import { Board, fileRankToSquare } from './Board';

export class TamerlaneEngine {
  private worker: Worker;
  private isReady: boolean = false;
  private resolveReady: ((value: unknown) => void) | null = null;
  private resolveSearch: ((value: any) => void) | null = null;
  public useWebSocket: boolean = false;
  private ws: WebSocket | null = null;

  constructor() {
    this.worker = new Worker(new URL('./engine-worker.ts', import.meta.url), { type: 'module' });
    
    this.worker.onmessage = (e) => {
      const { type, result } = e.data;
      if (type === 'ready') {
        this.isReady = true;
        if (this.resolveReady) {
          this.resolveReady(true);
          this.resolveReady = null;
        }
      } else if (type === 'best_move') {
        if (this.resolveSearch && !this.useWebSocket) {
          this.resolveSearch(this.parseMove(result));
          this.resolveSearch = null;
        }
      }
    };
    
    this.worker.postMessage({ type: 'init' });
  }

  public async waitReady() {
    if (this.isReady) return;
    return new Promise(resolve => {
      this.resolveReady = resolve;
    });
  }

  private connectWebSocket(): Promise<void> {
    return new Promise((resolve) => {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            resolve();
            return;
        }
        
        console.log("Connecting to Local Native Engine via WebSocket...");
        this.ws = new WebSocket('ws://127.0.0.1:8080');
        
        this.ws.onopen = () => {
            console.log("Connected to Local Native Engine!");
            resolve();
        };
        
        this.ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                if (data.type === 'bestmove') {
                    console.log(`[Native Engine] Depth: ${data.depth}, Nodes: ${data.nodes}, Score: ${data.score}, Threads: ${data.threads || '?'}`);
                    if (this.resolveSearch && this.useWebSocket) {
                        this.resolveSearch(this.parseMoveStr(data.bestMove));
                        this.resolveSearch = null;
                    }
                }
            } catch (e) {
                console.error("WebSocket message parsing error:", e);
            }
        };
        
        this.ws.onerror = (e) => {
            console.error("Local Native Engine connection failed. Is the server running? Run 'cargo run --release -- --server' inside the engine directory.", e);
            // Fallback to WASM
            this.useWebSocket = false;
            resolve();
        };
    });
  }

  public async getBestMove(board: Board, timeMs = 2000, difficulty = 1): Promise<any> {
    if (this.useWebSocket) {
        await this.connectWebSocket();
        if (this.useWebSocket && this.ws?.readyState === WebSocket.OPEN) {
            return new Promise((resolve) => {
                this.resolveSearch = resolve;
                this.ws!.send(JSON.stringify({
                    type: "search",
                    fen: board.generateFEN(),
                    time: this.useWebSocket ? 5000 : timeMs,
                    level: difficulty
                }));
            });
        }
    }
    
    // Fallback to WASM
    await this.waitReady();
    return new Promise((resolve) => {
      this.resolveSearch = resolve;
      this.worker.postMessage({
        type: 'search',
        fen: board.generateFEN(),
        time_ms: timeMs,
        difficulty
      });
    });
  }

  // Parses comma-separated from wasm format: "ff,fr,tf,tr,is_promo"
  private parseMove(resultStr: string) {
    if (!resultStr || resultStr === 'none') return null;
    const parts = resultStr.split(',');
    if (parts.length < 4) return null;
    return {
      from: fileRankToSquare(parseInt(parts[0]), parseInt(parts[1])),
      to: fileRankToSquare(parseInt(parts[2]), parseInt(parts[3])),
      isPromotion: parts.length > 4 && parts[4] === '1'
    };
  }
  
  // Parses string generic format: e.g. "d8e8" or "d8h8"; format "a1a2"
  private parseMoveStr(moveStr: string) {
      if (!moveStr || moveStr === 'none' || moveStr.length < 4) return null;
      
      const ff = moveStr.charCodeAt(0) - 'a'.charCodeAt(0) + 1;
      const fr = parseInt(moveStr.substring(1, 2), 10);
      const tf = moveStr.charCodeAt(2) - 'a'.charCodeAt(0) + 1;
      const tr = parseInt(moveStr.substring(3), 10);
      
      return {
          from: fileRankToSquare(ff, fr),
          to: fileRankToSquare(tf, tr),
          isPromotion: false // Simplification as promotion detection isn't strictly necessary for bot move execution if it just overrides piece, but in TS we handle it manually.
      };
  }
}
