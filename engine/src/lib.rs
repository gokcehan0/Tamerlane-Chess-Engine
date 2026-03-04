/// Tamerlane Chess Engine — WASM Entry Point
/// Exposes `get_best_move(fen, time_ms, difficulty)` to JavaScript.

pub mod types;
pub mod board;
pub mod attack;
pub mod movegen;
pub mod eval;
pub mod tt;
pub mod ordering;
pub mod search;

use wasm_bindgen::prelude::*;
use crate::types::*;
use crate::board::Board;
use crate::search::search_best_move;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
}

/// Initialize the engine (call once on startup).
#[wasm_bindgen]
pub fn init_engine() {
    board::init_globals();
}

/// Get the best move for the given FEN position.
/// Returns "from_file,from_rank,to_file,to_rank" (1-based) or "none".
/// difficulty: 0 = medium, 1 = master
#[wasm_bindgen]
pub fn get_best_move(fen: &str, time_ms: u32, difficulty: i32) -> String {
    board::init_globals();
    
    // Generate true random seed for evaluation jitter
    let random_val = js_sys::Math::random();
    let seed = (random_val * 1000000000.0) as u64;
    crate::eval::set_game_seed(seed);

    let mut board = Board::from_fen(fen);
    
    let (mv, score, depth, nodes) = search_best_move(&mut board, time_ms as u64, difficulty);
    
    if mv == MOVE_NONE {
        return "none".into();
    }
    
    let from = mv_from(mv);
    let to = mv_to(mv);
    let ff = board::file_brd(from);
    let fr = board::rank_brd(from);
    let tf = board::file_brd(to);
    let tr = board::rank_brd(to);
    
    let is_prom = mv_is_promotion(mv);
    
    console_log(&format!("Engine: depth {} complete, nodes={}, score={}, move={}{}->{}{}{}",
        depth, nodes, score,
        (b'a' + (ff - 1) as u8) as char, fr,
        (b'a' + (tf - 1) as u8) as char, tr,
        if is_prom { " (promotion)" } else { "" }
    ));
    
    format!("{},{},{},{},{}", ff, fr, tf, tr, if is_prom { 1 } else { 0 })
}

/// Get legal moves for the given FEN. Returns a comma-separated list of "from_file,from_rank,to_file,to_rank" moves.
#[wasm_bindgen]
pub fn get_legal_moves(fen: &str) -> String {
    board::init_globals();
    let mut board = Board::from_fen(fen);
    let moves = movegen::legal_moves(&mut board);
    
    let mut result = String::with_capacity(moves.len() * 10);
    for (i, &mv) in moves.iter().enumerate() {
        if i > 0 { result.push(';'); }
        let from = mv_from(mv);
        let to = mv_to(mv);
        let ff = board::file_brd(from);
        let fr = board::rank_brd(from);
        let tf = board::file_brd(to);
        let tr = board::rank_brd(to);
        result.push_str(&format!("{},{},{},{}", ff, fr, tf, tr));
    }
    result
}

/// Check if the position is checkmate or stalemate.
/// Returns: "playing", "check", "checkmate_white", "checkmate_black", "stalemate"
#[wasm_bindgen]
pub fn get_game_status(fen: &str) -> String {
    board::init_globals();
    let mut board = Board::from_fen(fen);
    let moves = movegen::legal_moves(&mut board);
    let in_chk = attack::in_check(&board);
    
    if moves.is_empty() {
        if in_chk {
            // Current side is checkmated — opponent wins
            match board.side {
                crate::types::Color::White => return "checkmate_black".into(),
                crate::types::Color::Black => return "checkmate_white".into(),
            }
        } else {
            return "stalemate".into();
        }
    }
    
    if in_chk {
        "check".into()
    } else {
        "playing".into()
    }
}
