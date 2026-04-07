/// Tamerlane Chess Engine — Transposition Table
/// Zobrist-based hash table for storing search results.
/// Supports both single-thread (WASM) and multi-thread (Native Lazy SMP) modes.

use crate::types::*;

// ─── TT Entry ───────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: u64,       // Full hash for verification
    pub depth: i8,       // Search depth
    pub score: i32,      // Evaluation score
    pub flag: TTFlag,    // Exact, Alpha, Beta
    pub best_move: Move, // Best move found
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TTFlag {
    Exact = 0,
    Alpha = 1, // Upper bound (failed low)
    Beta = 2,  // Lower bound (failed high)
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry {
            hash: 0,
            depth: -1,
            score: 0,
            flag: TTFlag::Exact,
            best_move: MOVE_NONE,
        }
    }
}

// ─── TTAccess Trait ──────────────────────────────────────────────
/// Unified interface for both single-threaded and atomic TT.

pub trait TTAccess {
    fn tt_probe(&self, hash: u64) -> Option<TTEntry>;
    fn tt_store(&self, hash: u64, depth: i8, score: i32, flag: TTFlag, best_move: Move);
    fn tt_clear(&self);
}

// ─── Single-Thread Transposition Table (WASM + default) ─────────

pub struct TranspositionTable {
    entries: std::cell::UnsafeCell<Vec<TTEntry>>,
    mask: usize,
}

// SAFETY: TranspositionTable is only used in single-threaded contexts (WASM / main thread)
unsafe impl Send for TranspositionTable {}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;
        let actual = num_entries.next_power_of_two() >> 1;
        let actual = actual.max(1024);
        TranspositionTable {
            entries: std::cell::UnsafeCell::new(vec![TTEntry::default(); actual]),
            mask: actual - 1,
        }
    }
}

impl TTAccess for TranspositionTable {
    fn tt_probe(&self, hash: u64) -> Option<TTEntry> {
        let idx = (hash as usize) & self.mask;
        let entries = unsafe { &*self.entries.get() };
        let entry = &entries[idx];
        if entry.hash == hash && entry.depth >= 0 {
            Some(*entry)
        } else {
            None
        }
    }

    fn tt_store(&self, hash: u64, depth: i8, score: i32, flag: TTFlag, best_move: Move) {
        let idx = (hash as usize) & self.mask;
        let entries = unsafe { &mut *self.entries.get() };
        let entry = &mut entries[idx];
        if entry.hash != hash || depth >= entry.depth {
            entry.hash = hash;
            entry.depth = depth;
            entry.score = score;
            entry.flag = flag;
            entry.best_move = best_move;
        }
    }

    fn tt_clear(&self) {
        let entries = unsafe { &mut *self.entries.get() };
        for entry in entries.iter_mut() {
            *entry = TTEntry::default();
        }
    }
}

// ─── Shared Atomic Transposition Table (for Lazy SMP) ────────────

#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(not(target_arch = "wasm32"))]
pub struct SharedTT {
    keys: Vec<AtomicU64>,
    data: Vec<AtomicU64>,
    mask: usize,
}

#[cfg(not(target_arch = "wasm32"))]
unsafe impl Send for SharedTT {}
#[cfg(not(target_arch = "wasm32"))]
unsafe impl Sync for SharedTT {}

#[cfg(not(target_arch = "wasm32"))]
impl SharedTT {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = 16; // two u64s
        let num_entries = (size_mb * 1024 * 1024) / entry_size;
        let actual = num_entries.next_power_of_two() >> 1;
        let actual = actual.max(1024);
        let mut keys = Vec::with_capacity(actual);
        let mut data = Vec::with_capacity(actual);
        for _ in 0..actual {
            keys.push(AtomicU64::new(0));
            data.push(AtomicU64::new(0));
        }
        SharedTT { keys, data, mask: actual - 1 }
    }
}

// Pack TT entry into a single u64:
// depth(8) | flag(2) | score(22) | best_move(32) = 64 bits
#[cfg(not(target_arch = "wasm32"))]
fn pack_data(depth: i8, score: i32, flag: TTFlag, best_move: Move) -> u64 {
    let d = (depth as u8) as u64;
    let f = (flag as u64) & 0x3;
    let s = ((score + 2_097_152) as u64) & 0x3F_FFFF;
    let m = (best_move as u64) & 0xFFFF_FFFF;
    (d << 56) | (f << 54) | (s << 32) | m
}

#[cfg(not(target_arch = "wasm32"))]
fn unpack_data(data: u64) -> (i8, i32, TTFlag, Move) {
    let depth = (data >> 56) as i8;
    let flag = match (data >> 54) & 0x3 {
        0 => TTFlag::Exact,
        1 => TTFlag::Alpha,
        _ => TTFlag::Beta,
    };
    let score = ((data >> 32) & 0x3F_FFFF) as i32 - 2_097_152;
    let best_move = (data & 0xFFFF_FFFF) as Move;
    (depth, score, flag, best_move)
}

#[cfg(not(target_arch = "wasm32"))]
impl TTAccess for SharedTT {
    fn tt_probe(&self, hash: u64) -> Option<TTEntry> {
        let idx = (hash as usize) & self.mask;
        let key = self.keys[idx].load(Ordering::Relaxed);
        if key == hash {
            let d = self.data[idx].load(Ordering::Relaxed);
            let (depth, score, flag, best_move) = unpack_data(d);
            if depth >= 0 {
                return Some(TTEntry { hash, depth, score, flag, best_move });
            }
        }
        None
    }

    fn tt_store(&self, hash: u64, depth: i8, score: i32, flag: TTFlag, best_move: Move) {
        let idx = (hash as usize) & self.mask;
        let existing_key = self.keys[idx].load(Ordering::Relaxed);
        if existing_key != hash {
            let packed = pack_data(depth, score, flag, best_move);
            self.keys[idx].store(hash, Ordering::Relaxed);
            self.data[idx].store(packed, Ordering::Relaxed);
        } else {
            let existing_data = self.data[idx].load(Ordering::Relaxed);
            let (existing_depth, _, _, _) = unpack_data(existing_data);
            if depth >= existing_depth {
                let packed = pack_data(depth, score, flag, best_move);
                self.keys[idx].store(hash, Ordering::Relaxed);
                self.data[idx].store(packed, Ordering::Relaxed);
            }
        }
    }

    fn tt_clear(&self) {
        for i in 0..self.keys.len() {
            self.keys[i].store(0, Ordering::Relaxed);
            self.data[i].store(0, Ordering::Relaxed);
        }
    }
}
