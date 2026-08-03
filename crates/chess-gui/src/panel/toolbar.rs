//! 工具栏：导航按钮、翻转棋盘、新局、PGN 操作。

/// 工具栏操作
pub enum ToolbarAction {
    GoToStart,
    GoBack,
    GoForward,
    GoToEnd,
    FlipBoard,
    NewGame,
    OpenPgn,
    SavePgn,
}

pub struct Toolbar;

impl Toolbar {
    pub fn new() -> Self {
        Self
    }

    /// 渲染工具栏并返回用户触发的操作列表
    pub fn show(
        &self,
        ui: &mut egui::Ui,
        can_back: bool,
        can_forward: bool,
    ) -> Vec<ToolbarAction> {
        let mut actions = Vec::new();

        // ── 导航 ──
        ui.horizontal(|ui| {
            ui.label("Nav:");
            if ui.button("⏮").on_hover_text("Start (Home)").clicked() {
                actions.push(ToolbarAction::GoToStart);
            }
            if ui
                .add_enabled(can_back, egui::Button::new("⏪"))
                .on_hover_text("Back (←)")
                .clicked()
            {
                actions.push(ToolbarAction::GoBack);
            }
            if ui
                .add_enabled(can_forward, egui::Button::new("⏩"))
                .on_hover_text("Forward (→)")
                .clicked()
            {
                actions.push(ToolbarAction::GoForward);
            }
            if ui.button("⏭").on_hover_text("End (End)").clicked() {
                actions.push(ToolbarAction::GoToEnd);
            }
        });

        // ── 操作 ──
        ui.horizontal(|ui| {
            if ui.button("↻ Flip").on_hover_text("Flip Board (R)").clicked() {
                actions.push(ToolbarAction::FlipBoard);
            }
            if ui.button("✦ New").on_hover_text("New Game (N)").clicked() {
                actions.push(ToolbarAction::NewGame);
            }
        });

        // ── PGN ──
        ui.horizontal(|ui| {
            if ui.button("📂 Open").on_hover_text("Open PGN file").clicked() {
                actions.push(ToolbarAction::OpenPgn);
            }
            if ui.button("💾 Save").on_hover_text("Save PGN file").clicked() {
                actions.push(ToolbarAction::SavePgn);
            }
        });

        actions
    }
}
