use crate::{
    Board, CastlingRights, Color, Piece, PieceKind, Result, Square, attack::is_square_attacked,
    fen, zobrist::Zobrist,
};

/// 局面
#[derive(Clone)]
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
        let mut position = Self {
            board,
            side_to_move,
            castling,
            en_passant,
            halfmove_clock,
            fullmove_number,
            zobrist_key: 0,
        };
        position.zobrist_key = Zobrist::compute(&position);
        position
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

    pub(crate) fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    pub(crate) fn switch_side(&mut self) {
        self.side_to_move = self.side_to_move.flip();
    }

    pub(crate) fn set_side_to_move(&mut self, color: Color) {
        self.side_to_move = color;
    }

    pub(crate) fn set_castling(&mut self, rights: CastlingRights) {
        self.castling = rights;
    }

    pub(crate) fn set_en_passant(&mut self, sq: Option<Square>) {
        self.en_passant = sq;
    }

    pub(crate) fn set_halfmove_clock(&mut self, value: u32) {
        self.halfmove_clock = value;
    }

    pub(crate) fn set_fullmove_number(&mut self, value: u32) {
        self.fullmove_number = value;
    }

    pub(crate) fn set_zobrist(&mut self, key: u64) {
        self.zobrist_key = key;
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.board.piece_at(sq)
    }

    /// 检测是否将军
    pub fn is_check(&self) -> bool {
        let side = self.side_to_move();
        let king = self
            .board
            .piece_kind(side, PieceKind::King)
            .lsb()
            .expect(&format!("missing king for {}", side));

        is_square_attacked(&self.board, king, side.flip())
    }
}
