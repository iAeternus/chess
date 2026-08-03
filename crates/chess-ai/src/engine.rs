use chess_core::{Move, Position};

pub trait ChessEngine {
    /// 搜索并返回最佳走法
    fn search(&mut self, position: &Position) -> Option<Move>;

    /// 引擎名称
    fn name(&self) -> &str;

    /// 设置搜索时间限制（毫秒）
    fn set_time_limit(&mut self, ms: u64);

    /// 设置搜索深度限制
    fn set_depth_limit(&mut self, depth: u32);
}
