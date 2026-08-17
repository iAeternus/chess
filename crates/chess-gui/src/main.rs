//! Chess GUI — Lichess 风格专业国际象棋桌面软件。

mod app;
mod board;
mod constants;
mod game;
mod panel;
mod piece;
mod theme;

use app::ViewEgui;

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
