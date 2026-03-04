/**
 * Tamerlane Chess - Constants
 * Game constants including piece movement directions and properties
 */

import { Color, PieceType } from './types';

// =============================================================================
// PIECE PROPERTIES
// =============================================================================

/** Maps piece type to its color */
export const PIECE_COLOR: Record<PieceType, Color> = {
    [PieceType.EMPTY]: Color.BOTH,
    // White pawns
    [PieceType.W_PAWN_PAWN]: Color.WHITE,
    [PieceType.W_PAWN_WARENGINE]: Color.WHITE,
    [PieceType.W_PAWN_CAMEL]: Color.WHITE,
    [PieceType.W_PAWN_ELEPHANT]: Color.WHITE,
    [PieceType.W_PAWN_MINISTER]: Color.WHITE,
    [PieceType.W_PAWN_KING]: Color.WHITE,
    [PieceType.W_PAWN_ADVISOR]: Color.WHITE,
    [PieceType.W_PAWN_GIRAFFE]: Color.WHITE,
    [PieceType.W_PAWN_CATAPULT]: Color.WHITE,
    [PieceType.W_PAWN_KNIGHT]: Color.WHITE,
    [PieceType.W_PAWN_ROOK]: Color.WHITE,
    // White pieces
    [PieceType.W_ROOK]: Color.WHITE,
    [PieceType.W_KNIGHT]: Color.WHITE,
    [PieceType.W_CATAPULT]: Color.WHITE,
    [PieceType.W_GIRAFFE]: Color.WHITE,
    [PieceType.W_MINISTER]: Color.WHITE,
    [PieceType.W_KING]: Color.WHITE,
    [PieceType.W_ADVISOR]: Color.WHITE,
    [PieceType.W_ELEPHANT]: Color.WHITE,
    [PieceType.W_CAMEL]: Color.WHITE,
    [PieceType.W_WARENGINE]: Color.WHITE,
    [PieceType.W_PRINCE]: Color.WHITE,
    [PieceType.W_ADKING]: Color.WHITE,
    // Black pawns
    [PieceType.B_PAWN_PAWN]: Color.BLACK,
    [PieceType.B_PAWN_WARENGINE]: Color.BLACK,
    [PieceType.B_PAWN_CAMEL]: Color.BLACK,
    [PieceType.B_PAWN_ELEPHANT]: Color.BLACK,
    [PieceType.B_PAWN_MINISTER]: Color.BLACK,
    [PieceType.B_PAWN_KING]: Color.BLACK,
    [PieceType.B_PAWN_ADVISOR]: Color.BLACK,
    [PieceType.B_PAWN_GIRAFFE]: Color.BLACK,
    [PieceType.B_PAWN_CATAPULT]: Color.BLACK,
    [PieceType.B_PAWN_KNIGHT]: Color.BLACK,
    [PieceType.B_PAWN_ROOK]: Color.BLACK,
    // Black pieces
    [PieceType.B_ROOK]: Color.BLACK,
    [PieceType.B_KNIGHT]: Color.BLACK,
    [PieceType.B_CATAPULT]: Color.BLACK,
    [PieceType.B_GIRAFFE]: Color.BLACK,
    [PieceType.B_MINISTER]: Color.BLACK,
    [PieceType.B_KING]: Color.BLACK,
    [PieceType.B_ADVISOR]: Color.BLACK,
    [PieceType.B_ELEPHANT]: Color.BLACK,
    [PieceType.B_CAMEL]: Color.BLACK,
    [PieceType.B_WARENGINE]: Color.BLACK,
    [PieceType.B_PRINCE]: Color.BLACK,
    [PieceType.B_ADKING]: Color.BLACK,
};

/** Piece name for image files */
export const PIECE_NAME: Record<PieceType, string> = {
    [PieceType.EMPTY]: 'Empty',
    [PieceType.W_PAWN_PAWN]: 'PawnPawn',
    [PieceType.W_PAWN_WARENGINE]: 'WarenginePawn',
    [PieceType.W_PAWN_CAMEL]: 'CamelPawn',
    [PieceType.W_PAWN_ELEPHANT]: 'ElephantPawn',
    [PieceType.W_PAWN_MINISTER]: 'MinisterPawn',
    [PieceType.W_PAWN_KING]: 'KingPawn',
    [PieceType.W_PAWN_ADVISOR]: 'AdvisorPawn',
    [PieceType.W_PAWN_GIRAFFE]: 'GiraffePawn',
    [PieceType.W_PAWN_CATAPULT]: 'CatapultPawn',
    [PieceType.W_PAWN_KNIGHT]: 'KnightPawn',
    [PieceType.W_PAWN_ROOK]: 'RookPawn',
    [PieceType.W_ROOK]: 'Rook',
    [PieceType.W_KNIGHT]: 'Knight',
    [PieceType.W_CATAPULT]: 'Catapult',
    [PieceType.W_GIRAFFE]: 'Giraffe',
    [PieceType.W_MINISTER]: 'Minister',
    [PieceType.W_KING]: 'King',
    [PieceType.W_ADVISOR]: 'Advisor',
    [PieceType.W_ELEPHANT]: 'Elephant',
    [PieceType.W_CAMEL]: 'Camel',
    [PieceType.W_WARENGINE]: 'Warengine',
    [PieceType.W_PRINCE]: 'Prince',
    [PieceType.W_ADKING]: 'AdKing',
    // Black pieces use same names
    [PieceType.B_PAWN_PAWN]: 'PawnPawn',
    [PieceType.B_PAWN_WARENGINE]: 'WarenginePawn',
    [PieceType.B_PAWN_CAMEL]: 'CamelPawn',
    [PieceType.B_PAWN_ELEPHANT]: 'ElephantPawn',
    [PieceType.B_PAWN_MINISTER]: 'MinisterPawn',
    [PieceType.B_PAWN_KING]: 'KingPawn',
    [PieceType.B_PAWN_ADVISOR]: 'AdvisorPawn',
    [PieceType.B_PAWN_GIRAFFE]: 'GiraffePawn',
    [PieceType.B_PAWN_CATAPULT]: 'CatapultPawn',
    [PieceType.B_PAWN_KNIGHT]: 'KnightPawn',
    [PieceType.B_PAWN_ROOK]: 'RookPawn',
    [PieceType.B_ROOK]: 'Rook',
    [PieceType.B_KNIGHT]: 'Knight',
    [PieceType.B_CATAPULT]: 'Catapult',
    [PieceType.B_GIRAFFE]: 'Giraffe',
    [PieceType.B_MINISTER]: 'Minister',
    [PieceType.B_KING]: 'King',
    [PieceType.B_ADVISOR]: 'Advisor',
    [PieceType.B_ELEPHANT]: 'Elephant',
    [PieceType.B_CAMEL]: 'Camel',
    [PieceType.B_WARENGINE]: 'Warengine',
    [PieceType.B_PRINCE]: 'Prince',
    [PieceType.B_ADKING]: 'AdKing',
};

// =============================================================================
// MOVEMENT DIRECTIONS
// =============================================================================

/** Knight movement pattern: L-shape (2+1) */
export const KNIGHT_DIRECTIONS = [-31, -29, -17, -13, 13, 17, 29, 31];

/** Catapult/Elephant movement: 1 or 2 squares diagonally */
export const CATAPULT_DIRECTIONS = [-32, -28, 28, 32, -16, -14, 14, 16];

/** Giraffe movement: 4+1 or 1+4 pattern */
export const GIRAFFE_DIRECTIONS = [-61, -59, -19, -11, 11, 19, 59, 61];

/** Minister movement: 2 squares diagonally */
export const MINISTER_DIRECTIONS = [-16, -14, 14, 16];

/** King movement: all 8 directions, 1 square */
export const KING_DIRECTIONS = [-16, -15, -14, -1, 1, 14, 15, 16];

/** Advisor movement: orthogonal, 1 square */
export const ADVISOR_DIRECTIONS = [-15, -1, 1, 15];

/** Camel movement: 3+1 or 1+3 pattern */
export const CAMEL_DIRECTIONS = [-46, -44, -18, -12, 12, 18, 44, 46];

/** Warengine (Dabbaba) movement: 2 squares orthogonally */
export const WARENGINE_DIRECTIONS = [-30, -2, 2, 30];

/** Rook movement directions (for sliding) */
export const ROOK_DIRECTIONS = [-15, -1, 1, 15];

// Giraffe Checkpoint Arrays (Reference implementation)
// Giraffe can only slide if ALL 3 checkpoint squares are empty
export const GIRAFFE_CHECK1 = [-16, -14, -16, -14, 14, 16, 14, 16]; // Diagonal checkpoint
export const GIRAFFE_CHECK2 = [-31, -29, -17, -13, 13, 17, 29, 31]; // Knight-distance checkpoint  
export const GIRAFFE_CHECK3 = [-46, -44, -18, -12, 12, 18, 44, 46]; // Camel-distance checkpoint
export const GIRAFFE_SLIDE_DIR = [-15, -15, -1, 1, -1, 1, 15, 15]; // Slide direction after checks pass

// =============================================================================
// FEN STRINGS
// =============================================================================

/** Starting position FEN for white at bottom */
export const START_FEN_WHITE = 'f1d1i1i1d1f/kamzgsvzmak1/pxcbyqehtnr/92/92/92/92/PXCBYQEHTNR/KAMZGSVZMAK1/F1D1I1I1D1F w';

/** Starting position FEN for black at bottom */
export const START_FEN_BLACK = 'F1D1I1I1D1F/KAMZGSVZMAK1/PXCBYQEHTNR/92/92/92/92/pxcbyqehtnr/kamzgsvzmak1/f1d1i1i1d1f w';

/** FEN character to piece type mapping */
export const FEN_CHAR_TO_PIECE: Record<string, PieceType> = {
    'P': PieceType.W_PAWN_PAWN,
    'X': PieceType.W_PAWN_WARENGINE,
    'C': PieceType.W_PAWN_CAMEL,
    'B': PieceType.W_PAWN_ELEPHANT,
    'Y': PieceType.W_PAWN_MINISTER,
    'Q': PieceType.W_PAWN_KING,
    'E': PieceType.W_PAWN_ADVISOR,
    'H': PieceType.W_PAWN_GIRAFFE,
    'T': PieceType.W_PAWN_CATAPULT,
    'N': PieceType.W_PAWN_KNIGHT,
    'R': PieceType.W_PAWN_ROOK,
    'K': PieceType.W_ROOK,
    'A': PieceType.W_KNIGHT,
    'M': PieceType.W_CATAPULT,
    'Z': PieceType.W_GIRAFFE,
    'G': PieceType.W_MINISTER,
    'S': PieceType.W_KING,
    'V': PieceType.W_ADVISOR,
    'F': PieceType.W_ELEPHANT,
    'D': PieceType.W_CAMEL,
    'I': PieceType.W_WARENGINE,
    'J': PieceType.W_PRINCE,
    'L': PieceType.W_ADKING,
    'p': PieceType.B_PAWN_PAWN,
    'x': PieceType.B_PAWN_WARENGINE,
    'c': PieceType.B_PAWN_CAMEL,
    'b': PieceType.B_PAWN_ELEPHANT,
    'y': PieceType.B_PAWN_MINISTER,
    'q': PieceType.B_PAWN_KING,
    'e': PieceType.B_PAWN_ADVISOR,
    'h': PieceType.B_PAWN_GIRAFFE,
    't': PieceType.B_PAWN_CATAPULT,
    'n': PieceType.B_PAWN_KNIGHT,
    'r': PieceType.B_PAWN_ROOK,
    'k': PieceType.B_ROOK,
    'a': PieceType.B_KNIGHT,
    'm': PieceType.B_CATAPULT,
    'z': PieceType.B_GIRAFFE,
    'g': PieceType.B_MINISTER,
    's': PieceType.B_KING,
    'v': PieceType.B_ADVISOR,
    'f': PieceType.B_ELEPHANT,
    'd': PieceType.B_CAMEL,
    'i': PieceType.B_WARENGINE,
    'j': PieceType.B_PRINCE,
    'l': PieceType.B_ADKING,
};

// =============================================================================
// PAWN ARRAYS
// =============================================================================

/** All white pawn piece types */
export const WHITE_PAWNS: PieceType[] = [
    PieceType.W_PAWN_PAWN,
    PieceType.W_PAWN_WARENGINE,
    PieceType.W_PAWN_CAMEL,
    PieceType.W_PAWN_ELEPHANT,
    PieceType.W_PAWN_MINISTER,
    PieceType.W_PAWN_KING,
    PieceType.W_PAWN_ADVISOR,
    PieceType.W_PAWN_GIRAFFE,
    PieceType.W_PAWN_CATAPULT,
    PieceType.W_PAWN_KNIGHT,
    PieceType.W_PAWN_ROOK,
];

/** All black pawn piece types */
export const BLACK_PAWNS: PieceType[] = [
    PieceType.B_PAWN_PAWN,
    PieceType.B_PAWN_WARENGINE,
    PieceType.B_PAWN_CAMEL,
    PieceType.B_PAWN_ELEPHANT,
    PieceType.B_PAWN_MINISTER,
    PieceType.B_PAWN_KING,
    PieceType.B_PAWN_ADVISOR,
    PieceType.B_PAWN_GIRAFFE,
    PieceType.B_PAWN_CATAPULT,
    PieceType.B_PAWN_KNIGHT,
    PieceType.B_PAWN_ROOK,
];

/** All pawn pieces (both colors) */
export const ALL_PAWNS: PieceType[] = [...WHITE_PAWNS, ...BLACK_PAWNS];
