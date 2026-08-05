//! 棋盘组件：组合布局计算、空间分配、渲染调度的顶层入口
//!
//! `BoardWidget` 是连接 egui 布局系统与棋盘渲染的桥梁
//! 它负责在 `Ui` 中分配空间、构建 `BoardLayout`、调用 `BoardRenderer` 进行绘制

use egui::{Sense, Vec2};

use crate::board::layout::BoardLayout;
use crate::board::renderer::BoardRenderer;
use crate::board::state::BoardState;
use crate::piece::texture::PieceTextureManager;

/// 棋盘组件
///
/// 使用 builder 模式：构造后调用 `show(ui)` 完成渲染
pub struct BoardWidget<'a> {
    pub renderer: &'a BoardRenderer,
    pub textures: &'a PieceTextureManager,
    pub state: &'a BoardState,
    pub flipped: bool,
    pub sense: Sense,
}

/// `BoardWidget::show()` 的返回值
///
/// 包含 egui 响应（用于交互检测）和布局（用于坐标转换）
pub struct BoardResponse {
    pub response: egui::Response,
    pub layout: BoardLayout,
}

impl BoardWidget<'_> {
    /// 在 egui 中渲染棋盘组件
    ///
    /// 1. 根据可用空间计算最佳棋盘尺寸
    /// 2. 分配一个正方形区域
    /// 3. 构建 `BoardLayout`
    /// 4. 调用 `BoardRenderer::paint()` 绘制全部元素
    /// 5. 返回 `BoardResponse` 供交互处理
    pub fn show(self, ui: &mut egui::Ui) -> BoardResponse {
        let available = ui.available_size();
        let side = BoardLayout::optimal_side(available);

        // 垂直居中（水平方向由 egui 垂直布局自然放置到左侧）
        let y_pad = ((available.y - side) / 2.0).max(0.0);
        ui.add_space(y_pad);

        let (response, painter) = ui.allocate_painter(Vec2::new(side, side), self.sense);
        let layout = BoardLayout::from_allocated_rect(response.rect);

        self.renderer
            .paint(&painter, &layout, self.state, self.textures, self.flipped);

        BoardResponse { response, layout }
    }
}
