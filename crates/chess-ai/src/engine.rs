use chess_core::{Move, Position};

// TODO: 考虑将其改为async trait
pub trait ChessEngine {
    /// 搜索并返回最佳走法
    fn search(&mut self, position: &Position) -> Option<Move>;
}
