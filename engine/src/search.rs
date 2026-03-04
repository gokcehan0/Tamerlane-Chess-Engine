/// Tamerlane Chess Engine — Search
/// Iterative deepening + Alpha-Beta + PVS + Null Move Pruning + LMR + Quiescence.

use crate::types::*;
use crate::board::*;
use crate::movegen::*;
use crate::attack::in_check;
use crate::eval::evaluate;
use crate::tt::*;
use crate::ordering::*;

// ─── Time ───────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn current_time_ms() -> u64 {
    js_sys::Date::now() as u64
}

#[cfg(not(target_arch = "wasm32"))]
fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

// ─── Search state ────────────────────────────────────────────────

pub struct SearchInfo {
    pub nodes: u64,
    pub time_limit_ms: u64,
    pub start_time: u64,
    pub stopped: bool,
    pub best_move: Move,
    pub best_score: i32,
}

impl SearchInfo {
    pub fn new(time_limit_ms: u64) -> Self {
        SearchInfo {
            nodes: 0,
            time_limit_ms,
            start_time: current_time_ms(),
            stopped: false,
            best_move: MOVE_NONE,
            best_score: 0,
        }
    }

    fn check_time(&mut self) {
        if self.nodes & 4095 == 0 {
            let elapsed = current_time_ms() - self.start_time;
            if elapsed >= self.time_limit_ms {
                self.stopped = true;
            }
        }
    }
}

// ─── Main search entry ──────────────────────────────────────────

pub fn search_best_move(board: &mut Board, time_limit_ms: u64, difficulty: i32) -> (Move, i32, i32, u64) {
    let mut info = SearchInfo::new(time_limit_ms);
    let mut tt = TranspositionTable::new(128); // 128 MB TT for massive endgame depth
    let mut killers = Killers::new();
    let mut history = HistoryTable::new();

    // Determine max depth based on difficulty
    let max_depth = if difficulty <= 0 {
        4  // Medium
    } else {
        64 // Master (limited by time)
    };

    let mut best_move = MOVE_NONE;
    let mut best_score = -INF;

    let mut reached_depth = 0;

    // Aspiration Window logic
    let mut alpha = -INF;
    let mut beta = INF;

    // Iterative deepening
    for depth in 1..=max_depth {
        info.stopped = false;
        
        if depth >= 5 {
            alpha = best_score - 50;
            beta = best_score + 50;
        }

        let mut score;
        loop {
            score = alpha_beta(board, depth, alpha, beta, 0, &mut info, &mut tt, &mut killers, &mut history, true);

            if info.stopped {
                break;
            }

            if score <= alpha {
                alpha = -INF;
            } else if score >= beta {
                beta = INF;
            } else {
                break;
            }
        }

        if info.stopped && depth > 1 {
            break; // Use previous iteration's result
        }

        reached_depth = depth;

        if info.best_move != MOVE_NONE {
            best_move = info.best_move;
            best_score = score;
        }

        // If we found mate, no need to search deeper
        if score.abs() > MATE_SCORE - 100 {
            break;
        }

        // Check time — don't start new iteration if >60% of time used
        let elapsed = current_time_ms() - info.start_time;
        if elapsed >= time_limit_ms * 60 / 100 {
            break;
        }
    }

    // Medium difficulty: sometimes pick a suboptimal move
    if difficulty <= 0 {
        best_move = maybe_weaken(board, best_move);
    }

    (best_move, best_score, reached_depth, info.nodes)
}

// ─── Alpha-Beta with PVS ─────────────────────────────────────────

fn alpha_beta(
    board: &mut Board,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    ply: usize,
    info: &mut SearchInfo,
    tt: &mut TranspositionTable,
    killers: &mut Killers,
    history: &mut HistoryTable,
    is_pv: bool,
) -> i32 {
    info.check_time();
    if info.stopped { return 0; }

    // Draw by 50-move rule
    if board.half_moves >= 100 {
        return DRAW_SCORE;
    }

    // Quiescence at leaf
    if depth <= 0 {
        return quiescence(board, alpha, beta, ply, info);
    }

    info.nodes += 1;

    let is_root = ply == 0;
    let in_chk = in_check(board);
    
    let static_eval = if in_chk { 0 } else { evaluate(board) };

    // Check extension
    let effective_depth = if in_chk { depth + 1 } else { depth };

    // ─── TT probe ─────────────────────────────────────────
    let mut tt_move = MOVE_NONE;
    if let Some(entry) = tt.probe(board.hash) {
        tt_move = entry.best_move;
        if !is_pv && entry.depth as i32 >= effective_depth {
            match entry.flag {
                TTFlag::Exact => return entry.score,
                TTFlag::Alpha => { if entry.score <= alpha { return alpha; } }
                TTFlag::Beta => { if entry.score >= beta { return beta; } }
            }
        }
    }

    // ─── Reverse Futility Pruning (Static Null Move) ───────
    if !is_pv && !in_chk && effective_depth < 4 {
        let margin = 120 * effective_depth;
        if static_eval - margin >= beta {
            return static_eval;
        }
    }

    // ─── Null Move Pruning ────────────────────────────────
    if !is_pv && !in_chk && depth >= 3 && has_non_pawn_material(board) {
        // Make null move (just flip side)
        let saved_hash = board.hash;
        board.hash ^= zobrist_side_key();
        board.side = board.side.flip();
        board.ply += 1;

        let r = if depth >= 6 { 3 } else { 2 };
        let null_score = -alpha_beta(board, depth - 1 - r, -beta, -beta + 1, ply + 1, info, tt, killers, history, false);

        // Unmake null move
        board.side = board.side.flip();
        board.ply -= 1;
        board.hash = saved_hash;

        if info.stopped { return 0; }
        if null_score >= beta {
            return beta;
        }
    }

    // ─── Internal Iterative Deepening (IID) ───────────────
    if tt_move == MOVE_NONE && is_pv && effective_depth >= 4 {
        let _ = alpha_beta(board, effective_depth - 2, alpha, beta, ply, info, tt, killers, history, true);
        if let Some(entry) = tt.probe(board.hash) {
            tt_move = entry.best_move;
        }
    }

    // ─── Generate and order moves ─────────────────────────
    let moves = legal_moves(board);

    if moves.is_empty() {
        if in_chk {
            return -(MATE_SCORE - ply as i32); // Checkmate
        } else {
            return DRAW_SCORE; // Stalemate
        }
    }

    let mut scored = score_moves(&moves, &board.pieces, tt_move, killers, history, ply);
    let mut best_move_local = MOVE_NONE;
    let mut best_score = -INF;
    let mut moves_searched = 0;

    for i in 0..scored.len() {
        let mv = pick_best(&mut scored, i);
        let is_capture = mv_captured(mv) != Piece::Empty;
        let is_prom = mv_is_promotion(mv);

        // ─── Futility Pruning ─────────────────────────────────
        if !is_pv && !in_chk && !is_capture && !is_prom && effective_depth <= 3 && moves_searched > 0 && best_score > -MATE_SCORE {
            let futil_margin = 200 * effective_depth;
            if static_eval + futil_margin <= alpha {
                continue;
            }
        }

        board.make_move(mv);

        let score;
        if moves_searched == 0 {
            // Full window search for first move
            score = -alpha_beta(board, effective_depth - 1, -beta, -alpha, ply + 1, info, tt, killers, history, is_pv);
        } else {
            // ─── Late Move Reductions ─────────────────
            let mut reduction = 0;
            if depth >= 3 && moves_searched >= 4 && !in_chk
                && mv_captured(mv) == Piece::Empty && !mv_is_promotion(mv) {
                reduction = 1;
                if moves_searched >= 8 { reduction = 2; }
                if depth >= 6 && moves_searched >= 12 { reduction = 3; }
            }

            // Null-window search with reduction
            let mut s = -alpha_beta(board, effective_depth - 1 - reduction, -alpha - 1, -alpha, ply + 1, info, tt, killers, history, false);

            // Re-search without reduction if improved alpha
            if s > alpha && reduction > 0 {
                s = -alpha_beta(board, effective_depth - 1, -alpha - 1, -alpha, ply + 1, info, tt, killers, history, false);
            }

            // Full window re-search if within PV window
            if s > alpha && s < beta {
                s = -alpha_beta(board, effective_depth - 1, -beta, -alpha, ply + 1, info, tt, killers, history, true);
            }

            score = s;
        }

        board.unmake_move();

        if info.stopped { return 0; }

        if score > best_score {
            best_score = score;
            best_move_local = mv;

            if score > alpha {
                alpha = score;

                if is_root {
                    info.best_move = mv;
                    info.best_score = score;
                }

                if score >= beta {
                    // Beta cutoff
                    if mv_captured(mv) == Piece::Empty {
                        killers.add(ply, mv);
                        history.add(mv_from(mv), mv_to(mv), depth);
                    }
                    tt.store(board.hash, depth as i8, beta, TTFlag::Beta, mv);
                    return beta;
                }
            }
        }

        moves_searched += 1;
    }

    // Store in TT
    let flag = if best_score <= alpha { TTFlag::Alpha } else { TTFlag::Exact };
    tt.store(board.hash, depth as i8, best_score, flag, best_move_local);

    best_score
}

// ─── Quiescence Search ──────────────────────────────────────────

fn quiescence(board: &mut Board, mut alpha: i32, beta: i32, ply: usize, info: &mut SearchInfo) -> i32 {
    info.check_time();
    if info.stopped { return 0; }
    info.nodes += 1;

    let stand_pat = evaluate(board);

    if stand_pat >= beta {
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Delta pruning
    if stand_pat + 600 < alpha {
        return alpha;
    }

    // Max quiescence depth
    if ply > 64 {
        return stand_pat;
    }

    let captures = legal_captures(board);

    let mut scored: Vec<(Move, i32)> = captures.iter().map(|&mv| {
        let captured = mv_captured(mv);
        let from = mv_from(mv);
        let attacker = board.pieces[from];
        (mv, score_capture(captured, attacker))
    }).collect();

    for i in 0..scored.len() {
        let mv = pick_best(&mut scored, i);
        board.make_move(mv);
        let score = -quiescence(board, -beta, -alpha, ply + 1, info);
        board.unmake_move();

        if info.stopped { return 0; }

        if score >= beta { return beta; }
        if score > alpha { alpha = score; }
    }

    alpha
}

// ─── Helpers ─────────────────────────────────────────────────────

fn has_non_pawn_material(board: &Board) -> bool {
    let side = board.side;
    for rank in 1..=10 {
        for file in 1..=11 {
            let s = sq(file, rank);
            let p = board.pieces[s];
            if p == Piece::Empty { continue; }
            if p.color() != Some(side) { continue; }
            if !p.is_pawn() && !p.is_king_type() {
                return true;
            }
        }
    }
    false
}

fn maybe_weaken(board: &mut Board, best_move: Move) -> Move {
    // Medium difficulty: 20% chance random legal move
    let roll = current_time_ms() % 100;
    if roll < 20 {
        let moves = legal_moves(board);
        if !moves.is_empty() {
            let idx = (current_time_ms() as usize) % moves.len();
            return moves[idx];
        }
    }
    best_move
}

pub fn move_to_string(mv: Move, _board: &Board) -> String {
    if mv == MOVE_NONE { return "none".into(); }
    let from = mv_from(mv);
    let to = mv_to(mv);
    let ff = file_brd(from);
    let fr = rank_brd(from);
    let tf = file_brd(to);
    let tr = rank_brd(to);
    format!("{}{}{}{}", 
        (b'a' + (ff - 1) as u8) as char, fr,
        (b'a' + (tf - 1) as u8) as char, tr)
}
