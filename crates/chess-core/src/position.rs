use crate::{Board, CastlingRights, Color, Piece, Result, Square, fen, zobrist::ZobristKeys};

/// 局面
pub struct Position {
    /// 棋盘状态
    board: Board,
    /// 当前行动放
    side_to_move: Color,
    /// 王车易位权限
    castling: CastlingRights,
    /// 吃过路兵目标格
    en_passant: Option<Square>,
    /// 50步规则计数器
    halfmove_clock: u32,
    /// 当前完整回合数，从1开始
    fullmove_number: u32,
    /// Zobrist哈希
    zobrist_key: u64,
}

impl Position {
    pub fn new(
        board: Board,
        side_to_move: Color,
        castling: CastlingRights,
        en_passant: Option<Square>,
        halfmove_clock: u32,
        fullmove_number: u32,
    ) -> Self {
        let zobrist_key = ZobristKeys::compute(&board, side_to_move, castling, en_passant);

        Self {
            board,
            side_to_move,
            castling,
            en_passant,
            halfmove_clock,
            fullmove_number,
            zobrist_key,
        }
    }

    pub fn startpos() -> Self {
        // SAFETY: 初始局面字符串合法
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn from_fen(fen: &str) -> Result<Self> {
        fen::fen2position(fen)
    }

    pub fn to_fen(&self) -> String {
        fen::position2fen(self)
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn castling(&self) -> CastlingRights {
        self.castling
    }

    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    pub fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    pub fn fullmove_number(&self) -> u32 {
        self.fullmove_number
    }

    pub fn zobrist_key(&self) -> u64 {
        self.zobrist_key
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.board.piece_at(sq)
    }
}
