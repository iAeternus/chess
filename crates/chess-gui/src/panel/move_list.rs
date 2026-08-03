//! 走法列表面板：顶部导航按钮 + SAN 格式走法表格。

use chess_core::Move;
use egui::{Color32, ScrollArea};

/// 走法列表操作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveListAction {
    /// 跳转到指定 ply（1-based）
    JumpToPly(usize),
    /// 导航操作
    GoToStart,
    GoBack,
    GoForward,
    GoToEnd,
}

pub struct MoveListPanel;

impl MoveListPanel {
    pub fn new() -> Self {
        Self
    }

    /// 渲染走法列表（含顶部导航按钮）
    ///
    /// `on_action` — 用户触发操作时的回调
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &self,
        ui: &mut egui::Ui,
        moves: &[Move],
        san_list: &[String],
        current_ply: usize,
        max_height: f32,
        can_back: bool,
        can_forward: bool,
        mut on_action: impl FnMut(MoveListAction),
    ) {
        ui.heading("Moves");
        ui.separator();

        // 导航按钮行
        ui.horizontal(|ui| {
            if ui
                .add_enabled(can_back, egui::Button::new("|◀"))
                .on_hover_text("Start (Home)")
                .clicked()
            {
                on_action(MoveListAction::GoToStart);
            }
            if ui
                .add_enabled(can_back, egui::Button::new("◀"))
                .on_hover_text("Back (←)")
                .clicked()
            {
                on_action(MoveListAction::GoBack);
            }
            if ui
                .add_enabled(can_forward, egui::Button::new("▶"))
                .on_hover_text("Forward (→)")
                .clicked()
            {
                on_action(MoveListAction::GoForward);
            }
            if ui
                .add_enabled(can_forward, egui::Button::new("▶|"))
                .on_hover_text("End (End)")
                .clicked()
            {
                on_action(MoveListAction::GoToEnd);
            }
        });

        if moves.is_empty() {
            ui.label("No moves yet");
            return;
        }

        ScrollArea::vertical()
            .max_height(max_height)
            .auto_shrink([false, true])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                egui::Grid::new("move_list_grid")
                    .striped(true)
                    .min_col_width(20.0)
                    .show(ui, |ui| {
                        let total_full_moves = moves.len().div_ceil(2);

                        for move_num in 1..=total_full_moves {
                            let white_idx = (move_num - 1) * 2;
                            let black_idx = white_idx + 1;

                            // 走法编号
                            ui.label(
                                egui::RichText::new(format!("{move_num}."))
                                    .color(Color32::from_rgb(130, 130, 130))
                                    .size(14.0),
                            );

                            // 白方走法
                            let w_ply = white_idx + 1;
                            let w_san = san_list
                                .get(white_idx)
                                .cloned()
                                .unwrap_or_else(|| format_move(moves[white_idx]));
                            let w_is_current = current_ply == w_ply;
                            if self.move_button(ui, &w_san, w_is_current) {
                                on_action(MoveListAction::JumpToPly(w_ply));
                            }

                            // 黑方走法
                            if black_idx < moves.len() {
                                let b_ply = black_idx + 1;
                                let b_san = san_list
                                    .get(black_idx)
                                    .cloned()
                                    .unwrap_or_else(|| format_move(moves[black_idx]));
                                let b_is_current = current_ply == b_ply;
                                if self.move_button(ui, &b_san, b_is_current) {
                                    on_action(MoveListAction::JumpToPly(b_ply));
                                }
                            }

                            ui.end_row();
                        }
                    });
            });
    }

    fn move_button(&self, ui: &mut egui::Ui, san: &str, is_current: bool) -> bool {
        let text = if is_current {
            egui::RichText::new(san)
                .color(Color32::from_rgb(255, 255, 100))
                .strong()
                .size(16.0)
        } else {
            egui::RichText::new(san).size(16.0)
        };

        ui.button(text).clicked()
    }
}

fn format_move(mv: Move) -> String {
    format!("{}{}", mv.from(), mv.to())
}
