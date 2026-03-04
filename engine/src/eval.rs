/// Tamerlane Chess Engine — Evaluation
/// Static position evaluation: material + PST + mobility + king safety + pawn advancement.

use crate::types::*;
use crate::board::*;
use std::sync::atomic::{AtomicU64, Ordering};

pub static EVAL_SEED: AtomicU64 = AtomicU64::new(0);

pub fn set_game_seed(seed: u64) {
    EVAL_SEED.store(seed, Ordering::Relaxed);
}

// ─── Material values (centipawns) ────────────────────────────────

/// Piece material values indexed by Piece kind_index
/// Rook=0, Knight=1, Catapult=2, Giraffe=3, Minister=4, King=5, Advisor=6, Elephant=7, Camel=8, Warengine=9, Pawn=10
const MATERIAL: [i32; 11] = [
    500,  // Rook (orthogonal slider — most powerful non-royal)
    325,  // Knight (2,1 leaper — strong in open positions)
    300,  // Catapult/Picket (diagonal slider, min 2 range)
    300,  // Giraffe (4,1 bent rider — complex but powerful)
    150,  // Minister (Ferz: 1 diagonal — weak piece)
    10000, // King (infinite — never actually used for trade)
    150,  // Advisor (Wazir: 1 orthogonal — weak piece)
    160,  // Elephant (Alfil: 1-2 diagonal jump — somewhat weak)
    160,  // Camel (3,1 leaper — colorbound but decent)
    140,  // Warengine (Dabbaba: 2 orthogonal jump — limited)
    100,  // Pawn
];

// ─── Piece-Square Tables (10 ranks × 11 files = 110 entries) ─────

/// PST for pawns — encourage advancement toward promotion
const PST_PAWN_WHITE: [i32; 110] = [
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  // rank 1 (start)
    5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,   // rank 2
    5, 10, 10, 15, 15, 15, 15, 15, 10, 10,  5,  // rank 3
   10, 15, 15, 20, 25, 25, 25, 20, 15, 15, 10,  // rank 4
   15, 20, 20, 30, 35, 35, 35, 30, 20, 20, 15,  // rank 5
   20, 25, 30, 40, 45, 50, 45, 40, 30, 25, 20,  // rank 6
   30, 35, 40, 50, 55, 60, 55, 50, 40, 35, 30,  // rank 7
   40, 45, 50, 60, 65, 70, 65, 60, 50, 45, 40,  // rank 8
   50, 55, 60, 70, 75, 80, 75, 70, 60, 55, 50,  // rank 9
   60, 65, 70, 80, 85, 90, 85, 80, 70, 65, 60,  // rank 10 (promotion)
];

/// PST for knights — centralized is better
const PST_KNIGHT: [i32; 110] = [
   -20,-10, -5, -5, -5, -5, -5, -5, -5,-10,-20,
   -10,  0,  5,  5, 10, 10, 10,  5,  5,  0,-10,
    -5,  5, 10, 15, 15, 20, 15, 15, 10,  5, -5,
    -5,  5, 15, 20, 25, 25, 25, 20, 15,  5, -5,
    -5, 10, 15, 25, 30, 30, 30, 25, 15, 10, -5,
    -5, 10, 15, 25, 30, 30, 30, 25, 15, 10, -5,
    -5,  5, 15, 20, 25, 25, 25, 20, 15,  5, -5,
    -5,  5, 10, 15, 15, 20, 15, 15, 10,  5, -5,
   -10,  0,  5,  5, 10, 10, 10,  5,  5,  0,-10,
   -20,-10, -5, -5, -5, -5, -5, -5, -5,-10,-20,
];

/// PST for rooks — favor open files and 7th rank
const PST_ROOK: [i32; 110] = [
    0,  0,  5, 10, 10, 10, 10, 10,  5,  0,  0,
    0,  0,  5, 10, 10, 10, 10, 10,  5,  0,  0,
   -5,  0,  5, 10, 10, 10, 10, 10,  5,  0, -5,
   -5,  0,  5, 10, 10, 10, 10, 10,  5,  0, -5,
   -5,  0,  5, 10, 10, 10, 10, 10,  5,  0, -5,
   -5,  0,  5, 10, 15, 15, 15, 10,  5,  0, -5,
   -5,  0,  5, 10, 15, 15, 15, 10,  5,  0, -5,
    5, 10, 15, 20, 20, 20, 20, 20, 15, 10,  5, // 7th rank bonus
    5, 10, 15, 20, 20, 20, 20, 20, 15, 10,  5,
    0,  0,  5, 10, 10, 10, 10, 10,  5,  0,  0,
];

/// PST for king — stay safe (back rank, away from center)
const PST_KING: [i32; 110] = [
    20, 30, 10,  0,  0,-10,  0,  0, 10, 30, 20,
    20, 20,  0,-10,-10,-20,-10,-10,  0, 20, 20,
   -10,-20,-20,-30,-30,-30,-30,-30,-20,-20,-10,
   -20,-30,-30,-40,-40,-40,-40,-40,-30,-30,-20,
   -30,-40,-40,-50,-50,-50,-50,-50,-40,-40,-30,
   -30,-40,-40,-50,-50,-50,-50,-50,-40,-40,-30,
   -20,-30,-30,-40,-40,-40,-40,-40,-30,-30,-20,
   -10,-20,-20,-30,-30,-30,-30,-30,-20,-20,-10,
    20, 20,  0,-10,-10,-20,-10,-10,  0, 20, 20,
    20, 30, 10,  0,  0,-10,  0,  0, 10, 30, 20,
];

/// Generic piece PST — slight center preference
/// Generic piece PST — slight center preference (lowered to avoid artificial rushing)
const PST_GENERIC: [i32; 110] = [
   -10, -5, -5,  0,  0,  0,  0,  0, -5, -5,-10,
    -5,  0,  5,  0,  0,  0,  0,  0,  5,  0, -5,
    -5,  0,  0,  5,  5,  5,  5,  5,  0,  0, -5,
     0,  0,  5, 10, 10, 10, 10, 10,  5,  0,  0,
     0,  0,  5, 10, 10, 10, 10, 10,  5,  0,  0,
     0,  0,  5, 10, 10, 10, 10, 10,  5,  0,  0,
     0,  0,  5, 10, 10, 10, 10, 10,  5,  0,  0,
    -5,  0,  0,  5,  5,  5,  5,  5,  0,  0, -5,
    -5,  0,  5,  0,  0,  0,  0,  0,  5,  0, -5,
   -10, -5, -5,  0,  0,  0,  0,  0, -5, -5,-10,
];

fn pst_index(file: i32, rank: i32, is_white: bool) -> usize {
    let r = if is_white { rank - 1 } else { 10 - rank };
    let f = file - 1;
    (r * 11 + f) as usize
}

fn get_pst(piece: Piece, file: i32, rank: i32) -> i32 {
    let is_white = piece.is_white();
    let idx = pst_index(file, rank, is_white);
    if idx >= 110 { return 0; }

    let kind = piece.kind_index();
    match kind {
        0 => PST_ROOK[idx],       // Rook
        1 => PST_KNIGHT[idx],     // Knight
        5 => PST_KING[idx],       // King/Prince/AdKing
        10 => PST_PAWN_WHITE[idx], // Pawn
        _ => PST_GENERIC[idx],    // Everything else
    }
}

// ─── Evaluation function ─────────────────────────────────────────

/// Evaluate position from the perspective of the side to move.
/// Positive = good for side to move.
pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0i32; // from White's perspective

    for rank in 1..=10 {
        for file in 1..=11 {
            let s = sq(file, rank);
            let p = board.pieces[s];
            if p == Piece::Empty { continue; }

            let mat = MATERIAL[p.kind_index()];
            let pst = get_pst(p, file as i32, rank as i32);
            let value = mat + pst;

            if p.is_white() {
                score += value;
            } else {
                score -= value;
            }
        }
    }

    // Mobility bonus (approximate - count pieces that aren't boxed in)
    // We don't want to do full move generation here for speed, so we use a simpler metric
    score += mobility_estimate(board);

    // King safety: penalize exposed king
    score += king_safety(board);

    // Return from side-to-move perspective
    // Return from side-to-move perspective
    let mut final_score = if board.side == Color::White { score } else { -score };

    // Deterministic pseudo-random jitter based on position hash 
    // to break ties and ensure opening variety without breaking TT consistency
    let seed = EVAL_SEED.load(Ordering::Relaxed);
    let jitter_base = board.hash ^ seed;
    let jitter = (jitter_base % 11) as i32 - 5; // -5 to +5 centipawns
    final_score += jitter;

    final_score
}

fn mobility_estimate(board: &Board) -> i32 {
    let mut white_mobility = 0i32;
    let mut black_mobility = 0i32;

    for rank in 1..=10 {
        for file in 1..=11 {
            let s = sq(file, rank);
            let p = board.pieces[s];
            if p == Piece::Empty || p.is_pawn() { continue; }

            let dirs: &[i32] = match p {
                Piece::WRook | Piece::BRook => &ROOK_DIRS,
                Piece::WKnight | Piece::BKnight => &KNIGHT_DIRS,
                Piece::WKing | Piece::BKing |
                Piece::WPrince | Piece::BPrince |
                Piece::WAdKing | Piece::BAdKing => &KING_DIRS,
                Piece::WMinister | Piece::BMinister => &MINISTER_DIRS,
                Piece::WAdvisor | Piece::BAdvisor => &ADVISOR_DIRS,
                Piece::WCamel | Piece::BCamel => &CAMEL_DIRS,
                Piece::WWarengine | Piece::BWarengine => &WARENGINE_DIRS,
                Piece::WElephant | Piece::BElephant => &ELEPHANT_DIRS,
                Piece::WCatapult | Piece::BCatapult => &MINISTER_DIRS, // Uses diagonal like minister for basic mobility counting
                // Giraffe is too complex for simple mobility counting, give it a flat bonus
                Piece::WGiraffe | Piece::BGiraffe => {
                    if p.is_white() { white_mobility += 4; } else { black_mobility += 4; }
                    continue;
                },
                _ => continue,
            };

            let is_slider = matches!(p, Piece::WRook | Piece::BRook | Piece::WCatapult | Piece::BCatapult);
            let mut count = 0i32;

            for &d in dirs {
                if is_slider {
                    let mut ts = (s as i32 + d) as usize;
                    while !is_off_board(ts) && board.pieces[ts] == Piece::Empty {
                        count += 1;
                        ts = (ts as i32 + d) as usize;
                    }
                    if !is_off_board(ts) {
                        let target_piece = board.pieces[ts];
                        if (p.is_white() && target_piece.is_black()) || (p.is_black() && target_piece.is_white()) {
                            count += 1; // can capture enemy
                        }
                    }
                } else {
                    let ts = (s as i32 + d) as usize;
                    if !is_off_board(ts) {
                        let target_piece = board.pieces[ts];
                        if target_piece == Piece::Empty || (p.is_white() && target_piece.is_black()) || (p.is_black() && target_piece.is_white()) {
                            count += 1;
                        }
                    }
                }
            }

            if p.is_white() {
                white_mobility += count;
            } else {
                black_mobility += count;
            }
        }
    }

    // ~2 centipawns per mobility point
    (white_mobility - black_mobility) * 2
}

fn king_safety(board: &Board) -> i32 {
    let mut score = 0i32;

    // White king safety
    if let Some(wk) = board.king_sq(Color::White) {
        let wk_rank = rank_brd(wk);
        let wk_file = file_brd(wk);
        // Penalize king in center of board (ranks 4-7)
        if wk_rank >= 4 && wk_rank <= 7 {
            score -= 30;
        }
        // Penalize king on edges (files 1, 11)
        if wk_file <= 2 || wk_file >= 10 {
            score -= 15;
        }
        // Count adjacent friendly pieces (shield)
        let mut shield = 0;
        for &d in &KING_DIRS {
            let ts = (wk as i32 + d) as usize;
            if !is_off_board(ts) && board.pieces[ts].is_white() {
                shield += 1;
            }
        }
        score += shield * 10;
    }

    // Black king safety (mirror)
    if let Some(bk) = board.king_sq(Color::Black) {
        let bk_rank = rank_brd(bk);
        let bk_file = file_brd(bk);
        if bk_rank >= 4 && bk_rank <= 7 {
            score += 30;
        }
        if bk_file <= 2 || bk_file >= 10 {
            score += 15;
        }
        let mut shield = 0;
        for &d in &KING_DIRS {
            let ts = (bk as i32 + d) as usize;
            if !is_off_board(ts) && board.pieces[ts].is_black() {
                shield += 1;
            }
        }
        score -= shield * 10;
    }

    score
}
