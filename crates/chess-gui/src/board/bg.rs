//! 棋盘背景渲染：外围填充、64格交替色、圆角边框

use chess_core::Square;
use egui::{CornerRadius, Pos2, Rect, Stroke, StrokeKind};

use crate::board::layout::BoardLayout;
use crate::theme::ThemeColors;

/// 圆角半径占格子的比例
const ROUNDING_RATIO: f32 = 0.06;

pub struct BoardBgRenderer;

impl BoardBgRenderer {
    /// 绘制棋盘背景：外围背景色、64 格交替色、内框圆角描边。
    pub fn paint(painter: &egui::Painter, layout: &BoardLayout, colors: &ThemeColors) {
        // 外围背景填充
        painter.rect_filled(
            Rect::from_min_size(Pos2::ZERO, layout.outer_rect.size()),
            0.0,
            colors.bg,
        );

        // 64 格
        for rank in 0..8u8 {
            for file in 0..8u8 {
                let sq = Square::from_coord(file, rank).unwrap(); // SAFETY: fille and rand is valid here
                let r = layout.square_rect(sq); // 格子模式不随翻转变化（180° 旋转下奇偶不变）
                let bg = if (file + rank) % 2 == 0 {
                    colors.board_light
                } else {
                    colors.board_dark
                };
                painter.rect_filled(r, 0.0, bg);
            }
        }

        // 内框圆角描边
        let rounding = CornerRadius::same((layout.square_size * ROUNDING_RATIO) as u8);
        painter.rect_stroke(
            layout.board_rect_local(),
            rounding,
            Stroke::new(layout.square_size * 0.04, colors.bg),
            StrokeKind::Middle,
        );
    }
}
