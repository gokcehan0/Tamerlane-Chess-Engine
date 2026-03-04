/**
 * Tamerlane Chess - Promotion Logic
 * Handles pawn promotion when reaching the last rank
 */

import { PieceType, Color } from './types';
import { PIECE_COLOR, WHITE_PAWNS, BLACK_PAWNS } from './constants';

// Promotion ranks
export const WHITE_PROMOTION_RANK = 10;
export const BLACK_PROMOTION_RANK = 1;

/**
 * Check if a piece is a pawn
 */
export function isPawn(piece: PieceType): boolean {
    return WHITE_PAWNS.includes(piece) || BLACK_PAWNS.includes(piece);
}

/**
 * Check if a pawn can promote at the given rank
 */
export function canPromote(piece: PieceType, rank: number): boolean {
    if (!isPawn(piece)) return false;

    const color = PIECE_COLOR[piece];
    if (color === Color.WHITE && rank === WHITE_PROMOTION_RANK) return true;
    if (color === Color.BLACK && rank === BLACK_PROMOTION_RANK) return true;

    return false;
}

/**
 * Get the promoted piece for a pawn
 * Each pawn type promotes to its corresponding piece
 */
export function getPromotedPiece(pawn: PieceType): PieceType {
    switch (pawn) {
        // White pawns
        case PieceType.W_PAWN_PAWN:
            return PieceType.W_ADKING; // Special: pawn of pawn → adking
        case PieceType.W_PAWN_ROOK:
            return PieceType.W_ROOK;
        case PieceType.W_PAWN_KNIGHT:
            return PieceType.W_KNIGHT;
        case PieceType.W_PAWN_CATAPULT:
            return PieceType.W_CATAPULT;
        case PieceType.W_PAWN_GIRAFFE:
            return PieceType.W_GIRAFFE;
        case PieceType.W_PAWN_MINISTER:
            return PieceType.W_MINISTER;
        case PieceType.W_PAWN_KING:
            return PieceType.W_PRINCE;
        case PieceType.W_PAWN_ADVISOR:
            return PieceType.W_ADVISOR;
        case PieceType.W_PAWN_ELEPHANT:
            return PieceType.W_ELEPHANT;
        case PieceType.W_PAWN_CAMEL:
            return PieceType.W_CAMEL;
        case PieceType.W_PAWN_WARENGINE:
            return PieceType.W_WARENGINE;

        // Black pawns
        case PieceType.B_PAWN_PAWN:
            return PieceType.B_ADKING; // Special: pawn of pawn → adking
        case PieceType.B_PAWN_ROOK:
            return PieceType.B_ROOK;
        case PieceType.B_PAWN_KNIGHT:
            return PieceType.B_KNIGHT;
        case PieceType.B_PAWN_CATAPULT:
            return PieceType.B_CATAPULT;
        case PieceType.B_PAWN_GIRAFFE:
            return PieceType.B_GIRAFFE;
        case PieceType.B_PAWN_MINISTER:
            return PieceType.B_MINISTER;
        case PieceType.B_PAWN_KING:
            return PieceType.B_PRINCE;
        case PieceType.B_PAWN_ADVISOR:
            return PieceType.B_ADVISOR;
        case PieceType.B_PAWN_ELEPHANT:
            return PieceType.B_ELEPHANT;
        case PieceType.B_PAWN_CAMEL:
            return PieceType.B_CAMEL;
        case PieceType.B_PAWN_WARENGINE:
            return PieceType.B_WARENGINE;

        default:
            return pawn; // No promotion
    }
}

/**
 * Get the pawn type from a promoted piece (for undo)
 */
export function getDemotedPiece(piece: PieceType): PieceType {
    switch (piece) {
        // White pieces → their pawns
        case PieceType.W_ADKING:
            return PieceType.W_PAWN_PAWN;
        case PieceType.W_ROOK:
            return PieceType.W_PAWN_ROOK;
        case PieceType.W_KNIGHT:
            return PieceType.W_PAWN_KNIGHT;
        case PieceType.W_CATAPULT:
            return PieceType.W_PAWN_CATAPULT;
        case PieceType.W_GIRAFFE:
            return PieceType.W_PAWN_GIRAFFE;
        case PieceType.W_MINISTER:
            return PieceType.W_PAWN_MINISTER;
        case PieceType.W_PRINCE:
            return PieceType.W_PAWN_KING;
        case PieceType.W_ADVISOR:
            return PieceType.W_PAWN_ADVISOR;
        case PieceType.W_ELEPHANT:
            return PieceType.W_PAWN_ELEPHANT;
        case PieceType.W_CAMEL:
            return PieceType.W_PAWN_CAMEL;
        case PieceType.W_WARENGINE:
            return PieceType.W_PAWN_WARENGINE;

        // Black pieces → their pawns
        case PieceType.B_ADKING:
            return PieceType.B_PAWN_PAWN;
        case PieceType.B_ROOK:
            return PieceType.B_PAWN_ROOK;
        case PieceType.B_KNIGHT:
            return PieceType.B_PAWN_KNIGHT;
        case PieceType.B_CATAPULT:
            return PieceType.B_PAWN_CATAPULT;
        case PieceType.B_GIRAFFE:
            return PieceType.B_PAWN_GIRAFFE;
        case PieceType.B_MINISTER:
            return PieceType.B_PAWN_MINISTER;
        case PieceType.B_PRINCE:
            return PieceType.B_PAWN_KING;
        case PieceType.B_ADVISOR:
            return PieceType.B_PAWN_ADVISOR;
        case PieceType.B_ELEPHANT:
            return PieceType.B_PAWN_ELEPHANT;
        case PieceType.B_CAMEL:
            return PieceType.B_PAWN_CAMEL;
        case PieceType.B_WARENGINE:
            return PieceType.B_PAWN_WARENGINE;

        default:
            return piece;
    }
}
