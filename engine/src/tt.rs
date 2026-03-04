/// Tamerlane Chess Engine — Transposition Table
/// Zobrist-based hash table for storing search results.

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

// ─── Transposition Table ─────────────────────────────────────────

pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    mask: usize,
}

impl TranspositionTable {
    /// Create a TT with `size` entries (will be rounded to power of 2)
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;
        // Round down to power of 2
        let actual = num_entries.next_power_of_two() >> 1;
        let actual = actual.max(1024);
        TranspositionTable {
            entries: vec![TTEntry::default(); actual],
            mask: actual - 1,
        }
    }

    #[inline]
    fn index(&self, hash: u64) -> usize {
        (hash as usize) & self.mask
    }

    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let idx = self.index(hash);
        let entry = &self.entries[idx];
        if entry.hash == hash && entry.depth >= 0 {
            Some(entry)
        } else {
            None
        }
    }

    pub fn store(&mut self, hash: u64, depth: i8, score: i32, flag: TTFlag, best_move: Move) {
        let idx = self.index(hash);
        let entry = &mut self.entries[idx];
        // Always replace if deeper or different position
        if entry.hash != hash || depth >= entry.depth {
            entry.hash = hash;
            entry.depth = depth;
            entry.score = score;
            entry.flag = flag;
            entry.best_move = best_move;
        }
    }

    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = TTEntry::default();
        }
    }
}
