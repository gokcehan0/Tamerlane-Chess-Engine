#![cfg(not(target_arch = "wasm32"))]

use std::env;
use tamerlane_engine::board::init_globals;

#[tokio::main]
async fn main() {
    init_globals();
    
    let args: Vec<String> = env::args().collect();
    let mut mode = "cli";
    
    for arg in &args {
        if arg == "--uci" { mode = "uci"; }
        if arg == "--server" { mode = "server"; }
    }
    
    match mode {
        "uci" => {
            tamerlane_engine::uci::run_uci();
        }
        "server" => {
            tamerlane_engine::server::run_server().await;
        }
        _ => {
            println!("=======================================================");
            println!("🏆 Tamerlane Chess Engine v3.0");
            println!("=======================================================");
            println!("This executable supports two powerful modes:\n");
            println!("  cargo run --release -- --uci");
            println!("      Run in Universal Chess Interface mode.");
            println!("      Ideal for terminal play or connecting to GUIs like CuteChess/Arena.\n");
            println!("  cargo run --release -- --server");
            println!("      Run as a Local WebSocket Server (ws://127.0.0.1:8080).");
            println!("      Ideal for connecting the Web UI directly to your Native PC CPU power.");
            println!("=======================================================");
        }
    }
}
