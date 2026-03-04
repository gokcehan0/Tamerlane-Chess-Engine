/// Tamerlane Chess Engine — Board representation
/// 15-wide mailbox (270 squares), FEN parsing, make/unmake, Zobrist hashing.

use crate::types::*;

// ─── Off-board detection ─────────────────────────────────────────

/// files_brd[sq] == OFF if sq is off the 10×11 playing area
const OFF: i32 = -1;

fn init_files_ranks() -> ([i32; MAILBOX_SIZE], [i32; MAILBOX_SIZE]) {
    let mut files = [OFF; MAILBOX_SIZE];
    let mut ranks = [OFF; MAILBOX_SIZE];
    for r in 1..=10i32 {
        for f in 1..=11i32 {
            let s = sq(f, r);
            files[s] = f;
            ranks[s] = r;
        }
    }
    // Citadels — mark as valid
    let wc = sq(6, 0);
    let bc = sq(6, 11);
    files[wc] = 6;
    ranks[wc] = 0;
    files[bc] = 6;
    ranks[bc] = 11;
    (files, ranks)
}

// We use static mutable arrays initialized once via init_globals()
static mut FILES_BRD: [i32; MAILBOX_SIZE] = [OFF; MAILBOX_SIZE];
static mut RANKS_BRD: [i32; MAILBOX_SIZE] = [OFF; MAILBOX_SIZE];
static mut ZOBRIST_KEYS: [[u64; NUM_PIECE_TYPES]; MAILBOX_SIZE] = [[0; NUM_PIECE_TYPES]; MAILBOX_SIZE];
static mut ZOBRIST_SIDE: u64 = 0;
static mut INITIALIZED: bool = false;

pub fn init_globals() {
    unsafe {
        if INITIALIZED { return; }
        let (f, r) = init_files_ranks();
        FILES_BRD = f;
        RANKS_BRD = r;
        // Zobrist
        let mut rng = SimpleRng::new(0xBEEF_CAFE_1234_5678);
        for sq_i in 0..MAILBOX_SIZE {
            for pc in 0..NUM_PIECE_TYPES {
                ZOBRIST_KEYS[sq_i][pc] = rng.next();
            }
        }
        ZOBRIST_SIDE = rng.next();
        INITIALIZED = true;
    }
}

#[inline]
pub fn is_off_board(s: usize) -> bool {
    if s >= MAILBOX_SIZE { return true; }
    unsafe { FILES_BRD[s] == OFF }
}

#[inline]
pub fn file_brd(s: usize) -> i32 {
    unsafe { FILES_BRD[s] }
}

#[inline]
pub fn rank_brd(s: usize) -> i32 {
    unsafe { RANKS_BRD[s] }
}

#[inline]
fn zobrist_piece(s: usize, p: Piece) -> u64 {
    unsafe { ZOBRIST_KEYS[s][p as usize] }
}

#[inline]
fn zobrist_side() -> u64 {
    unsafe { ZOBRIST_SIDE }
}

/// Public version of zobrist_side for null-move pruning in search
#[inline]
pub fn zobrist_side_key() -> u64 {
    unsafe { ZOBRIST_SIDE }
}

// ─── Simple RNG for Zobrist ──────────────────────────────────────

struct SimpleRng(u64);
impl SimpleRng {
    fn new(seed: u64) -> Self { Self(seed) }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

// ─── Undo info ──────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct UndoInfo {
    pub mv: Move,
    pub captured: Piece,
    pub hash: u64,
    pub half_moves: u16,
}

// ─── Board ──────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Board {
    pub pieces: [Piece; MAILBOX_SIZE],
    pub side: Color,
    pub hash: u64,
    pub ply: u16,
    pub half_moves: u16,
    pub history: Vec<UndoInfo>,
}

impl Board {
    pub fn new() -> Self {
        init_globals();
        Board {
            pieces: [Piece::Empty; MAILBOX_SIZE],
            side: Color::White,
            hash: 0,
            ply: 0,
            half_moves: 0,
            history: Vec::with_capacity(512),
        }
    }

    // ─── FEN parsing ─────────────────────────────────────────

    /// Parse our Tamerlane FEN: ranks separated by /, pieces in custom notation, then side.
    /// Example: "f1d1i1i1d1f/kamzgsvzmak1/pxcbyqehtnr/92/92/92/92/PXCBYQEHTNR/KAMZGSVZMAK1/F1D1I1I1D1F w"
    pub fn from_fen(fen: &str) -> Board {
        init_globals();
        let mut b = Board::new();
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let board_str = parts[0];
        let side_str = if parts.len() > 1 { parts[1] } else { "w" };

        let ranks: Vec<&str> = board_str.split('/').collect();
        // FEN ranks go from top (rank 10) to bottom (rank 1)
        for (ri, rank_str) in ranks.iter().enumerate() {
            let rank = 10 - ri as i32; // rank 10 down to rank 1
            let mut file = 1i32;
            let chars: Vec<char> = rank_str.chars().collect();
            let mut ci = 0;
            while ci < chars.len() && file <= 11 {
                let ch = chars[ci];
                if ch.is_ascii_digit() {
                    // Could be multi-digit empty count like "92" meaning 9+2=11
                    let mut empty = (ch as i32) - ('0' as i32);
                    // Check if next char is also digit
                    if ci + 1 < chars.len() && chars[ci + 1].is_ascii_digit() {
                        ci += 1;
                        empty = empty * 10 + ((chars[ci] as i32) - ('0' as i32));
                    }
                    file += empty;
                } else {
                    let piece = fen_char_to_piece(ch);
                    if piece != Piece::Empty && file >= 1 && file <= 11 {
                        let s = sq(file, rank);
                        b.pieces[s] = piece;
                        b.hash ^= zobrist_piece(s, piece);
                    }
                    file += 1;
                }
                ci += 1;
            }
        }

        b.side = if side_str == "b" { Color::Black } else { Color::White };
        if b.side == Color::Black {
            b.hash ^= zobrist_side();
        }
        b
    }

    /// Convert board to FEN string (compact)
    pub fn to_fen(&self) -> String {
        let mut fen = String::with_capacity(80);
        for rank in (1..=10).rev() {
            if rank < 10 { fen.push('/'); }
            let mut empty = 0;
            for file in 1..=11 {
                let s = sq(file, rank);
                let p = self.pieces[s];
                if p == Piece::Empty {
                    empty += 1;
                } else {
                    if empty > 0 {
                        push_empty_count(&mut fen, empty);
                        empty = 0;
                    }
                    fen.push(piece_to_fen_char(p));
                }
            }
            if empty > 0 {
                push_empty_count(&mut fen, empty);
            }
        }
        fen.push(' ');
        fen.push(if self.side == Color::White { 'w' } else { 'b' });
        fen
    }

    // ─── Make / Unmake ───────────────────────────────────────

    pub fn make_move(&mut self, mv: Move) {
        let from = mv_from(mv);
        let to = mv_to(mv);
        let piece = self.pieces[from];
        let captured = self.pieces[to];
        let is_prom = mv_is_promotion(mv);

        self.history.push(UndoInfo {
            mv,
            captured,
            hash: self.hash,
            half_moves: self.half_moves,
        });

        // Remove piece from source
        self.hash ^= zobrist_piece(from, piece);
        self.pieces[from] = Piece::Empty;

        // Remove captured piece
        if captured != Piece::Empty {
            self.hash ^= zobrist_piece(to, captured);
            self.half_moves = 0;
        } else if piece.is_pawn() {
            self.half_moves = 0;
        } else {
            self.half_moves += 1;
        }

        // Place piece at destination (or promoted piece)
        let final_piece = if is_prom {
            mv_promoted_to(mv)
        } else {
            piece
        };
        self.pieces[to] = final_piece;
        self.hash ^= zobrist_piece(to, final_piece);

        // Switch side
        self.hash ^= zobrist_side();
        self.side = self.side.flip();
        self.ply += 1;
    }

    pub fn unmake_move(&mut self) {
        let undo = self.history.pop().expect("No move to unmake");
        let mv = undo.mv;
        let from = mv_from(mv);
        let to = mv_to(mv);
        let is_prom = mv_is_promotion(mv);

        self.side = self.side.flip();
        self.ply -= 1;
        self.hash = undo.hash;
        self.half_moves = undo.half_moves;

        // What was at 'to' now? Either the moved piece or the promoted piece
        let piece_on_to = self.pieces[to];
        self.pieces[to] = Piece::Empty;

        // Restore original piece at 'from'
        let original_piece = if is_prom {
            // We need to figure out what the pawn was — derive from promoted piece color + type
            demote_piece(piece_on_to)
        } else {
            piece_on_to
        };
        self.pieces[from] = original_piece;

        // Restore captured piece at 'to'
        self.pieces[to] = undo.captured;
    }

    // ─── Accessors ───────────────────────────────────────────

    #[inline]
    pub fn piece_at(&self, s: usize) -> Piece {
        self.pieces[s]
    }

    /// Find king square for given color (priority: King > Prince > AdKing)
    pub fn king_sq(&self, color: Color) -> Option<usize> {
        let (king, prince, adking) = match color {
            Color::White => (Piece::WKing, Piece::WPrince, Piece::WAdKing),
            Color::Black => (Piece::BKing, Piece::BPrince, Piece::BAdKing),
        };
        let mut king_s = None;
        let mut prince_s = None;
        let mut adking_s = None;
        for r in 1..=10 {
            for f in 1..=11 {
                let s = sq(f, r);
                let p = self.pieces[s];
                if p == king { king_s = Some(s); }
                else if p == prince { prince_s = Some(s); }
                else if p == adking { adking_s = Some(s); }
            }
        }
        king_s.or(prince_s).or(adking_s)
    }

    /// Count how many royal pieces the given color has (King, Prince, AdKing)
    pub fn royal_count(&self, color: Color) -> u8 {
        let (king, prince, adking) = match color {
            Color::White => (Piece::WKing, Piece::WPrince, Piece::WAdKing),
            Color::Black => (Piece::BKing, Piece::BPrince, Piece::BAdKing),
        };
        let mut count = 0;
        for &p in self.pieces.iter() {
            if p == king || p == prince || p == adking {
                count += 1;
            }
        }
        count
    }

    /// Check citadel squares
    pub fn is_white_citadel(s: usize) -> bool {
        // White citadel is below rank 1, column 6
        // From TS: CITADEL_WHITE = 88 in their mailbox
        // In our mailbox: sq(6, 0) = (0+3)*15 + 6+1 = 45+7 = 52
        // But we need to match TS. Let's just use the stored value.
        s == sq(6, 0)
    }

    pub fn is_black_citadel(s: usize) -> bool {
        s == sq(6, 11)
    }

    pub fn is_citadel(s: usize) -> bool {
        Self::is_white_citadel(s) || Self::is_black_citadel(s)
    }
}

// ─── FEN character mappings ─────────────────────────────────────

fn fen_char_to_piece(ch: char) -> Piece {
    match ch {
        'P' => Piece::WPawnPawn,
        'X' => Piece::WPawnWarengine,
        'C' => Piece::WPawnCamel,
        'B' => Piece::WPawnElephant,
        'Y' => Piece::WPawnMinister,
        'Q' => Piece::WPawnKing,
        'E' => Piece::WPawnAdvisor,
        'H' => Piece::WPawnGiraffe,
        'T' => Piece::WPawnCatapult,
        'N' => Piece::WPawnKnight,
        'R' => Piece::WPawnRook,
        'K' => Piece::WRook,
        'A' => Piece::WKnight,
        'M' => Piece::WCatapult,
        'Z' => Piece::WGiraffe,
        'G' => Piece::WMinister,
        'S' => Piece::WKing,
        'V' => Piece::WAdvisor,
        'F' => Piece::WElephant,
        'D' => Piece::WCamel,
        'I' => Piece::WWarengine,
        'J' => Piece::WPrince,
        'L' => Piece::WAdKing,
        'p' => Piece::BPawnPawn,
        'x' => Piece::BPawnWarengine,
        'c' => Piece::BPawnCamel,
        'b' => Piece::BPawnElephant,
        'y' => Piece::BPawnMinister,
        'q' => Piece::BPawnKing,
        'e' => Piece::BPawnAdvisor,
        'h' => Piece::BPawnGiraffe,
        't' => Piece::BPawnCatapult,
        'n' => Piece::BPawnKnight,
        'r' => Piece::BPawnRook,
        'k' => Piece::BRook,
        'a' => Piece::BKnight,
        'm' => Piece::BCatapult,
        'z' => Piece::BGiraffe,
        'g' => Piece::BMinister,
        's' => Piece::BKing,
        'v' => Piece::BAdvisor,
        'f' => Piece::BElephant,
        'd' => Piece::BCamel,
        'i' => Piece::BWarengine,
        'j' => Piece::BPrince,
        'l' => Piece::BAdKing,
        _ => Piece::Empty,
    }
}

fn piece_to_fen_char(p: Piece) -> char {
    match p {
        Piece::WPawnPawn => 'P',
        Piece::WPawnWarengine => 'X',
        Piece::WPawnCamel => 'C',
        Piece::WPawnElephant => 'B',
        Piece::WPawnMinister => 'Y',
        Piece::WPawnKing => 'Q',
        Piece::WPawnAdvisor => 'E',
        Piece::WPawnGiraffe => 'H',
        Piece::WPawnCatapult => 'T',
        Piece::WPawnKnight => 'N',
        Piece::WPawnRook => 'R',
        Piece::WRook => 'K',
        Piece::WKnight => 'A',
        Piece::WCatapult => 'M',
        Piece::WGiraffe => 'Z',
        Piece::WMinister => 'G',
        Piece::WKing => 'S',
        Piece::WAdvisor => 'V',
        Piece::WElephant => 'F',
        Piece::WCamel => 'D',
        Piece::WWarengine => 'I',
        Piece::WPrince => 'J',
        Piece::WAdKing => 'L',
        Piece::BPawnPawn => 'p',
        Piece::BPawnWarengine => 'x',
        Piece::BPawnCamel => 'c',
        Piece::BPawnElephant => 'b',
        Piece::BPawnMinister => 'y',
        Piece::BPawnKing => 'q',
        Piece::BPawnAdvisor => 'e',
        Piece::BPawnGiraffe => 'h',
        Piece::BPawnCatapult => 't',
        Piece::BPawnKnight => 'n',
        Piece::BPawnRook => 'r',
        Piece::BRook => 'k',
        Piece::BKnight => 'a',
        Piece::BCatapult => 'm',
        Piece::BGiraffe => 'z',
        Piece::BMinister => 'g',
        Piece::BKing => 's',
        Piece::BAdvisor => 'v',
        Piece::BElephant => 'f',
        Piece::BCamel => 'd',
        Piece::BWarengine => 'i',
        Piece::BPrince => 'j',
        Piece::BAdKing => 'l',
        Piece::Empty => '.',
    }
}

fn push_empty_count(s: &mut String, count: i32) {
    if count > 9 {
        s.push_str(&format!("{}{}", count / 10, count % 10));
    } else {
        s.push(char::from_digit(count as u32, 10).unwrap());
    }
}

/// Convert promoted piece back to pawn (for unmake)
fn demote_piece(p: Piece) -> Piece {
    match p {
        Piece::WAdKing => Piece::WPawnPawn,
        Piece::WWarengine => Piece::WPawnWarengine,
        Piece::WCamel => Piece::WPawnCamel,
        Piece::WElephant => Piece::WPawnElephant,
        Piece::WMinister => Piece::WPawnMinister,
        Piece::WPrince => Piece::WPawnKing,
        Piece::WAdvisor => Piece::WPawnAdvisor,
        Piece::WGiraffe => Piece::WPawnGiraffe,
        Piece::WCatapult => Piece::WPawnCatapult,
        Piece::WKnight => Piece::WPawnKnight,
        Piece::WRook => Piece::WPawnRook,
        Piece::BAdKing => Piece::BPawnPawn,
        Piece::BWarengine => Piece::BPawnWarengine,
        Piece::BCamel => Piece::BPawnCamel,
        Piece::BElephant => Piece::BPawnElephant,
        Piece::BMinister => Piece::BPawnMinister,
        Piece::BPrince => Piece::BPawnKing,
        Piece::BAdvisor => Piece::BPawnAdvisor,
        Piece::BGiraffe => Piece::BPawnGiraffe,
        Piece::BCatapult => Piece::BPawnCatapult,
        Piece::BKnight => Piece::BPawnKnight,
        Piece::BRook => Piece::BPawnRook,
        _ => p,
    }
}

/// Standard starting position FEN
pub const START_FEN: &str = "f1d1i1i1d1f/kamzgsvzmak1/pxcbyqehtnr/92/92/92/92/PXCBYQEHTNR/KAMZGSVZMAK1/F1D1I1I1D1F w";
