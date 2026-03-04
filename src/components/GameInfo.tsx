
import React, { memo } from 'react';
import { Color } from '../core/types';
import { useTheme } from '../contexts/ThemeContext';

interface GameInfoProps {
    status: string;
    currentSide: Color;
    moveCount: number;
    whiteTime?: string;
    blackTime?: string;
    whiteName?: string;
    blackName?: string;
    opponentName?: string; // Fixed opponent name to always display
    connectionStatus?: 'connected' | 'disconnected' | 'reconnecting';
    isWhiteMe?: boolean;
    isBlackMe?: boolean;
}

/**
 * Game Info component - displays game status
 * Minimalist version (No header, no border)
 */
const GameInfo: React.FC<GameInfoProps> = memo(({ status, currentSide, moveCount, whiteTime, blackTime, whiteName, blackName, opponentName, connectionStatus, isWhiteMe, isBlackMe }) => {
    const { colors } = useTheme();
    const sideEmoji = currentSide === Color.WHITE ? '⚪' : '⚫';

    // Display opponent name FIXED at the top (doesn't change with turns)
    // If opponentName is provided, use it. Otherwise fallback to "Opponent"
    const displayName = opponentName || 'Opponent';

    return (
        <div className="game-info" style={{ padding: '0.25rem 0' }}>
            <div className="status-box" style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>

                {/* Players & Timers */}
                {(whiteTime || blackTime) && (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
                        {/* White Player */}
                        <div style={{
                            display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                            padding: '0.25rem 0.5rem', background: colors.card, borderRadius: '4px',
                            borderLeft: currentSide === Color.WHITE ? `3px solid ${colors.success}` : '3px solid transparent'
                        }}>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: colors.text, fontSize: '0.9rem' }}>
                                <span style={{ fontSize: '1.2em' }}>⚪</span>
                                <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'center', marginLeft: '4px' }}>
                                    <span style={{ fontWeight: '600', fontSize: '1rem', maxWidth: '120px', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', lineHeight: '1.2' }}>
                                        {whiteName || 'White'}
                                    </span>
                                    {connectionStatus && isWhiteMe && (
                                        <span style={{ 
                                            fontSize: '0.65rem', 
                                            fontWeight: 'bold',
                                            color: connectionStatus === 'connected' ? colors.success : (connectionStatus === 'reconnecting' ? '#f59e0b' : '#ef4444'),
                                            textTransform: 'uppercase',
                                            letterSpacing: '0.5px',
                                            marginTop: '2px'
                                        }}>
                                            {connectionStatus === 'connected' ? 'CONNECTED' : (connectionStatus === 'reconnecting' ? 'RECONNECTING' : 'OFFLINE')}
                                        </span>
                                    )}
                                </div>
                            </div>
                            <div style={{ fontFamily: 'monospace', fontWeight: 'bold', fontSize: '1.1rem', color: currentSide === Color.WHITE ? colors.success : colors.textSec }}>
                                {whiteTime || '--:--'}
                            </div>
                        </div>

                        {/* Black Player */}
                        <div style={{
                            display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                            padding: '0.25rem 0.5rem', background: colors.card, borderRadius: '4px',
                            borderLeft: currentSide === Color.BLACK ? `3px solid ${colors.success}` : '3px solid transparent'
                        }}>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: colors.text, fontSize: '0.9rem' }}>
                                <span style={{ fontSize: '1.2em' }}>⚫</span>
                                <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'center', marginLeft: '4px' }}>
                                    <span style={{ fontWeight: '600', fontSize: '1rem', maxWidth: '120px', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', lineHeight: '1.2' }}>
                                        {blackName || 'Black'}
                                    </span>
                                    {connectionStatus && isBlackMe && (
                                        <span style={{ 
                                            fontSize: '0.65rem', 
                                            fontWeight: 'bold',
                                            color: connectionStatus === 'connected' ? colors.success : (connectionStatus === 'reconnecting' ? '#f59e0b' : '#ef4444'),
                                            textTransform: 'uppercase',
                                            letterSpacing: '0.5px',
                                            marginTop: '2px'
                                        }}>
                                            {connectionStatus === 'connected' ? 'CONNECTED' : (connectionStatus === 'reconnecting' ? 'RECONNECTING' : 'OFFLINE')}
                                        </span>
                                    )}
                                </div>
                            </div>
                            <div style={{ fontFamily: 'monospace', fontWeight: 'bold', fontSize: '1.1rem', color: currentSide === Color.BLACK ? colors.success : colors.textSec }}>
                                {blackTime || '--:--'}
                            </div>
                        </div>
                    </div>
                )}

                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '0.9rem', marginTop: '0.25rem', padding: '0 0.5rem' }}>
                    <div className="current-turn" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                        <span className="emoji" style={{ fontSize: '1.2em' }}>⚔️</span>
                        <span className="text" style={{ fontWeight: 'bold', color: colors.text, fontSize: '1.1em' }}>{displayName}</span>
                    </div>
                    <div className="move-count" style={{ fontSize: '0.85rem', color: colors.textSec }}>
                        Move: {moveCount}
                    </div>
                </div>

                <div className="status-message" style={{ fontSize: '0.85rem', color: colors.textSec, padding: '0 0.5rem', lineHeight: '1.3' }}>
                    {status}
                </div>
            </div>
        </div>
    );
});

GameInfo.displayName = 'GameInfo';

export default GameInfo;
