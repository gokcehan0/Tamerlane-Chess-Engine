/**
 * Tamerlane Chess - Move Generator
 * Generates legal moves for all pieces
 */

import {
    Color,
    PieceType,
    Move,
    BOARD_SQUARES,
    NO_MOVE,
    MOVE_FLAG_NONE,
    MOVE_FLAG_PROMOTION,
    MOVE_FLAG_TO_BE_ADKING,
    MOVE_FLAG_SWITCH_KING,
    MOVE_FLAG_ADKING_FROM_CITADEL,
    MOVE_FLAG_SWITCH_ANY_PIECE,
} from './types';
import { Board, fileRankToSquare, isOffBoard } from './Board';
import {
    PIECE_COLOR,
    KNIGHT_DIRECTIONS,
    CATAPULT_DIRECTIONS,
    GIRAFFE_DIRECTIONS,
    GIRAFFE_CHECK1,
    GIRAFFE_CHECK2,
    GIRAFFE_CHECK3,
    GIRAFFE_SLIDE_DIR,
    MINISTER_DIRECTIONS,
    KING_DIRECTIONS,
    ADVISOR_DIRECTIONS,
    CAMEL_DIRECTIONS,
    WARENGINE_DIRECTIONS,
    ROOK_DIRECTIONS,
    WHITE_PAWNS,
    BLACK_PAWNS,
} from './constants';

/**
 * Creates a move integer from components
 */
export function createMove(
    from: number,
    to: number,
    captured: PieceType = PieceType.EMPTY,
    isPromotion: boolean = false,
    flags: number = MOVE_FLAG_NONE
): number {
    let move = from | (to << 8) | (captured << 16);
    if (isPromotion) {
        move |= MOVE_FLAG_PROMOTION;
    }
    move |= flags;
    return move;
}

/**
 * Extracts 'from' square from move integer
 */
export function getFromSquare(move: number): number {
    return move & 0xff;
}

/**
 * Extracts 'to' square from move integer
 */
export function getToSquare(move: number): number {
    return (move >> 8) & 0xff;
}

/**
 * Extracts captured piece from move integer
 */
export function getCapturedPiece(move: number): PieceType {
    return (move >> 16) & 0x3f;
}

/**
 * Checks if move is a promotion
 */
export function isPromotion(move: number): boolean {
    return (move & MOVE_FLAG_PROMOTION) !== 0;
}

/**
 * Checks if move has a specific flag
 */
export function hasFlag(move: number, flag: number): boolean {
    return (move & flag) !== 0;
}

/**
 * Adds a flag to a move
 */
export function addFlag(move: number, flag: number): number {
    return move | flag;
}

/**
 * Move Generator class
 */
export class MoveGenerator {
    private board: Board;
    private moveList: number[] = [];
    private moveListStart: number[] = [];

    constructor(board: Board) {
        this.board = board;
        this.moveListStart = new Array(128).fill(0);
    }

    /**
     * Generate all pseudo-legal moves for current position
     */
    generateMoves(): number[] {
        this.moveList = [];
        const side = this.board.getSide();

        // Update kings list for Tamerlane special rules
        this.board.updateKingsList();

        // Tamerlane Special Moves (Priority over regular moves)
        // 1. King Switch Places (when king in opponent citadel with multiple kings)
        this.generateKingSwitchMoves();

        // 2. AdKing Citadel Escape (forced move if AdKing in own citadel)
        if (this.generateAdKingCitadelEscape()) {
            return this.moveList; // Forced move, no other options
        }

        // 3. Sole King Teleport (when only one king and in check)
        // User requested "Restricted King Moves" - disabling this rule
        // this.generateSoleKingTeleport();

        // Regular piece moves
        const pieces = this.board.getPieces();
        const filesBrd = this.board.getFilesBrd();

        for (let sq = 0; sq < BOARD_SQUARES; sq++) {
            const piece = pieces[sq];
            if (piece === PieceType.EMPTY) continue;
            if (PIECE_COLOR[piece] !== side) continue;

            // Generate moves based on piece type
            this.generatePieceMoves(sq, piece, side, filesBrd);
        }

        return this.moveList;
    }

    /**
     * Generate moves for a specific piece
     */
    private isSliding(piece: PieceType): boolean {
        return piece === PieceType.W_ROOK || piece === PieceType.B_ROOK ||
            piece === PieceType.W_CATAPULT || piece === PieceType.B_CATAPULT;
    }

    /**
     * Get movement directions for a piece
     */
    private getDirections(piece: PieceType): number[] | null {
        switch (piece) {
            case PieceType.W_KNIGHT:
            case PieceType.B_KNIGHT:
                return KNIGHT_DIRECTIONS;

            case PieceType.W_ELEPHANT:
            case PieceType.B_ELEPHANT:
                // User requested 1 or 2 squares for Elephant
                return CATAPULT_DIRECTIONS;

            case PieceType.W_MINISTER:
            case PieceType.B_MINISTER:
                return MINISTER_DIRECTIONS;

            case PieceType.W_CATAPULT:
            case PieceType.B_CATAPULT:
                // Catapult acts as Bishop (Diagonal Slider)
                return MINISTER_DIRECTIONS;

            // Giraffe handled separately
            case PieceType.W_GIRAFFE:
            case PieceType.B_GIRAFFE:
                return null;

            case PieceType.W_KING:
            case PieceType.B_KING:
            case PieceType.W_PRINCE:
            case PieceType.B_PRINCE:
            case PieceType.W_ADKING:
            case PieceType.B_ADKING:
                return KING_DIRECTIONS;

            case PieceType.W_ADVISOR:
            case PieceType.B_ADVISOR:
                return ADVISOR_DIRECTIONS;

            case PieceType.W_CAMEL:
            case PieceType.B_CAMEL:
                return CAMEL_DIRECTIONS;

            case PieceType.W_WARENGINE:
            case PieceType.B_WARENGINE:
                return WARENGINE_DIRECTIONS;

            case PieceType.W_ROOK:
            case PieceType.B_ROOK:
                return ROOK_DIRECTIONS;

            default:
                return null;
        }
    }

    /**
     * Generate moves for a specific piece
     */
    private generatePieceMoves(
        sq: number,
        piece: PieceType,
        side: Color,
        filesBrd: number[]
    ): void {
        // Check if piece is a pawn
        if (this.isPawn(piece)) {
            this.generatePawnMoves(sq, piece, side, filesBrd);
            return;
        }

        // Giraffe special handling
        if (piece === PieceType.W_GIRAFFE || piece === PieceType.B_GIRAFFE) {
            this.generateGiraffeMoves(sq, side, filesBrd);
            return;
        }

        // Picket/Catapult special handling - must move at least 2 squares diagonally
        if (piece === PieceType.W_CATAPULT || piece === PieceType.B_CATAPULT) {
            this.generatePicketMoves(sq, side, filesBrd);
            return;
        }

        // Non-sliding pieces
        const directions = this.getDirections(piece);
        if (!directions) return;

        if (this.isSliding(piece)) {
            this.generateSlidingMoves(sq, directions, side, filesBrd);
        } else {
            this.generateNonSlidingMoves(sq, directions, side, filesBrd);
        }
    }



    /**
     * Generate Giraffe moves (Correct checkpoint algorithm from reference)
     * Giraffe moves: Must have 3 specific squares empty, then slides
     */
    private generateGiraffeMoves(sq: number, side: Color, filesBrd: number[]): void {
        const pieces = this.board.getPieces();

        // 8 directions for Giraffe
        for (let i = 0; i < 8; i++) {
            const check1 = sq + GIRAFFE_CHECK1[i];
            const check2 = sq + GIRAFFE_CHECK2[i];
            const check3 = sq + GIRAFFE_CHECK3[i];

            // All 3 checkpoints must be EMPTY (can pass through)
            if (pieces[check1] === PieceType.EMPTY &&
                pieces[check2] === PieceType.EMPTY &&
                pieces[check3] === PieceType.EMPTY) {

                // Start sliding from initial Giraffe position
                let toSq = sq + GIRAFFE_DIRECTIONS[i];

                while (!isOffBoard(toSq, filesBrd)) {
                    // Citadel Check
                    const dummyPiece = side === Color.WHITE ? PieceType.W_GIRAFFE : PieceType.B_GIRAFFE;
                    if (!this.canEnterSquare(dummyPiece, toSq, side)) {
                        toSq += GIRAFFE_SLIDE_DIR[i];
                        continue;
                    }

                    const targetPiece = this.board.getPiece(toSq);
                    if (targetPiece === PieceType.EMPTY) {
                        this.addMove(sq, toSq);
                    } else {
                        const targetColor = PIECE_COLOR[targetPiece];
                        if (targetColor !== undefined && targetColor !== side) {
                            this.addCaptureMove(sq, toSq, targetPiece);
                        }
                        break; // Blocked
                    }

                    // Continue sliding in the appropriate direction
                    toSq += GIRAFFE_SLIDE_DIR[i];
                }
            }
        }
    }

    /**
     * Generate Picket/Catapult moves - must move at least 2 squares diagonally
     * Like a Bishop but cannot move to adjacent diagonal square
     */
    private generatePicketMoves(sq: number, side: Color, filesBrd: number[]): void {
        const piece = this.board.getPiece(sq);
        
        // Diagonal directions (same as Bishop/Minister)
        for (const dir of MINISTER_DIRECTIONS) {
            let toSq = sq + dir;
            let distance = 1;

            while (!isOffBoard(toSq, filesBrd)) {
                // Skip first square (distance 1) - Picket must move at least 2 squares
                if (distance >= 2) {
                    // Citadel Check
                    if (this.canEnterSquare(piece, toSq, side)) {
                        const targetPiece = this.board.getPiece(toSq);

                        if (targetPiece === PieceType.EMPTY) {
                            this.addMove(sq, toSq);
                        } else {
                            if (PIECE_COLOR[targetPiece] !== undefined && PIECE_COLOR[targetPiece] !== side) {
                                this.addCaptureMove(sq, toSq, targetPiece);
                            }
                            break; // Blocked by piece
                        }
                    }
                } else {
                    // Distance 1 - check if blocked (cannot jump over pieces)
                    const targetPiece = this.board.getPiece(toSq);
                    if (targetPiece !== PieceType.EMPTY) {
                        break; // Blocked at first square, cannot continue
                    }
                }

                toSq += dir;
                distance++;
            }
        }
    }

    /**
     * Check if piece is a pawn
     */
    private isPawn(piece: PieceType): boolean {
        return WHITE_PAWNS.includes(piece) || BLACK_PAWNS.includes(piece);
    }

    /**
     * Generate pawn moves
     */
    private generatePawnMoves(
        sq: number,
        piece: PieceType,
        side: Color,
        filesBrd: number[]
    ): void {
        const forward = side === Color.WHITE ? 15 : -15;
        const diagonals = side === Color.WHITE ? [14, 16] : [-14, -16];

        // Forward move
        const forwardSq = sq + forward;
        if (!isOffBoard(forwardSq, filesBrd)) {
            // Citadel Check
            if (this.canEnterSquare(piece, forwardSq, side)) {
                if (this.board.getPiece(forwardSq) === PieceType.EMPTY) {
                    this.addMove(sq, forwardSq);
                }
            }
        }

        // Diagonal captures
        for (const diag of diagonals) {
            const captureSq = sq + diag;
            if (!isOffBoard(captureSq, filesBrd)) {
                // Citadel Check
                if (this.canEnterSquare(piece, captureSq, side)) {
                    const targetPiece = this.board.getPiece(captureSq);
                    const targetColor = PIECE_COLOR[targetPiece];
                    if (targetPiece !== PieceType.EMPTY && targetColor !== undefined && targetColor !== side) {
                        this.addCaptureMove(sq, captureSq, targetPiece);
                    }
                }
            }
        }
    }

    /**
     * Check if a square is a citadel
     */
    private isCitadel(sq: number): boolean {
        return this.board.isCitadel(sq);
    }

    /**
     * Check if a piece can enter the target square (Citadel rules)
     */
    private canEnterSquare(piece: PieceType, toSq: number, side: Color): boolean {
        if (!this.board.isCitadel(toSq)) return true;

        // It is a citadel.
        // Rule: Only King (or Adventitious King) can enter the OPPONENT'S citadel.
        // No piece can enter its OWN citadel.

        const isWhiteCitadel = this.board.isWhiteCitadel(toSq);
        const isBlackCitadel = this.board.isBlackCitadel(toSq);

        if (side === Color.WHITE) {
            // White piece
            if (isWhiteCitadel) return false; // Cannot enter own
            if (isBlackCitadel) {
                // Opponent's citadel. Only King/AdKing/Prince?
                return piece === PieceType.W_KING || piece === PieceType.W_ADKING || piece === PieceType.W_PRINCE;
            }
        } else {
            // Black piece
            if (isBlackCitadel) return false; // Cannot enter own
            if (isWhiteCitadel) {
                return piece === PieceType.B_KING || piece === PieceType.B_ADKING || piece === PieceType.B_PRINCE;
            }
        }
        return false;
    }

    /**
     * Generate non-sliding piece moves (knight, king, etc.)
     */
    private generateNonSlidingMoves(
        sq: number,
        directions: number[],
        side: Color,
        filesBrd: number[]
    ): void {
        for (const dir of directions) {
            const toSq = sq + dir;
            if (isOffBoard(toSq, filesBrd)) continue;

            // Citadel Check
            const piece = this.board.getPiece(sq);
            if (!this.canEnterSquare(piece, toSq, side)) continue;

            const targetPiece = this.board.getPiece(toSq);
            if (targetPiece === PieceType.EMPTY) {
                this.addMove(sq, toSq);
            } else {
                const targetColor = PIECE_COLOR[targetPiece];
                if (targetColor !== undefined && targetColor !== side) {
                    this.addCaptureMove(sq, toSq, targetPiece);
                }
            }
        }
    }

    /**
     * Generate sliding piece moves (rook)
     */
    private generateSlidingMoves(
        sq: number,
        directions: number[],
        side: Color,
        filesBrd: number[]
    ): void {
        const piece = this.board.getPiece(sq); // Get piece for citadel check

        for (const dir of directions) {
            let toSq = sq + dir;

            while (!isOffBoard(toSq, filesBrd)) {

                // Citadel Check (Block if cannot enter)
                if (!this.canEnterSquare(piece, toSq, side)) {
                    toSq += dir;
                    continue;
                }

                const targetPiece = this.board.getPiece(toSq);

                if (targetPiece === PieceType.EMPTY) {
                    this.addMove(sq, toSq);
                } else {
                    if (PIECE_COLOR[targetPiece] !== undefined && PIECE_COLOR[targetPiece] !== side) {
                        this.addCaptureMove(sq, toSq, targetPiece);
                    }
                    break; // Blocked by piece
                }

                toSq += dir;
            }
        }
    }

    /**
     * Add a quiet move to the move list
     */
    private addMove(from: number, to: number): void {
        this.moveList.push(createMove(from, to));
    }

    /**
     * Add a capture move to the move list
     */
    private addCaptureMove(from: number, to: number, captured: PieceType): void {
        this.moveList.push(createMove(from, to, captured));
    }

    /**
     * Tamerlane: King Switch Places
     * When king enters opponent citadel AND multiple kings exist, must switch with another king
     */
    private generateKingSwitchMoves(): void {
        const side = this.board.getSide();
        const pieces = this.board.getPieces();
        const kings = side === Color.WHITE ? this.board.getWhiteKings() : this.board.getBlackKings();

        if (kings.length <= 1) return; // Need at least 2 kings

        // Check if highest ranking king is in opponent citadel
        const highestKing = kings[0]; // Already sorted by priority in Board
        const kingSq = this.board.getPieceSquares(highestKing)[0];

        if (!kingSq) return;

        const inOpponentCitadel = (side === Color.WHITE && this.board.isBlackCitadel(kingSq)) ||
            (side === Color.BLACK && this.board.isWhiteCitadel(kingSq));

        if (!inOpponentCitadel) return;

        // Generate swap moves with other kings
        for (let i = 1; i < kings.length; i++) {
            const otherKing = kings[i];
            const otherSq = this.board.getPieceSquares(otherKing)[0];
            if (otherSq) {
                // Create swap move (king at citadel swaps with other king)
                this.moveList.push(createMove(kingSq, otherSq, PieceType.EMPTY, false, MOVE_FLAG_SWITCH_KING));
            }
        }
    }

    /**
     * Tamerlane: AdKing Citadel Escape
     * If AdKing starts in own citadel, must escape to Pawn of King's initial square
     * Returns true if this is a forced move (suppresses other moves)
     */
    private generateAdKingCitadelEscape(): boolean {
        const side = this.board.getSide();
        const pieces = this.board.getPieces();

        const adKingType = side === Color.WHITE ? PieceType.W_ADKING : PieceType.B_ADKING;
        const adKingSq = this.board.getPieceSquares(adKingType)[0];

        if (!adKingSq) return false;

        // Check if AdKing is in own citadel
        const inOwnCitadel = (side === Color.WHITE && this.board.isWhiteCitadel(adKingSq)) ||
            (side === Color.BLACK && this.board.isBlackCitadel(adKingSq));

        if (!inOwnCitadel) return false;

        // Must move to Pawn of King initial square (simplified: move towards center)
        // Reference code uses specific squares (92 for White, 177 for Black)
        const targetSq = side === Color.WHITE ? 92 : 177;

        if (pieces[targetSq] === PieceType.EMPTY) {
            this.moveList.push(createMove(adKingSq, targetSq, PieceType.EMPTY, false, MOVE_FLAG_ADKING_FROM_CITADEL));
        } else {
            this.moveList.push(createMove(adKingSq, targetSq, pieces[targetSq], false, MOVE_FLAG_ADKING_FROM_CITADEL));
        }

        return true; // Forced move
    }



    /**
     * Tamerlane: Pawn of King Forced Return to Initial Square
     * Reference: MoveToInitPosPawnofKing()
     * When wPromNumPofP==2, Pawn of King MUST return to initial square
     * Returns true if this is a forced move
     */
    private generatePawnOfKingReturn(): boolean {
        const side = this.board.getSide();
        const pieces = this.board.getPieces();
        const ranksBrd = this.board.getRanksBrd();
        const history = this.board.getMoveHistory();

        if (side === Color.WHITE) {
            const promCount = this.board.getWhitePawnOfPawnPromotions();
            if (promCount !== 2) return false;

            // Find Pawn of Pawn square
            const pawnOfPawnSq = this.board.getPieceSquares(PieceType.W_PAWN_PAWN)[0];
            if (!pawnOfPawnSq) return false;

            // Check if at promotion rank (10)
            if (ranksBrd[pawnOfPawnSq] !== 10) return false;

            // Check if king not in check (simplified - assume ok)
            // const kingSq = this.board.getPieceSquares(PieceType.W_KING)[0];
            // if (isSquareAttacked(this.board, kingSq, Color.BLACK)) return false;

            // Check previous move history (2 moves back)
            if (history.length >= 2) {
                const prevMove = history[history.length - 2];
                const prevFromRank = ranksBrd[prevMove.from];

                // WfromRank = 9 (reference code)
                if (prevFromRank === 9) {
                    // Forced return to initial square (97 for White Pawn of King)
                    const targetSq = 97;
                    const targetPiece = pieces[targetSq];

                    if (targetPiece === PieceType.EMPTY) {
                        this.moveList.push(createMove(pawnOfPawnSq, targetSq, PieceType.EMPTY, false, MOVE_FLAG_TO_BE_ADKING));
                    } else {
                        this.moveList.push(createMove(pawnOfPawnSq, targetSq, targetPiece, false, MOVE_FLAG_TO_BE_ADKING));
                    }
                    return true; // Forced move
                }
            }
        } else { // BLACK
            const promCount = this.board.getBlackPawnOfPawnPromotions();
            if (promCount !== 2) return false;

            const pawnOfPawnSq = this.board.getPieceSquares(PieceType.B_PAWN_PAWN)[0];
            if (!pawnOfPawnSq) return false;

            if (ranksBrd[pawnOfPawnSq] !== 1) return false;

            if (history.length >= 2) {
                const prevMove = history[history.length - 2];
                const prevFromRank = ranksBrd[prevMove.from];

                // BfromRank = 2
                if (prevFromRank === 2) {
                    const targetSq = 172; // Black Pawn of King initial square
                    const targetPiece = pieces[targetSq];

                    if (targetPiece === PieceType.EMPTY) {
                        this.moveList.push(createMove(pawnOfPawnSq, targetSq, PieceType.EMPTY, false, MOVE_FLAG_TO_BE_ADKING));
                    } else {
                        this.moveList.push(createMove(pawnOfPawnSq, targetSq, targetPiece, false, MOVE_FLAG_TO_BE_ADKING));
                    }
                    return true;
                }
            }
        }

        return false;
    }

    /**
     * Get the generated move list
     */
    getMoveList(): number[] {
        return this.moveList;
    }
}

export default MoveGenerator;
