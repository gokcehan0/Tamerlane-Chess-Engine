# ♟️ Tamerlane Chess Engine

![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)
![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=WebAssembly&logoColor=white)

A high-performance, strictly mathematical open-source chess engine exclusively designed for **Tamerlane (Timur) Chess**. It dominates the massive 11x10 board featuring Citadels and historical leaping pieces (Giraffes, Camels, War Engines).

This repository is built as a **Dual-Architecture Monorepo**. The core Rust engine handles all search and evaluation logic, exposed both as a WebAssembly module for browser play and as a native WebSocket server for full multi-threaded power. The frontend is a dynamic React web UI.

**🌐 Play instantly in your browser (WASM version):** [tamerlane-chess-engine.vercel.app](https://tamerlane-chess-engine.vercel.app)

![Tamerlane Chess Bot UI](screenshot/bot.png)

---

## 🧠 Core Architecture & Sub-Algorithms

Since Tamerlane Chess has 112 squares and 22 highly exotic piece types, conventional 64-square engine bitboard architectures are not applicable. This engine implements an array-based mailbox architecture combined with sophisticated, hand-crafted evaluation (HCE) algorithms and extremely aggressive search pruning.

### Search & Pruning Algorithms

*   **Lazy SMP (Symmetric Multi-Processing):** The Native Rust server automatically detects system CPU cores and spawns decoupled parallel search threads.
    *   **Lock-Free Architecture:** Threads do not lock each other. Instead, they share an `AtomicU64` Transposition Table (`SharedTT`) wrapped in a `OnceLock`.
    *   **Data Packing:** Depth, scores, flags, and best moves are bit-packed into a single 64-bit integer, allowing single-instruction atomic reads/writes (`Ordering::Relaxed`).
    *   **Throughput:** Achieves over **1.7 Million nodes/sec** on modern CPUs (16+ threads), yielding up to 17x speedups over the single-threaded WASM version.
*   **Principal Variation Search (PVS) with Iterative Deepening:** The backbone of the tree search. It assumes the first move searched is the best, searching subsequent moves with a zero-window (`alpha, alpha+1`) to rapidly prove they are worse, saving massive computational time.
*   **Null Move Pruning (NMP):** If the engine has a solid position, it temporarily passes its turn (a "null move"). If the resulting position is *still* too strong for the opponent to break via a shallow search, the branch is immediately pruned.
*   **Late Move Reductions (LMR):** Moves that are ordered poorly (e.g., non-captures searched late in the node) are assumed to be bad. The engine searches them at a reduced depth (`depth - 1 - reduction`). If the move surprisingly beats `alpha`, a full-depth re-search is triggered.
*   **Quiescence Search (QS) & Delta Pruning:** Operates at the leaves of the search tree. It continues calculating captures (ignoring quiet moves) until a "quiet" position is reached, mathematically terminating the **Horizon Effect**. Delta Pruning instantly aborts capture paths that cannot possibly raise the score above `alpha` based on piece values.
*   **Singular Extensions:** Identifies forced replies. If only one move is viable in a node, the search depth is artificially extended to calculate ultimate tactical certainty without hitting the depth limit.
*   **Root-Level Blunder Avoidance:** A final safety net. Before returning a move, the engine verifies using Static Exchange Evaluation (SEE) that it is not placing a high-value piece (e.g., Giraffe, General) onto an unprotected, heavily attacked square, defaulting to a fallback safe move if necessary.

### Positional Evaluation (Hand-Crafted)

The engine features a deeply strategic, coordinated evaluation function tailored specifically to Tamerlane mechanics:
*   **Overextension Penalty (Phalanx Enforcement):** Minor leaping pieces (Knights, Pickets) are heavily penalized for diving deep into enemy territory without Pawn support. This forces the engine into coordinated *Phalanx* attacks rather than chaotic, single-piece charges.
*   **Leaper-Based King Safety:** Algorithmic threat detection evaluating "distance arrays" specific to Tamerlane's unique pieces (Giraffes, Camels) closing in on the King's zone.
*   **Endgame Checkmate Nets:** Complete override of the evaluator during crushing advantages (>800cp). The engine stops centralizing, ignores material gain, and ruthlessly hunts the King to the edges to force a concrete Checkmate.

---

## 🚀 Installation & Setup

### Prerequisites

Before you begin, make sure the following tools are installed on your system:

| Tool | Required For | Download |
|---|---|---|
| **Node.js** (v18+) | Frontend (React UI) | [nodejs.org](https://nodejs.org/) |
| **npm** | Dependency management | Comes with Node.js |
| **Rust Toolchain** (1.70+) | Native engine (server & UCI) | [rustup.rs](https://rustup.rs/) |

> **Note:** If you only want to play using the browser WASM mode, you only need Node.js. The Rust toolchain is only required for the Native Server and UCI modes.

### Step 1: Clone the Repository

```bash
git clone https://github.com/gokcehan0/Tamerlane-Chess-Engine.git
cd Tamerlane-Chess-Engine
```

### Step 2: Install Frontend Dependencies

```bash
npm install
```

This installs all React, TypeScript, Vite, and WASM bridge dependencies.

### Step 3: Choose Your Mode

The engine supports three distinct execution modes. Pick the one that fits your goal.

---

#### Mode A — Browser Play (WASM)

The simplest way to play. The Rust engine is pre-compiled to WebAssembly and runs entirely inside the browser. No Rust toolchain needed.

*   **Performance:** Single-threaded, Depth 6-8.

```bash
npm run dev
```

Open `http://localhost:3000` in your browser. You can start playing immediately.

---

#### Mode B — Maximum Power (Native Lazy SMP + WebSocket)

For serious players who want to unleash their full CPU. The Rust engine runs natively as a local WebSocket server, bypassing all browser limitations and utilizing **100% of available CPU threads** via Lazy SMP.

*   **Performance:** Multi-threaded (all cores), Depth 10-16+. Plays at an aggressive Master level.

**Terminal 1 — Start the Native Engine Server:**

```bash
cd engine
cargo run --release -- --server
```

You will see output like:
```
🚀 Tamerlane Engine Native Server Started (Lazy SMP)
🔌 Listening on 127.0.0.1:8080
🧵 Using 16 search threads (out of 16 available)
```

**Terminal 2 — Start the Web UI:**

```bash
npm run dev
```

Open `http://localhost:3000` in your browser. In the game sidebar, check the **"Use Local Native Engine"** checkbox. The UI will instantly connect to your native engine. You can verify it is working by opening the browser developer console (`F12`) — you will see logs like:

```
[Native Engine] Depth: 11, Nodes: 1872560, Score: -4, Threads: 16
```

---

#### Mode C — UCI Protocol (Terminal / Arena / CuteChess)

For integration with standard chess GUI programs (Arena, CuteChess) or for developers who want to parse raw engine output over stdin/stdout.

*   **Performance:** Multi-threaded. Communicates via standard **Universal Chess Interface (UCI)**.

```bash
cd engine
cargo run --release -- --uci
```

You can now type UCI commands directly:

```
position fen <your_fen_string>
go movetime 5000
```

The engine will respond with `bestmove d8d7` (or equivalent).

To use with a GUI program, point it to the compiled binary at `engine/target/release/test_engine.exe` and pass `--uci` as a launch argument.

---

## 🛠️ Repository Architecture

```text
Tamerlane-Chess-Engine/
├── engine/                <-- Core Rust Engine (Dual Bin/Lib Target)
│   ├── src/
│   │   ├── search.rs      <-- PVS, Lazy SMP Thread Pools, Alpha-Beta
│   │   ├── eval.rs        <-- HCE, Tamerlane piece values, King safety
│   │   ├── tt.rs          <-- AtomicU64 Lock-free Shared Transposition Table
│   │   ├── server.rs      <-- WebSocket server for native UI bridging
│   │   └── uci.rs         <-- Standard UCI protocol implementation
│   └── Cargo.toml         <-- Rust dependencies (Tokio, Tungstenite, Serde)
│
├── src/                   <-- React / TypeScript Web UI
│   ├── components/        <-- Chessboard, Dashboard, Game State Hooks
│   └── core/              <-- WASM bridge, WebSocket client, Web Worker
│
└── package.json           <-- Node dependencies
```

## 🔮 Future Roadmap

*   **Staged Move Generation:** Generating captures first to improve early Alpha-Beta cutoffs before generating quiet moves, saving node calculations.
*   **Zobrist Pawn Hash Tables:** Specialized TT just for static pawn evaluation performance.
*   **NNUE Integration:** Upgrading the Hand-Crafted Evaluation arrays to a custom Neural Network trained exclusively on high-elo Tamerlane games.
