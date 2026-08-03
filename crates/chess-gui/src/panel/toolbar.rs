//! 工具栏：导航按钮、翻转棋盘、新局。

pub struct Toolbar;

pub enum ToolbarAction {
    GoToStart, GoBack, GoForward, GoToEnd,
    FlipBoard, NewGame, OpenPgn,
}

impl Toolbar {
    pub fn new() -> Self { Self }

    pub fn show(&self, ui: &mut egui::Ui, can_back: bool, can_forward: bool) -> Vec<ToolbarAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            if ui.button("|<").clicked() { actions.push(ToolbarAction::GoToStart); }
            if ui.add_enabled(can_back, egui::Button::new("<")).clicked() { actions.push(ToolbarAction::GoBack); }
            if ui.add_enabled(can_forward, egui::Button::new(">")).clicked() { actions.push(ToolbarAction::GoForward); }
            if ui.button(">|").clicked() { actions.push(ToolbarAction::GoToEnd); }
        });
        ui.horizontal(|ui| {
            if ui.button("↻").clicked() { actions.push(ToolbarAction::FlipBoard); }
            if ui.button("New").clicked() { actions.push(ToolbarAction::NewGame); }
        });
        actions
    }
}
