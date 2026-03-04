/// Tamerlane Chess Engine — Attack detection
/// Determines if a square is attacked by a given side.

use crate::types::*;
use crate::board::*;

/// Is square `s` attacked by `attacker` side?
pub fn is_attacked(board: &Board, s: usize, attacker: Color) -> bool {
    // 1. Pawn attacks (diagonal captures)
    if attacker == Color::White {
        // White pawns move UP (rank increases), so a white pawn at s-14 or s-16 attacks s
        let s1 = (s as i32 - 14) as usize;
        let s2 = (s as i32 - 16) as usize;
        if !is_off_board(s1) && is_pawn_of_color(board.pieces[s1], Color::White) { return true; }
        if !is_off_board(s2) && is_pawn_of_color(board.pieces[s2], Color::White) { return true; }
    } else {
        // Black pawns move DOWN, so a black pawn at s+14 or s+16 attacks s
        let s1 = (s as i32 + 14) as usize;
        let s2 = (s as i32 + 16) as usize;
        if !is_off_board(s1) && is_pawn_of_color(board.pieces[s1], Color::Black) { return true; }
        if !is_off_board(s2) && is_pawn_of_color(board.pieces[s2], Color::Black) { return true; }
    }

    // 2. Knight
    if check_leaper(board, s, attacker, &KNIGHT_DIRS, &|p| matches!(p, Piece::WKnight | Piece::BKnight)) { return true; }

    // 3. King / Prince / AdKing
    if check_leaper(board, s, attacker, &KING_DIRS, &|p| p.is_king_type()) { return true; }

    // 4. Minister (Ferz: 1 diagonal)
    if check_leaper(board, s, attacker, &MINISTER_DIRS, &|p| matches!(p, Piece::WMinister | Piece::BMinister)) { return true; }

    // 5. Advisor (Wazir: 1 orthogonal)
    if check_leaper(board, s, attacker, &ADVISOR_DIRS, &|p| matches!(p, Piece::WAdvisor | Piece::BAdvisor)) { return true; }

    // 6. Camel (3,1 leaper)
    if check_leaper(board, s, attacker, &CAMEL_DIRS, &|p| matches!(p, Piece::WCamel | Piece::BCamel)) { return true; }

    // 7. Warengine (Dabbaba: 2 orthogonal jump)
    if check_leaper(board, s, attacker, &WARENGINE_DIRS, &|p| matches!(p, Piece::WWarengine | Piece::BWarengine)) { return true; }

    // 8. Elephant (Alfil: 1 or 2 diagonal jump) — uses elephant directions (both 1-step and 2-step)
    if check_leaper(board, s, attacker, &ELEPHANT_DIRS, &|p| matches!(p, Piece::WElephant | Piece::BElephant)) { return true; }

    // 9. Rook (orthogonal slider)
    if check_slider(board, s, attacker, &ROOK_DIRS, &|p| matches!(p, Piece::WRook | Piece::BRook)) { return true; }

    // 10. Catapult/Picket (diagonal slider, minimum 2 squares)
    if check_picket(board, s, attacker) { return true; }

    // 11. Giraffe (special bent rider)
    if check_giraffe(board, s, attacker) { return true; }

    false
}

fn is_pawn_of_color(p: Piece, color: Color) -> bool {
    if p == Piece::Empty { return false; }
    if !p.is_pawn() { return false; }
    match color {
        Color::White => p.is_white(),
        Color::Black => p.is_black(),
    }
}

fn check_leaper(board: &Board, s: usize, attacker: Color, dirs: &[i32], pred: &dyn Fn(Piece) -> bool) -> bool {
    for &d in dirs {
        let ts = (s as i32 + d) as usize;
        if is_off_board(ts) { continue; }
        let p = board.pieces[ts];
        if p != Piece::Empty && p.color() == Some(attacker) && pred(p) {
            return true;
        }
    }
    false
}

fn check_slider(board: &Board, s: usize, attacker: Color, dirs: &[i32], pred: &dyn Fn(Piece) -> bool) -> bool {
    for &d in dirs {
        let mut ts = (s as i32 + d) as usize;
        while !is_off_board(ts) {
            let p = board.pieces[ts];
            if p != Piece::Empty {
                if p.color() == Some(attacker) && pred(p) { return true; }
                break;
            }
            ts = (ts as i32 + d) as usize;
        }
    }
    false
}

fn check_picket(board: &Board, s: usize, attacker: Color) -> bool {
    // Catapult: diagonal slider, but must be at least 2 squares away
    for &d in &MINISTER_DIRS {
        let mut ts = (s as i32 + d) as usize;
        let mut dist = 1;
        while !is_off_board(ts) {
            let p = board.pieces[ts];
            if p != Piece::Empty {
                if dist >= 2 {
                    if p.color() == Some(attacker) && matches!(p, Piece::WCatapult | Piece::BCatapult) {
                        return true;
                    }
                }
                break;
            }
            ts = (ts as i32 + d) as usize;
            dist += 1;
        }
    }
    false
}

fn check_giraffe(board: &Board, s: usize, attacker: Color) -> bool {
    // Check all enemy giraffes and see if they can attack this square
    let giraffe_type = if attacker == Color::White { Piece::WGiraffe } else { Piece::BGiraffe };
    for r in 1..=10 {
        for f in 1..=11 {
            let gs = sq(f, r);
            if board.pieces[gs] == giraffe_type {
                if can_giraffe_reach(board, gs, s) { return true; }
            }
        }
    }
    false
}

fn can_giraffe_reach(board: &Board, from: usize, to: usize) -> bool {
    // Giraffe: 1 diagonal step + slide orthogonally, with 3 checkpoint squares
    let diag_ortho: [(i32, [i32; 2]); 4] = [
        (-16, [-15, -1]),
        (-14, [-15, 1]),
        (14, [15, -1]),
        (16, [15, 1]),
    ];
    for &(diag, orthos) in &diag_ortho {
        let dsq = (from as i32 + diag) as usize;
        if is_off_board(dsq) { continue; }
        if board.pieces[dsq] != Piece::Empty { continue; } // Cannot jump over first square
        if dsq == to { continue; } // Too close
        for &ortho in &orthos {
            let s1 = (dsq as i32 + ortho) as usize;
            if is_off_board(s1) { continue; }
            if s1 == to { continue; } // Too close (min range 3)
            if board.pieces[s1] != Piece::Empty { continue; }
            // Slide from s1 along ortho direction
            let mut curr = (s1 as i32 + ortho) as usize;
            while !is_off_board(curr) {
                if curr == to { return true; }
                if board.pieces[curr] != Piece::Empty { break; }
                curr = (curr as i32 + ortho) as usize;
            }
        }
    }
    false
}

/// Is the side-to-move's king in check?
pub fn in_check(board: &Board) -> bool {
    let ksq = board.king_sq(board.side);
    match ksq {
        Some(s) => is_attacked(board, s, board.side.flip()),
        None => false,
    }
}
