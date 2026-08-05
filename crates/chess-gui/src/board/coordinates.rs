//! 坐标标注渲染：行列标签（a-h, 1-8），Lichess coords-out 风格

use egui::{Align2, FontId, Pos2};

use crate::board::layout::BoardLayout;
use crate::theme::ThemeColors;

pub struct CoordinateRenderer;

impl CoordinateRenderer {
    /// 绘制行号（1-8）和列号（a-h）。
    ///
    /// 行号位于左侧边距垂直居中，列号位于底部边距水平居中。
    pub fn paint(
        painter: &egui::Painter,
        layout: &BoardLayout,
        colors: &ThemeColors,
        flipped: bool,
    ) {
        let files: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let board = layout.board_rect_local();
        let font = FontId::monospace(layout.square_size * 0.16);

        for i in 0..8u8 {
            let ri = if flipped { i } else { 7 - i };
            let fi = if flipped { 7 - i } else { i };

            // 行号（1-8）：左侧外边距垂直居中
            let rx = layout.coord_margin / 2.0;
            let ry = board.min.y + ri as f32 * layout.square_size + layout.square_size / 2.0;
            painter.text(
                Pos2::new(rx, ry),
                Align2::CENTER_CENTER,
                (i + 1).to_string(),
                font.clone(),
                colors.coord_light,
            );

            // 列号（a-h）：底部外边距水平居中
            let fx = board.min.x + fi as f32 * layout.square_size + layout.square_size / 2.0;
            let fy = board.max.y + layout.coord_margin / 2.0;
            painter.text(
                Pos2::new(fx, fy),
                Align2::CENTER_CENTER,
                files[i as usize].to_string(),
                font.clone(),
                colors.coord_light,
            );
        }
    }
}
