//! 走法列表面板。

use chess_core::Move;
use egui::ScrollArea;

pub struct MoveListPanel;

impl MoveListPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show_with_height(
        &self,
        ui: &mut egui::Ui,
        moves: &[Move],
        current_index: usize,
        max_height: f32,
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
                for (i, mv) in moves.iter().enumerate() {
                    let move_num = i / 2 + 1;
                    let is_current = i == current_index.saturating_sub(1);

                    let notation = format!("{}{}", mv.from(), mv.to());

                    let label = if i % 2 == 0 {
                        format!("{}. {}", move_num, notation)
                    } else {
                        notation
                    };

                    if is_current {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 255, 100),
                            &label,
                        );
                    } else {
                        ui.label(&label);
                    }
                }
            });
    }
}
