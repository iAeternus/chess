//! 走法列表面板：SAN 格式、可点击跳转、双列布局。

use chess_core::Move;
use egui::{Color32, ScrollArea};

pub struct MoveListPanel;

impl MoveListPanel {
    pub fn new() -> Self {
        Self
    }

    /// 渲染走法列表
    ///
    /// * `moves` — 所有走法（Move 数组）
    /// * `current_ply` — 当前所在的 ply（用于高亮）
    /// * `max_height` — 最大高度
    /// * `on_jump` — 点击某个走法时的回调，参数为 ply 编号（1-based，即第几个半移动之后）
    pub fn show(
        &self,
        ui: &mut egui::Ui,
        moves: &[Move],
        san_list: &[String],
        current_ply: usize,
        max_height: f32,
        mut on_jump: impl FnMut(usize),
    ) {
        ui.heading("Moves");
        ui.separator();

        if moves.is_empty() {
            ui.label("No moves yet");
            return;
        }

        ScrollArea::vertical()
            .max_height(max_height)
            .auto_shrink([false, true])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                // 双列网格：move_number | white_move | black_move
                egui::Grid::new("move_list_grid")
                    .striped(true)
                    .min_col_width(20.0)
                    .show(ui, |ui| {
                        let total_full_moves = moves.len().div_ceil(2);

                        for move_num in 1..=total_full_moves {
                            let white_idx = (move_num - 1) * 2; // 0, 2, 4, ...
                            let black_idx = white_idx + 1; // 1, 3, 5, ...

                            // 走法编号
                            ui.label(
                                egui::RichText::new(format!("{move_num}."))
                                    .color(Color32::from_rgb(150, 150, 150)),
                            );

                            // 白方走法
                            let w_ply = white_idx + 1; // ply 编号（1-based）
                            let w_san = san_list
                                .get(white_idx)
                                .cloned()
                                .unwrap_or_else(|| format_move(moves[white_idx]));
                            let w_is_current = current_ply == w_ply;
                            if self.move_button(ui, &w_san, w_is_current) {
                                on_jump(w_ply);
                            }

                            // 黑方走法（可能不存在）
                            if black_idx < moves.len() {
                                let b_ply = black_idx + 1;
                                let b_san = san_list
                                    .get(black_idx)
                                    .cloned()
                                    .unwrap_or_else(|| format_move(moves[black_idx]));
                                let b_is_current = current_ply == b_ply;
                                if self.move_button(ui, &b_san, b_is_current) {
                                    on_jump(b_ply);
                                }
                            }

                            ui.end_row();
                        }
                    });
            });
    }

    /// 单个走法按钮
    fn move_button(&self, ui: &mut egui::Ui, san: &str, is_current: bool) -> bool {
        let text = if is_current {
            egui::RichText::new(san)
                .color(Color32::from_rgb(255, 255, 100))
                .strong()
        } else {
            egui::RichText::new(san)
        };

        ui.button(text).clicked()
    }
}

/// 回退：坐标表示
fn format_move(mv: Move) -> String {
    format!("{}{}", mv.from(), mv.to())
}
