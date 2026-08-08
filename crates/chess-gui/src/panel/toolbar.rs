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
    ///
    /// `replay_mode` — Replay 模式下禁用 New 按钮
    pub fn show(&self, ui: &mut egui::Ui, replay_mode: bool) -> Vec<ToolbarAction> {
        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            let buttons = [
                ("↻ Flip", "Flip Board (R)", !false, ToolbarAction::FlipBoard),
                ("✦ New", "New Game (N)", !replay_mode, ToolbarAction::NewGame),
                ("📂 Open", "Open PGN file", true, ToolbarAction::OpenPgn),
                ("💾 Save", "Save PGN file", true, ToolbarAction::SavePgn),
            ];

            for (icon, tooltip, enabled, action) in buttons {
                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(egui::RichText::new(icon).size(16.0)),
                    )
                    .on_hover_text(tooltip)
                    .clicked()
                {
                    actions.push(action);
                }
            }
        });

        actions
    }
}
