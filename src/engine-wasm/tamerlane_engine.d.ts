/* tslint:disable */
/* eslint-disable */

/**
 * Get the best move for the given FEN position.
 * Returns "from_file,from_rank,to_file,to_rank" (1-based) or "none".
 * difficulty: 0 = medium, 1 = master
 */
export function get_best_move(fen: string, time_ms: number, difficulty: number): string;

/**
 * Check if the position is checkmate or stalemate.
 * Returns: "playing", "check", "checkmate_white", "checkmate_black", "stalemate"
 */
export function get_game_status(fen: string): string;

/**
 * Get legal moves for the given FEN. Returns a comma-separated list of "from_file,from_rank,to_file,to_rank" moves.
 */
export function get_legal_moves(fen: string): string;

/**
 * Initialize the engine (call once on startup).
 */
export function init_engine(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly get_best_move: (a: number, b: number, c: number, d: number) => [number, number];
    readonly get_game_status: (a: number, b: number) => [number, number];
    readonly get_legal_moves: (a: number, b: number) => [number, number];
    readonly init_engine: () => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
