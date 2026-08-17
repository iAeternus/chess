mod app;
mod constants;
mod game;
mod gui;

use crate::gui::view::ViewEgui;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Chess",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_min_inner_size([820.0, 650.0])
                .with_title("Chess — Professional Analysis Board"),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(ViewEgui::new(cc)))),
    )
}
