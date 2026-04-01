/// Tamerlane Chess Engine — Search
/// Iterative deepening + Alpha-Beta + PVS + Null Move Pruning + LMR + Quiescence
/// + Razoring + Countermove heuristic + History penalties.

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

// Cross-platform logging
#[cfg(target_arch = "wasm32")]
fn log_msg(s: &str) {
    crate::console_log(s);
}

#[cfg(not(target_arch = "wasm32"))]
fn log_msg(s: &str) {
    println!("{}", s);
}

// ─── LMR reduction table (precomputed) ──────────────────────────

/// Logarithmic LMR formula: reduction = max(1, ln(depth) * ln(moves_searched) / 2.25)
fn lmr_reduction(depth: i32, moves_searched: i32) -> i32 {
    if depth < 3 || moves_searched < 5 {
        return 0;
    }
    let d = (depth as f64).ln();
    let m = (moves_searched as f64).ln();
    let r = (d * m / 2.25) as i32;
    r.max(1).min(depth - 1)
}

// ─── TT Mate Score Adjustment ──────────────────────────────────────
// Mate scores must be adjusted for ply when stored/probed in TT.
// Without this, the engine sees phantom mates and sacrifices pieces.

#[inline]
fn score_to_tt(score: i32, ply: usize) -> i32 {
    if score > MATE_SCORE - 200 {
        score + ply as i32
    } else if score < -(MATE_SCORE - 200) {
        score - ply as i32
    } else {
        score
    }
}

#[inline]
fn score_from_tt(score: i32, ply: usize) -> i32 {
    if score > MATE_SCORE - 200 {
        score - ply as i32
    } else if score < -(MATE_SCORE - 200) {
        score + ply as i32
    } else {
        score
    }
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

/// Diagnostic stats to track which algorithms are firing
pub struct SearchStats {
    pub tt_hits: u64,
    pub tt_cutoffs: u64,
    pub null_move_tried: u64,
    pub null_move_cutoffs: u64,
    pub razor_prunes: u64,
    pub rfp_prunes: u64,
    pub futility_prunes: u64,
    pub lmp_prunes: u64,
    pub lmr_reductions: u64,
    pub lmr_re_searches: u64,
    pub qsearch_nodes: u64,
    pub qsearch_check_evasions: u64,
    pub repetition_draws: u64,
}

impl SearchStats {
    pub fn new() -> Self {
        SearchStats {
            tt_hits: 0, tt_cutoffs: 0,
            null_move_tried: 0, null_move_cutoffs: 0,
            razor_prunes: 0, rfp_prunes: 0,
            futility_prunes: 0, lmp_prunes: 0,
            lmr_reductions: 0, lmr_re_searches: 0,
            qsearch_nodes: 0, qsearch_check_evasions: 0,
            repetition_draws: 0,
        }
    }
}
// ─── Main search entry ──────────────────────────────────────────

struct GlobalSearchState {
    tt: TranspositionTable,
    history: HistoryTable,
    countermoves: CounterMoveTable,
}

static mut GLOBAL_STATE: Option<GlobalSearchState> = None;

pub fn search_best_move(board: &mut Board, time_limit_ms: u64, difficulty: i32) -> (Move, i32, i32, u64) {
    let mut info = SearchInfo::new(time_limit_ms);
    
    unsafe {
        if GLOBAL_STATE.is_none() {
            GLOBAL_STATE = Some(GlobalSearchState {
                tt: TranspositionTable::new(32),
                history: HistoryTable::new(),
                countermoves: CounterMoveTable::new(),
            });
        }
    }
    
    let state = unsafe { GLOBAL_STATE.as_mut().unwrap() };
    
    // Killers are cleared every move since they are ply-dependent
    let mut killers = Killers::new();
    let mut stats = SearchStats::new();

    let tt = &mut state.tt;
    let history = &mut state.history;
    let countermoves = &mut state.countermoves;

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
        
        // Only use aspiration window after we have a valid score from previous iteration
        if depth >= 5 && reached_depth > 0 && best_score.abs() < MATE_SCORE - 200 {
            alpha = best_score - 35;
            beta = best_score + 35;
        } else {
            alpha = -INF;
            beta = INF;
        }

        let mut score_dropped = false;
        let mut score;
        loop {
            score = alpha_beta(board, depth, alpha, beta, 0, &mut info, tt, &mut killers, history, countermoves, true, &mut stats);

            if info.stopped {
                break;
            }

            if score <= alpha {
                alpha = -INF;
                score_dropped = true; // We failed low, tactical crisis possible
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

        // Log each completed depth
        log_msg(&format!("  depth {} score {} nodes={}", depth, score, info.nodes));

        // If we found mate, no need to search deeper
        if score.abs() > MATE_SCORE - 100 {
            break;
        }

        // Dynamic time management: if we failed low and are resolving a crisis,
        // we can use up to 85% of our total time before aborting the next depth iteration.
        // Otherwise, we use the standard 60% safe soft limit.
        let elapsed = current_time_ms() - info.start_time;
        let soft_limit_pct = if score_dropped { 85 } else { 60 };
        if elapsed >= time_limit_ms * soft_limit_pct / 100 {
            break;
        }
    }

    // Log diagnostic stats
    log_msg(&format!("STATS: TT hits={} cutoffs={} | NMP tried={} cutoffs={} | Razor={} RFP={} | Futility={} LMP={} | LMR={} re-search={} | QS nodes={} chk_evasions={} | RepDraw={}",
        stats.tt_hits, stats.tt_cutoffs,
        stats.null_move_tried, stats.null_move_cutoffs,
        stats.razor_prunes, stats.rfp_prunes,
        stats.futility_prunes, stats.lmp_prunes,
        stats.lmr_reductions, stats.lmr_re_searches,
        stats.qsearch_nodes, stats.qsearch_check_evasions,
        stats.repetition_draws
    ));

    // ─── ROOT BLUNDER CHECK ─────────────────────────────────
    // Safety net: verify the best move doesn't put a piece on an attacked,
    // undefended square without adequate compensation (capture).
    if best_move != MOVE_NONE && difficulty > 0 {
        let from = mv_from(best_move);
        let to = mv_to(best_move);
        let piece = board.pieces[from];
        let captured = mv_captured(best_move);
        let piece_val = see_value(piece);
        let capt_val = if captured != Piece::Empty { see_value(captured) } else { 0 };

        // Only check non-pawn, non-king quiet moves or captures worth less
        if !piece.is_pawn() && !piece.is_king_type() && piece_val > 120 {
            let enemy_color = if piece.is_white() { Color::Black } else { Color::White };
            let friendly_color = if piece.is_white() { Color::White } else { Color::Black };

            // After moving, check if destination is attacked by enemy
            board.make_move(best_move);
            let dest_attacked = crate::attack::is_attacked(board, to, enemy_color);
            let dest_defended = crate::attack::is_attacked(board, to, friendly_color);
            board.unmake_move();

            if dest_attacked && !dest_defended && capt_val < piece_val / 2 {
                log_msg(&format!("BLUNDER CHECK: move puts {} (val={}) on attacked undefended sq! capt_val={}", 
                    piece.kind_index(), piece_val, capt_val));

                // Try to find the next best safe move from root
                let moves = legal_moves(board);
                let is_white_side = board.side == Color::White;
                let mut scored = score_moves(&moves, &board.pieces, best_move, &killers, &history, 0, MOVE_NONE, is_white_side);
                
                let mut fallback_move = MOVE_NONE;
                let mut fallback_score = -INF;
                let mut safe_moves_tried = 0;

                for i in 0..scored.len() {
                    let mv = pick_best(&mut scored, i);
                    if mv == best_move { continue; } // skip the blunder

                    let mv_from_sq = mv_from(mv);
                    let mv_to_sq = mv_to(mv);
                    let mv_piece = board.pieces[mv_from_sq];
                    let mv_capt = mv_captured(mv);
                    let mv_piece_val = see_value(mv_piece);
                    let mv_capt_val = if mv_capt != Piece::Empty { see_value(mv_capt) } else { 0 };

                    // Strict safety check on this alternative
                    let mut is_safe = true;
                    if !mv_piece.is_pawn() && !mv_piece.is_king_type() && mv_piece_val > 120 {
                        board.make_move(mv);
                        let atk = crate::attack::is_attacked(board, mv_to_sq, enemy_color);
                        board.unmake_move();

                        // If the destination is attacked, and we are not capturing something worth at least half our piece, reject it.
                        // We no longer care if it's defended, because trading a 300 val piece for a 100 val piece is bad even if defended.
                        if atk && mv_capt_val < mv_piece_val / 2 {
                            is_safe = false;
                        }
                    }

                    if is_safe {
                        // Temporarily bypass stopped strictly for this shallow safety check
                        let was_stopped = info.stopped;
                        info.stopped = false;

                        // Do a quick shallow search (depth 2) to get a rough score for this safe alternative
                        board.make_move(mv);
                        let s = -alpha_beta(board, 2, -INF, INF, 1, &mut info, tt, &mut killers, history, countermoves, true, &mut stats);
                        board.unmake_move();

                        info.stopped = was_stopped;

                        if s > fallback_score {
                            fallback_score = s;
                            fallback_move = mv;
                        }
                        
                        safe_moves_tried += 1;
                        // Check top 3 safe alternatives
                        if safe_moves_tried >= 3 {
                            break;
                        }
                    }
                }

                if fallback_move != MOVE_NONE {
                    log_msg(&format!("BLUNDER AVOIDED: switching to safe move, score={}", fallback_score));
                    best_move = fallback_move;
                    best_score = fallback_score;
                }
            }
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
    countermoves: &mut CounterMoveTable,
    is_pv: bool,
    stats: &mut SearchStats,
) -> i32 {
    info.check_time();
    if info.stopped { return 0; }

    // Draw by 50-move rule
    if board.half_moves >= 100 {
        return DRAW_SCORE;
    }

    // Repetition detection
    if ply > 0 {
        for i in (0..board.history.len()).rev() {
            if board.history[i].hash == board.hash {
                stats.repetition_draws += 1;
                return DRAW_SCORE;
            }
            if board.history[i].half_moves == 0 {
                break;
            }
        }
    }

    // Quiescence at leaf
    if depth <= 0 {
        return quiescence(board, alpha, beta, ply, info, stats);
    }

    info.nodes += 1;

    let is_root = ply == 0;
    let in_chk = in_check(board);
    
    let static_eval = if in_chk { 0 } else if is_root { crate::eval::evaluate_with_log(board) } else { evaluate(board) };

    // Check extension
    let effective_depth = if in_chk { depth + 1 } else { depth };

    // ─── TT probe ─────────────────────────────────────────
    let mut tt_move = MOVE_NONE;
    if let Some(entry) = tt.probe(board.hash) {
        tt_move = entry.best_move;
        stats.tt_hits += 1;
        if !is_pv && entry.depth as i32 >= effective_depth {
            let tt_score = score_from_tt(entry.score, ply);
            stats.tt_cutoffs += 1;
            match entry.flag {
                TTFlag::Exact => return tt_score,
                TTFlag::Alpha => { if tt_score <= alpha { return alpha; } }
                TTFlag::Beta => { if tt_score >= beta { return beta; } }
            }
        }
    }

    // ─── Razoring ──────────────────────────────────────────
    if !is_pv && !in_chk && effective_depth <= 3 && tt_move == MOVE_NONE {
        let razor_margin = 200 + 150 * effective_depth;
        if static_eval + razor_margin <= alpha {
            let qscore = quiescence(board, alpha, beta, ply, info, stats);
            if qscore <= alpha {
                stats.razor_prunes += 1;
                return alpha;
            }
        }
    }

    // ─── Reverse Futility Pruning (Static Null Move) ───────
    if !is_pv && !in_chk && effective_depth < 4 {
        let margin = 120 * effective_depth;
        if static_eval - margin >= beta {
            stats.rfp_prunes += 1;
            return static_eval;
        }
    }

    // ─── Null Move Pruning ────────────────────────────────
    if !is_pv && !in_chk && depth >= 3 && has_non_pawn_material(board) {
        stats.null_move_tried += 1;
        let saved_hash = board.hash;
        board.hash ^= zobrist_side_key();
        board.side = board.side.flip();
        board.ply += 1;
        let r = if depth >= 6 { 3 } else { 2 };
        let null_score = -alpha_beta(board, depth - 1 - r, -beta, -beta + 1, ply + 1, info, tt, killers, history, countermoves, false, stats);
        board.side = board.side.flip();
        board.ply -= 1;
        board.hash = saved_hash;
        if info.stopped { return 0; }
        if null_score >= beta {
            stats.null_move_cutoffs += 1;
            return beta;
        }
    }

    // ─── Internal Iterative Deepening (IID) ───────────────
    if tt_move == MOVE_NONE && is_pv && effective_depth >= 4 {
        let _ = alpha_beta(board, effective_depth - 2, alpha, beta, ply, info, tt, killers, history, countermoves, true, stats);
        if let Some(entry) = tt.probe(board.hash) {
            tt_move = entry.best_move;
        }
    }

    // ─── Get countermove for current position ─────────────
    let counter_mv = if ply > 0 && !board.history.is_empty() {
        let last_undo = board.history.last().unwrap();
        let last_mv = last_undo.mv;
        let last_to = mv_to(last_mv);
        let last_piece = board.pieces[last_to];
        countermoves.probe(last_piece, last_to)
    } else {
        MOVE_NONE
    };

    // ─── Generate and order moves ─────────────────────────
    let moves = legal_moves(board);

    if moves.is_empty() {
        if in_chk {
            return -(MATE_SCORE - ply as i32); // Checkmate
        } else {
            return DRAW_SCORE; // Stalemate
        }
    }

    // Singular Extension: If exactly one legal move exists (forced reply), extend depth
    let singular_ext = if moves.len() == 1 && !is_root { 1 } else { 0 };
    let new_depth = effective_depth + singular_ext;

    let is_white_side = board.side == Color::White;
    let mut scored = score_moves(&moves, &board.pieces, tt_move, killers, history, ply, counter_mv, is_white_side);
    let mut best_move_local = MOVE_NONE;
    let mut best_score = -INF;
    let mut moves_searched = 0;
    let mut quiet_moves_tried: Vec<Move> = Vec::new();

    // Save previous move info for countermove updates
    let prev_piece_for_cm;
    let prev_to_for_cm;
    if ply > 0 && !board.history.is_empty() {
        let last_undo = board.history.last().unwrap();
        let last_mv = last_undo.mv;
        prev_to_for_cm = mv_to(last_mv);
        prev_piece_for_cm = board.pieces[prev_to_for_cm];
    } else {
        prev_piece_for_cm = Piece::Empty;
        prev_to_for_cm = 0;
    }

    for i in 0..scored.len() {
        let mv = pick_best(&mut scored, i);
        let is_capture = mv_captured(mv) != Piece::Empty;
        let is_prom = mv_is_promotion(mv);
        let is_quiet = !is_capture && !is_prom;

        // ─── Futility Pruning ─────────────────────────────────
        if !is_pv && !in_chk && !is_capture && !is_prom && effective_depth <= 2 && moves_searched > 0 && best_score > -MATE_SCORE {
            let futil_margin = 150 * effective_depth;
            if static_eval + futil_margin <= alpha {
                stats.futility_prunes += 1;
                continue;
            }
        }

        // ─── Late Move Pruning ─────────────────────────────────
        if !is_pv && !in_chk && is_quiet && effective_depth <= 2 && moves_searched >= 12 + 5 * effective_depth as usize {
            stats.lmp_prunes += 1;
            continue;
        }

        if is_quiet {
            quiet_moves_tried.push(mv);
        }

        board.make_move(mv);

        let score;
        if moves_searched == 0 {
            score = -alpha_beta(board, new_depth - 1, -beta, -alpha, ply + 1, info, tt, killers, history, countermoves, is_pv, stats);
        } else {
            // ─── Late Move Reductions ─────────────────
            let mut reduction = 0;
            if is_quiet && !in_chk {
                reduction = lmr_reduction(depth, moves_searched as i32);

                if killers.is_killer(ply, mv).is_some() || mv == counter_mv {
                    reduction = (reduction - 1).max(0);
                }
                let h_score = history.score(mv_from(mv), mv_to(mv), is_white_side);
                if h_score < -100 {
                    reduction += 1;
                } else if h_score > 200 {
                    reduction = (reduction - 1).max(0);
                }
                reduction = reduction.min(new_depth - 2).max(0);
                if reduction > 0 {
                    stats.lmr_reductions += 1;
                }
            }

            let mut s = -alpha_beta(board, new_depth - 1 - reduction, -alpha - 1, -alpha, ply + 1, info, tt, killers, history, countermoves, false, stats);

            if s > alpha && reduction > 0 {
                stats.lmr_re_searches += 1;
                s = -alpha_beta(board, new_depth - 1, -alpha - 1, -alpha, ply + 1, info, tt, killers, history, countermoves, false, stats);
            }

            if s > alpha && s < beta {
                s = -alpha_beta(board, new_depth - 1, -beta, -alpha, ply + 1, info, tt, killers, history, countermoves, true, stats);
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
                    if is_quiet {
                        killers.add(ply, mv);
                        history.add(mv_from(mv), mv_to(mv), depth, is_white_side);
                        
                        countermoves.store(prev_piece_for_cm, prev_to_for_cm, mv);
                        
                        for &qm in &quiet_moves_tried {
                            if qm != mv {
                                history.penalize(mv_from(qm), mv_to(qm), depth, is_white_side);
                            }
                        }
                    }
                    tt.store(board.hash, depth as i8, score_to_tt(beta, ply), TTFlag::Beta, mv);
                    return beta;
                }
            }
        }

        moves_searched += 1;
    }

    // Store in TT
    let flag = if best_score <= alpha { TTFlag::Alpha } else { TTFlag::Exact };
    tt.store(board.hash, depth as i8, score_to_tt(best_score, ply), flag, best_move_local);

    best_score
}

// ─── Quiescence Search ──────────────────────────────────────────

fn quiescence(board: &mut Board, mut alpha: i32, beta: i32, ply: usize, info: &mut SearchInfo, stats: &mut SearchStats) -> i32 {
    info.check_time();
    if info.stopped { return 0; }
    info.nodes += 1;
    stats.qsearch_nodes += 1;

    let in_chk = in_check(board);

    // If in check, search ALL legal moves but LIMIT depth to prevent explosion
    // Only allow 3 extra plies of check evasions beyond ply 64 baseline
    if in_chk {
        stats.qsearch_check_evasions += 1;
        let moves = legal_moves(board);
        if moves.is_empty() {
            return -(MATE_SCORE - ply as i32); // Checkmate!
        }

        // Hard limit on check evasion depth — this was causing 1700+ evasions
        if ply > 20 {
            return evaluate(board);
        }

        for &mv in &moves {
            board.make_move(mv);
            let score = -quiescence(board, -beta, -alpha, ply + 1, info, stats);
            board.unmake_move();

            if info.stopped { return 0; }
            if score >= beta { return beta; }
            if score > alpha { alpha = score; }
        }
        return alpha;
    }

    // Not in check: normal quiescence with stand-pat
    let stand_pat = evaluate(board);

    if stand_pat >= beta {
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Delta pruning: tightened from 600 to 500
    if stand_pat + 500 < alpha {
        return alpha;
    }

    // Max Q-search depth
    if ply > 20 {
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

        // Smart SEE pruning: skip only VERY bad captures
        // Only prune if attacker is 300+ more valuable than victim AND square is defended
        let captured = mv_captured(mv);
        let from = mv_from(mv);
        let to = mv_to(mv);
        let attacker = board.pieces[from];
        if see_value(attacker) > see_value(captured) + 300 && !mv_is_promotion(mv) {
            let enemy_color = if attacker.is_white() { Color::Black } else { Color::White };
            if crate::attack::is_attacked(board, to, enemy_color) {
                continue; // Skip clearly losing captures of defended pieces
            }
        }

        board.make_move(mv);
        let score = -quiescence(board, -beta, -alpha, ply + 1, info, stats);
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

fn maybe_weaken(_board: &mut Board, best_move: Move) -> Move {
    // Medium difficulty is already limited to depth 4.
    // We no longer inject random moves — that caused the engine to give away pieces.
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
