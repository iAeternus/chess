//! 高亮渲染：最后一步、将军光晕、选中、拖拽来源、合法走法提示

use chess_core::MoveFlag;
use egui::{Color32, Mesh, Stroke, epaint};

use crate::board::layout::BoardLayout;
use crate::board::state::BoardState;
use crate::theme::ThemeColors;

pub struct HighlightRenderer;

impl HighlightRenderer {
    /// 按层叠顺序绘制所有高亮效果。
    pub fn paint(
        painter: &egui::Painter,
        layout: &BoardLayout,
        state: &BoardState,
        colors: &ThemeColors,
        flipped: bool,
    ) {
        let sq = layout.square_size;

        // 最后一步走法 from/to 高亮
        if let Some(mv) = state.last_move {
            for sq_sq in [mv.from(), mv.to()] {
                let color = if sq_sq == mv.to() {
                    colors.last_move_to
                } else {
                    colors.last_move_from
                };
                let r = layout.square_rect(sq_sq, flipped);
                painter.rect_filled(r, 0.0, color);
            }
        }

        // 将军光晕
        if let Some(king_sq) = state.king_in_check {
            Self::paint_check_glow(painter, layout.square_center(king_sq, flipped), sq);
        }

        // 选中高亮
        if let Some(sq_sel) = state.selected_square {
            let r = layout.square_rect(sq_sel, flipped);
            painter.rect_filled(r, 0.0, colors.selected_highlight);
        }

        // 拖拽来源高亮
        if let Some((_piece, from, _pos)) = &state.drag {
            let r = layout.square_rect(*from, flipped);
            painter.rect_filled(r, 0.0, colors.drag_source);
        }

        // 合法走法提示：圆点（普通走法）/ 圆环（吃子）
        for mv in &state.legal_moves {
            let tgt = mv.to();
            let c = layout.square_center(tgt, flipped);

            let is_capture =
                state.position.piece_at(tgt).is_some() || mv.flag() == MoveFlag::EnPassant;

            if is_capture {
                // 空心圆环
                painter.circle_stroke(c, sq * 0.42, Stroke::new(sq * 0.06, colors.capture_ring));
            } else {
                // 实心圆点
                painter.circle_filled(c, sq * 0.15, colors.legal_move_dot);
            }
        }
    }

    /// 绘制将军光晕
    ///
    /// Lichess 风格：
    /// - 中心红色较强
    /// - 外圈渐隐
    /// - 半径小于棋格，避免覆盖整个格子
    fn paint_check_glow(painter: &egui::Painter, center: egui::Pos2, square_size: f32) {
        let radius = square_size * 0.48;
        let segments = 64;
        let mut mesh = Mesh::default();

        // 中心：强红色
        mesh.vertices.push(epaint::Vertex {
            pos: center,
            uv: egui::epaint::WHITE_UV,
            color: Color32::from_rgba_unmultiplied(220, 0, 0, 140),
        });

        // 外圈：完全透明
        for i in 0..=segments {
            let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
            let pos = center + egui::vec2(angle.cos(), angle.sin()) * radius;
            mesh.vertices.push(epaint::Vertex {
                pos,
                uv: egui::epaint::WHITE_UV,
                color: Color32::from_rgba_unmultiplied(220, 0, 0, 0),
            });
        }
        for i in 0..segments {
            mesh.indices.extend([0, (i + 1) as u32, (i + 2) as u32]);
        }
        painter.add(egui::Shape::mesh(mesh));
    }
}
