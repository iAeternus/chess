//! 棋盘渲染器 — lichess 风格：绿色圆点、吃子圆环、将王光晕、拖拽虚影。

use chess_core::{Move, Piece, Position, Square};
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use crate::piece::texture::PieceTextureManager;
use crate::theme::ThemeColors;

const MARGIN_RATIO: f32 = 0.35;
const MIN_BOARD_SIZE: f32 = 480.0;

/// 拖拽状态
pub struct DragState {
    pub piece: Piece,
    pub from: Square,
}

pub struct BoardRenderer {
    pub flipped: bool,
    colors: ThemeColors,
}

impl BoardRenderer {
    pub fn new(colors: ThemeColors) -> Self {
        Self { flipped: false, colors }
    }

    pub fn set_colors(&mut self, colors: ThemeColors) { self.colors = colors; }

    pub fn board_rect(ui: &egui::Ui) -> Rect {
        let available = ui.available_size();
        let side = available.x.min(available.y).max(MIN_BOARD_SIZE);
        let x0 = ui.cursor().min.x + (available.x - side) / 2.0;
        let y0 = ui.cursor().min.y + (available.y - side) / 2.0;
        Rect::from_min_size(Pos2::new(x0, y0), Vec2::new(side, side))
    }

    pub fn inner_rect(board_rect: Rect) -> Rect {
        let sq = board_rect.width() / (8.0 + 2.0 * MARGIN_RATIO);
        let m = sq * MARGIN_RATIO;
        Rect::from_min_size(
            Pos2::new(board_rect.min.x + m, board_rect.min.y + m),
            Vec2::new(sq * 8.0, sq * 8.0),
        )
    }

    pub fn square_size(board_rect: Rect) -> f32 {
        Self::inner_rect(board_rect).width() / 8.0
    }

    pub fn pos_to_square(&self, board_rect: Rect, pos: Pos2) -> Option<Square> {
        let inner = Self::inner_rect(board_rect);
        if !inner.contains(pos) { return None; }
        let sq_sz = inner.width() / 8.0;
        let f = ((pos.x - inner.min.x) / sq_sz) as u8;
        let r = 7 - ((pos.y - inner.min.y) / sq_sz) as u8;
        if f >= 8 || r >= 8 { return None; }
        if self.flipped { Square::from_coord(7 - f, 7 - r) }
        else { Square::from_coord(f, r) }
    }

    pub fn square_center(board_rect: Rect, sq: Square, flipped: bool) -> Pos2 {
        let inner = Self::inner_rect(board_rect);
        let sz = inner.width() / 8.0;
        let (f, r) = if flipped { (7 - sq.file(), 7 - sq.rank()) } else { (sq.file(), sq.rank()) };
        Pos2::new(inner.min.x + f as f32 * sz + sz / 2.0, inner.min.y + (7 - r) as f32 * sz + sz / 2.0)
    }

    fn sq_rect(board_rect: Rect, sq: Square, flipped: bool) -> Rect {
        let inner = Self::inner_rect(board_rect);
        let sz = inner.width() / 8.0;
        let (f, r) = if flipped { (7 - sq.file(), 7 - sq.rank()) } else { (sq.file(), sq.rank()) };
        let x = inner.min.x + f as f32 * sz;
        let y = inner.min.y + (7 - r) as f32 * sz;
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(sz, sz))
    }

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
        // drag: (DragState, 鼠标在 board_rect 内的位置)
        drag: Option<(DragState, Pos2)>,
    ) {
        let inner = Self::inner_rect(board_rect);
        let sq_size = inner.width() / 8.0;

        let (_response, p) = ui.allocate_painter(board_rect.size(), Sense::click_and_drag());
        let off = board_rect.min.to_vec2();

        // ── BG ──
        p.rect_filled(Rect::from_min_size(Pos2::ZERO, board_rect.size()), 0.0, self.colors.bg);

        // ── Squares ──
        for rank in 0..8u8 { for file in 0..8u8 {
            let (df, dr) = if self.flipped { (7 - file, 7 - rank) } else { (file, 7 - rank) };
            let x = inner.min.x + df as f32 * sq_size - off.x;
            let y = inner.min.y + dr as f32 * sq_size - off.y;
            let r = Rect::from_min_size(Pos2::new(x, y), Vec2::new(sq_size, sq_size));
            let bg = if (file + rank) % 2 == 0 { self.colors.board_light } else { self.colors.board_dark };
            p.rect_filled(r, 0.0, bg);
        }}

        // ── Last move ──
        if let Some(mv) = last_move {
            for sq in [mv.from(), mv.to()] {
                let c = if sq == mv.to() { self.colors.last_move_to } else { self.colors.last_move_from };
                let r = Self::sq_rect(board_rect, sq, self.flipped);
                p.rect_filled(Rect::from_min_size(Pos2::new(r.min.x - off.x, r.min.y - off.y), r.size()), 0.0, c);
            }
        }

        // ── Check glow (radial) ──
        if in_check {
            let king_sq = position.board().king_square(position.side_to_move());
            let c = Self::square_center(board_rect, king_sq, self.flipped);
            let cx = c.x - off.x; let cy = c.y - off.y;
            // Outer
            p.circle_filled(Pos2::new(cx, cy), sq_size * 0.95, self.colors.check_glow_outer);
            // Mid
            p.circle_filled(Pos2::new(cx, cy), sq_size * 0.75, self.colors.check_glow_mid);
            // Inner
            p.circle_filled(Pos2::new(cx, cy), sq_size * 0.55, self.colors.check_glow_inner);
            // King square with check highlight
            let r = Self::sq_rect(board_rect, king_sq, self.flipped);
            p.rect_filled(Rect::from_min_size(Pos2::new(r.min.x - off.x, r.min.y - off.y), r.size()), 0.0, self.colors.selected_highlight);
        }

        // ── Drag source highlight ──
        if let Some((ref ds, _)) = drag {
            let r = Self::sq_rect(board_rect, ds.from, self.flipped);
            p.rect_filled(Rect::from_min_size(Pos2::new(r.min.x - off.x, r.min.y - off.y), r.size()), 0.0, self.colors.drag_source);
        }

        // ── Selected highlight ──
        if let Some(sq) = selected {
            let r = Self::sq_rect(board_rect, sq, self.flipped);
            p.rect_filled(Rect::from_min_size(Pos2::new(r.min.x - off.x, r.min.y - off.y), r.size()), 0.0, self.colors.selected_highlight);
        }

        // ── Legal move dots / capture rings ──
        for mv in legal_moves {
            let tgt = mv.to();
            let c = Self::square_center(board_rect, tgt, self.flipped);
            let cx = c.x - off.x; let cy = c.y - off.y;
            let is_cap = position.piece_at(tgt).is_some() || mv.flag() == chess_core::MoveFlag::EnPassant;
            if is_cap {
                p.circle_stroke(Pos2::new(cx, cy), sq_size * 0.42,
                    Stroke::new(sq_size * 0.06, self.colors.capture_ring));
            } else {
                p.circle_filled(Pos2::new(cx, cy), sq_size * 0.15, self.colors.legal_move_dot);
            }
        }

        // ── Pieces ──
        for rank in 0..8u8 { for file in 0..8u8 {
            let sq = Square::from_coord(file, rank).unwrap();
            // Skip piece being dragged (drawn separately)
            if let Some((ref ds, _)) = drag {
                if sq == ds.from { continue; }
            }
            if let Some(piece) = position.piece_at(sq) {
                let c = Self::square_center(board_rect, sq, self.flipped);
                textures.render(&p, piece.color, piece.kind,
                    Pos2::new(c.x - off.x, c.y - off.y), sq_size * 0.85);
            }
        }}

        // ── Drag ghost + floating piece ──
        if let Some((ref ds, mouse_pos)) = drag {
            // Ghost at source (painter-local)
            let sc = Self::square_center(board_rect, ds.from, self.flipped);
            textures.render(&p, ds.piece.color, ds.piece.kind,
                Pos2::new(sc.x - off.x, sc.y - off.y), sq_size * 0.85);

            // Floating piece at mouse (mouse_pos is already widget-local = painter-local)
            let tint = Color32::from_rgba_premultiplied(255, 255, 255, 200);
            let half = sq_size * 0.85 / 2.0;
            let fr = Rect::from_min_max(
                Pos2::new(mouse_pos.x - half, mouse_pos.y - half),
                Pos2::new(mouse_pos.x + half, mouse_pos.y + half),
            );
            let tex = textures.get(ds.piece.color, ds.piece.kind);
            p.image(tex.id(), fr, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), tint);
        }

        // ── Labels ──
        let font = FontId::monospace(sq_size * 0.26);
        let files: [char; 8] = ['a','b','c','d','e','f','g','h'];
        let margin = inner.min.x;
        for i in 0..8u8 {
            let di = if self.flipped { 7 - i } else { i };
            let fx = inner.min.x + di as f32 * sq_size + sq_size / 2.0 - off.x;
            let fy = inner.max.y + margin * 0.7 - off.y;
            p.text(Pos2::new(fx, fy), Align2::CENTER_CENTER, files[i as usize].to_string(), font.clone(), self.colors.label_color);
            let rx = margin * 0.5 - off.x;
            let ry = inner.min.y + (7 - di) as f32 * sq_size + sq_size / 2.0 - off.y;
            p.text(Pos2::new(rx, ry), Align2::CENTER_CENTER, (i + 1).to_string(), font.clone(), self.colors.label_color);
        }
    }
}
