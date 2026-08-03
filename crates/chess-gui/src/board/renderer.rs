//! 棋盘渲染器 — Lichess 风格：绿色圆点、吃子圆环、将军光晕、拖拽浮子、箭头。
//!
//! 使用 `egui::Painter` 进行全部绘制，不依赖棋盘图片。

use chess_core::{MoveFlag, Square};
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2};

use crate::board::state::BoardState;
use crate::piece::texture::PieceTextureManager;
use crate::theme::ThemeColors;

/// 圆角半径占格子的比例
const ROUNDING_RATIO: f32 = 0.06;
/// 棋子占格子的比例
const PIECE_RATIO: f32 = 0.90;

/// 棋盘填充可用空间的比例（Lichess uniboard 风格）
const BOARD_SCALE: f32 = 0.95;
/// 坐标外边距的等效格子数（coords-out 模式下总外边距）
const COORD_MARGIN: f32 = 0.5;

/// 坐标标注显示模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordMode {
    /// 坐标在棋盘格子外侧（Lichess 默认），带外边距防止裁切
    Outside,
}

pub struct BoardRenderer {
    /// 是否翻转棋盘（黑方视角）
    pub flipped: bool,
    colors: ThemeColors,
    #[allow(dead_code)]
    coord_mode: CoordMode,
}

impl BoardRenderer {
    pub fn new(colors: ThemeColors) -> Self {
        Self {
            flipped: false,
            colors,
            coord_mode: CoordMode::Outside,
        }
    }

    pub fn set_colors(&mut self, colors: ThemeColors) {
        self.colors = colors;
    }

    /// 棋盘缩放系数（占可用空间的比例）
    pub fn board_scale(&self) -> f32 {
        BOARD_SCALE
    }

    // 布局计算（静态方法）

    /// 计算 8×8 格子区域（从 board_rect 减去坐标外边距）
    pub fn inner_rect(board_rect: Rect) -> Rect {
        let outer_side = board_rect.width();
        let sq_size = outer_side / (8.0 + COORD_MARGIN);
        let margin = sq_size * COORD_MARGIN / 2.0;
        Rect::from_min_max(
            Pos2::new(board_rect.min.x + margin, board_rect.min.y + margin),
            Pos2::new(board_rect.max.x - margin, board_rect.max.y - margin),
        )
    }

    /// 单个格子的边长（像素）
    #[allow(dead_code)]
    pub fn square_size(board_rect: Rect) -> f32 {
        Self::inner_rect(board_rect).width() / 8.0
    }

    /// 屏幕坐标 → 棋盘格子（失败返回 None）
    pub fn pos_to_square(&self, board_rect: Rect, pos: Pos2) -> Option<Square> {
        let inner = Self::inner_rect(board_rect);
        if !inner.contains(pos) {
            return None;
        }
        let sq_sz = inner.width() / 8.0;
        let f = ((pos.x - inner.min.x) / sq_sz) as u8;
        let r = 7 - ((pos.y - inner.min.y) / sq_sz) as u8;
        if f >= 8 || r >= 8 {
            return None;
        }
        if self.flipped {
            Square::from_coord(7 - f, 7 - r)
        } else {
            Square::from_coord(f, r)
        }
    }

    /// 格子中心在屏幕上的坐标
    pub fn square_center(board_rect: Rect, sq: Square, flipped: bool) -> Pos2 {
        let inner = Self::inner_rect(board_rect);
        let sz = inner.width() / 8.0;
        let (f, r) = if flipped {
            (7 - sq.file(), 7 - sq.rank())
        } else {
            (sq.file(), sq.rank())
        };
        Pos2::new(
            inner.min.x + f as f32 * sz + sz / 2.0,
            inner.min.y + (7 - r) as f32 * sz + sz / 2.0,
        )
    }

    // 内部辅助

    /// 格子在屏幕上的矩形
    fn sq_rect(board_rect: Rect, sq: Square, flipped: bool) -> Rect {
        let inner = Self::inner_rect(board_rect);
        let sz = inner.width() / 8.0;
        let (f, r) = if flipped {
            (7 - sq.file(), 7 - sq.rank())
        } else {
            (sq.file(), sq.rank())
        };
        let x = inner.min.x + f as f32 * sz;
        let y = inner.min.y + (7 - r) as f32 * sz;
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(sz, sz))
    }

    /// 将绝对坐标转换为 painter 相对坐标
    fn to_local(rect: &Rect, off: Vec2) -> Rect {
        Rect::from_min_size(
            Pos2::new(rect.min.x - off.x, rect.min.y - off.y),
            rect.size(),
        )
    }

    /// 将绝对坐标点转换为 painter 相对坐标
    fn pt_local(pos: Pos2, off: Vec2) -> Pos2 {
        Pos2::new(pos.x - off.x, pos.y - off.y)
    }

    // 主渲染

    /// 在已分配的 Painter 上绘制全部棋盘元素
    ///
    /// * `p` — 已通过 `ui.allocate_painter()` 分配的 Painter
    /// * `board_rect` — 分配返回的实际屏幕矩形（`response.rect`）
    /// * `state` — 棋盘渲染状态
    /// * `textures` — 棋子纹理管理器
    pub fn paint(
        &self,
        p: &egui::Painter,
        board_rect: Rect,
        state: &BoardState,
        textures: &PieceTextureManager,
    ) {
        let inner = Self::inner_rect(board_rect);
        let sq_size = inner.width() / 8.0;
        let off = board_rect.min.to_vec2();

        // 背景
        p.rect_filled(
            Rect::from_min_size(Pos2::ZERO, board_rect.size()),
            0.0,
            self.colors.bg,
        );

        // 64 格
        for rank in 0..8u8 {
            for file in 0..8u8 {
                let (df, dr) = if self.flipped {
                    (7 - file, 7 - rank)
                } else {
                    (file, 7 - rank)
                };
                let x = inner.min.x + df as f32 * sq_size - off.x;
                let y = inner.min.y + dr as f32 * sq_size - off.y;
                let r = Rect::from_min_size(Pos2::new(x, y), Vec2::new(sq_size, sq_size));
                let bg = if (file + rank) % 2 == 0 {
                    self.colors.board_light
                } else {
                    self.colors.board_dark
                };
                p.rect_filled(r, 0.0, bg);
            }
        }

        // 棋盘圆角边框
        let rounding = egui::CornerRadius::same((sq_size * ROUNDING_RATIO) as u8);
        let inner_local = Rect::from_min_size(
            Pos2::new(inner.min.x - off.x, inner.min.y - off.y),
            inner.size(),
        );
        p.rect_stroke(
            inner_local,
            rounding,
            Stroke::new(sq_size * 0.04, self.colors.bg),
            egui::StrokeKind::Middle,
        );

        // 最后一步高亮
        if let Some(mv) = state.last_move {
            for sq in [mv.from(), mv.to()] {
                let color = if sq == mv.to() {
                    self.colors.last_move_to
                } else {
                    self.colors.last_move_from
                };
                let r = Self::to_local(&Self::sq_rect(board_rect, sq, self.flipped), off);
                p.rect_filled(r, 0.0, color);
            }
        }

        // 将军光晕
        // 将军光晕（Lichess 风格：多层同心圆模拟径向渐变，中心亮红→边缘透明）
        if let Some(king_sq) = state.king_in_check {
            let c = Self::square_center(board_rect, king_sq, self.flipped);
            let cx = c.x - off.x;
            let cy = c.y - off.y;

            // 5 层从外到内绘制，内层覆盖外层中心
            let mid_fade = Color32::from_rgba_premultiplied(200, 0, 0, 120);
            let outer_fade = Color32::from_rgba_premultiplied(180, 0, 0, 40);

            p.circle_filled(
                Pos2::new(cx, cy),
                sq_size * 0.95,
                self.colors.check_glow_outer,
            );
            p.circle_filled(Pos2::new(cx, cy), sq_size * 0.85, outer_fade);
            p.circle_filled(Pos2::new(cx, cy), sq_size * 0.70, mid_fade);
            p.circle_filled(
                Pos2::new(cx, cy),
                sq_size * 0.50,
                self.colors.check_glow_mid,
            );
            p.circle_filled(
                Pos2::new(cx, cy),
                sq_size * 0.30,
                self.colors.check_glow_inner,
            );
        }

        // 选中高亮
        if let Some(sq) = state.selected_square {
            let r = Self::to_local(&Self::sq_rect(board_rect, sq, self.flipped), off);
            p.rect_filled(r, 0.0, self.colors.selected_highlight);
        }

        // 拖拽来源高亮
        if let Some((_piece, from, _pos)) = &state.drag {
            let r = Self::to_local(&Self::sq_rect(board_rect, *from, self.flipped), off);
            p.rect_filled(r, 0.0, self.colors.drag_source);
        }

        // 合法走法提示：圆点（普通）/ 圆环（吃子）
        for mv in &state.legal_moves {
            let tgt = mv.to();
            let c = Self::square_center(board_rect, tgt, self.flipped);
            let cx = c.x - off.x;
            let cy = c.y - off.y;

            let is_capture =
                state.position.piece_at(tgt).is_some() || mv.flag() == MoveFlag::EnPassant;

            if is_capture {
                // 空心圆环
                p.circle_stroke(
                    Pos2::new(cx, cy),
                    sq_size * 0.42,
                    Stroke::new(sq_size * 0.06, self.colors.capture_ring),
                );
            } else {
                // 实心圆点
                p.circle_filled(
                    Pos2::new(cx, cy),
                    sq_size * 0.15,
                    self.colors.legal_move_dot,
                );
            }
        }

        // 箭头
        for arrow in &state.arrows {
            self.draw_arrow(&p, board_rect, arrow, off, sq_size);
        }

        // 棋子
        for rank in 0..8u8 {
            for file in 0..8u8 {
                let sq = Square::from_coord(file, rank).unwrap();

                // 跳过正在拖拽的棋子（单独绘制浮子）
                if let Some((_piece, from, _pos)) = &state.drag
                    && sq == *from
                {
                    continue;
                }

                if let Some(piece) = state.position.piece_at(sq) {
                    let c = Self::square_center(board_rect, sq, self.flipped);
                    textures.render(
                        &p,
                        piece.color,
                        piece.kind,
                        Self::pt_local(c, off),
                        sq_size * PIECE_RATIO,
                    );
                }
            }
        }

        // 拖拽浮子
        if let Some((piece, from, mouse_pos)) = &state.drag {
            // 来源格画半透明残影（Lichess: opacity 0.3）
            let sc = Self::square_center(board_rect, *from, self.flipped);
            let ghost_half = sq_size * PIECE_RATIO / 2.0;
            let ghost_rect = Rect::from_min_max(
                Pos2::new(sc.x - off.x - ghost_half, sc.y - off.y - ghost_half),
                Pos2::new(sc.x - off.x + ghost_half, sc.y - off.y + ghost_half),
            );
            let tex = textures.get(piece.color, piece.kind);
            let ghost_tint = Color32::from_rgba_premultiplied(255, 255, 255, 77);
            p.image(
                tex.id(),
                ghost_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                ghost_tint,
            );

            // 鼠标位置画跟随棋子（完全不透明）
            let half = sq_size * PIECE_RATIO / 2.0;
            let fr = Rect::from_min_max(
                Pos2::new(mouse_pos.x - half, mouse_pos.y - half),
                Pos2::new(mouse_pos.x + half, mouse_pos.y + half),
            );
            p.image(
                tex.id(),
                fr,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        // 坐标标注（Lichess coords-out 风格：格子外侧边距）
        let files: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let margin = (board_rect.width() - inner.width()) / 2.0;
        let font = FontId::monospace(sq_size * 0.16);

        for i in 0..8u8 {
            let ri = if self.flipped { i } else { 7 - i };
            let fi = if self.flipped { 7 - i } else { i };

            // 行号（1-8）：左侧外边距垂直居中
            let rx = board_rect.min.x + margin / 2.0 - off.x;
            let ry = inner.min.y + ri as f32 * sq_size + sq_size / 2.0 - off.y;
            p.text(
                Pos2::new(rx, ry),
                Align2::CENTER_CENTER,
                (i + 1).to_string(),
                font.clone(),
                self.colors.coord_light,
            );

            // 列号（a-h）：底部外边距水平居中
            let fx = inner.min.x + fi as f32 * sq_size + sq_size / 2.0 - off.x;
            let fy = inner.max.y + margin / 2.0 - off.y;
            p.text(
                Pos2::new(fx, fy),
                Align2::CENTER_CENTER,
                files[i as usize].to_string(),
                font.clone(),
                self.colors.coord_light,
            );
        }
    }

    // 箭头绘制

    fn draw_arrow(
        &self,
        p: &egui::Painter,
        board_rect: Rect,
        arrow: &crate::board::state::BoardArrow,
        off: Vec2,
        sq_size: f32,
    ) {
        let from_center = Self::square_center(board_rect, arrow.from, self.flipped);
        let to_center = Self::square_center(board_rect, arrow.to, self.flipped);

        let start = Self::pt_local(from_center, off);
        let end = Self::pt_local(to_center, off);

        // 箭头线宽
        let width = sq_size * 0.12;

        // 绘制线段（到目标格子边缘，留出箭头头部空间）
        let dir = end - start;
        let len = dir.length();
        if len < 1.0 {
            return;
        }
        let unit = dir / len;

        // 箭头头部三角形大小
        let head_len = sq_size * 0.35;
        let head_width = sq_size * 0.18;

        // 线段终点（在箭头头部之前）
        let line_end = end - unit * head_len * 0.6;

        // 绘制线段
        p.line_segment([start, line_end], Stroke::new(width, arrow.color));

        // 绘制箭头头部三角形
        let perp = Vec2::new(-unit.y, unit.x);
        let tip = end;
        let left = end - unit * head_len + perp * head_width;
        let right = end - unit * head_len - perp * head_width;

        p.add(egui::Shape::convex_polygon(
            vec![tip, left, right],
            arrow.color,
            Stroke::new(1.0_f32, arrow.color),
        ));
    }
}
