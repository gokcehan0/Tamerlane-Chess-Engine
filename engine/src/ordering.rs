/// Tamerlane Chess Engine — Move Ordering
/// Orders moves for better alpha-beta pruning: TT move > captures (MVV-LVA) > killers > history.

use crate::types::*;

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
pub const SCORE_CAPTURE_BASE: i32 = 1_000_000;
pub const SCORE_KILLER1: i32 = 900_000;
pub const SCORE_KILLER2: i32 = 800_000;

pub fn score_capture(captured: Piece, attacker: Piece) -> i32 {
    SCORE_CAPTURE_BASE + victim_value(captured) * 100 - attacker_value(attacker)
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

// ─── History heuristic ──────────────────────────────────────────

/// History table: [piece_kind * 2 + color][to_sq] — compact representation
/// Using piece_kind (0-10) * 2 + color (0-1) = 22 entries × 270 squares
pub struct HistoryTable {
    pub table: Box<[[i32; 270]; 22]>,
}

impl HistoryTable {
    pub fn new() -> Self {
        HistoryTable {
            table: Box::new([[0; 270]; 22]),
        }
    }

    fn index(piece: Piece) -> usize {
        let kind = piece.kind_index();
        let color = if piece.is_white() { 0 } else { 1 };
        kind * 2 + color
    }

    pub fn add(&mut self, from: usize, to: usize, depth: i32) {
        // Simple: use 'to' square only (more common approach)
        let _ = from;
        // We don't have the piece info here, so just use from/to hash
        let idx = (from % 22) as usize;
        self.table[idx][to] += depth * depth;
        if self.table[idx][to] > 500_000 {
            for row in self.table.iter_mut() {
                for val in row.iter_mut() {
                    *val /= 2;
                }
            }
        }
    }

    pub fn score(&self, from: usize, to: usize) -> i32 {
        let idx = from % 22;
        self.table[idx][to]
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
            } else if let Some(ks) = killers.is_killer(ply, mv) {
                score = ks;
            } else {
                let from = mv_from(mv);
                let to = mv_to(mv);
                score = history.score(from, to);
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
