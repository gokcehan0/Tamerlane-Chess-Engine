/// Tamerlane Chess Engine — Move Ordering
/// Orders moves for better alpha-beta pruning: TT move > captures (SEE) > killers > countermoves > history.

use crate::types::*;
use crate::board::*;

// ─── SEE piece values ────────────────────────────────────────────

/// Simple piece values for SEE (Static Exchange Evaluation)
pub fn see_value(p: Piece) -> i32 {
    match p.kind_index() {
        0 => 500,   // Rook
        1 => 325,   // Knight
        2 => 300,   // Catapult
        3 => 300,   // Giraffe
        4 => 150,   // Minister
        5 => 10000, // King
        6 => 150,   // Advisor
        7 => 160,   // Elephant
        8 => 160,   // Camel
        9 => 140,   // Warengine
        10 => 100,  // Pawn
        _ => 0,
    }
}

// ─── MVV-LVA tables ──────────────────────────────────────────────

fn victim_value(p: Piece) -> i32 {
    match p.kind_index() {
        0 => 600,   // Rook
        1 => 400,   // Knight
        2 => 350,   // Catapult
        3 => 350,   // Giraffe
        4 => 200,   // Minister
        5 => 9000,  // King
        6 => 200,   // Advisor
        7 => 200,   // Elephant
        8 => 250,   // Camel
        9 => 200,   // Warengine
        10 => 100,  // Pawn
        _ => 0,
    }
}

fn attacker_value(p: Piece) -> i32 {
    match p.kind_index() {
        10 => 1,    // Pawn (best attacker)
        4 => 2,     // Minister
        6 => 2,     // Advisor
        9 => 3,     // Warengine
        7 => 3,     // Elephant
        8 => 4,     // Camel
        2 => 5,     // Catapult
        3 => 5,     // Giraffe
        1 => 6,     // Knight
        0 => 7,     // Rook
        5 => 8,     // King
        _ => 10,
    }
}

// ─── Move scoring ────────────────────────────────────────────────

pub const SCORE_TT_MOVE: i32 = 10_000_000;
pub const SCORE_GOOD_CAPTURE: i32 = 5_000_000;   // Winning/equal captures (SEE >= 0)
pub const SCORE_CAPTURE_BASE: i32 = 1_000_000;
pub const SCORE_KILLER1: i32 = 900_000;
pub const SCORE_KILLER2: i32 = 800_000;
pub const SCORE_COUNTERMOVE: i32 = 700_000;
pub const SCORE_BAD_CAPTURE: i32 = -100_000;      // Losing captures (SEE < 0)

pub fn score_capture(captured: Piece, attacker: Piece) -> i32 {
    let mvv_lva = victim_value(captured) * 100 - attacker_value(attacker);
    // Simple SEE approximation: if attacker is less valuable than victim, it's a good capture
    if see_value(attacker) <= see_value(captured) {
        SCORE_GOOD_CAPTURE + mvv_lva
    } else {
        // Might still be good (e.g. protected), but rank lower
        SCORE_CAPTURE_BASE + mvv_lva
    }
}

// ─── Killer moves ────────────────────────────────────────────────

pub const MAX_PLY: usize = 128;

pub struct Killers {
    pub moves: [[Move; 2]; MAX_PLY],
}

impl Killers {
    pub fn new() -> Self {
        Killers {
            moves: [[MOVE_NONE; 2]; MAX_PLY],
        }
    }

    pub fn add(&mut self, ply: usize, mv: Move) {
        if ply >= MAX_PLY { return; }
        if self.moves[ply][0] != mv {
            self.moves[ply][1] = self.moves[ply][0];
            self.moves[ply][0] = mv;
        }
    }

    pub fn is_killer(&self, ply: usize, mv: Move) -> Option<i32> {
        if ply >= MAX_PLY { return None; }
        if self.moves[ply][0] == mv { return Some(SCORE_KILLER1); }
        if self.moves[ply][1] == mv { return Some(SCORE_KILLER2); }
        None
    }
}

// ─── Countermove heuristic ──────────────────────────────────────

/// Countermove table: for each previous move's (from, to), store the best reply.
/// Indexed by [from_sq][to_sq] — 270 × 270 would be too large.
/// Instead use [piece_type * 2 + color][to_sq]
pub struct CounterMoveTable {
    pub table: Box<[[Move; MAILBOX_SIZE]; 22]>,
}

impl CounterMoveTable {
    pub fn new() -> Self {
        CounterMoveTable {
            table: Box::new([[MOVE_NONE; MAILBOX_SIZE]; 22]),
        }
    }

    fn index(piece: Piece) -> usize {
        let kind = piece.kind_index();
        let color = if piece.is_white() { 0 } else { 1 };
        kind * 2 + color
    }

    pub fn store(&mut self, prev_piece: Piece, prev_to: usize, counter_mv: Move) {
        if prev_piece == Piece::Empty || prev_to >= MAILBOX_SIZE { return; }
        let idx = Self::index(prev_piece);
        self.table[idx][prev_to] = counter_mv;
    }

    pub fn probe(&self, prev_piece: Piece, prev_to: usize) -> Move {
        if prev_piece == Piece::Empty || prev_to >= MAILBOX_SIZE { return MOVE_NONE; }
        let idx = Self::index(prev_piece);
        self.table[idx][prev_to]
    }
}

// ─── History heuristic ──────────────────────────────────────────

/// Butterfly history table: indexed by [side][from_sq * 270 + to_sq]
/// This is the standard approach — each (from, to) pair gets its own score.
/// We use a flat Vec to avoid enormous stack allocations.
pub struct HistoryTable {
    /// [0] = White history, [1] = Black history
    /// Each is indexed by from_sq (0..270) * 270 + to_sq (0..270)
    /// But most squares are off-board, so effective size is much smaller.
    /// We use from_sq * MAILBOX_SIZE + to_sq for indexing.
    pub white: Vec<i32>,
    pub black: Vec<i32>,
}

const HIST_SIZE: usize = MAILBOX_SIZE * MAILBOX_SIZE;

impl HistoryTable {
    pub fn new() -> Self {
        HistoryTable {
            white: vec![0i32; HIST_SIZE],
            black: vec![0i32; HIST_SIZE],
        }
    }

    #[inline]
    fn idx(from: usize, to: usize) -> usize {
        from * MAILBOX_SIZE + to
    }

    /// Record a history bonus for a move that caused a beta cutoff
    pub fn add(&mut self, from: usize, to: usize, depth: i32, is_white: bool) {
        let i = Self::idx(from, to);
        if i >= HIST_SIZE { return; }
        let table = if is_white { &mut self.white } else { &mut self.black };
        table[i] += depth * depth;
        // Gravity: prevent overflow by halving all scores
        if table[i] > 400_000 {
            for val in table.iter_mut() {
                *val /= 2;
            }
        }
    }

    /// Penalize quiet moves that didn't cause a cutoff
    pub fn penalize(&mut self, from: usize, to: usize, depth: i32, is_white: bool) {
        let i = Self::idx(from, to);
        if i >= HIST_SIZE { return; }
        let table = if is_white { &mut self.white } else { &mut self.black };
        table[i] -= depth * depth;
        if table[i] < -400_000 {
            table[i] = -400_000;
        }
    }

    pub fn score(&self, from: usize, to: usize, is_white: bool) -> i32 {
        let i = Self::idx(from, to);
        if i >= HIST_SIZE { return 0; }
        if is_white { self.white[i] } else { self.black[i] }
    }
}

// ─── Order moves ─────────────────────────────────────────────────

pub fn score_moves(
    moves: &[Move],
    board_pieces: &[Piece],
    tt_move: Move,
    killers: &Killers,
    history: &HistoryTable,
    ply: usize,
    countermove: Move,
    is_white: bool,
) -> Vec<(Move, i32)> {
    let mut scored: Vec<(Move, i32)> = Vec::with_capacity(moves.len());

    for &mv in moves {
        let score;
        if mv == tt_move && tt_move != MOVE_NONE {
            score = SCORE_TT_MOVE;
        } else {
            let captured = mv_captured(mv);
            if captured != Piece::Empty {
                let from_sq = mv_from(mv);
                let attacker = board_pieces[from_sq];
                score = score_capture(captured, attacker);
            } else if mv_is_promotion(mv) {
                score = SCORE_GOOD_CAPTURE + 100; // Promotions are very important
            } else if let Some(ks) = killers.is_killer(ply, mv) {
                score = ks;
            } else if mv == countermove && countermove != MOVE_NONE {
                score = SCORE_COUNTERMOVE;
            } else {
                let from = mv_from(mv);
                let to = mv_to(mv);
                score = history.score(from, to, is_white);
            }
        }
        scored.push((mv, score));
    }

    scored
}

/// Pick the best-scored move (selection sort — lazy, one at a time)
pub fn pick_best(moves: &mut [(Move, i32)], start: usize) -> Move {
    let mut best_idx = start;
    let mut best_score = moves[start].1;

    for i in (start + 1)..moves.len() {
        if moves[i].1 > best_score {
            best_score = moves[i].1;
            best_idx = i;
        }
    }

    moves.swap(start, best_idx);
    moves[start].0
}
