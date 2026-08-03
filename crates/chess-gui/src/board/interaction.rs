//! 棋盘交互处理：点击、拖拽。

use chess_core::Square;
use egui::Pos2;

/// 拖拽状态（Phase 5 实现拖拽移动）
#[allow(dead_code)]
pub struct DragState {
    pub from_square: Square,
    pub current_pos: Pos2,
}
