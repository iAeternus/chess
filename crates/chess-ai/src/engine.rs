use chess_core::{Move, Position};

pub trait ChessEngine {
    /// 搜索并返回最佳走法
    fn search(&mut self, position: &Position) -> Option<Move>;

    /// 引擎名称
    fn name(&self) -> &str;
}
