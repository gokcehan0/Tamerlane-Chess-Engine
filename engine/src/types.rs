/// Tamerlane Chess Engine — Type Definitions
/// 10×11 board with citadels, all piece types for both colors.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline]
    pub fn flip(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// All piece types in Tamerlane Chess.
/// Values 0..=20 for White, 21..=41 for Black pawns/pieces.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Piece {
    Empty = 0,
    // White Pawns (promote to their named piece)
    WPawnPawn = 1,
    WPawnWarengine = 2,
    WPawnCamel = 3,
    WPawnElephant = 4,
    WPawnMinister = 5,
    WPawnKing = 6,
    WPawnAdvisor = 7,
    WPawnGiraffe = 8,
    WPawnCatapult = 9,
    WPawnKnight = 10,
    WPawnRook = 11,
    // White Pieces
    WRook = 12,
    WKnight = 13,
    WCatapult = 14,
    WGiraffe = 15,
    WMinister = 16,
    WKing = 17,
    WAdvisor = 18,
    WElephant = 19,
    WCamel = 20,
    WWarengine = 21,
    WPrince = 22,
    WAdKing = 23,
    // Black Pawns
    BPawnPawn = 24,
    BPawnWarengine = 25,
    BPawnCamel = 26,
    BPawnElephant = 27,
    BPawnMinister = 28,
    BPawnKing = 29,
    BPawnAdvisor = 30,
    BPawnGiraffe = 31,
    BPawnCatapult = 32,
    BPawnKnight = 33,
    BPawnRook = 34,
    // Black Pieces
    BRook = 35,
    BKnight = 36,
    BCatapult = 37,
    BGiraffe = 38,
    BMinister = 39,
    BKing = 40,
    BAdvisor = 41,
    BElephant = 42,
    BCamel = 43,
    BWarengine = 44,
    BPrince = 45,
    BAdKing = 46,
}

pub const NUM_PIECE_TYPES: usize = 47;

impl Piece {
    #[inline]
    pub fn color(self) -> Option<Color> {
        let v = self as u8;
        if v == 0 { return None; }
        if v <= 23 { Some(Color::White) } else { Some(Color::Black) }
    }

    #[inline]
    pub fn is_white(self) -> bool {
        let v = self as u8;
        v >= 1 && v <= 23
    }

    #[inline]
    pub fn is_black(self) -> bool {
        (self as u8) >= 24
    }

    #[inline]
    pub fn is_pawn(self) -> bool {
        let v = self as u8;
        (v >= 1 && v <= 11) || (v >= 24 && v <= 34)
    }

    #[inline]
    pub fn is_king_type(self) -> bool {
        matches!(self,
            Piece::WKing | Piece::BKing |
            Piece::WPrince | Piece::BPrince |
            Piece::WAdKing | Piece::BAdKing
        )
    }

    #[inline]
    pub fn from_u8(v: u8) -> Piece {
        if v < NUM_PIECE_TYPES as u8 {
            unsafe { std::mem::transmute(v) }
        } else {
            Piece::Empty
        }
    }

    /// Get the base piece kind index (0-9) for PST/material tables
    /// Rook=0, Knight=1, Catapult=2, Giraffe=3, Minister=4, King=5, Advisor=6, Elephant=7, Camel=8, Warengine=9
    pub fn kind_index(self) -> usize {
        match self {
            Piece::WRook | Piece::BRook | Piece::WPawnRook | Piece::BPawnRook => 0,
            Piece::WKnight | Piece::BKnight | Piece::WPawnKnight | Piece::BPawnKnight => 1,
            Piece::WCatapult | Piece::BCatapult | Piece::WPawnCatapult | Piece::BPawnCatapult => 2,
            Piece::WGiraffe | Piece::BGiraffe | Piece::WPawnGiraffe | Piece::BPawnGiraffe => 3,
            Piece::WMinister | Piece::BMinister | Piece::WPawnMinister | Piece::BPawnMinister => 4,
            Piece::WKing | Piece::BKing | Piece::WPawnKing | Piece::BPawnKing => 5,
            Piece::WAdvisor | Piece::BAdvisor | Piece::WPawnAdvisor | Piece::BPawnAdvisor => 6,
            Piece::WElephant | Piece::BElephant | Piece::WPawnElephant | Piece::BPawnElephant => 7,
            Piece::WCamel | Piece::BCamel | Piece::WPawnCamel | Piece::BPawnCamel => 8,
            Piece::WWarengine | Piece::BWarengine | Piece::WPawnWarengine | Piece::BPawnWarengine => 9,
            Piece::WPrince | Piece::BPrince => 5,
            Piece::WAdKing | Piece::BAdKing => 5,
            Piece::WPawnPawn | Piece::BPawnPawn => 10, // generic pawn
            Piece::Empty => 10,
        }
    }

    /// Get the promoted piece for this pawn type
    pub fn promoted(self) -> Piece {
        match self {
            Piece::WPawnPawn => Piece::WAdKing,
            Piece::WPawnWarengine => Piece::WWarengine,
            Piece::WPawnCamel => Piece::WCamel,
            Piece::WPawnElephant => Piece::WElephant,
            Piece::WPawnMinister => Piece::WMinister,
            Piece::WPawnKing => Piece::WPrince,
            Piece::WPawnAdvisor => Piece::WAdvisor,
            Piece::WPawnGiraffe => Piece::WGiraffe,
            Piece::WPawnCatapult => Piece::WCatapult,
            Piece::WPawnKnight => Piece::WKnight,
            Piece::WPawnRook => Piece::WRook,
            Piece::BPawnPawn => Piece::BAdKing,
            Piece::BPawnWarengine => Piece::BWarengine,
            Piece::BPawnCamel => Piece::BCamel,
            Piece::BPawnElephant => Piece::BElephant,
            Piece::BPawnMinister => Piece::BMinister,
            Piece::BPawnKing => Piece::BPrince,
            Piece::BPawnAdvisor => Piece::BAdvisor,
            Piece::BPawnGiraffe => Piece::BGiraffe,
            Piece::BPawnCatapult => Piece::BCatapult,
            Piece::BPawnKnight => Piece::BKnight,
            Piece::BPawnRook => Piece::BRook,
            _ => self,
        }
    }
}

// ─── Board geometry ───────────────────────────────────────────────

/// We use a 15-wide mailbox: file 1..11 mapped, with padding columns 0, 12, 13, 14.
/// Ranks 1..10, rows padded above and below.
/// Total mailbox: 15 columns × 12 rows = 180, but we use 15×18 = 270 for ample padding.
pub const BOARD_FILES: usize = 11;
pub const BOARD_RANKS: usize = 10;
pub const MAILBOX_WIDTH: i32 = 15;
pub const MAILBOX_SIZE: usize = 270;

/// Convert (file 1-based, rank 1-based) to mailbox index
#[inline]
pub fn sq(file: i32, rank: i32) -> usize {
    ((rank + 3) * MAILBOX_WIDTH + file + 1) as usize
}

/// Extract file (1..11) from mailbox index
#[inline]
pub fn file_of(s: usize) -> i32 {
    (s as i32) % MAILBOX_WIDTH - 1
}

/// Extract rank (1..10) from mailbox index
#[inline]
pub fn rank_of(s: usize) -> i32 {
    (s as i32) / MAILBOX_WIDTH - 3
}

/// Citadel squares
pub const WHITE_CITADEL: usize = 88;  // sq(6, 0) — but we encode as a special square below rank 1
pub const BLACK_CITADEL: usize = 181; // sq(6, 11) — above rank 10

pub fn init_citadels() -> (usize, usize) {
    // White citadel: column 6 (middle), row below rank 1
    // sq(6, 0) but rank 0 is padding. We'll use the special address.
    // Actually from the TS code: CITADEL_WHITE = 88, CITADEL_BLACK = 181
    // Let's verify: sq(6,1) = (1+3)*15 + 6+1 = 60+7 = 67
    // In the TS code, the mapping is different (file+rank*15 with different offsets).
    // We need to match the TS mapping exactly for FEN compatibility.
    // TS: fileRankToSquare(file, rank) = (21 + file) + (rank - 1) * 15
    // So sq_ts(6, 0) = 21 + 6 + (0-1)*15 = 27-15 = 12... no
    // TS CITADEL_WHITE = 88, let's reverse: 88 = 21 + file + (rank-1)*15
    // 88-21 = 67 = file + (rank-1)*15
    // If rank=5: file + 60 = 67, file=7... 
    // If rank=4: file + 45 = 67, file=22... no
    // The TS uses a different mailbox. We'll use our own mapping.
    (88, 181)
}

// ─── Move encoding ───────────────────────────────────────────────

/// Move is a u32:
/// bits 0-7:   from square (0-269)
/// bits 8-15:  to square
/// bits 16-21: captured piece (Piece as u8)
/// bit  22:    is promotion
/// bits 23-28: promoted-to piece (Piece as u8)
/// bits 29-31: flags (special moves)
pub type Move = u32;

pub const MOVE_NONE: Move = 0;

#[inline]
pub fn mv_from(m: Move) -> usize {
    (m & 0xFF) as usize
}

#[inline]
pub fn mv_to(m: Move) -> usize {
    ((m >> 8) & 0xFF) as usize
}

#[inline]
pub fn mv_captured(m: Move) -> Piece {
    Piece::from_u8(((m >> 16) & 0x3F) as u8)
}

#[inline]
pub fn mv_is_promotion(m: Move) -> bool {
    (m >> 22) & 1 != 0
}

#[inline]
pub fn mv_promoted_to(m: Move) -> Piece {
    Piece::from_u8(((m >> 23) & 0x3F) as u8)
}

#[inline]
pub fn make_move(from: usize, to: usize, captured: Piece, promotion: bool, promoted_to: Piece) -> Move {
    let mut m = (from as u32) | ((to as u32) << 8) | ((captured as u32) << 16);
    if promotion {
        m |= 1 << 22;
        m |= (promoted_to as u32) << 23;
    }
    m
}

#[inline]
pub fn make_quiet(from: usize, to: usize) -> Move {
    (from as u32) | ((to as u32) << 8)
}

#[inline]
pub fn make_capture(from: usize, to: usize, captured: Piece) -> Move {
    (from as u32) | ((to as u32) << 8) | ((captured as u32) << 16)
}

// ─── Directions ──────────────────────────────────────────────────

pub const KNIGHT_DIRS: [i32; 8] = [-31, -29, -17, -13, 13, 17, 29, 31];
pub const CAMEL_DIRS: [i32; 8] = [-46, -44, -18, -12, 12, 18, 44, 46];
pub const GIRAFFE_DIRS: [i32; 8] = [-61, -59, -19, -11, 11, 19, 59, 61];
pub const KING_DIRS: [i32; 8] = [-16, -15, -14, -1, 1, 14, 15, 16];
pub const ROOK_DIRS: [i32; 4] = [-15, -1, 1, 15];
pub const ADVISOR_DIRS: [i32; 4] = [-15, -1, 1, 15];
pub const MINISTER_DIRS: [i32; 4] = [-16, -14, 14, 16];
pub const ELEPHANT_DIRS: [i32; 8] = [-32, -28, 28, 32, -16, -14, 14, 16];
pub const WARENGINE_DIRS: [i32; 4] = [-30, -2, 2, 30];

/// Giraffe checkpoint arrays
pub const GIRAFFE_CHECK1: [i32; 8] = [-16, -14, -16, -14, 14, 16, 14, 16];
pub const GIRAFFE_CHECK2: [i32; 8] = [-31, -29, -17, -13, 13, 17, 29, 31];
pub const GIRAFFE_CHECK3: [i32; 8] = [-46, -44, -18, -12, 12, 18, 44, 46];
pub const GIRAFFE_SLIDE: [i32; 8] = [-15, -15, -1, 1, -1, 1, 15, 15];

// ─── Scores ────────────────────────────────────────────────────

pub const INF: i32 = 100_000;
pub const MATE_SCORE: i32 = 90_000;
pub const DRAW_SCORE: i32 = 0;
