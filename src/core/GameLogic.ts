/**
 * Tamerlane Chess - Game Logic
 * Handles game status, check/checkmate detection, and legal move filtering
 */

import { Board } from './Board';
import { MoveGenerator, createMove, getFromSquare, getToSquare, getCapturedPiece, hasFlag } from './MoveGenerator';
import { isSquareAttacked } from './Attack';
import { Color, PieceType, MOVE_FLAG_SWITCH_KING, MOVE_FLAG_ADKING_FROM_CITADEL, MOVE_FLAG_SWITCH_ANY_PIECE } from './types';
import { PIECE_COLOR } from './constants';

export enum GameStatus {
    PLAYING,
    CHECK,
    CHECKMATE,
    STALEMATE
}

/**
 * Checks if the king of the given color is in check
 */
export function isInCheck(board: Board, color: Color): boolean {
    const kingSq = findKingSquare(board, color);

    // If no king found (should not happen in normal game), not in check
    if (kingSq === -1) return false;

    const opponentColor = color === Color.WHITE ? Color.BLACK : Color.WHITE;
    return isSquareAttacked(board, kingSq, opponentColor);
}

/**
 * Finds the square of the King for the given color
 * Handles Tamerlane specific rules: King -> Prince -> AdKing priority
 */
export function findKingSquare(board: Board, color: Color): number {
    const pieces = board.getPieces();
    const filesBrd = board.getFilesBrd();

    // Priority: King > Prince > AdKing
    const kingType = color === Color.WHITE ? PieceType.W_KING : PieceType.B_KING;
    const princeType = color === Color.WHITE ? PieceType.W_PRINCE : PieceType.B_PRINCE;
    const adKingType = color === Color.WHITE ? PieceType.W_ADKING : PieceType.B_ADKING;

    let kingSq = -1;
    let princeSq = -1;
    let adKingSq = -1;

    // Scan board
    for (let i = 0; i < pieces.length; i++) {
        if (filesBrd[i] === 209) continue; // OFF_BOARD constant

        const piece = pieces[i];
        if (piece === kingType) kingSq = i;
        else if (piece === princeType) princeSq = i;
        else if (piece === adKingType) adKingSq = i;
    }

    // Return highest priority king available
    if (kingSq !== -1) return kingSq;
    if (princeSq !== -1) return princeSq;
    return adKingSq;
}

/**
 * Generates only legal moves (filters out moves that leave king in check)
 */
export function getLegalMoves(board: Board, generator: MoveGenerator): number[] {
    const pseudoLegalMoves = generator.generateMoves();
    const legalMoves: number[] = [];
    const side = board.getSide();

    for (const move of pseudoLegalMoves) {
        if (isMoveLegal(board, move, side)) {
            legalMoves.push(move);
        }
    }

    return legalMoves;
}

/**
 * Checks if a move is legal (doesn't leave king in check)
 */
function isMoveLegal(board: Board, move: number, side: Color): boolean {
    const from = getFromSquare(move);
    const to = getToSquare(move);
    const captured = getCapturedPiece(move);

    // Tamerlane Special Moves - always legal (don't leave king in check by definition)
    if (hasFlag(move, MOVE_FLAG_SWITCH_KING) ||
        hasFlag(move, MOVE_FLAG_ADKING_FROM_CITADEL) ||
        hasFlag(move, MOVE_FLAG_SWITCH_ANY_PIECE)) {
        return true; // Special moves bypass check validation
    }

    // Simulate move using a fast clone to avoid corrupting piece lists and arrays
    const testBoard = board.clone();
    testBoard.movePiece(from, to);

    return !isInCheck(testBoard, side);
}

/**
 * Determines the current game status
 */
export function getGameStatus(board: Board, generator: MoveGenerator): { status: GameStatus, winner?: Color, checkSquare?: number } {
    const side = board.getSide();
    const opponent = side === Color.WHITE ? Color.BLACK : Color.WHITE;

    // 1. Check Citadel Victory (if opponent king entered my citadel)
    // Tamerlane Rule: If a King enters the opponent's citadel, the game is a DRAW on terms of honor? 
    // Usually it is considered a WIN in computer implementations or a Draw.
    // However, the user provided reference suggests moves INTO Citadel are for "AdKing Move FROM Citadel".
    // But traditionally, entering the enemy citadel is the goal.
    // I will implement it as a WIN for now.

    // Logic: If 'Opponent' (who just moved before 'side' turn started but we check status now)
    // moved their King into 'Side's' Citadel.
    // Wait, status is checked for CURRENT side (to move).
    // So 'side' is the VICTIM if they are checkmated.
    // BUT if the PREVIOUS move by 'opponent' put their king in 'side's' citadel, 'opponent' wins.

    const oppKingSq = findKingSquare(board, opponent);
    if (oppKingSq !== -1) {
        // If Opponent (Black) King is in My (White) Citadel (12) -> Opponent Wins
        if (side === Color.WHITE && board.isWhiteCitadel(oppKingSq)) {
            return { status: GameStatus.CHECKMATE, winner: Color.BLACK };
        }
        // If Opponent (White) King is in My (Black) Citadel (0) -> Opponent Wins
        if (side === Color.BLACK && board.isBlackCitadel(oppKingSq)) {
            return { status: GameStatus.CHECKMATE, winner: Color.WHITE };
        }
    }

    const legalMoves = getLegalMoves(board, generator);
    const inCheck = isInCheck(board, side);

    if (legalMoves.length === 0) {
        if (inCheck) {
            return {
                status: GameStatus.CHECKMATE,
                winner: side === Color.WHITE ? Color.BLACK : Color.WHITE,
                checkSquare: findKingSquare(board, side)
            };
        } else {
            return { status: GameStatus.STALEMATE };
        }
    }

    if (inCheck) {
        return { 
            status: GameStatus.CHECK, 
            checkSquare: findKingSquare(board, side)
        };
    }

    return { status: GameStatus.PLAYING };
}
