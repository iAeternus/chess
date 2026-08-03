//! 主题系统：Dark/Light 主题定义与 egui 样式配置。
//!
//! 棋盘颜色始终使用固定配色（不受主题切换影响）。

use egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme { Dark, Light }

impl Default for AppTheme {
    fn default() -> Self { Self::Dark }
}

pub struct ThemeColors {
    pub bg: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub accent: Color32,
    pub board_light: Color32,
    pub board_dark: Color32,
    pub check_glow_inner: Color32,
    pub check_glow_mid: Color32,
    pub check_glow_outer: Color32,
    pub selected_highlight: Color32,
    pub legal_move_dot: Color32,
    pub capture_ring: Color32,
    pub last_move_from: Color32,
    pub last_move_to: Color32,
    pub label_color: Color32,
    pub drag_source: Color32,
}

impl AppTheme {
    /// 棋盘颜色始终使用 Light 模式配色（不受主题影响）
    pub fn colors(&self) -> ThemeColors {
        // 棋盘固定配色（lichess 风格）
        let board_light = Color32::from_rgb(240, 217, 181);
        let board_dark = Color32::from_rgb(181, 136, 99);

        match self {
            Self::Dark => ThemeColors {
                bg: Color32::from_rgb(30, 30, 30),
                surface: Color32::from_rgb(40, 40, 40),
                text: Color32::from_rgb(220, 220, 220),
                accent: Color32::from_rgb(100, 150, 255),
                board_light,
                board_dark,
                check_glow_inner: Color32::from_rgba_premultiplied(255, 40, 40, 90),
                check_glow_mid: Color32::from_rgba_premultiplied(255, 60, 60, 50),
                check_glow_outer: Color32::from_rgba_premultiplied(255, 80, 80, 25),
                selected_highlight: Color32::from_rgba_premultiplied(100, 180, 100, 140),
                legal_move_dot: Color32::from_rgba_premultiplied(100, 200, 100, 150),
                capture_ring: Color32::from_rgba_premultiplied(100, 180, 100, 200),
                last_move_from: Color32::from_rgba_premultiplied(255, 255, 100, 50),
                last_move_to: Color32::from_rgba_premultiplied(255, 255, 50, 70),
                label_color: Color32::from_rgb(140, 140, 140),
                drag_source: Color32::from_rgba_premultiplied(140, 210, 140, 100),
            },
            Self::Light => ThemeColors {
                bg: Color32::from_rgb(245, 245, 245),
                surface: Color32::from_rgb(255, 255, 255),
                text: Color32::from_rgb(30, 30, 30),
                accent: Color32::from_rgb(60, 100, 200),
                board_light,
                board_dark,
                check_glow_inner: Color32::from_rgba_premultiplied(255, 40, 40, 90),
                check_glow_mid: Color32::from_rgba_premultiplied(255, 60, 60, 50),
                check_glow_outer: Color32::from_rgba_premultiplied(255, 80, 80, 25),
                selected_highlight: Color32::from_rgba_premultiplied(100, 180, 100, 140),
                legal_move_dot: Color32::from_rgba_premultiplied(100, 200, 100, 150),
                capture_ring: Color32::from_rgba_premultiplied(100, 180, 100, 200),
                last_move_from: Color32::from_rgba_premultiplied(255, 240, 100, 50),
                last_move_to: Color32::from_rgba_premultiplied(255, 230, 50, 70),
                label_color: Color32::from_rgb(100, 100, 100),
                drag_source: Color32::from_rgba_premultiplied(140, 210, 140, 100),
            },
        }
    }

    pub fn apply_egui_theme(&self, ctx: &egui::Context) {
        let mut visuals = match self {
            Self::Dark => egui::Visuals::dark(),
            Self::Light => egui::Visuals::light(),
        };
        let colors = self.colors();
        visuals.panel_fill = colors.bg;
        visuals.window_fill = colors.bg;
        visuals.extreme_bg_color = colors.surface;
        visuals.override_text_color = Some(colors.text);

        let (wi, wh) = match self {
            Self::Dark => (Color32::from_rgb(55, 55, 60), Color32::from_rgb(70, 70, 78)),
            Self::Light => (Color32::from_rgb(215, 215, 220), Color32::from_rgb(195, 195, 205)),
        };
        visuals.widgets.noninteractive.bg_fill = colors.surface;
        visuals.widgets.inactive.bg_fill = wi;
        visuals.widgets.hovered.bg_fill = wh;
        visuals.widgets.active.bg_fill = colors.accent;
        visuals.selection.bg_fill = colors.accent;
        ctx.set_visuals(visuals);
    }
}
