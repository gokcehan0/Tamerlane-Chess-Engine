/// Tamerlane Chess Engine — Evaluation
/// Static position evaluation: material + PST + mobility + king safety + pawn advancement.

use crate::types::*;
use crate::board::*;
use crate::attack::is_attacked;
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
    0,    // King — NEVER count in eval (always on board, counting causes phantom scores)
    150,  // Advisor (Wazir: 1 orthogonal — weak piece)
    160,  // Elephant (Alfil: 1-2 diagonal jump — somewhat weak)
    160,  // Camel (3,1 leaper — colorbound but decent)
    140,  // Warengine (Dabbaba: 2 orthogonal jump — limited)
    100,  // Pawn
];

// ─── Piece-Square Tables (10 ranks × 11 files = 110 entries) ─────

/// PST for pawns — strongly encourage center pawn advancement
const PST_PAWN_WHITE: [i32; 110] = [
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  // rank 1 (start)
    5,  5, 10, 15, 20, 20, 20, 15, 10,  5,  5,  // rank 2 — center pawns get bonus for being ready
   10, 15, 20, 30, 35, 40, 35, 30, 20, 15, 10,  // rank 3 — big reward for center advance!
   15, 20, 25, 35, 45, 50, 45, 35, 25, 20, 15,  // rank 4 — controlling center is great
   20, 25, 30, 40, 50, 55, 50, 40, 30, 25, 20,  // rank 5 — deep center control
   25, 30, 35, 45, 55, 60, 55, 45, 35, 30, 25,  // rank 6
   30, 35, 40, 50, 60, 65, 60, 50, 40, 35, 30,  // rank 7
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

/// PST for camel — penalize early development, reward staying on back ranks
/// Camel is colorbound (3,1 leaper): don't rush it to center in the opening
const PST_CAMEL: [i32; 110] = [
    15, 10, 10, 10,  5,  5,  5, 10, 10, 10, 15,  // rank 1 — bonus for starting position
    10,  5,  5,  0,  0,  0,  0,  0,  5,  5, 10,  // rank 2 — still okay on back ranks
     0,  0, -5, -5,-10,-10,-10, -5, -5,  0,  0,  // rank 3 — penalize early center approach
    -5, -5,-10,-15,-20,-20,-20,-15,-10, -5, -5,  // rank 4 — strong penalty in center
    -5, -5,-10,-10,-15,-15,-15,-10,-10, -5, -5,  // rank 5 — still somewhat bad
     0,  0, -5, -5, -5, -5, -5, -5, -5,  0,  0,  // rank 6 — neutral as game progresses
     5,  5,  0,  5,  5,  5,  5,  5,  0,  5,  5,  // rank 7 — okay in midgame
    10,  5,  5, 10, 10, 10, 10, 10,  5,  5, 10,  // rank 8 — good for invasion
    10,  5,  5,  0,  0,  0,  0,  0,  5,  5, 10,  // rank 9
    15, 10, 10, 10,  5,  5,  5, 10, 10, 10, 15,  // rank 10
];

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
        8 => PST_CAMEL[idx],      // Camel — custom PST to prevent early development
        10 => PST_PAWN_WHITE[idx], // Pawn
        _ => PST_GENERIC[idx],    // Everything else
    }
}

// ─── Evaluation function ─────────────────────────────────────────

/// Evaluate position from the perspective of the side to move.
/// Positive = good for side to move.
pub fn evaluate(board: &Board) -> i32 {
    evaluate_internal(board, false)
}

/// Evaluate with optional breakdown logging
pub fn evaluate_with_log(board: &Board) -> i32 {
    evaluate_internal(board, true)
}

fn evaluate_internal(board: &Board, log: bool) -> i32 {
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

    let mat_pst_score = score;

    let mob = mobility_estimate(board);
    score += mob;

    let ks = king_safety(board);
    score += ks;

    let dev = development_penalty(board);
    score += dev;

    let ps = pawn_structure(board);
    score += ps;

    let eg = endgame_eval(board);
    score += eg;

    let hp = hanging_pieces(board);
    score += hp;

    let tempo = 12;
    let final_score = if board.side == Color::White { score + tempo } else { -score + tempo };

    if log && final_score.abs() > 500 {
        #[cfg(target_arch = "wasm32")]
        crate::console_log(&format!("EVAL BREAKDOWN: mat+pst={} mob={} ks={} dev={} ps={} eg={} hp={} total={} final={}",
            mat_pst_score, mob, ks, dev, ps, eg, hp, score, final_score));
        #[cfg(not(target_arch = "wasm32"))]
        println!("EVAL BREAKDOWN: mat+pst={} mob={} ks={} dev={} ps={} eg={} hp={} total={} final={}",
            mat_pst_score, mob, ks, dev, ps, eg, hp, score, final_score);
    }

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

    // ~3 centipawns per mobility point (increased from 2 for stronger positional play)
    (white_mobility - black_mobility) * 3
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

/// Development penalty: penalize moving non-pawn pieces before developing center pawns.
/// In the opening (low ply count), if center pawns haven't moved but pieces have,
/// apply a penalty to discourage premature piece development.
fn development_penalty(board: &Board) -> i32 {
    // Only apply in early game (first ~20 half-moves)
    if board.ply > 20 { return 0; }

    let mut score = 0i32;

    // Check if White center pawns are still on starting rank (rank 3 = pawn row)
    // Files 4-8 (d through h) are center files on the 11-file board
    let mut w_center_pawns_unmoved = 0;
    for file in 4..=8 {
        let s = sq(file, 3); // White pawn starting rank
        let p = board.pieces[s];
        if p.is_pawn() && p.is_white() {
            w_center_pawns_unmoved += 1;
        }
    }

    // Check if White pieces have moved from back ranks
    // Camels start at rank 1 (files 3 and 9 in starting FEN: D positions)
    // If camel is NOT on rank 1 or 2 but center pawns haven't moved, penalize
    let mut w_pieces_developed = 0;
    for rank in 3..=10 {
        for file in 1..=11 {
            let s = sq(file, rank);
            let p = board.pieces[s];
            if p == Piece::Empty || p.is_pawn() || p.is_king_type() { continue; }
            if p.is_white() {
                w_pieces_developed += 1;
            }
        }
    }

    // Penalize developing pieces while center pawns are still home
    // More unmoved center pawns + more developed pieces = bigger penalty
    if w_center_pawns_unmoved >= 3 && w_pieces_developed > 0 {
        score -= w_pieces_developed * 15; // 15cp per developed piece when pawns are home
    }

    // Mirror for Black
    let mut b_center_pawns_unmoved = 0;
    for file in 4..=8 {
        let s = sq(file, 8); // Black pawn starting rank
        let p = board.pieces[s];
        if p.is_pawn() && p.is_black() {
            b_center_pawns_unmoved += 1;
        }
    }

    let mut b_pieces_developed = 0;
    for rank in 1..=8 {
        for file in 1..=11 {
            let s = sq(file, rank);
            let p = board.pieces[s];
            if p == Piece::Empty || p.is_pawn() || p.is_king_type() { continue; }
            if p.is_black() {
                b_pieces_developed += 1;
            }
        }
    }

    if b_center_pawns_unmoved >= 3 && b_pieces_developed > 0 {
        score += b_pieces_developed * 15;
    }

    score
}

/// Pawn structure evaluation: doubled pawns, isolated pawns, passed pawns.
/// Doubled pawns = multiple friendly pawns on the same file.
/// Isolated pawns = no friendly pawns on adjacent files.
/// Passed pawns = no enemy pawns ahead on this file or adjacent files.
fn pawn_structure(board: &Board) -> i32 {
    let mut score = 0i32;

    // Count pawns per file for each side
    let mut w_pawns_on_file = [0i32; 12]; // files 1..11, index 0 unused
    let mut b_pawns_on_file = [0i32; 12];
    let mut w_pawn_most_advanced = [0i32; 12]; // highest rank for white pawn
    let mut b_pawn_most_advanced = [11i32; 12]; // lowest rank for black pawn

    for rank in 1..=10 {
        for file in 1..=11 {
            let s = sq(file, rank);
            let p = board.pieces[s];
            if !p.is_pawn() { continue; }
            if p.is_white() {
                w_pawns_on_file[file as usize] += 1;
                if rank as i32 > w_pawn_most_advanced[file as usize] {
                    w_pawn_most_advanced[file as usize] = rank as i32;
                }
            } else {
                b_pawns_on_file[file as usize] += 1;
                if (rank as i32) < b_pawn_most_advanced[file as usize] {
                    b_pawn_most_advanced[file as usize] = rank as i32;
                }
            }
        }
    }

    for file in 1..=11usize {
        // ─── Doubled pawns penalty ─────────────────────
        if w_pawns_on_file[file] > 1 {
            score -= (w_pawns_on_file[file] - 1) * 15; // -15cp per extra pawn
        }
        if b_pawns_on_file[file] > 1 {
            score += (b_pawns_on_file[file] - 1) * 15;
        }

        // ─── Isolated pawns penalty ────────────────────
        if w_pawns_on_file[file] > 0 {
            let left = if file > 1 { w_pawns_on_file[file - 1] } else { 0 };
            let right = if file < 11 { w_pawns_on_file[file + 1] } else { 0 };
            if left == 0 && right == 0 {
                score -= 12; // Isolated white pawn
            }
        }
        if b_pawns_on_file[file] > 0 {
            let left = if file > 1 { b_pawns_on_file[file - 1] } else { 0 };
            let right = if file < 11 { b_pawns_on_file[file + 1] } else { 0 };
            if left == 0 && right == 0 {
                score += 12; // Isolated black pawn
            }
        }

        // ─── Passed pawns bonus ────────────────────────
        // White passed pawn: no black pawns ahead on this file or adjacent files
        if w_pawns_on_file[file] > 0 {
            let adv_rank = w_pawn_most_advanced[file];
            let mut is_passed = true;
            for check_file in (file.saturating_sub(1))..=(file + 1).min(11) {
                // Check if any black pawn is at rank >= adv_rank (ahead of this white pawn)
                for r in (adv_rank + 1)..=10 {
                    let s = sq(check_file as i32, r);
                    if s < MAILBOX_SIZE {
                        let p = board.pieces[s];
                        if p.is_pawn() && p.is_black() {
                            is_passed = false;
                            break;
                        }
                    }
                }
                if !is_passed { break; }
            }
            if is_passed && adv_rank > 1 {
                // Bonus scales with how far advanced the pawn is
                let bonus = 10 + (adv_rank - 1) * 5; // rank 2=15, rank 5=30, rank 9=50
                score += bonus;
            }
        }

        // Black passed pawn
        if b_pawns_on_file[file] > 0 {
            let adv_rank = b_pawn_most_advanced[file];
            let mut is_passed = true;
            for check_file in (file.saturating_sub(1))..=(file + 1).min(11) {
                for r in 1..adv_rank {
                    let s = sq(check_file as i32, r);
                    if s < MAILBOX_SIZE {
                        let p = board.pieces[s];
                        if p.is_pawn() && p.is_white() {
                            is_passed = false;
                            break;
                        }
                    }
                }
                if !is_passed { break; }
            }
            if is_passed && adv_rank < 10 {
                let bonus = 10 + (10 - adv_rank) * 5;
                score -= bonus;
            }
        }
    }

    score
}

/// Endgame evaluation: when material is low, encourage king centralization
/// and penalize the losing king being on the edge.
fn endgame_eval(board: &Board) -> i32 {
    // Count total non-pawn, non-king material
    let mut w_material = 0i32;
    let mut b_material = 0i32;

    for rank in 1..=10 {
        for file in 1..=11 {
            let s = sq(file, rank);
            let p = board.pieces[s];
            if p == Piece::Empty || p.is_pawn() || p.is_king_type() { continue; }
            let val = MATERIAL[p.kind_index()];
            if p.is_white() { w_material += val; } else { b_material += val; }
        }
    }

    let total_material = w_material + b_material;

    // Only apply endgame bonuses when material is low (less than ~2 rooks + minor piece each side)
    if total_material > 2400 { return 0; }

    let mut score = 0i32;

    // Encourage the winning side's king to move toward center
    // and the losing side's king to be pushed to edge
    let phase = 1.0 - (total_material as f64 / 2400.0); // 0.0 = midgame, 1.0 = pure endgame

    if let Some(wk) = board.king_sq(Color::White) {
        let wk_file = file_brd(wk);
        let wk_rank = rank_brd(wk);
        // Center distance: how far from center (file 6, rank 5.5)
        let file_dist = (wk_file - 6).abs();
        let rank_dist = ((wk_rank * 2 - 11).abs()) / 2; // approx distance from rank 5-6
        let center_dist = file_dist + rank_dist;
        // In endgame, reward king being close to center
        let king_centrality = (8 - center_dist).max(0) * 5;
        score += (king_centrality as f64 * phase) as i32;
    }

    if let Some(bk) = board.king_sq(Color::Black) {
        let bk_file = file_brd(bk);
        let bk_rank = rank_brd(bk);
        let file_dist = (bk_file - 6).abs();
        let rank_dist = ((bk_rank * 2 - 11).abs()) / 2;
        let center_dist = file_dist + rank_dist;
        let king_centrality = (8 - center_dist).max(0) * 5;
        score -= (king_centrality as f64 * phase) as i32;
    }

    // If one side has significant material advantage, encourage pushing enemy king to edge
    if w_material > b_material + 200 {
        if let Some(bk) = board.king_sq(Color::Black) {
            let bk_file = file_brd(bk);
            let bk_rank = rank_brd(bk);
            let edge_dist_file = (bk_file - 1).min(11 - bk_file);
            let edge_dist_rank = (bk_rank - 1).min(10 - bk_rank);
            let edge_dist = edge_dist_file.min(edge_dist_rank);
            // Bonus for pushing enemy king toward edge
            score += ((4 - edge_dist).max(0) * 10) as i32;
        }
    } else if b_material > w_material + 200 {
        if let Some(wk) = board.king_sq(Color::White) {
            let wk_file = file_brd(wk);
            let wk_rank = rank_brd(wk);
            let edge_dist_file = (wk_file - 1).min(11 - wk_file);
            let edge_dist_rank = (wk_rank - 1).min(10 - wk_rank);
            let edge_dist = edge_dist_file.min(edge_dist_rank);
            score -= ((4 - edge_dist).max(0) * 10) as i32;
        }
    }

    score
}

/// Hanging piece detection: small penalty for undefended pieces under attack.
/// Capped to prevent eval inflation that causes the engine to overvalue positions.
fn hanging_pieces(board: &Board) -> i32 {
    let mut score = 0i32;

    const PIECE_VAL: [i32; 11] = [
        500, 325, 300, 300, 150, 0, 150, 160, 160, 140, 100,
        //                       ^ King = 0, never penalize
    ];

    let mut total_white_penalty = 0i32;
    let mut total_black_penalty = 0i32;

    for rank in 1..=10 {
        for file in 1..=11 {
            let s = sq(file, rank);
            let p = board.pieces[s];
            if p == Piece::Empty || p.is_pawn() || p.is_king_type() { continue; }

            let piece_value = PIECE_VAL[p.kind_index()];
            if piece_value == 0 { continue; }

            let is_white = p.is_white();
            let enemy_color = if is_white { Color::Black } else { Color::White };
            let friendly_color = if is_white { Color::White } else { Color::Black };

            // Only penalize if attacked AND undefended
            if is_attacked(board, s, enemy_color) && !is_attacked(board, s, friendly_color) {
                // Small penalty: 25% of piece value
                let penalty = piece_value * 25 / 100;
                if is_white {
                    total_white_penalty += penalty;
                } else {
                    total_black_penalty += penalty;
                }
            }
        }
    }

    // Cap penalties to prevent eval inflation (max 400cp effect)
    total_white_penalty = total_white_penalty.min(400);
    total_black_penalty = total_black_penalty.min(400);

    score -= total_white_penalty;
    score += total_black_penalty;

    score
}

/// Check if a square is attacked by an enemy pawn
fn is_pawn_attacking(board: &Board, sq_idx: usize, by_color: Color) -> bool {
    if by_color == Color::White {
        // White pawns at sq-14 or sq-16 attack this square
        let s1 = (sq_idx as i32 - 14) as usize;
        let s2 = (sq_idx as i32 - 16) as usize;
        if !is_off_board(s1) { let p = board.pieces[s1]; if p.is_pawn() && p.is_white() { return true; } }
        if !is_off_board(s2) { let p = board.pieces[s2]; if p.is_pawn() && p.is_white() { return true; } }
    } else {
        // Black pawns at sq+14 or sq+16 attack this square
        let s1 = (sq_idx as i32 + 14) as usize;
        let s2 = (sq_idx as i32 + 16) as usize;
        if !is_off_board(s1) { let p = board.pieces[s1]; if p.is_pawn() && p.is_black() { return true; } }
        if !is_off_board(s2) { let p = board.pieces[s2]; if p.is_pawn() && p.is_black() { return true; } }
    }
    false
}
