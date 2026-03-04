import React, { memo, useMemo } from 'react';
import { PieceType, Color } from '../core/types';
import { PIECE_COLOR, PIECE_NAME } from '../core/constants';

interface PieceProps {
    type: PieceType;
}

/**
 * Piece component - displays piece image
 */
const Piece: React.FC<PieceProps> = memo(({ type }) => {
    const imageUrl = useMemo(() => {
        if (type === PieceType.EMPTY) return null;

        const color = PIECE_COLOR[type] === Color.WHITE ? 'w' : 'b';
        const name = PIECE_NAME[type];
        return `/images/${color}${name}.png`;
    }, [type]);

    if (!imageUrl) return null;

    return (
        <img
            className="piece"
            src={imageUrl}
            alt={PIECE_NAME[type]}
            draggable={false}
        />
    );
});

Piece.displayName = 'Piece';

export default Piece;
