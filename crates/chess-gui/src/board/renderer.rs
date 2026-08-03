//! 棋盘渲染器 — Lichess 风格：绿色圆点、吃子圆环、将军光晕、拖拽浮子、箭头。
//!
//! 使用 `egui::Painter` 进行全部绘制，不依赖棋盘图片。

use chess_core::{MoveFlag, Square};
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use crate::board::state::BoardState;
use crate::piece::texture::PieceTextureManager;
use crate::theme::ThemeColors;

/// 棋盘边距占一格的比例
const MARGIN_RATIO: f32 = 0.35;
/// 最小棋盘尺寸（像素）
const MIN_BOARD_SIZE: f32 = 480.0;

pub struct BoardRenderer {
    /// 是否翻转棋盘（黑方视角）
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

    // ── 布局计算（静态方法）─────────────────────────────────

    /// 计算棋盘整体矩形：可用空间内最大的居中正方形
    pub fn board_rect(ui: &egui::Ui) -> Rect {
        let available = ui.available_size();
        let side = available.x.min(available.y).max(MIN_BOARD_SIZE);
        let x0 = ui.cursor().min.x + (available.x - side) / 2.0;
        let y0 = ui.cursor().min.y + (available.y - side) / 2.0;
        Rect::from_min_size(Pos2::new(x0, y0), Vec2::new(side, side))
    }

    /// 8×8 格子的内部区域（不含坐标边距）
    pub fn inner_rect(board_rect: Rect) -> Rect {
        let sq = board_rect.width() / (8.0 + 2.0 * MARGIN_RATIO);
        let m = sq * MARGIN_RATIO;
        Rect::from_min_size(
            Pos2::new(board_rect.min.x + m, board_rect.min.y + m),
            Vec2::new(sq * 8.0, sq * 8.0),
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

    // ── 内部辅助 ────────────────────────────────────────────

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

    // ── 主渲染 ──────────────────────────────────────────────

    /// 渲染棋盘的全部视觉元素
    ///
    /// * `ui` — egui 上下文
    /// * `board_rect` — 棋盘在屏幕上的矩形
    /// * `state` — 棋盘渲染状态
    /// * `textures` — 棋子纹理管理器
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        board_rect: Rect,
        state: &BoardState,
        textures: &PieceTextureManager,
    ) {
        let inner = Self::inner_rect(board_rect);
        let sq_size = inner.width() / 8.0;

        // 仅渲染用 Painter，交互由外部 ui.interact(...) 处理
        let (_response, p) = ui.allocate_painter(board_rect.size(), Sense::hover());
        let off = board_rect.min.to_vec2();

        // ── 背景 ──
        p.rect_filled(
            Rect::from_min_size(Pos2::ZERO, board_rect.size()),
            0.0,
            self.colors.bg,
        );

        // ── 64 格 ──
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

        // ── 最后一步高亮 ──
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

        // ── 将军光晕 ──
        if let Some(king_sq) = state.king_in_check {
            let c = Self::square_center(board_rect, king_sq, self.flipped);
            let cx = c.x - off.x;
            let cy = c.y - off.y;

            // 三层同心圆产生渐变光晕效果
            p.circle_filled(
                Pos2::new(cx, cy),
                sq_size * 0.95,
                self.colors.check_glow_outer,
            );
            p.circle_filled(
                Pos2::new(cx, cy),
                sq_size * 0.75,
                self.colors.check_glow_mid,
            );
            p.circle_filled(
                Pos2::new(cx, cy),
                sq_size * 0.55,
                self.colors.check_glow_inner,
            );

            // 王所在格子也高亮
            let r = Self::to_local(
                &Self::sq_rect(board_rect, king_sq, self.flipped),
                off,
            );
            p.rect_filled(r, 0.0, self.colors.selected_highlight);
        }

        // ── 选中高亮 ──
        if let Some(sq) = state.selected_square {
            let r = Self::to_local(&Self::sq_rect(board_rect, sq, self.flipped), off);
            p.rect_filled(r, 0.0, self.colors.selected_highlight);
        }

        // ── 拖拽来源高亮 ──
        if let Some((_piece, from, _pos)) = &state.drag {
            let r = Self::to_local(&Self::sq_rect(board_rect, *from, self.flipped), off);
            p.rect_filled(r, 0.0, self.colors.drag_source);
        }

        // ── 合法走法提示：圆点（普通）/ 圆环（吃子） ──
        for mv in &state.legal_moves {
            let tgt = mv.to();
            let c = Self::square_center(board_rect, tgt, self.flipped);
            let cx = c.x - off.x;
            let cy = c.y - off.y;

            let is_capture = state.position.piece_at(tgt).is_some()
                || mv.flag() == MoveFlag::EnPassant;

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

        // ── 箭头 ──
        for arrow in &state.arrows {
            self.draw_arrow(&p, board_rect, arrow, off, sq_size);
        }

        // ── 棋子 ──
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
                        sq_size * 0.85,
                    );
                }
            }
        }

        // ── 拖拽浮子 ──
        if let Some((piece, from, mouse_pos)) = &state.drag {
            // 来源格画半透明残影
            let sc = Self::square_center(board_rect, *from, self.flipped);
            textures.render(
                &p,
                piece.color,
                piece.kind,
                Self::pt_local(sc, off),
                sq_size * 0.85,
            );

            // 鼠标位置画跟随棋子（半透明）
            let tint = Color32::from_rgba_premultiplied(255, 255, 255, 200);
            let half = sq_size * 0.85 / 2.0;
            let fr = Rect::from_min_max(
                Pos2::new(mouse_pos.x - half, mouse_pos.y - half),
                Pos2::new(mouse_pos.x + half, mouse_pos.y + half),
            );
            let tex = textures.get(piece.color, piece.kind);
            p.image(
                tex.id(),
                fr,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                tint,
            );
        }

        // ── 坐标标注 ──
        let font = FontId::monospace(sq_size * 0.26);
        let files: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

        // 下方文件标注 + 左侧行号标注
        let margin = inner.min.x; // 边距大小
        for i in 0..8u8 {
            let di = if self.flipped { 7 - i } else { i };

            // 文件标注（a-h）：棋盘下方
            let fx = inner.min.x + di as f32 * sq_size + sq_size / 2.0 - off.x;
            let fy = inner.max.y + margin * 0.7 - off.y;
            p.text(
                Pos2::new(fx, fy),
                Align2::CENTER_CENTER,
                files[i as usize].to_string(),
                font.clone(),
                self.colors.label_color,
            );

            // 行号标注（1-8）：棋盘左侧
            let rx = margin * 0.5 - off.x;
            let ry = inner.min.y + (7 - di) as f32 * sq_size + sq_size / 2.0 - off.y;
            p.text(
                Pos2::new(rx, ry),
                Align2::CENTER_CENTER,
                (i + 1).to_string(),
                font.clone(),
                self.colors.label_color,
            );
        }
    }

    // ── 箭头绘制 ────────────────────────────────────────────

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
