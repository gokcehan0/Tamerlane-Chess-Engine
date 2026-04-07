#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, BufRead, Write};
#[cfg(not(target_arch = "wasm32"))]
use crate::board::Board;
#[cfg(not(target_arch = "wasm32"))]
use crate::search::{search_best_move, move_to_string};
#[cfg(not(target_arch = "wasm32"))]
use crate::types::Color;

#[cfg(not(target_arch = "wasm32"))]
pub fn run_uci() {
    println!("id name TamerlaneEngine v3.0");
    println!("id author Gokce");
    println!("uciok");

    let mut board = Board::new();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let msg = line.unwrap_or_default();
        let tokens: Vec<&str> = msg.split_whitespace().collect();
        if tokens.is_empty() { continue; }

        match tokens[0] {
            "uci" => {
                println!("id name TamerlaneEngine v3.0");
                println!("id author Gokce");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                board = Board::new();
            }
            "position" => {
                if tokens.len() > 1 && tokens[1] == "fen" {
                    let fen = tokens[2..].join(" ");
                    board = Board::from_fen(&fen);
                } else if tokens.len() > 1 && tokens[1] == "startpos" {
                    board = Board::new();
                }
            }
            "go" => {
                // simple parse for movetime
                let mut movetime = 2000;
                let mut difficulty = 1;
                for i in 1..tokens.len() {
                    if tokens[i] == "movetime" && i + 1 < tokens.len() {
                        movetime = tokens[i+1].parse::<u64>().unwrap_or(2000);
                    }
                    if tokens[i] == "level" && i + 1 < tokens.len() {
                        difficulty = tokens[i+1].parse::<i32>().unwrap_or(1);
                    }
                }
                
                let (best_move, _, _, _) = search_best_move(&mut board, movetime, difficulty);
                let best_move_str = move_to_string(best_move, &board);
                println!("bestmove {}", best_move_str);
                let _ = io::stdout().flush();
            }
            "quit" => {
                break;
            }
            _ => {
                println!("Unknown command: {}", tokens[0]);
            }
        }
    }
}
