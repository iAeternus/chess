//! 引擎信息面板（占位）。

use egui::Ui;

/// 引擎信息面板
pub struct EngineInfoPanel;

impl EngineInfoPanel {
    pub fn new() -> Self {
        Self
    }

    /// 渲染引擎信息面板
    pub fn show(&self, ui: &mut Ui, engine_name: Option<&str>) {
        ui.heading("Engine");
        ui.separator();

        if let Some(name) = engine_name {
            ui.label(format!("Engine: {name}"));
        } else {
            ui.label("Engine unavailable");
        }

        ui.label("Depth: --");
        ui.label("Evaluation: --");
        ui.label("Best Move: --");
    }
}
