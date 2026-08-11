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
    // UI
    pub bg: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub accent: Color32,

    // 棋盘（Lichess 风格）
    pub board_light: Color32,
    pub board_dark: Color32,
    pub coord_light: Color32,
    pub coord_dark: Color32,

    /// 将军中心颜色
    pub check_glow_inner: Color32,
    /// 将军外围透明颜色
    pub check_glow_outer: Color32,

    /// 选中格
    pub selected_highlight: Color32,

    /// 普通走法圆点
    pub legal_move_dot: Color32,

    /// 吃子圆环
    pub capture_ring: Color32,

    /// 最近一步
    pub last_move_from: Color32,
    pub last_move_to: Color32,

    /// 拖拽来源
    pub drag_source: Color32,

    /// 分析箭头
    pub arrow_color: Color32,
    /// 箭头预览
    pub arrow_preview_color: Color32,

    /// 非活跃 widget 背景
    pub widget_inactive_bg: Color32,
    /// 悬停 widget 背景
    pub widget_hovered_bg: Color32,

    /// 右键拖拽创建箭头时的初始颜色
    pub arrow_drag_color: Color32,

    /// 拖拽棋子时来源格的半透明残影颜色
    pub drag_ghost_tint: Color32,

    /// 走法列表当前走法高亮背景
    pub move_list_highlight_bg: Color32,
    /// 走法列表编号/表头颜色
    pub move_list_dim_text: Color32,
    /// 引擎信息面板次要文本颜色
    pub panel_dim_text: Color32,
}

impl AppTheme {
    /// 获取主题颜色
    ///
    /// 棋盘颜色始终为 Lichess 风格：浅格 #eeeed2，深格 #769656
    pub fn colors(&self) -> ThemeColors {
        match self {
            Self::Dark => ThemeColors {
                bg: Color32::from_rgb(30, 30, 30),
                surface: Color32::from_rgb(40, 40, 40),
                text: Color32::from_rgb(220, 220, 220),
                accent: Color32::from_rgb(100, 150, 255),
                widget_inactive_bg: Color32::from_rgb(55, 55, 60),
                widget_hovered_bg: Color32::from_rgb(70, 70, 78),
                ..Default::default()
            },

            Self::Light => ThemeColors {
                bg: Color32::from_rgb(245, 245, 245),
                surface: Color32::from_rgb(255, 255, 255),
                text: Color32::from_rgb(30, 30, 30),
                accent: Color32::from_rgb(60, 100, 200),
                widget_inactive_bg: Color32::from_rgb(215, 215, 220),
                widget_hovered_bg: Color32::from_rgb(195, 195, 205),
                ..Default::default()
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

        visuals.widgets.noninteractive.bg_fill = colors.surface;
        visuals.widgets.inactive.bg_fill = colors.widget_inactive_bg;
        visuals.widgets.hovered.bg_fill = colors.widget_hovered_bg;
        visuals.widgets.active.bg_fill = colors.accent;
        visuals.selection.bg_fill = colors.accent;

        ctx.set_visuals(visuals);
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            // UI 默认
            bg: Color32::from_rgb(30, 30, 30),
            surface: Color32::from_rgb(40, 40, 40),
            text: Color32::from_rgb(220, 220, 220),
            accent: Color32::from_rgb(100, 150, 255),

            // Lichess Classic Board
            board_light: Color32::from_rgb(240, 217, 181), // #f0d9b5
            board_dark: Color32::from_rgb(181, 136, 99),   // #b58863

            // 坐标
            coord_light: Color32::from_rgba_unmultiplied(255, 255, 255, 200), // 深色格上的浅色坐标
            coord_dark: Color32::from_rgba_unmultiplied(100, 70, 40, 220),    // 浅色格上的深色坐标

            // 将军光晕
            check_glow_inner: Color32::from_rgba_unmultiplied(220, 40, 30, 160),
            check_glow_outer: Color32::from_rgba_unmultiplied(220, 40, 30, 0),

            // 选中
            selected_highlight: Color32::from_rgba_unmultiplied(180, 200, 80, 150),

            // 合法走法
            legal_move_dot: Color32::from_rgba_unmultiplied(0, 0, 0, 60),
            capture_ring: Color32::from_rgba_unmultiplied(0, 0, 0, 90),

            // 最后一步
            last_move_from: Color32::from_rgba_unmultiplied(250, 230, 80, 100),
            last_move_to: Color32::from_rgba_unmultiplied(240, 210, 50, 140),

            // 拖拽
            drag_source: Color32::from_rgba_unmultiplied(100, 180, 100, 120),

            // 分析模式箭头
            arrow_color: Color32::from_rgba_unmultiplied(40, 140, 40, 150),
            arrow_preview_color: Color32::from_rgba_unmultiplied(40, 140, 40, 80),

            // Widget
            widget_inactive_bg: Color32::from_rgb(55, 55, 60),
            widget_hovered_bg: Color32::from_rgb(70, 70, 78),

            // 箭头拖拽初始颜色
            arrow_drag_color: Color32::from_rgba_unmultiplied(0, 200, 0, 100),

            // 拖拽残影
            drag_ghost_tint: Color32::from_rgba_premultiplied(255, 255, 255, 77),

            // 面板
            move_list_highlight_bg: Color32::from_rgba_premultiplied(100, 150, 255, 60),
            move_list_dim_text: Color32::from_rgb(130, 130, 130),
            panel_dim_text: Color32::from_rgb(150, 150, 150),
        }
    }
}
