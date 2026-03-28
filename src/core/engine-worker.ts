// @ts-ignore
import init, { init_engine, get_best_move } from '../engine-wasm/tamerlane_engine.js';

let isReady = false;

self.onmessage = async (e) => {
  const { type, fen, time_ms, difficulty } = e.data;
  
  if (type === 'init') {
    if (!isReady) {
      // Initialize WASM module
      await init();
      // Initialize engine globals (Zobrist keys, etc.)
      init_engine();
      isReady = true;
    }
    self.postMessage({ type: 'ready' });
  } else if (type === 'search') {
    if (!isReady) {
      self.postMessage({ type: 'error', message: 'Engine not ready' });
      return;
    }
    
    // get_best_move returns "from_file,from_rank,to_file,to_rank,is_prom"
    const result = get_best_move(fen, time_ms, difficulty);
    self.postMessage({ type: 'best_move', result });
  }
};
