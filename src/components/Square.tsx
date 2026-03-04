import React, { memo } from 'react';
import { PieceType } from '../core/types';
import Piece from './Piece';

interface SquareProps {
    square: number;
    piece: PieceType;
    isLight: boolean;
    isSelected: boolean;
    isValidMove: boolean;
    onClick: () => void;
    isCitadel?: boolean;
    citadelPosition?: 'left' | 'right';
    isInCheck?: boolean;
    style?: React.CSSProperties;
    customColor?: string;
}

/**
 * Square component - individual board square
 */
const Square: React.FC<SquareProps> = memo(({
    square,
    piece,
    isLight,
    isSelected,
    isValidMove,
    onClick,
    isCitadel = false,
    citadelPosition,
    isInCheck = false,
    style,
    customColor,
}) => {
    const classNames = [
        'square',
        !customColor && (isLight ? 'light' : 'dark'),
        isValidMove && 'valid-move',
        isCitadel && 'citadel',
        citadelPosition && `citadel-${citadelPosition}`,
        isInCheck && 'in-check',
    ].filter(Boolean).join(' ');

    const combinedStyle: React.CSSProperties = {
        ...style,
        ...(customColor ? { backgroundColor: customColor } : {})
    };

    return (
        <div
            className={classNames}
            data-square={square}
            onClick={onClick}
            style={combinedStyle}
        >
            {piece !== PieceType.EMPTY && <Piece type={piece} />}
            {isValidMove && piece === PieceType.EMPTY && <div className="move-indicator" />}
        </div>
    );
});

Square.displayName = 'Square';

export default Square;
