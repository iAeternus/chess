//! 棋盘交互处理：点击、拖拽开始/移动/释放
//!
//! 从 `ChessApp` 中提取，使用 `BoardLayout` 进行坐标转换

use chess_core::{Piece, Square};
use egui::Pos2;

use crate::board::layout::BoardLayout;
use crate::game::controller::{GameController, GameMode, SelectionResult};

/// 处理棋盘上的所有鼠标交互
///
/// 返回 `Some((from, to))` 表示需要升变选择
pub fn handle_interaction(
    response: &egui::Response,
    layout: &BoardLayout,
    controller: &mut GameController,
    drag: &mut Option<(Piece, Square, Pos2)>,
    ctx: &egui::Context,
    mode: GameMode,
    flipped: bool,
) -> Option<(Square, Square)> {
    let mut pending_promotion = None;

    // 拖拽开始
    if response.drag_started()
        && let Some(pos) = response.interact_pointer_pos()
        && let Some(sq) = layout.pos_to_square(pos, flipped)
    {
        let piece = controller.current_position().piece_at(sq);
        if let Some(p) = piece {
            let side = controller.current_position().side_to_move();
            let can_move = mode == GameMode::Analysis || p.color == side;

            if can_move {
                // 选中棋子（仅在棋子有合法走法时生效）
                let result = controller.select_square(sq);
                if matches!(result, SelectionResult::Selected { .. }) {
                    *drag = Some((p, sq, pos));
                }
            }
        }
    }

    // 拖拽移动
    if response.dragged()
        && let Some(pos) = response.interact_pointer_pos()
        && let Some((_piece, _from, drag_pos)) = drag
    {
        *drag_pos = pos;
        ctx.request_repaint();
    }

    // 拖拽释放
    if response.drag_stopped()
        && let Some((_piece, _from, pos)) = drag.take()
    {
        if let Some(target) = layout.pos_to_square(pos, flipped) {
            pending_promotion = execute_drag_drop(controller, target);
        } else {
            controller.clear_selection();
        }
    }

    // 点击
    if response.clicked()
        && let Some(local) = response.interact_pointer_pos()
        && let Some(sq) = layout.pos_to_square(local, flipped)
        && let SelectionResult::NeedsPromotion { from, to } = controller.select_square(sq)
    {
        pending_promotion = Some((from, to));
    }

    pending_promotion
}

/// 执行拖拽释放：检查目标格子是否有合法走法，处理升变判断
fn execute_drag_drop(
    controller: &mut GameController,
    target: Square,
) -> Option<(Square, Square)> {
    let legal_moves = controller.legal_moves_for_selected();
    let matching: Vec<_> = legal_moves
        .iter()
        .filter(|m| m.to() == target)
        .copied()
        .collect();

    if matching.is_empty() {
        controller.clear_selection();
        return None;
    }

    // 检查是否需要升变选择
    let has_promotion = matching.iter().any(|m| m.is_promotion());
    if has_promotion && matching.len() > 1 {
        let from = matching[0].from();
        return Some((from, target));
    }

    // 直接执行
    controller.make_move(matching[0]);
    None
}
