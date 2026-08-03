//! 主题系统：Dark/Light 主题定义与 egui 样式配置。
//!
//! 棋盘使用 Lichess 风格配色（绿/奶油色），不受 UI 主题切换影响。

use egui::Color32;

/// 应用主题
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTheme {
    #[default]
    Dark,
    Light,
}

/// 所有可配置的颜色
pub struct ThemeColors {
    // UI 颜色
    pub bg: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub accent: Color32,

    // 棋盘颜色
    pub board_light: Color32,
    pub board_dark: Color32,

    // 棋盘标注颜色
    pub label_color: Color32,

    // 将军光晕（三层径向渐变）
    pub check_glow_inner: Color32,
    pub check_glow_mid: Color32,
    pub check_glow_outer: Color32,

    // 选中高亮
    pub selected_highlight: Color32,

    // 合法走法提示
    pub legal_move_dot: Color32,
    pub capture_ring: Color32,

    // 最后一步高亮
    pub last_move_from: Color32,
    pub last_move_to: Color32,

    // 拖拽来源
    pub drag_source: Color32,
}

impl AppTheme {
    /// 获取主题颜色
    ///
    /// 棋盘颜色始终为 Lichess 风格：浅格 #eeeed2，深格 #769656
    pub fn colors(&self) -> ThemeColors {
        // Lichess 标准棋盘配色
        let board_light = Color32::from_rgb(238, 238, 210); // #eeeed2
        let board_dark = Color32::from_rgb(118, 150, 86); // #769656

        match self {
            Self::Dark => ThemeColors {
                bg: Color32::from_rgb(30, 30, 30),
                surface: Color32::from_rgb(40, 40, 40),
                text: Color32::from_rgb(220, 220, 220),
                accent: Color32::from_rgb(100, 150, 255),

                board_light,
                board_dark,
                label_color: Color32::from_rgb(160, 160, 160),

                check_glow_inner: Color32::from_rgba_premultiplied(255, 40, 40, 90),
                check_glow_mid: Color32::from_rgba_premultiplied(255, 60, 60, 50),
                check_glow_outer: Color32::from_rgba_premultiplied(255, 80, 80, 25),

                selected_highlight: Color32::from_rgba_premultiplied(100, 180, 100, 140),
                legal_move_dot: Color32::from_rgba_premultiplied(0, 0, 0, 50),
                capture_ring: Color32::from_rgba_premultiplied(0, 0, 0, 60),

                last_move_from: Color32::from_rgba_premultiplied(255, 255, 100, 60),
                last_move_to: Color32::from_rgba_premultiplied(255, 255, 50, 80),

                drag_source: Color32::from_rgba_premultiplied(140, 210, 140, 100),
            },
            Self::Light => ThemeColors {
                bg: Color32::from_rgb(245, 245, 245),
                surface: Color32::from_rgb(255, 255, 255),
                text: Color32::from_rgb(30, 30, 30),
                accent: Color32::from_rgb(60, 100, 200),

                board_light,
                board_dark,
                label_color: Color32::from_rgb(120, 120, 120),

                check_glow_inner: Color32::from_rgba_premultiplied(255, 40, 40, 90),
                check_glow_mid: Color32::from_rgba_premultiplied(255, 60, 60, 50),
                check_glow_outer: Color32::from_rgba_premultiplied(255, 80, 80, 25),

                selected_highlight: Color32::from_rgba_premultiplied(100, 180, 100, 140),
                legal_move_dot: Color32::from_rgba_premultiplied(0, 0, 0, 40),
                capture_ring: Color32::from_rgba_premultiplied(0, 0, 0, 55),

                last_move_from: Color32::from_rgba_premultiplied(255, 240, 100, 60),
                last_move_to: Color32::from_rgba_premultiplied(255, 230, 50, 80),

                drag_source: Color32::from_rgba_premultiplied(140, 210, 140, 100),
            },
        }
    }

    /// 应用 egui 全局样式
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
        visuals.widgets.inactive.bg_fill = wi;
        visuals.widgets.hovered.bg_fill = wh;
        visuals.widgets.active.bg_fill = colors.accent;
        visuals.selection.bg_fill = colors.accent;

        ctx.set_visuals(visuals);
    }
}
