/// Tamerlane Chess Engine — Move Generator
/// Generates all pseudo-legal moves for the side to move.

use crate::types::*;
use crate::board::*;
use crate::attack::is_attacked;

/// Generate all pseudo-legal moves for the current side.
pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(200);
    let side = board.side;

    for rank in 1..=10 {
        for file in 1..=11 {
            let s = sq(file, rank);
            let piece = board.pieces[s];
            if piece == Piece::Empty { continue; }
            if piece.color() != Some(side) { continue; }

            if piece.is_pawn() {
                gen_pawn_moves(board, s, piece, side, &mut moves);
            } else {
                gen_piece_moves(board, s, piece, side, &mut moves);
            }
        }
    }
    moves
}

/// Generate only capture moves (for quiescence search)
pub fn generate_captures(board: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(80);
    let side = board.side;

    for rank in 1..=10 {
        for file in 1..=11 {
            let s = sq(file, rank);
            let piece = board.pieces[s];
            if piece == Piece::Empty { continue; }
            if piece.color() != Some(side) { continue; }

            if piece.is_pawn() {
                gen_pawn_captures(board, s, piece, side, &mut moves);
            } else {
                gen_piece_captures(board, s, piece, side, &mut moves);
            }
        }
    }
    moves
}

// ─── Citadel restrictions ─────────────────────────────────────────

#[inline]
fn can_enter_square(piece: Piece, to_sq: usize, side: Color) -> bool {
    // If it's not a citadel, everyone can enter
    if to_sq != WHITE_CITADEL && to_sq != BLACK_CITADEL {
        return true;
    }

    if side == Color::White {
        if to_sq == WHITE_CITADEL { return false; } // Cannot enter own
        if to_sq == BLACK_CITADEL {
            return matches!(piece, Piece::WKing | Piece::WAdKing | Piece::WPrince);
        }
    } else {
        if to_sq == BLACK_CITADEL { return false; } // Cannot enter own
        if to_sq == WHITE_CITADEL {
            return matches!(piece, Piece::BKing | Piece::BAdKing | Piece::BPrince);
        }
    }
    false
}

// ─── Pawn moves ──────────────────────────────────────────────────

fn gen_pawn_moves(board: &Board, s: usize, piece: Piece, side: Color, moves: &mut Vec<Move>) {
    let forward = if side == Color::White { 15i32 } else { -15i32 };
    let cap_dirs: [i32; 2] = if side == Color::White { [14, 16] } else { [-14, -16] };
    let promo_rank = if side == Color::White { 10 } else { 1 };

    // Forward move (1 square, must be empty)
    let fwd = (s as i32 + forward) as usize;
    if !is_off_board(fwd) && can_enter_square(piece, fwd, side) && board.pieces[fwd] == Piece::Empty {
        if rank_brd(fwd) == promo_rank {
            let prom = piece.promoted();
            moves.push(make_move(s, fwd, Piece::Empty, true, prom));
        } else {
            moves.push(make_quiet(s, fwd));
        }
    }

    // Diagonal captures
    for &d in &cap_dirs {
        let ts = (s as i32 + d) as usize;
        if is_off_board(ts) || !can_enter_square(piece, ts, side) { continue; }
        let target = board.pieces[ts];
        if target != Piece::Empty && target.color() != Some(side) {
            if rank_brd(ts) == promo_rank {
                let prom = piece.promoted();
                moves.push(make_move(s, ts, target, true, prom));
            } else {
                moves.push(make_capture(s, ts, target));
            }
        }
    }
}

fn gen_pawn_captures(board: &Board, s: usize, piece: Piece, side: Color, moves: &mut Vec<Move>) {
    let cap_dirs: [i32; 2] = if side == Color::White { [14, 16] } else { [-14, -16] };
    let promo_rank = if side == Color::White { 10 } else { 1 };

    for &d in &cap_dirs {
        let ts = (s as i32 + d) as usize;
        if is_off_board(ts) || !can_enter_square(piece, ts, side) { continue; }
        let target = board.pieces[ts];
        if target != Piece::Empty && target.color() != Some(side) {
            if rank_brd(ts) == promo_rank {
                let prom = piece.promoted();
                moves.push(make_move(s, ts, target, true, prom));
            } else {
                moves.push(make_capture(s, ts, target));
            }
        }
    }

    // Promotion without capture (still a tactical move)
    let forward = if side == Color::White { 15i32 } else { -15i32 };
    let fwd = (s as i32 + forward) as usize;
    if !is_off_board(fwd) && can_enter_square(piece, fwd, side) && board.pieces[fwd] == Piece::Empty && rank_brd(fwd) == promo_rank {
        let prom = piece.promoted();
        moves.push(make_move(s, fwd, Piece::Empty, true, prom));
    }
}

// ─── Piece moves ─────────────────────────────────────────────────

fn gen_piece_moves(board: &Board, s: usize, piece: Piece, side: Color, moves: &mut Vec<Move>) {
    match piece {
        Piece::WRook | Piece::BRook => gen_sliding(board, s, piece, side, &ROOK_DIRS, moves),
        Piece::WKnight | Piece::BKnight => gen_leaper(board, s, piece, side, &KNIGHT_DIRS, moves),
        Piece::WCatapult | Piece::BCatapult => gen_picket(board, s, piece, side, moves),
        Piece::WGiraffe | Piece::BGiraffe => gen_giraffe(board, s, piece, side, moves),
        Piece::WMinister | Piece::BMinister => gen_leaper(board, s, piece, side, &MINISTER_DIRS, moves),
        Piece::WKing | Piece::BKing |
        Piece::WPrince | Piece::BPrince |
        Piece::WAdKing | Piece::BAdKing => gen_leaper(board, s, piece, side, &KING_DIRS, moves),
        Piece::WAdvisor | Piece::BAdvisor => gen_leaper(board, s, piece, side, &ADVISOR_DIRS, moves),
        Piece::WElephant | Piece::BElephant => gen_leaper(board, s, piece, side, &ELEPHANT_DIRS, moves),
        Piece::WCamel | Piece::BCamel => gen_leaper(board, s, piece, side, &CAMEL_DIRS, moves),
        Piece::WWarengine | Piece::BWarengine => gen_leaper(board, s, piece, side, &WARENGINE_DIRS, moves),
        _ => {}
    }
}

fn gen_piece_captures(board: &Board, s: usize, piece: Piece, side: Color, moves: &mut Vec<Move>) {
    match piece {
        Piece::WRook | Piece::BRook => gen_sliding_captures(board, s, piece, side, &ROOK_DIRS, moves),
        Piece::WKnight | Piece::BKnight => gen_leaper_captures(board, s, piece, side, &KNIGHT_DIRS, moves),
        Piece::WCatapult | Piece::BCatapult => gen_picket_captures(board, s, piece, side, moves),
        Piece::WGiraffe | Piece::BGiraffe => gen_giraffe_captures(board, s, piece, side, moves),
        Piece::WMinister | Piece::BMinister => gen_leaper_captures(board, s, piece, side, &MINISTER_DIRS, moves),
        Piece::WKing | Piece::BKing |
        Piece::WPrince | Piece::BPrince |
        Piece::WAdKing | Piece::BAdKing => gen_leaper_captures(board, s, piece, side, &KING_DIRS, moves),
        Piece::WAdvisor | Piece::BAdvisor => gen_leaper_captures(board, s, piece, side, &ADVISOR_DIRS, moves),
        Piece::WElephant | Piece::BElephant => gen_leaper_captures(board, s, piece, side, &ELEPHANT_DIRS, moves),
        Piece::WCamel | Piece::BCamel => gen_leaper_captures(board, s, piece, side, &CAMEL_DIRS, moves),
        Piece::WWarengine | Piece::BWarengine => gen_leaper_captures(board, s, piece, side, &WARENGINE_DIRS, moves),
        _ => {}
    }
}

// ─── Leaper (non-sliding) moves ──────────────────────────────────

fn gen_leaper(board: &Board, s: usize, piece: Piece, side: Color, dirs: &[i32], moves: &mut Vec<Move>) {
    for &d in dirs {
        let ts = (s as i32 + d) as usize;
        if is_off_board(ts) || !can_enter_square(piece, ts, side) { continue; }
        let target = board.pieces[ts];
        if target == Piece::Empty {
            moves.push(make_quiet(s, ts));
        } else if target.color() != Some(side) {
            moves.push(make_capture(s, ts, target));
        }
    }
}

fn gen_leaper_captures(board: &Board, s: usize, piece: Piece, side: Color, dirs: &[i32], moves: &mut Vec<Move>) {
    for &d in dirs {
        let ts = (s as i32 + d) as usize;
        if is_off_board(ts) || !can_enter_square(piece, ts, side) { continue; }
        let target = board.pieces[ts];
        if target != Piece::Empty && target.color() != Some(side) {
            moves.push(make_capture(s, ts, target));
        }
    }
}

// ─── Slider moves (Rook) ────────────────────────────────────────

fn gen_sliding(board: &Board, s: usize, piece: Piece, side: Color, dirs: &[i32], moves: &mut Vec<Move>) {
    for &d in dirs {
        let mut ts = (s as i32 + d) as usize;
        while !is_off_board(ts) {
            if !can_enter_square(piece, ts, side) {
                ts = (ts as i32 + d) as usize;
                continue;
            }
            let target = board.pieces[ts];
            if target == Piece::Empty {
                moves.push(make_quiet(s, ts));
            } else {
                if target.color() != Some(side) {
                    moves.push(make_capture(s, ts, target));
                }
                break;
            }
            ts = (ts as i32 + d) as usize;
        }
    }
}

fn gen_sliding_captures(board: &Board, s: usize, piece: Piece, side: Color, dirs: &[i32], moves: &mut Vec<Move>) {
    for &d in dirs {
        let mut ts = (s as i32 + d) as usize;
        while !is_off_board(ts) {
            if !can_enter_square(piece, ts, side) {
                ts = (ts as i32 + d) as usize;
                continue;
            }
            let target = board.pieces[ts];
            if target != Piece::Empty {
                if target.color() != Some(side) {
                    moves.push(make_capture(s, ts, target));
                }
                break;
            }
            ts = (ts as i32 + d) as usize;
        }
    }
}

// ─── Catapult/Picket (diagonal slider, min 2 squares) ────────────

fn gen_picket(board: &Board, s: usize, piece: Piece, side: Color, moves: &mut Vec<Move>) {
    for &d in &MINISTER_DIRS {
        let mut ts = (s as i32 + d) as usize;
        let mut dist = 1;
        while !is_off_board(ts) {
            if !can_enter_square(piece, ts, side) {
                ts = (ts as i32 + d) as usize;
                dist += 1;
                continue;
            }
            let target = board.pieces[ts];
            if target == Piece::Empty {
                if dist >= 2 {
                    moves.push(make_quiet(s, ts));
                }
            } else {
                if dist >= 2 && target.color() != Some(side) {
                    moves.push(make_capture(s, ts, target));
                }
                break;
            }
            ts = (ts as i32 + d) as usize;
            dist += 1;
        }
    }
}

fn gen_picket_captures(board: &Board, s: usize, piece: Piece, side: Color, moves: &mut Vec<Move>) {
    for &d in &MINISTER_DIRS {
        let mut ts = (s as i32 + d) as usize;
        let mut dist = 1;
        while !is_off_board(ts) {
            if !can_enter_square(piece, ts, side) {
                ts = (ts as i32 + d) as usize;
                dist += 1;
                continue;
            }
            let target = board.pieces[ts];
            if target != Piece::Empty {
                if dist >= 2 && target.color() != Some(side) {
                    moves.push(make_capture(s, ts, target));
                }
                break;
            }
            ts = (ts as i32 + d) as usize;
            dist += 1;
        }
    }
}

// ─── Giraffe (bent rider: 1 diagonal + slide orthogonal) ────────

fn gen_giraffe(board: &Board, s: usize, piece: Piece, side: Color, moves: &mut Vec<Move>) {
    giraffe_helper(board, s, piece, side, moves, false);
}

fn gen_giraffe_captures(board: &Board, s: usize, piece: Piece, side: Color, moves: &mut Vec<Move>) {
    giraffe_helper(board, s, piece, side, moves, true);
}

fn giraffe_helper(board: &Board, s: usize, piece: Piece, side: Color, moves: &mut Vec<Move>, captures_only: bool) {
    for i in 0..8 {
        let check1 = (s as i32 + GIRAFFE_CHECK1[i]) as usize;
        let check2 = (s as i32 + GIRAFFE_CHECK2[i]) as usize;
        let check3 = (s as i32 + GIRAFFE_CHECK3[i]) as usize;

        if is_off_board(check1) || is_off_board(check2) || is_off_board(check3) { continue; }

        // ALL 3 checkpoints must be empty to start sliding
        if board.pieces[check1] == Piece::Empty &&
           board.pieces[check2] == Piece::Empty &&
           board.pieces[check3] == Piece::Empty {
            
            let mut curr = (s as i32 + GIRAFFE_DIRS[i]) as usize; // Initial slide position
            
            while !is_off_board(curr) {
                if !can_enter_square(piece, curr, side) {
                    curr = (curr as i32 + GIRAFFE_SLIDE[i]) as usize;
                    continue;
                }
                
                let target = board.pieces[curr];
                
                if target == Piece::Empty {
                    if !captures_only {
                        moves.push(make_quiet(s, curr));
                    }
                } else {
                    if target.color() != Some(side) {
                        moves.push(make_capture(s, curr, target));
                    }
                    break;
                }
                curr = (curr as i32 + GIRAFFE_SLIDE[i]) as usize;
            }
        }
    }
}

// ─── Legal move filtering ────────────────────────────────────────

/// Filter pseudo-legal moves to only legal ones (doesn't leave king in check)
pub fn legal_moves(board: &mut Board) -> Vec<Move> {
    let pseudo = generate_moves(board);
    let mut legal = Vec::with_capacity(pseudo.len());
    let side = board.side;

    for mv in pseudo {
        board.make_move(mv);
        // After make_move, side has flipped. Check if our king is attacked.
        let king = board.king_sq(side);
        let in_check = match king {
            Some(ks) => is_attacked(board, ks, side.flip()),
            None => false,
        };
        board.unmake_move();
        if !in_check {
            legal.push(mv);
        }
    }
    legal
}

/// Generate legal captures only
pub fn legal_captures(board: &mut Board) -> Vec<Move> {
    let pseudo = generate_captures(board);
    let mut legal = Vec::with_capacity(pseudo.len());
    let side = board.side;

    for mv in pseudo {
        board.make_move(mv);
        let king = board.king_sq(side);
        let in_check = match king {
            Some(ks) => is_attacked(board, ks, side.flip()),
            None => false,
        };
        board.unmake_move();
        if !in_check {
            legal.push(mv);
        }
    }
    legal
}
