//! 棋盘渲染器 — 组合所有子渲染器，提供统一渲染入口
//!
//! 不再直接处理布局计算、坐标转换或具体绘制逻辑，
//! 仅负责按正确顺序调用各子渲染器

use crate::board::arrows::ArrowRenderer;
use crate::board::background::BoardBackgroundRenderer;
use crate::board::coordinates::CoordinateRenderer;
use crate::board::highlight::HighlightRenderer;
use crate::board::layout::BoardLayout;
use crate::board::pieces::PieceRenderer;
use crate::board::state::BoardState;
use crate::piece::texture::PieceTextureManager;
use crate::theme::ThemeColors;

pub struct BoardRenderer {
    colors: ThemeColors,
}

impl BoardRenderer {
    pub fn new(colors: ThemeColors) -> Self {
        Self { colors }
    }

    pub fn set_colors(&mut self, colors: ThemeColors) {
        self.colors = colors;
    }

    /// 绘制全部棋盘元素。
    ///
    /// * `painter` — 已通过 `ui.allocate_painter()` 分配的 Painter（原点在 outer_rect.min）
    /// * `layout` — 由 `BoardLayout::from_allocated_rect()` 构建的布局
    /// * `state` — 棋盘渲染状态
    /// * `textures` — 棋子纹理管理器
    /// * `flipped` — 是否翻转棋盘（黑方视角）
    pub fn paint(
        &self,
        painter: &egui::Painter,
        layout: &BoardLayout,
        state: &BoardState,
        textures: &PieceTextureManager,
        flipped: bool,
    ) {
        BoardBackgroundRenderer::paint(painter, layout, &self.colors);
        HighlightRenderer::paint(painter, layout, state, &self.colors, flipped);
        PieceRenderer::paint(painter, layout, state, textures, flipped);
        if !state.arrows.is_empty() {
            ArrowRenderer::paint(
                painter,
                layout,
                &state.arrows,
                state.arrow_preview.as_ref(),
                flipped,
            );
        }
        CoordinateRenderer::paint(painter, layout, &self.colors, flipped);
    }
}
