//! 坐标标注渲染：行列标签（a-h, 1-8），Lichess coords-out 风格

use chess_core::{Color, Square};
use egui::{Align2, FontId, Pos2};

use crate::board::layout::BoardLayout;
use crate::theme::ThemeColors;

pub struct CoordRenderer;

impl CoordRenderer {
    /// 绘制行号（1-8）和列号（a-h）。
    ///
    /// 行号位于左侧边距垂直居中，列号位于底部边距水平居中。
    /// 标签由视角格反查棋盘格推导，翻转由 core 的 `Square::view` 处理。
    pub fn paint(
        painter: &egui::Painter,
        layout: &BoardLayout,
        colors: &ThemeColors,
        view_from: Color,
    ) {
        let board = layout.board_rect_local();
        let font = FontId::monospace(layout.square_size * 0.16);

        // 根据主题选择坐标颜色：Dark 用浅色，Light 用深色
        let coord_color = if colors.bg.r() < 128 {
            colors.coord_light
        } else {
            colors.coord_dark
        };

        for i in 0..8u8 {
            // 行号（1-8）：左侧外边距垂直居中
            let row_sq = Square::from_coord(0, 7 - i).unwrap().view(view_from);
            let rx = layout.coord_margin / 2.0;
            let ry = board.min.y + i as f32 * layout.square_size + layout.square_size / 2.0;
            painter.text(
                Pos2::new(rx, ry),
                Align2::CENTER_CENTER,
                (row_sq.rank() + 1).to_string(),
                font.clone(),
                coord_color,
            );

            // 列号（a-h）：底部外边距水平居中
            let col_sq = Square::from_coord(i, 0).unwrap().view(view_from);
            let fx = board.min.x + i as f32 * layout.square_size + layout.square_size / 2.0;
            let fy = board.max.y + layout.coord_margin / 2.0;
            painter.text(
                Pos2::new(fx, fy),
                Align2::CENTER_CENTER,
                ((b'a' + col_sq.file()) as char).to_string(),
                font.clone(),
                coord_color,
            );
        }
    }
}
