import React from 'react';
import { PieceType } from '../core/types';
import Piece from './Piece';
import { useTheme } from '../contexts/ThemeContext';

interface CapturedPiecesProps {
    whiteCaptured: PieceType[]; // Pieces captured BY White (Black pieces)
    blackCaptured: PieceType[]; // Pieces captured BY Black (White pieces)
}

export default function CapturedPieces({ whiteCaptured, blackCaptured }: CapturedPiecesProps) {
    const { colors } = useTheme();

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', width: '100%', marginBottom: '1rem' }}>
            {/* Pieces captured by Black (White pieces lost) */}
            <div style={{ padding: '0.5rem', background: colors.card, borderRadius: '6px', border: `1px solid ${colors.border}` }}>
                <div style={{ fontSize: '0.75rem', color: colors.textSec, marginBottom: '0.25rem' }}>Captured by Black</div>
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '2px', minHeight: '24px' }}>
                    {blackCaptured.map((piece, idx) => (
                        <div key={idx} style={{ width: '24px', height: '24px' }}>
                            <Piece type={piece} />
                        </div>
                    ))}
                </div>
            </div>

            {/* Pieces captured by White (Black pieces lost) */}
            <div style={{ padding: '0.5rem', background: colors.card, borderRadius: '6px', border: `1px solid ${colors.border}` }}>
                <div style={{ fontSize: '0.75rem', color: colors.textSec, marginBottom: '0.25rem' }}>Captured by White</div>
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '2px', minHeight: '24px' }}>
                    {whiteCaptured.map((piece, idx) => (
                        <div key={idx} style={{ width: '24px', height: '24px' }}>
                            <Piece type={piece} />
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
