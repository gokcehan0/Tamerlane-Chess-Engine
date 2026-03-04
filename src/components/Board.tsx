import React, { memo } from 'react';
import { PieceType, CITADEL_WHITE, CITADEL_BLACK } from '../core/types';
import Square from './Square';
import { fileRankToSquare } from '../core/Board';
import { useTheme } from '../contexts/ThemeContext';

interface BoardProps {
    pieces: PieceType[];
    filesBrd: number[];
    ranksBrd: number[];
    selectedSquare: number | null;
    validMoves: number[];
    checkSquare?: number | null; // Square of the king in check
    onSquareClick: (square: number) => void;
    isFlipped?: boolean; // True when playing as black
    isReadOnly?: boolean; // True for spectators
}

/**
 * Board component - renders the 11x10 Tamerlane chess board
 * Supports flipped view for black player
 */
const Board: React.FC<BoardProps> = memo(({
    pieces,
    filesBrd,
    ranksBrd,
    selectedSquare,
    validMoves,
    checkSquare,
    onSquareClick,
    isFlipped = false,
    isReadOnly = false,
}) => {
    const { boardTheme } = useTheme();

    const squares: React.ReactNode[] = [];

    // Render main board (11x10)
    // When flipped, we render in reverse order
    if (isFlipped) {
        // Black's perspective: rank 1 at top, rank 10 at bottom
        for (let rank = 1; rank <= 10; rank++) {
            for (let file = 11; file >= 1; file--) {
                const sq = fileRankToSquare(file, rank);
                const isLight = (file + rank) % 2 === 0;

                squares.push(
                    <Square
                        key={sq}
                        square={sq}
                        piece={pieces[sq]}
                        isLight={isLight}
                        isSelected={selectedSquare === sq}
                        isValidMove={validMoves.includes(sq)}
                        isInCheck={checkSquare === sq}
                        onClick={() => onSquareClick(sq)}
                        customColor={isLight ? boardTheme.lightSquare : boardTheme.darkSquare}
                    />
                );
            }
        }
    } else {
        // White's perspective: rank 10 at top, rank 1 at bottom
        for (let rank = 10; rank >= 1; rank--) {
            for (let file = 1; file <= 11; file++) {
                const sq = fileRankToSquare(file, rank);
                const isLight = (file + rank) % 2 === 0;

                squares.push(
                    <Square
                        key={sq}
                        square={sq}
                        piece={pieces[sq]}
                        isLight={isLight}
                        isSelected={selectedSquare === sq}
                        isValidMove={validMoves.includes(sq)}
                        isInCheck={checkSquare === sq}
                        onClick={() => onSquareClick(sq)}
                        customColor={isLight ? boardTheme.lightSquare : boardTheme.darkSquare}
                    />
                );
            }
        }
    }

    // Citadel positions change based on flip
    const leftCitadel = isFlipped ? CITADEL_WHITE : CITADEL_BLACK;
    const rightCitadel = isFlipped ? CITADEL_BLACK : CITADEL_WHITE;
    const leftCitadelLight = isFlipped ? true : false;
    const rightCitadelLight = isFlipped ? false : true;

    return (
        <div className={`board-container ${isFlipped ? 'flipped' : ''}`}>
            <div className="board">
                {squares}
            </div>

            {/* Citadel squares - positioned precisely using percentages relative to the 11x10 grid */}
            
            {/* White Citadel: Connected to Rank 2, Right side (standard) */
            /* Standard: Right of File 11. Top: ~80% (Rank 8 from top? No, Rank 2 is 2nd from bottom). 
               10 Ranks => 10% each. Rank 1 (100-90%), Rank 2 (90-80%). So Top: 80%.
               File: Right side (100%).
            */
            /* Flipped: Connected to Rank 2.
               Rank 1 is Top (0-10%). Rank 2 is (10-20%). Top: 10%.
               Files: 11 (Left) -> 1 (Right).
               White Citadel (File 12?) would be Left of File 11.
               So Left: -9.09%.
            */}

            <div className="citadels">
                {/* Left Citadel Logic */}
                <Square
                    key={leftCitadel}
                    square={leftCitadel}
                    piece={pieces[leftCitadel]}
                    isLight={leftCitadelLight}
                    isSelected={selectedSquare === leftCitadel}
                    isValidMove={validMoves.includes(leftCitadel)}
                    onClick={() => { if (!isReadOnly) onSquareClick(leftCitadel); }}
                    isCitadel
                    citadelPosition="left"
                    style={{
                        position: 'absolute',
                        width: '9.09%', 
                        height: '10%',
                        cursor: isReadOnly ? 'default' : 'pointer',
                        // Dynamic Positioning
                        left: isFlipped ? '-9.09%' : '-9.09%', // Always sticking out left?
                        // Wait.
                        // Standard: Black Citadel is Left of Rank 9. Rank 9 is 2nd from top. Top: 10%. Left: -9.09%. Correct.
                        // Flipped: White Citadel is Left of Rank 2. Rank 2 is 2nd from top. Top: 10%. Left: -9.09%. Correct.
                        top: '10%', 
                        border: '1px solid #444' // Visual clarity
                    }}
                    customColor={leftCitadelLight ? boardTheme.lightSquare : boardTheme.darkSquare}
                />

                {/* Right Citadel Logic */}
                <Square
                    key={rightCitadel}
                    square={rightCitadel}
                    piece={pieces[rightCitadel]}
                    isLight={rightCitadelLight}
                    isSelected={selectedSquare === rightCitadel}
                    isValidMove={validMoves.includes(rightCitadel)}
                    onClick={() => { if (!isReadOnly) onSquareClick(rightCitadel); }}
                    isCitadel
                    citadelPosition="right"
                    style={{
                        position: 'absolute',
                        width: '9.09%',
                        height: '10%',
                        cursor: isReadOnly ? 'default' : 'pointer',
                        // Dynamic Positioning
                        right: '-9.09%', // Always sticking out right?
                        // Standard: White Citadel is Right of Rank 2. Rank 2 is 2nd from bottom. Top: 80%. Right: -9.09%.
                        // Flipped: Black Citadel is Right of Rank 9. Rank 9 is 2nd from Bottom. Top: 80%. Right: -9.09%.
                        top: '80%',
                        border: '1px solid #444'
                    }}
                    customColor={rightCitadelLight ? boardTheme.lightSquare : boardTheme.darkSquare}
                />
            </div>
        </div>
    );
});

Board.displayName = 'Board';

export default Board;
