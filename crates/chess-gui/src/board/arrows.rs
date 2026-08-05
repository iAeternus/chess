//! 箭头渲染：分析模式下的棋盘标注箭头

use egui::{Stroke, Vec2};

use crate::board::layout::BoardLayout;
use crate::board::state::BoardArrow;

pub struct ArrowRenderer;

impl ArrowRenderer {
    /// 绘制所有箭头（线段 + 三角形箭头头部）。
    pub fn paint(
        painter: &egui::Painter,
        layout: &BoardLayout,
        arrows: &[BoardArrow],
        flipped: bool,
    ) {
        for arrow in arrows {
            Self::draw_arrow(painter, layout, arrow, flipped);
        }
    }

    fn draw_arrow(
        painter: &egui::Painter,
        layout: &BoardLayout,
        arrow: &BoardArrow,
        flipped: bool,
    ) {
        let from_center = layout.square_center(arrow.from, flipped);
        let to_center = layout.square_center(arrow.to, flipped);

        let start = from_center;
        let end = to_center;

        // 箭头线宽
        let width = layout.square_size * 0.12;

        // 方向向量
        let dir = end - start;
        let len = dir.length();
        if len < 1.0 {
            return;
        }
        let unit = dir / len;

        // 箭头头部三角形大小
        let head_len = layout.square_size * 0.35;
        let head_width = layout.square_size * 0.18;

        // 线段终点（在箭头头部之前）
        let line_end = end - unit * head_len * 0.6;

        // 绘制线段
        painter.line_segment([start, line_end], Stroke::new(width, arrow.color));

        // 绘制箭头头部三角形
        let perp = Vec2::new(-unit.y, unit.x);
        let tip = end;
        let left = end - unit * head_len + perp * head_width;
        let right = end - unit * head_len - perp * head_width;

        painter.add(egui::Shape::convex_polygon(
            vec![tip, left, right],
            arrow.color,
            Stroke::new(1.0_f32, arrow.color),
        ));
    }
}
