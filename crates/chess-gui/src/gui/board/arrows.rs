//! 箭头渲染：分析模式下的棋盘标注箭头

use chess_core::Color;
use egui::{Shape, Stroke, Vec2};

use crate::gui::board::layout::BoardLayout;
use crate::gui::board::state::BoardArrow;

pub struct ArrowRenderer;

impl ArrowRenderer {
    /// 绘制所有箭头（线段 + 三角形箭头头部）
    pub fn paint(
        painter: &egui::Painter,
        layout: &BoardLayout,
        arrows: &[BoardArrow],
        preview: Option<&BoardArrow>,
        view_from: Color,
    ) {
        for arrow in arrows {
            Self::draw_arrow(painter, layout, arrow, view_from);
        }

        // 临时箭头
        if let Some(arrow) = preview {
            Self::draw_arrow(painter, layout, arrow, view_from);
        }
    }

    fn draw_arrow(
        painter: &egui::Painter,
        layout: &BoardLayout,
        arrow: &BoardArrow,
        view_from: Color,
    ) {
        let start = layout.square_center(arrow.from.view(view_from));
        let end = layout.square_center(arrow.to.view(view_from));

        let dir = end - start;
        let len = dir.length();

        if len < 1.0 {
            return;
        }

        let unit = dir / len;
        let perp = Vec2::new(-unit.y, unit.x);
        let shaft_width = layout.square_size * 0.07;
        let head_len = layout.square_size * 0.22;
        let head_width = layout.square_size * 0.16;

        // 箭头根部
        let base = end - unit * head_len;
        // 箭身
        painter.line_segment([start, base], Stroke::new(shaft_width, arrow.color));
        // 箭头
        let left = base + perp * head_width;
        let right = base - perp * head_width;

        painter.add(Shape::convex_polygon(
            vec![end, left, right],
            arrow.color,
            Stroke::NONE,
        ));
    }
}
