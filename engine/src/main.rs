use tamerlane_engine::board::Board;
use tamerlane_engine::search::search_best_move;
use tamerlane_engine::board::init_globals;
use std::env;

fn main() {
    init_globals();
    
    // Get FEN from args or use start pos
    let args: Vec<String> = env::args().collect();
    let fen = if args.len() > 1 {
        &args[1]
    } else {
        "f1d1i1i1d1f/kamzgsvzmak1/pxcbyqehtnr/92/92/92/92/PXCBYQEHTNR/KAMZGSVZMAK1/F1D1I1I1D1F w"
    };
    
    let time_ms = if args.len() > 2 {
        args[2].parse().unwrap_or(1000)
    } else {
        1000
    };
    
    let diff = if args.len() > 3 {
        args[3].parse().unwrap_or(1)
    } else {
        1
    };
    
    println!("Testing Tamerlane Engine Natively");
    println!("FEN: {}", fen);
    println!("Search Time: {}ms, Difficulty: {}", time_ms, diff);
    
    let mut board = Board::from_fen(fen);
    
    use std::time::Instant;
    let start = Instant::now();
    let (best_move, score, depth, nodes) = search_best_move(&mut board, time_ms, diff);
    let elapsed = start.elapsed();
    
    let mv_str = tamerlane_engine::search::move_to_string(best_move, &board);
    
    println!("Time taken: {:?}", elapsed);
    println!("Depth reached: {}", depth);
    println!("Nodes searched: {}", nodes);
    println!("Nodes/sec: {:.0}", (nodes as f64) / elapsed.as_secs_f64());
    println!("Best Move: {}", mv_str);
    println!("Score: {}", score);
}
