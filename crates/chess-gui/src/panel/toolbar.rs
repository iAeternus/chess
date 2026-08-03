//! 工具栏：单行布局（Flip / New / Open / Save）。

/// 工具栏操作
pub enum ToolbarAction {
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
    pub fn show(&self, ui: &mut egui::Ui) -> Vec<ToolbarAction> {
        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            // 棋盘操作
            if ui
                .button("↻ Flip")
                .on_hover_text("Flip Board (R)")
                .clicked()
            {
                actions.push(ToolbarAction::FlipBoard);
            }
            if ui
                .button("✦ New")
                .on_hover_text("New Game (N)")
                .clicked()
            {
                actions.push(ToolbarAction::NewGame);
            }
            ui.separator();
            // 文件操作
            if ui
                .button("📂 Open")
                .on_hover_text("Open PGN file")
                .clicked()
            {
                actions.push(ToolbarAction::OpenPgn);
            }
            if ui
                .button("💾 Save")
                .on_hover_text("Save PGN file")
                .clicked()
            {
                actions.push(ToolbarAction::SavePgn);
            }
        });

        actions
    }
}
