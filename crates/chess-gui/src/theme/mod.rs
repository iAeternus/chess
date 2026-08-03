//! 主题系统：Dark/Light 主题定义与 egui 样式配置。

use egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Dark,
    Light,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::Dark
    }
}

pub struct ThemeColors {
    pub bg: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub accent: Color32,
    pub board_light: Color32,
    pub board_dark: Color32,
    pub check_highlight: Color32,
    pub selected_highlight: Color32,
    pub legal_move_dot: Color32,
    pub last_move_from: Color32,
    pub last_move_to: Color32,
    /// 坐标标注颜色（高对比度）
    pub label_color: Color32,
}

impl AppTheme {
    pub fn colors(&self) -> ThemeColors {
        match self {
            Self::Dark => ThemeColors {
                bg: Color32::from_rgb(30, 30, 30),
                surface: Color32::from_rgb(40, 40, 40),
                text: Color32::from_rgb(220, 220, 220),
                accent: Color32::from_rgb(100, 150, 255),
                board_light: Color32::from_rgb(58, 58, 58),
                board_dark: Color32::from_rgb(38, 38, 38),
                check_highlight: Color32::from_rgba_premultiplied(255, 60, 60, 120),
                selected_highlight: Color32::from_rgba_premultiplied(100, 180, 100, 140),
                legal_move_dot: Color32::from_rgba_premultiplied(180, 180, 180, 90),
                last_move_from: Color32::from_rgba_premultiplied(255, 255, 100, 50),
                last_move_to: Color32::from_rgba_premultiplied(255, 255, 50, 70),
                label_color: Color32::from_rgb(140, 140, 140),
            },
            Self::Light => ThemeColors {
                bg: Color32::from_rgb(245, 245, 245),
                surface: Color32::from_rgb(255, 255, 255),
                text: Color32::from_rgb(30, 30, 30),
                accent: Color32::from_rgb(60, 100, 200),
                board_light: Color32::from_rgb(240, 217, 181),
                board_dark: Color32::from_rgb(181, 136, 99),
                check_highlight: Color32::from_rgba_premultiplied(255, 50, 50, 110),
                selected_highlight: Color32::from_rgba_premultiplied(80, 160, 80, 130),
                legal_move_dot: Color32::from_rgba_premultiplied(100, 100, 100, 70),
                last_move_from: Color32::from_rgba_premultiplied(255, 240, 100, 50),
                last_move_to: Color32::from_rgba_premultiplied(255, 230, 50, 70),
                label_color: Color32::from_rgb(100, 100, 100),
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

        // Widget 颜色根据主题适配
        let (widget_inactive, widget_hovered) = match self {
            Self::Dark => (
                Color32::from_rgb(55, 55, 60),
                Color32::from_rgb(70, 70, 78),
            ),
            Self::Light => (
                Color32::from_rgb(215, 215, 220),
                Color32::from_rgb(195, 195, 205),
            ),
        };
        visuals.widgets.noninteractive.bg_fill = colors.surface;
        visuals.widgets.inactive.bg_fill = widget_inactive;
        visuals.widgets.hovered.bg_fill = widget_hovered;
        visuals.widgets.active.bg_fill = colors.accent;
        visuals.selection.bg_fill = colors.accent;

        ctx.set_visuals(visuals);
    }
}
