/**
 * Tamerlane Chess - Attack Logic
 * Determines if a square is attacked by opponent pieces
 */

import { Board, isOffBoard } from './Board';
import { Color, PieceType } from './types';
import {
    KNIGHT_DIRECTIONS,
    CATAPULT_DIRECTIONS,
    GIRAFFE_DIRECTIONS,
    MINISTER_DIRECTIONS,
    KING_DIRECTIONS,
    ADVISOR_DIRECTIONS,
    CAMEL_DIRECTIONS,
    WARENGINE_DIRECTIONS,
    ROOK_DIRECTIONS,
    PIECE_COLOR,
    WHITE_PAWNS,
    BLACK_PAWNS
} from './constants';

/**
 * Checks if a square is attacked by the given side
 * @param board The game board
 * @param sq The square to check
 * @param attackingSide The side that might be attacking
 */
export function isSquareAttacked(board: Board, sq: number, attackingSide: Color): boolean {
    const pieces = board.getPieces();
    const filesBrd = board.getFilesBrd();

    // 1. Check Pawn Attacks
    // Pawns attack diagonally forward (relative to their color)
    // If attackingSide is WHITE, they attack from sq-14 and sq-16 (because they move +14/+16 to capture)
    // Wait! A white pawn at sq-14 moves +14 to land on sq. So we check sq-14.
    if (attackingSide === Color.WHITE) {
        if (!isOffBoard(sq - 14, filesBrd) && isPawnOfColor(pieces[sq - 14], Color.WHITE)) return true;
        if (!isOffBoard(sq - 16, filesBrd) && isPawnOfColor(pieces[sq - 16], Color.WHITE)) return true;
    } else { // BLACK
        if (!isOffBoard(sq + 14, filesBrd) && isPawnOfColor(pieces[sq + 14], Color.BLACK)) return true;
        if (!isOffBoard(sq + 16, filesBrd) && isPawnOfColor(pieces[sq + 16], Color.BLACK)) return true;
    }

    // 2. Check Stepping Pieces (Knight, Camel, etc.)
    // We check if a piece of that type is at the source square (move symmetry)

    // Knight & Pawn-of-Knight
    // Pawn types removed, they attack as pawns.
    if (checkSteppingAttack(board, sq, attackingSide, KNIGHT_DIRECTIONS, [
        PieceType.W_KNIGHT, PieceType.B_KNIGHT
    ])) return true;

    // Elephant & Pawn-of-Elephant (NOW A SLIDER: 1 or 2 squares)
    // Uses CATAPULT_DIRECTIONS from constants which has 8 directions (1 and 2 step)
    // MUST BE SLIDING for user visual expectations (though constant has fixed steps, function treats as slider)
    // Pawn types removed.
    const elephantTypes = [
        PieceType.W_ELEPHANT, PieceType.B_ELEPHANT
    ];
    if (checkSteppingAttack(board, sq, attackingSide, CATAPULT_DIRECTIONS, elephantTypes)) return true;

    // Minister & Pawn-of-Minister
    if (checkSteppingAttack(board, sq, attackingSide, MINISTER_DIRECTIONS, [
        PieceType.W_MINISTER, PieceType.B_MINISTER
    ])) return true;

    // Catapult/Picket (Special: Must be at least 2 squares away diagonally)
    // Cannot attack adjacent diagonal square
    if (checkPicketAttack(board, sq, attackingSide, MINISTER_DIRECTIONS, [
        PieceType.W_CATAPULT, PieceType.B_CATAPULT
    ])) return true;

    // King / Prince / AdKing & Pawn-of-King
    // Pawns removed.
    const kingTypes = [
        PieceType.W_KING, PieceType.B_KING,
        PieceType.W_PRINCE, PieceType.B_PRINCE,
        PieceType.W_ADKING, PieceType.B_ADKING
    ];
    if (checkSteppingAttack(board, sq, attackingSide, KING_DIRECTIONS, kingTypes)) return true;

    // Advisor & Pawn-of-Advisor
    if (checkSteppingAttack(board, sq, attackingSide, ADVISOR_DIRECTIONS, [
        PieceType.W_ADVISOR, PieceType.B_ADVISOR
    ])) return true;

    // Camel & Pawn-of-Camel
    if (checkSteppingAttack(board, sq, attackingSide, CAMEL_DIRECTIONS, [
        PieceType.W_CAMEL, PieceType.B_CAMEL
    ])) return true;

    // Warengine & Pawn-of-Warengine
    if (checkSteppingAttack(board, sq, attackingSide, WARENGINE_DIRECTIONS, [
        PieceType.W_WARENGINE, PieceType.B_WARENGINE
    ])) return true;

    // 3. Check Sliding Pieces (Rook, Giraffe)

    // Rook & Pawn-of-Rook
    if (checkSlidingAttack(board, sq, attackingSide, ROOK_DIRECTIONS, [
        PieceType.W_ROOK, PieceType.B_ROOK
    ])) return true;

    // Giraffe (Complex Bent Rider Logic)
    // Checked by iterating enemy giraffes
    const enemyGiraffe = attackingSide === Color.WHITE ? PieceType.W_GIRAFFE : PieceType.B_GIRAFFE;
    const enemyPawnGiraffe = attackingSide === Color.WHITE ? PieceType.W_PAWN_GIRAFFE : PieceType.B_PAWN_GIRAFFE;

    // Check main Giraffes
    const giraffes = board.getPieceSquares(enemyGiraffe);
    for (const gSq of giraffes) {
        if (canGiraffeAttack(board, gSq, sq)) return true;
    }
    // Pawn-of-Giraffe is a PAWN, handled by generic pawn logic.
    // Removed pawnGiraffes check.

    return false;
}

/**
 * Helper to check stepping piece attacks (symmetry required)
 */
function checkSteppingAttack(
    board: Board,
    sq: number,
    attackingSide: Color,
    directions: number[],
    validPieceTypes: PieceType[]
): boolean {
    const pieces = board.getPieces();
    const filesBrd = board.getFilesBrd();

    for (const dir of directions) {
        // We look "backwards" using the reverse direction (or same if symmetric)
        // Since directions like KNIGHT are symmetric (+/- pairs), we can just iterate them.
        // e.g. If a Knight is at sq+dir, it can jump to sq.

        const targetSq = sq + dir; // Check this square
        if (isOffBoard(targetSq, filesBrd)) continue;

        const piece = pieces[targetSq];
        if (piece !== PieceType.EMPTY) {
            if (PIECE_COLOR[piece] === attackingSide && validPieceTypes.includes(piece)) {
                return true;
            }
        }
    }
    return false;
}

/**
 * Helper to check sliding piece attacks
 */
function checkSlidingAttack(
    board: Board,
    sq: number,
    attackingSide: Color,
    directions: number[],
    validPieceTypes: PieceType[]
): boolean {
    const pieces = board.getPieces();
    const filesBrd = board.getFilesBrd();

    for (const dir of directions) {
        let targetSq = sq + dir;

        while (!isOffBoard(targetSq, filesBrd)) {
            const piece = pieces[targetSq];

            if (piece !== PieceType.EMPTY) {
                // Found a piece
                if (PIECE_COLOR[piece] === attackingSide && validPieceTypes.includes(piece)) {
                    return true; // It's an attacker
                }
                // If it's not the attacker, or it's a piece that blocks, we stop.
                // Even if it's the attacker (caught above), we stop (can't attack through itself).
                break;
            }

            targetSq += dir;
        }
    }
    return false;
}

/**
 * Helper to check Picket/Catapult attack (minimum 2 squares diagonally)
 * Similar to sliding but skips the first adjacent diagonal square
 */
function checkPicketAttack(
    board: Board,
    sq: number,
    attackingSide: Color,
    directions: number[],
    validPieceTypes: PieceType[]
): boolean {
    const pieces = board.getPieces();
    const filesBrd = board.getFilesBrd();

    for (const dir of directions) {
        let targetSq = sq + dir;
        let distance = 1;

        while (!isOffBoard(targetSq, filesBrd)) {
            const piece = pieces[targetSq];

            if (piece !== PieceType.EMPTY) {
                // Found a piece
                if (distance >= 2) {
                    // Picket can only attack from distance 2+
                    if (PIECE_COLOR[piece] === attackingSide && validPieceTypes.includes(piece)) {
                        return true; // It's an attacker at valid distance
                    }
                }
                // Blocked by piece (whether attacker at wrong distance or other piece)
                break;
            }

            targetSq += dir;
            distance++;
        }
    }
    return false;
}

/**
 * Helper to check if a specific Giraffe attacks a square
 */
function canGiraffeAttack(board: Board, from: number, to: number): boolean {
    const filesBrd = board.getFilesBrd();
    const pieces = board.getPieces();

    const diagonalSteps = [
        { diag: -16, orthos: [-15, -1] },
        { diag: -14, orthos: [-15, 1] },
        { diag: 14, orthos: [15, -1] },
        { diag: 16, orthos: [15, 1] }
    ];

    for (const { diag, orthos } of diagonalSteps) {
        const dSq = from + diag;
        // Step 1: Diag empty? (Cannot jump)
        if (isOffBoard(dSq, filesBrd)) continue;
        if (pieces[dSq] !== PieceType.EMPTY) continue;
        if (dSq === to) return false; // Too close (Giraffe min range is > 3)

        for (const ortho of orthos) {
            const s1 = dSq + ortho;
            if (isOffBoard(s1, filesBrd)) continue;
            if (s1 === to) return false; // Too close (min 3)
            if (pieces[s1] !== PieceType.EMPTY) continue;

            // Slide starting from 3rd square (1 diag + 2 straight)
            let curr = s1 + ortho;
            while (!isOffBoard(curr, filesBrd)) {
                if (curr === to) return true; // Hit!
                if (pieces[curr] !== PieceType.EMPTY) break; // Blocked
                curr += ortho;
            }
        }
    }
    return false;
}



/**
 * Helper to check if a piece is a pawn of a specific color
 */
function isPawnOfColor(piece: PieceType, color: Color): boolean {
    if (piece === PieceType.EMPTY) return false;
    if (PIECE_COLOR[piece] !== color) return false;

    if (color === Color.WHITE) {
        return WHITE_PAWNS.includes(piece);
    } else {
        return BLACK_PAWNS.includes(piece);
    }
}
