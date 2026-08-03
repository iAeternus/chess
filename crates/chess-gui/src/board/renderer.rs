//! 棋盘渲染器。
//!
//! 使用 egui Painter API 绘制 8×8 棋盘格子、外部坐标标注、高亮和棋子。
//!
//! 布局结构：
//! ```text
//! ┌──────────────────────────────┐  ← board_rect（外层，含 margin）
//! │ 8                            │
//! │ 7    ┌──────────────────┐    │
//! │ 6    │                  │    │
//! │ 5    │   8×8 格子       │    │  ← inner_rect（实际棋盘）
//! │ 4    │   (inner_rect)   │    │
//! │ 3    │                  │    │
//! │ 2    │                  │    │
//! │ 1    └──────────────────┘    │
//! │      a  b  c  d  e  f  g  h │
//! └──────────────────────────────┘
//! ```

use chess_core::{Move, Position, Square};
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use crate::piece::texture::PieceTextureManager;
use crate::theme::ThemeColors;

/// 棋盘周围标注区域占格子尺寸的比例
const MARGIN_RATIO: f32 = 0.35;
/// 最小棋盘尺寸（像素）
const MIN_BOARD_SIZE: f32 = 480.0;

/// 棋盘渲染器
pub struct BoardRenderer {
    pub flipped: bool,
    colors: ThemeColors,
}

impl BoardRenderer {
    pub fn new(colors: ThemeColors) -> Self {
        Self {
            flipped: false,
            colors,
        }
    }

    pub fn set_colors(&mut self, colors: ThemeColors) {
        self.colors = colors;
    }

    /// 计算棋盘外层矩形（含标注 margin），居中并限制最小尺寸
    pub fn board_rect(ui: &egui::Ui) -> Rect {
        let available = ui.available_size();
        let side = available.x.min(available.y).max(MIN_BOARD_SIZE);
        let x0 = ui.cursor().min.x + (available.x - side) / 2.0;
        let y0 = ui.cursor().min.y + (available.y - side) / 2.0;
        Rect::from_min_size(Pos2::new(x0, y0), Vec2::new(side, side))
    }

    /// 内层 8×8 格子区域（不含标注 margin）
    pub fn inner_rect(board_rect: Rect) -> Rect {
        let sq = board_rect.width() / (8.0 + 2.0 * MARGIN_RATIO);
        let margin = sq * MARGIN_RATIO;
        Rect::from_min_size(
            Pos2::new(board_rect.min.x + margin, board_rect.min.y + margin),
            Vec2::new(sq * 8.0, sq * 8.0),
        )
    }

    /// 单个格子尺寸
    pub fn square_size(board_rect: Rect) -> f32 {
        Self::inner_rect(board_rect).width() / 8.0
    }

    /// 像素坐标 → 棋盘格子。pos 相对于 board_rect.min
    pub fn pos_to_square(&self, board_rect: Rect, pos: Pos2) -> Option<Square> {
        let inner = Self::inner_rect(board_rect);
        if !inner.contains(pos) {
            return None;
        }

        let sq_size = inner.width() / 8.0;
        let rel_x = pos.x - inner.min.x;
        let rel_y = pos.y - inner.min.y;

        let file = (rel_x / sq_size) as u8;
        let rank = 7 - (rel_y / sq_size) as u8; // screen y=0 → rank=7

        if file >= 8 || rank >= 8 {
            return None;
        }

        if self.flipped {
            Square::from_coord(7 - file, 7 - rank)
        } else {
            Square::from_coord(file, rank)
        }
    }

    /// 格子中心像素位置（相对于 board_rect.min）
    pub fn square_center(board_rect: Rect, sq: Square, flipped: bool) -> Pos2 {
        let inner = Self::inner_rect(board_rect);
        let sq_size = inner.width() / 8.0;
        let (file, rank) = if flipped {
            (7 - sq.file(), 7 - sq.rank())
        } else {
            (sq.file(), sq.rank())
        };
        let x = inner.min.x + file as f32 * sq_size + sq_size / 2.0;
        let y = inner.min.y + (7 - rank) as f32 * sq_size + sq_size / 2.0;
        Pos2::new(x, y)
    }

    /// 格子矩形（相对于 board_rect.min）
    fn square_rect(board_rect: Rect, sq: Square, flipped: bool) -> Rect {
        let inner = Self::inner_rect(board_rect);
        let sq_size = inner.width() / 8.0;
        let (file, rank) = if flipped {
            (7 - sq.file(), 7 - sq.rank())
        } else {
            (sq.file(), sq.rank())
        };
        let x = inner.min.x + file as f32 * sq_size;
        let y = inner.min.y + (7 - rank) as f32 * sq_size;
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(sq_size, sq_size))
    }

    /// 绘制完整棋盘
    ///
    /// `board_rect` 必须由调用方预先计算（通过 `Self::board_rect(ui)`），
    /// 这样调用方可以用同一个 rect 做点击交互。
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        board_rect: Rect,
        position: &Position,
        textures: &PieceTextureManager,
        selected: Option<Square>,
        legal_moves: &[Move],
        last_move: Option<Move>,
        in_check: bool,
    ) {
        let inner = Self::inner_rect(board_rect);
        let sq_size = inner.width() / 8.0;

        // 分配可交互区域
        let (_response, base_painter) =
            ui.allocate_painter(board_rect.size(), Sense::click_and_drag());

        let painter_offset = board_rect.min.to_vec2();

        // ── 1. 棋盘底色 ──
        base_painter.rect_filled(
            Rect::from_min_size(Pos2::ZERO, board_rect.size()),
            0.0,
            self.colors.bg,
        );

        // ── 2. 64 个格子 ──
        for rank in 0..8u8 {
            for file in 0..8u8 {
                let (draw_file, draw_rank) = if self.flipped {
                    (7 - file, 7 - rank)
                } else {
                    (file, 7 - rank)
                };

                let x = inner.min.x + draw_file as f32 * sq_size - painter_offset.x;
                let y = inner.min.y + draw_rank as f32 * sq_size - painter_offset.y;
                let rect =
                    Rect::from_min_size(Pos2::new(x, y), Vec2::new(sq_size, sq_size));

                let is_light = (file + rank) % 2 == 0;
                let bg = if is_light {
                    self.colors.board_light
                } else {
                    self.colors.board_dark
                };

                base_painter.rect_filled(rect, 0.0, bg);
            }
        }

        // ── 3. 最后一步高亮 ──
        if let Some(mv) = last_move {
            for sq in [mv.from(), mv.to()] {
                let rect = Self::square_rect(board_rect, sq, self.flipped);
                let color = if sq == mv.to() {
                    self.colors.last_move_to
                } else {
                    self.colors.last_move_from
                };
                base_painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(
                            rect.min.x - painter_offset.x,
                            rect.min.y - painter_offset.y,
                        ),
                        rect.size(),
                    ),
                    0.0,
                    color,
                );
            }
        }

        // ── 4. 将军高亮 ──
        if in_check {
            let king_sq = position.board().king_square(position.side_to_move());
            let rect = Self::square_rect(board_rect, king_sq, self.flipped);
            base_painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(rect.min.x - painter_offset.x, rect.min.y - painter_offset.y),
                    rect.size(),
                ),
                0.0,
                self.colors.check_highlight,
            );
        }

        // ── 5. 选中格子高亮 ──
        if let Some(sq) = selected {
            let rect = Self::square_rect(board_rect, sq, self.flipped);
            base_painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(rect.min.x - painter_offset.x, rect.min.y - painter_offset.y),
                    rect.size(),
                ),
                0.0,
                self.colors.selected_highlight,
            );
        }

        // ── 6. 合法走法指示 ──
        for mv in legal_moves {
            let target = mv.to();
            let center = Self::square_center(board_rect, target, self.flipped);
            let cx = center.x - painter_offset.x;
            let cy = center.y - painter_offset.y;
            let dot_radius = sq_size * 0.15;

            if position.piece_at(target).is_some()
                || mv.flag() == chess_core::MoveFlag::EnPassant
            {
                base_painter.circle_stroke(
                    Pos2::new(cx, cy),
                    sq_size * 0.42,
                    Stroke::new(dot_radius * 0.8, self.colors.legal_move_dot),
                );
            } else {
                base_painter.circle_filled(
                    Pos2::new(cx, cy),
                    dot_radius,
                    self.colors.legal_move_dot,
                );
            }
        }

        // ── 7. 棋子渲染 ──
        for rank in 0..8u8 {
            for file in 0..8u8 {
                let sq = Square::from_coord(file, rank).unwrap();
                if let Some(piece) = position.piece_at(sq) {
                    let center = Self::square_center(board_rect, sq, self.flipped);
                    let adjusted = Pos2::new(
                        center.x - painter_offset.x,
                        center.y - painter_offset.y,
                    );
                    textures.render(&base_painter, piece.color, piece.kind, adjusted, sq_size * 0.85);
                }
            }
        }

        // ── 8. 坐标标注（在 margin 区域） ──
        let font = FontId::monospace(sq_size * 0.26);
        let files: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let margin = inner.min.x; // margin 宽度

        for i in 0..8u8 {
            let display_i = if self.flipped { 7 - i } else { i };

            // 文件标注（底部 margin）
            let fx = inner.min.x + display_i as f32 * sq_size + sq_size / 2.0 - painter_offset.x;
            let fy_top = inner.max.y + margin * 0.7 - painter_offset.y;
            base_painter.text(
                Pos2::new(fx, fy_top),
                Align2::CENTER_CENTER,
                files[i as usize].to_string(),
                font.clone(),
                self.colors.label_color,
            );

            // 行标注（左侧 margin）
            let rx = margin * 0.5 - painter_offset.x;
            let ry = inner.min.y + (7 - display_i) as f32 * sq_size + sq_size / 2.0
                - painter_offset.y;
            base_painter.text(
                Pos2::new(rx, ry),
                Align2::CENTER_CENTER,
                (i + 1).to_string(),
                font.clone(),
                self.colors.label_color,
            );
        }
    }
}
