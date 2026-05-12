/**
 * Bot Game Statistics Service
 * Records game results to Firebase Realtime Database.
 * Only active when VITE_STATS_DB_URL environment variable is set.
 */

import { PIECE_NAME, PIECE_COLOR } from './constants';
import { PieceType, Color, OFF_BOARD, CITADEL_WHITE, CITADEL_BLACK } from './types';

// Firebase config from environment variables
const STATS_API_KEY = import.meta.env.VITE_STATS_API_KEY || '';
const STATS_DB_URL = import.meta.env.VITE_STATS_DB_URL || '';
const STATS_PROJECT_ID = import.meta.env.VITE_STATS_PROJECT_ID || '';

/** Check if stats tracking is enabled */
function isEnabled(): boolean {
    return !!(STATS_DB_URL && STATS_API_KEY);
}

/**
 * Convert internal square index to human-readable notation.
 * Board is 11x10 with files a-k and ranks 1-10.
 * Internal formula: sq = 46 + file + rank * 15 (file: 1-11, rank: 1-10)
 * Citadels: White = sq 88 (file 12, rank 2), Black = sq 181 (file 0, rank 9)
 */
function squareToNotation(sq: number, filesBrd: number[], ranksBrd: number[]): string {
    const file = filesBrd[sq];
    const rank = ranksBrd[sq];

    if (file === OFF_BOARD || rank === OFF_BOARD) return '??';

    // Citadel squares
    if (sq === CITADEL_WHITE) return 'CW';
    if (sq === CITADEL_BLACK) return 'CB';

    // Files: 1=a, 2=b, ..., 11=k
    const fileChar = String.fromCharCode(96 + file); // 97='a', so 96+1='a'
    return `${fileChar}${rank}`;
}

/**
 * Get piece abbreviation for notation
 */
function getPieceAbbr(piece: PieceType): string {
    switch (piece) {
        // Major pieces
        case PieceType.W_KING: case PieceType.B_KING: return 'K';
        case PieceType.W_PRINCE: case PieceType.B_PRINCE: return 'Pr';
        case PieceType.W_ADKING: case PieceType.B_ADKING: return 'AK';
        case PieceType.W_ROOK: case PieceType.B_ROOK: return 'R';
        case PieceType.W_KNIGHT: case PieceType.B_KNIGHT: return 'N';
        case PieceType.W_MINISTER: case PieceType.B_MINISTER: return 'M';
        case PieceType.W_ADVISOR: case PieceType.B_ADVISOR: return 'A';
        case PieceType.W_ELEPHANT: case PieceType.B_ELEPHANT: return 'E';
        case PieceType.W_CAMEL: case PieceType.B_CAMEL: return 'C';
        case PieceType.W_GIRAFFE: case PieceType.B_GIRAFFE: return 'G';
        case PieceType.W_CATAPULT: case PieceType.B_CATAPULT: return 'T';
        case PieceType.W_WARENGINE: case PieceType.B_WARENGINE: return 'W';
        // Pawns - use lowercase
        default: return '';
    }
}

export interface MoveRecord {
    from: number;
    to: number;
    piece: PieceType;
    captured?: PieceType;
}

export interface GameResult {
    result: 'checkmate' | 'stalemate';
    winner: 'white' | 'black' | 'draw';
    difficulty: string;
    moveCount: number;
    durationSeconds: number;
    moves: MoveRecord[];
    filesBrd: number[];
    ranksBrd: number[];
}

/**
 * Format a single move into human-readable notation
 */
function formatMove(move: MoveRecord, filesBrd: number[], ranksBrd: number[]): string {
    const pieceAbbr = getPieceAbbr(move.piece);
    const fromSq = squareToNotation(move.from, filesBrd, ranksBrd);
    const toSq = squareToNotation(move.to, filesBrd, ranksBrd);
    const capture = move.captured ? 'x' : '-';
    return `${pieceAbbr}${fromSq}${capture}${toSq}`;
}

/**
 * Format entire move list into notation string
 * Example: "1. e2-e4 e7-e5 2. Ng1-f3 Nb8-c6"
 */
function formatNotation(moves: MoveRecord[], filesBrd: number[], ranksBrd: number[]): string {
    const parts: string[] = [];
    for (let i = 0; i < moves.length; i += 2) {
        const moveNum = Math.floor(i / 2) + 1;
        const whiteMove = formatMove(moves[i], filesBrd, ranksBrd);
        const blackMove = i + 1 < moves.length ? ' ' + formatMove(moves[i + 1], filesBrd, ranksBrd) : '';
        parts.push(`${moveNum}. ${whiteMove}${blackMove}`);
    }
    return parts.join(' ');
}

/**
 * Save a completed bot game result to Firebase Realtime Database.
 * Uses REST API directly to avoid requiring the full Firebase SDK.
 */
export async function saveBotGameResult(result: GameResult): Promise<void> {
    if (!isEnabled()) {
        console.log('[Stats] Tracking disabled (no env vars)');
        return;
    }

    try {
        const notation = formatNotation(result.moves, result.filesBrd, result.ranksBrd);
        
        // Also create a simple move list for easy parsing
        const moveList = result.moves.map(m => 
            formatMove(m, result.filesBrd, result.ranksBrd)
        );

        const payload = {
            timestamp: Date.now(),
            date: new Date().toISOString(),
            result: result.result,
            winner: result.winner,
            difficulty: result.difficulty,
            moveCount: result.moveCount,
            durationSeconds: result.durationSeconds,
            notation,       // "1. e2-e4 e7-e5 2. ..."
            moveList,       // ["e2-e4", "e7-e5", ...]
        };

        // Use Firebase REST API to push new entry
        const url = `${STATS_DB_URL}/games.json?key=${STATS_API_KEY}`;
        const response = await fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
        });

        if (!response.ok) {
            console.warn('[Stats] Failed to save:', response.status, await response.text());
        } else {
            console.log('[Stats] Game result saved successfully');
        }
    } catch (err) {
        // Silently fail - stats should never break the game
        console.warn('[Stats] Error saving result:', err);
    }
}
