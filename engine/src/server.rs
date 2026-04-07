#[cfg(not(target_arch = "wasm32"))]
use futures_util::{sink::SinkExt, stream::StreamExt};
#[cfg(not(target_arch = "wasm32"))]
use tokio::net::TcpListener;
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::accept_async;
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::tungstenite::Message;
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use crate::board::Board;
#[cfg(not(target_arch = "wasm32"))]
use crate::search::{search_best_move_parallel, move_to_string};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Deserialize)]
struct ClientMessage {
    #[serde(rename = "type")]
    msg_type: String,
    fen: Option<String>,
    time: Option<u64>,
    level: Option<i32>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize)]
struct ServerResponse {
    #[serde(rename = "type")]
    msg_type: String,
    score: i32,
    nodes: u64,
    depth: i32,
    threads: usize,
    #[serde(rename = "bestMove")]
    best_move: String,
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn run_server() {
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    // Use all available threads for maximum power
    let num_threads = num_cpus;
    
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    println!("=======================================================");
    println!("🚀 Tamerlane Engine Native Server Started (Lazy SMP)");
    println!("🔌 Listening on {}", addr);
    println!("🧵 Using {} search threads (out of {} available)", num_threads, num_cpus);
    println!("⚠️  Make sure 'Use Local Native Engine' is ON in UI.");
    println!("=======================================================");

    while let Ok((stream, _)) = listener.accept().await {
        let threads = num_threads;
        tokio::spawn(handle_connection(stream, threads));
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn handle_connection(stream: tokio::net::TcpStream, num_threads: usize) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            println!("Error during the websocket handshake occurred: {}", e);
            return;
        }
    };
    
    let (mut write, mut read) = ws_stream.split();
    println!("✅ React Interface connected via WebSocket.");

    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => break,
        };

        if msg.is_text() {
            let msg_text = msg.to_text().unwrap();
            let parsed: Result<ClientMessage, _> = serde_json::from_str(msg_text);
            
            if let Ok(client_msg) = parsed {
                if client_msg.msg_type == "search" {
                    let mut board = Board::new();
                    if let Some(fen_str) = client_msg.fen {
                        board = Board::from_fen(&fen_str);
                    }
                    
                    let movetime = client_msg.time.unwrap_or(5000); // 5 saniye düşünme süresi
                    let difficulty = client_msg.level.unwrap_or(1);
                    let threads = num_threads;
                    
                    // Run the parallel search in a blocking thread to not block tokio
                    let result = tokio::task::spawn_blocking(move || {
                        search_best_move_parallel(&mut board, movetime, difficulty, threads)
                    }).await;
                    
                    if let Ok((best_move, score, depth, nodes)) = result {
                        let best_move_str = {
                            let dummy_board = Board::new();
                            move_to_string(best_move, &dummy_board)
                        };
                        
                        let response = ServerResponse {
                            msg_type: "bestmove".to_string(),
                            score,
                            nodes,
                            depth,
                            threads: num_threads,
                            best_move: best_move_str,
                        };
                        
                        let response_text = serde_json::to_string(&response).unwrap();
                        let _ = write.send(Message::Text(response_text)).await;
                    }
                }
            }
        }
    }
    println!("❌ React Interface disconnected.");
}
