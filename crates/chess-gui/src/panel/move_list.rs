//! 走法列表面板：顶部导航按钮 + SAN 格式走法表格。

use chess_core::Move;
use egui::{Color32, ScrollArea, Sense};
use egui_extras::{Column, TableBuilder};

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

/// 当前走法高亮背景色
const HIGHLIGHT_BG: Color32 = Color32::from_rgba_premultiplied(100, 150, 255, 60);
/// 走法编号颜色
const DIM_COLOR: Color32 = Color32::from_rgb(130, 130, 130);

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
            let buttons = [
                ("⏮", can_back, MoveListAction::GoToStart, "Start (Home)"),
                ("◀", can_back, MoveListAction::GoBack, "Back (←)"),
                ("▶", can_forward, MoveListAction::GoForward, "Forward (→)"),
                ("⏭", can_forward, MoveListAction::GoToEnd, "End (End)"),
            ];

            for (icon, enabled, action, tooltip) in buttons {
                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(egui::RichText::new(icon).size(20.0)),
                    )
                    .on_hover_text(tooltip)
                    .clicked()
                {
                    on_action(action);
                }
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
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::exact(40.0)) // 回合编号
                    .column(Column::remainder()) // 白棋
                    .column(Column::remainder()) // 黑棋
                    .header(24.0, |mut header| {
                        header.col(|ui| {
                            ui.with_layout(
                                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                |ui| {
                                    ui.label(egui::RichText::new("#").color(DIM_COLOR).strong());
                                },
                            );
                        });

                        header.col(|ui| {
                            ui.with_layout(
                                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                |ui| {
                                    ui.label(egui::RichText::new("White").strong());
                                },
                            );
                        });

                        header.col(|ui| {
                            ui.with_layout(
                                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                |ui| {
                                    ui.label(egui::RichText::new("Black").strong());
                                },
                            );
                        });
                    })
                    .body(|mut body| {
                        let total_full_moves = moves.len().div_ceil(2);

                        for move_num in 1..=total_full_moves {
                            let white_idx = (move_num - 1) * 2;
                            let black_idx = white_idx + 1;

                            body.row(26.0, |mut row| {
                                // move number
                                row.col(|ui| {
                                    ui.with_layout(
                                        egui::Layout::centered_and_justified(
                                            egui::Direction::LeftToRight,
                                        ),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(format!("{move_num}."))
                                                    .color(DIM_COLOR)
                                                    .strong()
                                                    .size(17.0),
                                            );
                                        },
                                    );
                                });

                                // white move
                                row.col(|ui| {
                                    let ply = white_idx + 1;

                                    let san =
                                        san_list.get(white_idx).map(String::as_str).unwrap_or("??");

                                    if self.move_cell(ui, san, current_ply == ply) {
                                        on_action(MoveListAction::JumpToPly(ply));
                                    }
                                });

                                // black move
                                row.col(|ui| {
                                    if black_idx < moves.len() {
                                        let ply = black_idx + 1;

                                        let san = san_list
                                            .get(black_idx)
                                            .map(String::as_str)
                                            .unwrap_or("??");

                                        if self.move_cell(ui, san, current_ply == ply) {
                                            on_action(MoveListAction::JumpToPly(ply));
                                        }
                                    }
                                });
                            });
                        }
                    });
            });
    }

    /// 渲染单个走法单元格
    fn move_cell(&self, ui: &mut egui::Ui, san: &str, is_current: bool) -> bool {
        let mut text = egui::RichText::new(san).size(16.0);

        if is_current {
            text = text.strong();
        }

        let response = egui::Frame::NONE
            .fill(if is_current {
                HIGHLIGHT_BG
            } else {
                Color32::TRANSPARENT
            })
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| ui.add(egui::Label::new(text).sense(Sense::click())),
                )
                .inner
            });

        response.inner.clicked()
    }
}
