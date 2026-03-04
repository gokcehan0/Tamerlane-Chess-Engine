import { Board, fileRankToSquare } from './Board';

export class TamerlaneEngine {
  private worker: Worker;
  private isReady: boolean = false;
  private resolveReady: ((value: unknown) => void) | null = null;
  private resolveSearch: ((value: any) => void) | null = null;

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
        if (this.resolveSearch) {
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

  // Same interface as the old StockfishAI
  public async getBestMove(board: Board, timeMs = 1000, difficulty = 0): Promise<any> {
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
}
