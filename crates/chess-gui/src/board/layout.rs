//! 棋盘布局系统：所有尺寸计算与坐标转换的单一真相来源
//!
//! `BoardLayout` 负责计算整个棋盘组件的空间划分，包括：
//! - 外层区域（outer_rect）：棋盘 + 坐标标注的总区域
//! - 棋盘区域（board_rect）：8×8 格子网格
//! - 单格尺寸（square_size）
//! - 坐标边距（coord_margin）
//!
//! 所有方法返回**painter-local**坐标（相对于 outer_rect.min）

use chess_core::Square;
use egui::{Pos2, Rect, Vec2};

/// 坐标外边距的等效格子数（总边距，两侧各一半）
const COORD_MARGIN: f32 = 0.5;
/// 棋盘占可用空间的比例
const BOARD_SCALE: f32 = 0.95;
/// 最小棋盘外边长（像素）
const MIN_BOARD_SIDE: f32 = 400.0;

#[derive(Debug, Clone)]
pub struct BoardLayout {
    /// 整个棋盘组件的屏幕矩形（包含坐标边距）
    pub outer_rect: Rect,
    /// 8×8 棋盘格子的屏幕矩形
    #[allow(dead_code)]
    pub board_rect: Rect,
    /// 单格边长（像素）
    pub square_size: f32,
    /// 坐标边距（像素，每侧宽度）
    pub coord_margin: f32,
}

impl BoardLayout {
    /// 根据可用空间计算最佳棋盘外边长
    ///
    /// 棋盘始终为正方形，取 available 宽高中的较小值，
    /// 并按 `BOARD_SCALE` 缩放以提供呼吸空间
    pub fn optimal_side(available: Vec2) -> f32 {
        let max_side = available.x.min(available.y);
        if max_side < MIN_BOARD_SIDE {
            max_side
        } else {
            max_side * BOARD_SCALE
        }
    }

    /// 从已分配的屏幕矩形构建布局
    ///
    /// `outer_rect` 应来自 `ui.allocate_painter()` 返回的 `response.rect`
    pub fn from_allocated_rect(outer_rect: Rect) -> Self {
        let outer_side = outer_rect.width();
        let square_size = outer_side / (8.0 + COORD_MARGIN);
        let coord_margin = square_size * COORD_MARGIN / 2.0;
        let board_rect = Rect::from_min_max(
            Pos2::new(
                outer_rect.min.x + coord_margin,
                outer_rect.min.y + coord_margin,
            ),
            Pos2::new(
                outer_rect.max.x - coord_margin,
                outer_rect.max.y - coord_margin,
            ),
        );

        Self {
            outer_rect,
            board_rect,
            square_size,
            coord_margin,
        }
    }

    /// 棋盘区域在 painter-local 坐标系下的矩形
    ///
    /// painter-local 原点为 `outer_rect.min`
    #[inline]
    pub fn board_rect_local(&self) -> Rect {
        Rect::from_min_size(
            Pos2::new(self.coord_margin, self.coord_margin),
            Vec2::new(8.0 * self.square_size, 8.0 * self.square_size),
        )
    }

    /// 单个格子在 painter-local 坐标系下的矩形
    pub fn square_rect(&self, sq: Square, flipped: bool) -> Rect {
        let (f, r) = if flipped {
            (7 - sq.file(), 7 - sq.rank())
        } else {
            (sq.file(), sq.rank())
        };
        let board = self.board_rect_local();
        let x = board.min.x + f as f32 * self.square_size;
        // screen y 向下增长，rank 0 在底部，rank 7 在顶部
        let y = board.min.y + (7 - r) as f32 * self.square_size;
        Rect::from_min_size(
            Pos2::new(x, y),
            Vec2::new(self.square_size, self.square_size),
        )
    }

    /// 格子中心在 painter-local 坐标系下的坐标
    pub fn square_center(&self, sq: Square, flipped: bool) -> Pos2 {
        self.square_rect(sq, flipped).center()
    }

    /// painter-local 坐标 -> 棋盘格子
    ///
    /// 如果坐标不在棋盘区域内则返回 `None`
    pub fn pos_to_square(&self, pos: Pos2, flipped: bool) -> Option<Square> {
        let board = self.board_rect_local();
        if !board.contains(pos) {
            return None;
        }
        let f = ((pos.x - board.min.x) / self.square_size) as u8;
        let r = 7 - ((pos.y - board.min.y) / self.square_size) as u8;
        if f >= 8 || r >= 8 {
            return None;
        }
        if flipped {
            Square::from_coord(7 - f, 7 - r)
        } else {
            Square::from_coord(f, r)
        }
    }
}
