use chess_core::{Move, Position};

pub trait ChessEngine: Send {
    /// 搜索并返回最佳走法
    fn search(&mut self, position: &Position) -> Option<Move>;
}
