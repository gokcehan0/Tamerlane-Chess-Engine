import React, { useState, useEffect, useReducer, useRef } from 'react';
import BoardComponent from './Board';
import { TamerlaneEngine } from '../core/TamerlaneEngine';
type Difficulty = 'medium' | 'master';
import { getLegalMoves } from '../core/GameLogic';
import { MoveGenerator, getFromSquare, getToSquare, createMove, isPromotion } from '../core/MoveGenerator';
import { Board } from '../core/Board';
import { Color, PieceType } from '../core/types';
import GameInfo from './GameInfo';
import CapturedPieces from './CapturedPieces';
import { getGameStatus, GameStatus } from '../core/GameLogic';

interface GameState {
    boardObj: Board;
    selectedSquare: number | null;
    validMoves: number[];
    isGameOver: boolean;
    gameStatusMsg: string;
    whiteCaptured: PieceType[];
    blackCaptured: PieceType[];
    checkSquare?: number;
    moveCount: number;
}

type GameAction = 
    | { type: 'SELECT_SQUARE'; square: number }
    | { type: 'MAKE_MOVE'; from: number; to: number; piece: PieceType; isPromotion?: boolean }
    | { type: 'NEW_GAME' };

function gameReducer(state: GameState, action: GameAction): GameState {
    switch (action.type) {
        case 'SELECT_SQUARE': {
            const { square } = action;
            const piece = state.boardObj.getPiece(square);
            const isSelectable = piece !== PieceType.EMPTY && state.boardObj.getSide() === (PIECE_COLOR[piece] || Color.WHITE);

            if (isSelectable) {
                const moveGen = new MoveGenerator(state.boardObj);
                const legalMoves = getLegalMoves(state.boardObj, moveGen);
                const pieceMoves = legalMoves
                    .filter(m => getFromSquare(m) === square)
                    .map(m => getToSquare(m));

                return { ...state, selectedSquare: square, validMoves: pieceMoves };
            }
            return { ...state, selectedSquare: null, validMoves: [] };
        }
        case 'MAKE_MOVE': {
            const { from, to, piece } = action;
            
            // Clone board
            const newBoard = state.boardObj.clone ? state.boardObj.clone() : new Board();
            if (!state.boardObj.clone) {
                 newBoard.parseFEN(state.boardObj.generateFEN());
            }
            
            // Apply standard move logic
            
            // Check for capture
            let capturedPiece = newBoard.getPiece(to);
            if (capturedPiece === PieceType.EMPTY) {
                 const moveGen = new MoveGenerator(newBoard);
                 const allMoves = getLegalMoves(newBoard, moveGen);
                 const actualMove = allMoves.find(m => getFromSquare(m) === from && getToSquare(m) === to);
                 if (actualMove) {
                     capturedPiece = (actualMove >> 16) & 0x3f;
                 }
            }

            newBoard.movePiece(from, to);
            // Check for promotion via explicit flag from move generator / engine
            let finalPiece = piece;
            if (action.isPromotion) {
                // Determine promoted piece based on pawn type
                switch (piece) {
                    case PieceType.W_PAWN_PAWN: finalPiece = PieceType.W_ADKING; break;
                    case PieceType.W_PAWN_WARENGINE: finalPiece = PieceType.W_WARENGINE; break;
                    case PieceType.W_PAWN_CAMEL: finalPiece = PieceType.W_CAMEL; break;
                    case PieceType.W_PAWN_ELEPHANT: finalPiece = PieceType.W_ELEPHANT; break;
                    case PieceType.W_PAWN_MINISTER: finalPiece = PieceType.W_MINISTER; break;
                    case PieceType.W_PAWN_KING: finalPiece = PieceType.W_PRINCE; break;
                    case PieceType.W_PAWN_ADVISOR: finalPiece = PieceType.W_ADVISOR; break;
                    case PieceType.W_PAWN_GIRAFFE: finalPiece = PieceType.W_GIRAFFE; break;
                    case PieceType.W_PAWN_CATAPULT: finalPiece = PieceType.W_CATAPULT; break;
                    case PieceType.W_PAWN_KNIGHT: finalPiece = PieceType.W_KNIGHT; break;
                    case PieceType.W_PAWN_ROOK: finalPiece = PieceType.W_ROOK; break;
                    
                    case PieceType.B_PAWN_PAWN: finalPiece = PieceType.B_ADKING; break;
                    case PieceType.B_PAWN_WARENGINE: finalPiece = PieceType.B_WARENGINE; break;
                    case PieceType.B_PAWN_CAMEL: finalPiece = PieceType.B_CAMEL; break;
                    case PieceType.B_PAWN_ELEPHANT: finalPiece = PieceType.B_ELEPHANT; break;
                    case PieceType.B_PAWN_MINISTER: finalPiece = PieceType.B_MINISTER; break;
                    case PieceType.B_PAWN_KING: finalPiece = PieceType.B_PRINCE; break;
                    case PieceType.B_PAWN_ADVISOR: finalPiece = PieceType.B_ADVISOR; break;
                    case PieceType.B_PAWN_GIRAFFE: finalPiece = PieceType.B_GIRAFFE; break;
                    case PieceType.B_PAWN_CATAPULT: finalPiece = PieceType.B_CATAPULT; break;
                    case PieceType.B_PAWN_KNIGHT: finalPiece = PieceType.B_KNIGHT; break;
                    case PieceType.B_PAWN_ROOK: finalPiece = PieceType.B_ROOK; break;
                }
                // Overwrite the piece on the board with the newly promoted piece
                // We clear 'to' square and place the final piece
                newBoard.clearPiece(to);
                newBoard.setPiece(to, finalPiece);
            }

            newBoard.switchSide();
            
            // Track state for UI
            const nextMoveGen = new MoveGenerator(newBoard);
            const statusObj = getGameStatus(newBoard, nextMoveGen);
            let statusMsg = newBoard.getSide() === Color.WHITE ? 'White to move' : 'Black to move';
            let isGameOver = false;
            let currentCheckSquare = statusObj.checkSquare;

            if (statusObj.status === GameStatus.CHECK) {
                statusMsg = `⚠️ Check!`;
            } else if (statusObj.status === GameStatus.CHECKMATE) {
                statusMsg = `🏆 Checkmate! ${statusObj.winner === Color.WHITE ? 'White' : 'Black'} wins!`;
                isGameOver = true;
            } else if (statusObj.status === GameStatus.STALEMATE) {
                statusMsg = `🤝 Stalemate! Draw.`;
                isGameOver = true;
            }

            // Track Captures
            const newWhiteCaptured = [...state.whiteCaptured];
            const newBlackCaptured = [...state.blackCaptured];
            if (capturedPiece !== PieceType.EMPTY) {
                if (newBoard.getSide() === Color.WHITE) {
                    newBlackCaptured.push(capturedPiece); // Black just moved and captured
                } else {
                    newWhiteCaptured.push(capturedPiece); // White just moved and captured
                }
            }

            return {
                ...state,
                boardObj: newBoard,
                selectedSquare: null,
                validMoves: [],
                isGameOver,
                gameStatusMsg: statusMsg,
                moveCount: state.moveCount + 1,
                whiteCaptured: newWhiteCaptured,
                blackCaptured: newBlackCaptured,
                checkSquare: currentCheckSquare
            };
        }
        case 'NEW_GAME': {
            const newBoard = new Board();
            newBoard.setupStartPosition();
            return { 
                boardObj: newBoard, 
                selectedSquare: null, 
                validMoves: [], 
                isGameOver: false,
                gameStatusMsg: 'White to move',
                moveCount: 0,
                whiteCaptured: [],
                blackCaptured: [],
                checkSquare: undefined
            };
        }
        default:
            return state;
    }
}

// Helper for piece colors
const PIECE_COLOR: Record<PieceType, Color> = {} as any;
for (let key in PieceType) {
    if (isNaN(Number(key))) continue;
    const type = Number(key) as PieceType;
    PIECE_COLOR[type] = type <= PieceType.W_WARENGINE || type === PieceType.W_PRINCE || type === PieceType.W_ADKING ? Color.WHITE : Color.BLACK;
}
PIECE_COLOR[PieceType.EMPTY] = Color.BOTH;

export default function Game() {
    const [state, dispatch] = useReducer(gameReducer, null as any);
    const [difficulty, setDifficulty] = useState<Difficulty>('master');
    const engineRef = useRef<TamerlaneEngine | null>(null);

    // Initialize Game & Bot
    useEffect(() => {
        if (!state) {
            dispatch({ type: 'NEW_GAME' });
        }
        if (!engineRef.current) {
            engineRef.current = new TamerlaneEngine();
            // Wait for it to be ready
            engineRef.current.waitReady().then(() => console.log('Tamerlane Engine Ready'));
        }
    }, [difficulty, state]);

    // Bot Move Logic
    useEffect(() => {
        if (state && state.boardObj.getSide() === Color.BLACK) {
            const bot = engineRef.current;
            if (bot) {
                const searchTime = 2000; // Give the engine 2 seconds to think
                const diffLevel = difficulty === 'master' ? 1 : 0;
                bot.getBestMove(state.boardObj, searchTime, diffLevel).then((move) => {
                    if (move) {
                        const piece = state.boardObj.getPiece(move.from);
                        dispatch({ type: 'MAKE_MOVE', from: move.from, to: move.to, piece, isPromotion: move.isPromotion });
                    } else {
                        console.log('Bot has no moves - game over');
                    }
                });
            }
        }
    }, [state?.boardObj]);

    if (!state) return <div>Loading...</div>;

    const handleSquareClick = (square: number) => {
        if (state.boardObj.getSide() !== Color.WHITE) return; // Wait for bot

        if (state.selectedSquare !== null && state.validMoves.includes(square)) {
            const piece = state.boardObj.getPiece(state.selectedSquare);
            // Check if player move is a promotion
            const isPromo = isPromotion(state.validMoves.find(m => getFromSquare(m) === state.selectedSquare && getToSquare(m) === square) || 0);
            dispatch({ type: 'MAKE_MOVE', from: state.selectedSquare, to: square, piece, isPromotion: isPromo });
        } else {
            dispatch({ type: 'SELECT_SQUARE', square });
        }
    };

    return (
        <div className="app">
            <div className="main-container">
                <aside className="sidebar" style={{ minWidth: '300px' }}>
                    <GameInfo
                        status={state.gameStatusMsg}
                        currentSide={state.boardObj.getSide()}
                        moveCount={state.moveCount}
                        whiteName="You"
                        blackName="Tamerlane Bot"
                        opponentName="Tamerlane Bot"
                        isWhiteMe={true}
                        isBlackMe={false}
                    />

                    <CapturedPieces whiteCaptured={state.whiteCaptured} blackCaptured={state.blackCaptured} />

                    <div className="controls">
                        {/* Bot Game Over Banner */}
                        {state && state.isGameOver && (
                            <div style={{
                                padding: '1rem',
                                marginBottom: '1rem',
                                background: state.boardObj.getSide() === Color.WHITE ? 'rgba(239, 68, 68, 0.15)' : 'rgba(34, 197, 94, 0.15)',
                                borderRadius: '8px',
                                border: `1px solid ${state.boardObj.getSide() === Color.WHITE ? '#ef4444' : '#22c55e'}`,
                                textAlign: 'center'
                            }}>
                                <h3 style={{
                                    color: state.boardObj.getSide() === Color.WHITE ? '#f87171' : '#4ade80',
                                    marginBottom: '0.25rem',
                                    fontSize: '1.5rem',
                                    textTransform: 'uppercase',
                                    letterSpacing: '1px'
                                }}>
                                    {state.boardObj.getSide() === Color.WHITE ? '💀 DEFEAT' : '🏆 VICTORY'}
                                </h3>
                                <p style={{ color: '#ccc', fontSize: '0.9rem', margin: 0 }}>
                                    Checkmate!
                                </p>
                            </div>
                        )}

                        {/* Difficulty Selector */}
                        <div style={{ marginBottom: '0.75rem', display: 'flex', gap: '0.5rem' }}>
                            <button
                                onClick={() => setDifficulty('medium')}
                                style={{
                                    flex: 1,
                                    padding: '0.5rem',
                                    background: difficulty === 'medium' ? '#f59e0b' : '#2a2a2a',
                                    color: difficulty === 'medium' ? '#000' : '#aaa',
                                    border: `1px solid ${difficulty === 'medium' ? '#f59e0b' : '#444'}`,
                                    borderRadius: '6px',
                                    cursor: 'pointer',
                                    fontWeight: difficulty === 'medium' ? 'bold' : 'normal',
                                    fontSize: '0.85rem',
                                    transition: 'all 0.2s'
                                }}
                            >
                                ⚔️ Medium
                            </button>
                            <button
                                onClick={() => setDifficulty('master')}
                                style={{
                                    flex: 1,
                                    padding: '0.5rem',
                                    background: difficulty === 'master' ? '#ef4444' : '#2a2a2a',
                                    color: difficulty === 'master' ? '#fff' : '#aaa',
                                    border: `1px solid ${difficulty === 'master' ? '#ef4444' : '#444'}`,
                                    borderRadius: '6px',
                                    cursor: 'pointer',
                                    fontWeight: difficulty === 'master' ? 'bold' : 'normal',
                                    fontSize: '0.85rem',
                                    transition: 'all 0.2s'
                                }}
                            >
                                🔥 Master
                            </button>
                        </div>
                        
                        <div style={{ marginBottom: '0.75rem', display: 'flex', alignItems: 'center', gap: '0.5rem', padding: '0.5rem', background: '#1e1e1e', borderRadius: '6px', border: '1px solid #333' }}>
                            <input 
                                type="checkbox" 
                                id="nativeEngineToggle"
                                onChange={(e) => {
                                    if (engineRef.current) {
                                        engineRef.current.useWebSocket = e.target.checked;
                                    }
                                }}
                                style={{ cursor: 'pointer' }}
                            />
                            <label htmlFor="nativeEngineToggle" style={{ color: '#ccc', fontSize: '0.85rem', cursor: 'pointer', userSelect: 'none' }}>
                                Use Local Native Engine (Requires <code style={{color: '#4ade80'}}>cargo run -- --server</code>)
                            </label>
                        </div>

                        <button className="btn btn-primary" style={{ width: '100%' }} onClick={() => dispatch({ type: 'NEW_GAME' })}>
                            New Game
                        </button>
                    </div>
                </aside>

                <div className="game-area">
                    <BoardComponent
                        pieces={state.boardObj.getPieces()}
                        filesBrd={state.boardObj.getFilesBrd()}
                        ranksBrd={state.boardObj.getRanksBrd()}
                        selectedSquare={state.selectedSquare}
                        validMoves={state.validMoves}
                        checkSquare={state.checkSquare}
                        onSquareClick={handleSquareClick}
                        isFlipped={false} // Lock perspective to White
                    />
                </div>
            </div>
        </div>
    );
}
