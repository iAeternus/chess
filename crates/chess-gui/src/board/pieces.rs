//! 棋子渲染：静态棋子、拖拽残影、拖拽浮子

use chess_core::Square;
use egui::{Pos2, Rect};

use crate::board::layout::BoardLayout;
use crate::board::state::BoardState;
use crate::piece::texture::PieceTextureManager;
use crate::theme::ThemeColors;

/// 棋子占格子的比例
const PIECE_RATIO: f32 = 0.90;

pub struct PieceRenderer;

impl PieceRenderer {
    /// 按顺序绘制：静态棋子 -> 拖拽残影 -> 拖拽浮子。
    ///
    /// 遍历视角格，经 `Square::view` 反查棋盘格棋子，视角转换全部委托 core
    pub fn paint(
        painter: &egui::Painter,
        layout: &BoardLayout,
        state: &BoardState,
        textures: &PieceTextureManager,
        colors: &ThemeColors,
    ) {
        let sq_size = layout.square_size;
        let view_from = state.view_from;

        // 静态棋子（所有 64 格，跳过正在拖拽的来源格）
        for rank in 0..8u8 {
            for file in 0..8u8 {
                let view_sq = Square::from_coord(file, rank).unwrap();
                let sq = view_sq.view(view_from);

                // 跳过正在拖拽的棋子（单独绘制浮子）
                if let Some((_piece, from, _pos)) = &state.drag
                    && sq == *from
                {
                    continue;
                }

                if let Some(piece) = state.position.piece_at(sq) {
                    let c = layout.square_center(view_sq);
                    textures.render(painter, piece.color, piece.kind, c, sq_size * PIECE_RATIO);
                }
            }
        }

        // 拖拽浮子 + 残影
        if let Some((piece, from, mouse_pos)) = &state.drag {
            // 来源格半透明残影（Lichess: opacity 0.3）
            let sc = layout.square_center(from.view(view_from));
            let ghost_half = sq_size * PIECE_RATIO / 2.0;
            let ghost_rect = Rect::from_min_max(
                Pos2::new(sc.x - ghost_half, sc.y - ghost_half),
                Pos2::new(sc.x + ghost_half, sc.y + ghost_half),
            );
            let tex = textures.get(piece.color, piece.kind);
            let ghost_tint = colors.drag_ghost_tint;
            painter.image(
                tex.id(),
                ghost_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                ghost_tint,
            );

            // 鼠标位置跟随棋子（完全不透明）
            let half = sq_size * PIECE_RATIO / 2.0;
            let fr = Rect::from_min_max(
                Pos2::new(mouse_pos.x - half, mouse_pos.y - half),
                Pos2::new(mouse_pos.x + half, mouse_pos.y + half),
            );
            painter.image(
                tex.id(),
                fr,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }
}
