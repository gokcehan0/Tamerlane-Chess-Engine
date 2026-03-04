/**
 * Tamerlane Chess - Type Definitions
 * Core type definitions for the game engine
 */

// =============================================================================
// ENUMS
// =============================================================================

/** Player colors */
export enum Color {
    WHITE = 0,
    BLACK = 1,
    BOTH = 2,
}

export interface BoardState {
    board: PieceType[];
    turn: Color;
}

/** All piece types in Tamerlane Chess */
export enum PieceType {
    EMPTY = 0,
    // White Pawns (pieces that can promote)
    W_PAWN_PAWN = 1,
    W_PAWN_WARENGINE = 2,
    W_PAWN_CAMEL = 3,
    W_PAWN_ELEPHANT = 4,
    W_PAWN_MINISTER = 5,
    W_PAWN_KING = 6,
    W_PAWN_ADVISOR = 7,
    W_PAWN_GIRAFFE = 8,
    W_PAWN_CATAPULT = 9,
    W_PAWN_KNIGHT = 10,
    W_PAWN_ROOK = 11,
    // White Pieces
    W_ROOK = 12,
    W_KNIGHT = 13,
    W_CATAPULT = 14,
    W_GIRAFFE = 15,
    W_MINISTER = 16,
    W_KING = 17,
    W_ADVISOR = 18,
    W_ELEPHANT = 19,
    W_CAMEL = 20,
    W_WARENGINE = 21,
    // Black Pawns
    B_PAWN_PAWN = 22,
    B_PAWN_WARENGINE = 23,
    B_PAWN_CAMEL = 24,
    B_PAWN_ELEPHANT = 25,
    B_PAWN_MINISTER = 26,
    B_PAWN_KING = 27,
    B_PAWN_ADVISOR = 28,
    B_PAWN_GIRAFFE = 29,
    B_PAWN_CATAPULT = 30,
    B_PAWN_KNIGHT = 31,
    B_PAWN_ROOK = 32,
    // Black Pieces
    B_ROOK = 33,
    B_KNIGHT = 34,
    B_CATAPULT = 35,
    B_GIRAFFE = 36,
    B_MINISTER = 37,
    B_KING = 38,
    B_ADVISOR = 39,
    B_ELEPHANT = 40,
    B_CAMEL = 41,
    B_WARENGINE = 42,
    // Special pieces
    W_PRINCE = 43,
    W_ADKING = 44,
    B_PRINCE = 45,
    B_ADKING = 46,
}

// =============================================================================
// INTERFACES
// =============================================================================

/** Represents a piece on the board */
export interface Piece {
    type: PieceType;
    color: Color;
}

/** Represents a move */
export interface Move {
    from: number;
    to: number;
    piece: PieceType;
    captured: PieceType;
    isPromotion: boolean;
    promotedTo?: PieceType;
    flags: number;
}

/** Game state snapshot for undo/redo */
export interface GameStateSnapshot {
    pieces: PieceType[];
    side: Color;
    ply: number;
    posKey: number;
}

/** Move history entry */
export interface HistoryEntry {
    move: Move;
    posKey: number;
}

/** Board position (file and rank) */
export interface Position {
    file: number;
    rank: number;
}

/** Online game state */
export interface OnlineGameState {
    isOnline: boolean;
    roomCode: string;
    playerColor: Color;
    isRoomCreator: boolean;
    opponentConnected: boolean;
    currentTurn: Color;
}

// =============================================================================
// CONSTANTS
// =============================================================================

/** Board dimensions */
export const BOARD_FILES = 11;
export const BOARD_RANKS = 10;
export const BOARD_SQUARES = 270; // Internal representation

/** Special squares */
export const NO_SQUARE = 208;
export const OFF_BOARD = 209;

/** Citadel squares */
export const CITADEL_WHITE = 88;
export const CITADEL_BLACK = 181;

/** Move flags */
export const MOVE_FLAG_NONE = 0;
export const MOVE_FLAG_PROMOTION = 0x400000;
export const MOVE_FLAG_TO_BE_ADKING = 0x800000;       // Pawn becomes AdKing
export const MOVE_FLAG_SWITCH_KING = 0x1000000;       // King switches with another king
export const MOVE_FLAG_ADKING_FROM_CITADEL = 0x2000000; // AdKing escapes citadel
export const MOVE_FLAG_SWITCH_ANY_PIECE = 0x4000000;  // Sole king teleport

/** Maximum values */
export const MAX_GAME_MOVES = 2048;
export const MAX_POSITION_MOVES = 1000;
export const MAX_DEPTH = 112;
export const NO_MOVE = 0;
