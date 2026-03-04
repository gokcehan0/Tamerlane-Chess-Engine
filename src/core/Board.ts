/**
 * Tamerlane Chess - Board Class
 * Represents the game board and piece positions
 */

import {
    Color,
    PieceType,
    Position,
    BOARD_SQUARES,
    NO_SQUARE,
    OFF_BOARD,
    CITADEL_WHITE,
    CITADEL_BLACK,
} from './types';
import { PIECE_COLOR, FEN_CHAR_TO_PIECE, START_FEN_WHITE } from './constants';

/**
 * Converts file and rank to square index
 */
export function fileRankToSquare(file: number, rank: number): number {
    return 46 + file + rank * 15;
}

/**
 * Gets file from square index
 */
export function getFile(sq: number, filesBrd: number[]): number {
    return filesBrd[sq];
}

/**
 * Gets rank from square index
 */
export function getRank(sq: number, ranksBrd: number[]): number {
    return ranksBrd[sq];
}

/**
 * Checks if square is off the board
 */
export function isOffBoard(sq: number, filesBrd: number[]): boolean {
    if (sq < 0 || sq >= BOARD_SQUARES) return true;
    return filesBrd[sq] === OFF_BOARD;
}

/**
 * Checks if square is a citadel
 */
export function isCitadel(sq: number): boolean {
    return sq === CITADEL_WHITE || sq === CITADEL_BLACK;
}

/**
 * Board class representing the game board
 */
export class Board {
    /** Piece on each square */
    private pieces: PieceType[];

    /** File index for each square */
    private filesBrd: number[];

    /** Rank index for each square */
    private ranksBrd: number[];

    /** Current side to move */
    private side: Color;

    /** Half-move count (ply) */
    private ply: number;

    /** Position hash key */
    private posKey: number;

    /** Piece list: stores squares for each piece type */
    private pieceList: Map<PieceType, number[]>;

    /** Number of each piece type on board */
    private pieceCount: Map<PieceType, number>;

    /** Track all kings for advanced Tamerlane rules */
    private whiteKings: PieceType[] = [];
    private blackKings: PieceType[] = [];

    /** Pawn of Pawn promotion counters (for special rules) */
    private whitePawnOfPawnPromotions: number = 0;
    private blackPawnOfPawnPromotions: number = 0;

    /** Move history for special pawn rules */
    private moveHistory: Array<{ from: number, to: number }> = [];

    constructor() {
        this.pieces = new Array(BOARD_SQUARES).fill(PieceType.EMPTY);
        this.filesBrd = new Array(BOARD_SQUARES).fill(OFF_BOARD);
        this.ranksBrd = new Array(BOARD_SQUARES).fill(OFF_BOARD);
        this.side = Color.WHITE;
        this.ply = 0;
        this.posKey = 0;
        this.pieceList = new Map();
        this.pieceCount = new Map();
        this.whiteKings = [];
        this.blackKings = [];

        this.initFilesRanks();
    }

    /**
     * Initialize file and rank arrays
     */
    private initFilesRanks(): void {
        // Reset all to off board
        for (let i = 0; i < BOARD_SQUARES; i++) {
            this.filesBrd[i] = OFF_BOARD;
            this.ranksBrd[i] = OFF_BOARD;
        }

        // Set up main board (11x10)
        for (let rank = 1; rank <= 10; rank++) {
            for (let file = 1; file <= 11; file++) {
                const sq = fileRankToSquare(file, rank);
                this.filesBrd[sq] = file;
                this.ranksBrd[sq] = rank;
            }
        }

        // Citadel squares
        this.filesBrd[CITADEL_BLACK] = 0;
        this.ranksBrd[CITADEL_BLACK] = 9;
        this.filesBrd[CITADEL_WHITE] = 12;
        this.ranksBrd[CITADEL_WHITE] = 2;
    }

    /**
     * Reset the board to empty state
     */
    reset(): void {
        this.pieces = new Array(BOARD_SQUARES).fill(PieceType.EMPTY);
        this.side = Color.WHITE;
        this.ply = 0;
        this.posKey = 0;
        this.pieceList.clear();
        this.pieceCount.clear();
        this.initFilesRanks();
    }

    /**
     * Parse FEN string and set up the board
     */
    parseFEN(fen: string): void {
        this.reset();

        let rank = 10;
        let file = 1;
        let fenIndex = 0;

        while (rank >= 1 && fenIndex < fen.length) {
            const char = fen[fenIndex];

            if (char === ' ') {
                break;
            } else if (char === '/') {
                rank--;
                file = 1;
            } else if (char >= '1' && char <= '9') {
                // Number indicates empty squares
                let count = parseInt(char);
                // Check for two-digit numbers
                if (fenIndex + 1 < fen.length && fen[fenIndex + 1] >= '0' && fen[fenIndex + 1] <= '9') {
                    count = count * 10 + parseInt(fen[fenIndex + 1]);
                    fenIndex++;
                }
                file += count;
            } else if (FEN_CHAR_TO_PIECE[char] !== undefined) {
                const piece = FEN_CHAR_TO_PIECE[char];
                const sq = fileRankToSquare(file, rank);
                this.setPiece(sq, piece);
                file++;
            }

            fenIndex++;
        }

        // Parse side to move
        fenIndex++; // Skip space
        if (fenIndex < fen.length) {
            this.side = fen[fenIndex] === 'w' ? Color.WHITE : Color.BLACK;
        }
    }

    /**
     * Generate FEN string from current board state
     */
    generateFEN(): string {
        let fen = '';

        for (let rank = 10; rank >= 1; rank--) {
            let emptyCount = 0;

            for (let file = 1; file <= 11; file++) {
                const sq = fileRankToSquare(file, rank);
                const piece = this.pieces[sq];

                if (piece === PieceType.EMPTY) {
                    emptyCount++;
                } else {
                    if (emptyCount > 0) {
                        fen += emptyCount.toString();
                        emptyCount = 0;
                    }
                    // Find char for piece
                    const char = Object.keys(FEN_CHAR_TO_PIECE).find(key => FEN_CHAR_TO_PIECE[key] === piece);
                    fen += char || '?';
                }
            }

            if (emptyCount > 0) {
                fen += emptyCount.toString();
            }

            if (rank > 1) {
                fen += '/';
            }
        }

        fen += ' ' + (this.side === Color.WHITE ? 'w' : 'b');
        return fen;
    }

    /**
     * Set a piece on the board
     */
    setPiece(sq: number, piece: PieceType): void {
        this.pieces[sq] = piece;

        if (piece !== PieceType.EMPTY) {
            if (!this.pieceList.has(piece)) {
                this.pieceList.set(piece, []);
            }
            this.pieceList.get(piece)?.push(sq);

            const count = this.pieceCount.get(piece) || 0;
            this.pieceCount.set(piece, count + 1);
        }
    }

    /**
     * Clear a piece from the board
     */
    clearPiece(sq: number): void {
        const piece = this.pieces[sq];
        if (piece !== PieceType.EMPTY) {
            this.pieces[sq] = PieceType.EMPTY;

            const list = this.pieceList.get(piece);
            if (list) {
                const index = list.indexOf(sq);
                if (index !== -1) {
                    list.splice(index, 1);
                }
            }

            const count = this.pieceCount.get(piece) || 0;
            if (count > 0) {
                this.pieceCount.set(piece, count - 1);
            }
        }
    }

    /**
     * Move a piece from one square to another
     */
    movePiece(from: number, to: number): void {
        const piece = this.pieces[from];
        this.clearPiece(from);
        this.clearPiece(to); // Capture if any
        this.setPiece(to, piece);
    }

    /**
     * Get piece at square
     */
    getPiece(sq: number): PieceType {
        return this.pieces[sq];
    }

    /**
     * Get all pieces array
     */
    getPieces(): PieceType[] {
        return this.pieces;
    }

    /**
     * Get squares occupied by a specific piece type
     */
    getPieceSquares(piece: PieceType): number[] {
        return this.pieceList.get(piece) || [];
    }

    /**
     * Get files board array
     */
    getFilesBrd(): number[] {
        return this.filesBrd;
    }

    /**
     * Get ranks board array
     */
    getRanksBrd(): number[] {
        return this.ranksBrd;
    }

    /**
     * Get current side to move
     */
    getSide(): Color {
        return this.side;
    }

    /**
     * Set side to move
     */
    setSide(side: Color): void {
        this.side = side;
    }

    /**
     * Switch side to move
     */
    switchSide(): void {
        this.side = this.side === Color.WHITE ? Color.BLACK : Color.WHITE;
    }

    /**
     * Set up standard starting position
     */
    setupStartPosition(): void {
        this.parseFEN(START_FEN_WHITE);
    }

    /**
     * Check if a square is a citadel
     */
    isCitadel(sq: number): boolean {
        return sq === CITADEL_WHITE || sq === CITADEL_BLACK;
    }

    /**
     * Check if a square is the white citadel
     */
    isWhiteCitadel(sq: number): boolean {
        return sq === CITADEL_WHITE;
    }

    /**
     * Check if a square is the black citadel
     */
    isBlackCitadel(sq: number): boolean {
        return sq === CITADEL_BLACK;
    }

    /**
     * Get all white royal pieces (King, Prince, AdKing)
     */
    getWhiteKings(): PieceType[] {
        return this.whiteKings;
    }

    /**
     * Get all black royal pieces (King, Prince, AdKing)
     */
    getBlackKings(): PieceType[] {
        return this.blackKings;
    }

    /**
     * Update kings list - call after piece moves
     */
    updateKingsList(): void {
        this.whiteKings = [];
        this.blackKings = [];

        // Check all royal pieces
        const royalTypes = [
            PieceType.W_KING, PieceType.W_PRINCE, PieceType.W_ADKING,
            PieceType.B_KING, PieceType.B_PRINCE, PieceType.B_ADKING
        ];

        for (const royal of royalTypes) {
            const count = this.pieceCount.get(royal) || 0;
            if (count > 0) {
                if (royal === PieceType.W_KING || royal === PieceType.W_PRINCE || royal === PieceType.W_ADKING) {
                    this.whiteKings.push(royal);
                } else {
                    this.blackKings.push(royal);
                }
            }
        }
    }

    /**
     * Get Pawn of Pawn promotion count
     */
    getWhitePawnOfPawnPromotions(): number {
        return this.whitePawnOfPawnPromotions;
    }

    getBlackPawnOfPawnPromotions(): number {
        return this.blackPawnOfPawnPromotions;
    }

    /**
     * Increment Pawn of Pawn promotion counter
     */
    incrementWhitePawnOfPawnPromotions(): void {
        this.whitePawnOfPawnPromotions++;
    }

    incrementBlackPawnOfPawnPromotions(): void {
        this.blackPawnOfPawnPromotions++;
    }

    /**
     * Add move to history
     */
    addMoveToHistory(from: number, to: number): void {
        this.moveHistory.push({ from, to });
    }

    /**
     * Get move history
     */
    getMoveHistory(): Array<{ from: number, to: number }> {
        return this.moveHistory;
    }

    /**
     * Create a deep copy of the board
     */
    clone(): Board {
        const newBoard = new Board();
        
        // Copy primitive properties
        newBoard.side = this.side;
        newBoard.ply = this.ply;
        newBoard.posKey = this.posKey;
        newBoard.whitePawnOfPawnPromotions = this.whitePawnOfPawnPromotions;
        newBoard.blackPawnOfPawnPromotions = this.blackPawnOfPawnPromotions;

        // Copy Arrays (Deep copy needed for pieces?)
        // Pieces is array of numbers (primitive), so slice/spread is fine
        newBoard.pieces = [...this.pieces];
        newBoard.filesBrd = [...this.filesBrd];
        newBoard.ranksBrd = [...this.ranksBrd];

        // Copy Maps
        // pieceList is Map<PieceType, number[]>
        newBoard.pieceList = new Map();
        this.pieceList.forEach((val, key) => {
            newBoard.pieceList.set(key, [...val]);
        });
        
        // pieceCount is Map<PieceType, number>
        newBoard.pieceCount = new Map(this.pieceCount);

        // Copy Special Arrays
        newBoard.whiteKings = [...this.whiteKings];
        newBoard.blackKings = [...this.blackKings];
        newBoard.moveHistory = this.moveHistory.map(m => ({ ...m }));

        return newBoard;
    }
}
