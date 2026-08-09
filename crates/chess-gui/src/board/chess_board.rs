//! ChessBoard — 棋盘统一组件
//!
//! 整合布局计算、空间分配、渲染调度和交互处理。
//! 作为独立组件，可嵌入任意 egui 布局。

use chess_core::{Move, Piece, Square};
use egui::{PointerButton, Pos2, Sense, Vec2};

use crate::board::layout::BoardLayout;
use crate::board::renderer::BoardRenderer;
use crate::board::state::{BoardArrow, BoardState};
use crate::game::controller::{GameController, GameMode, SelectionResult};
use crate::piece::texture::PieceTextureManager;
use crate::theme::ThemeColors;

/// 棋盘交互事件
#[derive(Debug)]
#[allow(dead_code)]
pub enum BoardEvent {
    /// 走法已执行
    MoveMade(Move),
    /// 需要选择升变棋子
    PromotionNeeded { from: Square, to: Square },
    /// 创建/删除箭头
    ArrowToggle { from: Square, to: Square },
    /// 实时预览
    ArrowPreview { arrow: Option<BoardArrow> },
    /// 清空
    ClearArrows,
}

/// ChessBoard::show() 的返回值
#[allow(dead_code)]
pub struct ChessBoardResponse {
    /// egui 交互响应（用于外部 hover 等检测）
    pub response: egui::Response,
    /// 棋盘布局（用于外部坐标转换）
    pub layout: BoardLayout,
    /// 交互事件列表
    pub events: Vec<BoardEvent>,
}

/// 棋盘统一组件
///
/// 拥有渲染器、翻转状态。每帧调用 `show()` 完成渲染 + 交互。
pub struct ChessBoard {
    renderer: BoardRenderer,
    flipped: bool,

    /// 当前右键拖动中的箭头
    arrow_drag: Option<BoardArrow>,
}

impl ChessBoard {
    /// 创建棋盘组件
    pub fn new(renderer: BoardRenderer) -> Self {
        Self {
            renderer,
            flipped: false,
            arrow_drag: None,
        }
    }

    /// 设置是否翻转（黑方视角）
    pub fn set_flipped(&mut self, flipped: bool) {
        self.flipped = flipped;
    }

    /// 更新渲染器颜色（主题切换时调用）
    pub fn set_colors(&mut self, colors: ThemeColors) {
        self.renderer.set_colors(colors);
    }

    /// 根据可用空间计算最佳棋盘边长（外框，含坐标边距）
    pub fn optimal_side(available: Vec2) -> f32 {
        BoardLayout::optimal_side(available)
    }

    /// 渲染棋盘并处理交互
    ///
    /// # 参数
    /// - `ui`: 父布局提供的 egui::Ui（棋盘在此 Ui 中分配空间）
    /// - `state`: 当前棋盘状态（局面、选中、高亮、拖拽等）
    /// - `textures`: 棋子纹理
    /// - `controller`: 对局控制器（用于选中、走法执行）
    /// - `drag`: 拖拽状态（ChessApp 持有，此方法读取并修改）
    /// - `ctx`: egui 上下文（拖拽时触发重绘）
    /// - `mode`: 对局模式（影响交互规则）
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &BoardState,
        textures: &PieceTextureManager,
        controller: &mut GameController,
        drag: &mut Option<(Piece, Square, Pos2)>,
        ctx: &egui::Context,
        mode: GameMode,
    ) -> ChessBoardResponse {
        // 1. 尺寸计算
        // 基于此 Ui 的可用空间（即父布局分配的空间），计算最佳棋盘尺寸
        let available = ui.available_size();
        let side = Self::optimal_side(available);

        // 2. 空间分配
        // 使用 egui 标准布局机制：allocate_painter 保留一个 side×side 的方形区域
        let sense = if mode == GameMode::Replay {
            Sense::hover()
        } else {
            Sense::click().union(Sense::drag())
        };
        let (response, painter) = ui.allocate_painter(Vec2::new(side, side), sense);

        // 3. 构建布局
        let layout = BoardLayout::from_allocated_rect(response.rect);

        // 4. 渲染
        self.renderer
            .paint(&painter, &layout, state, textures, self.flipped);

        // 5. 交互处理
        let events = Self::handle_interaction(
            &response,
            &layout,
            controller,
            drag,
            ctx,
            mode,
            self.flipped,
            &mut self.arrow_drag,
        );

        ChessBoardResponse {
            response,
            layout,
            events,
        }
    }

    /// 交互逻辑
    fn handle_interaction(
        response: &egui::Response,
        layout: &BoardLayout,
        controller: &mut GameController,
        drag: &mut Option<(Piece, Square, Pos2)>,
        ctx: &egui::Context,
        mode: GameMode,
        flipped: bool,
        arrow_drag: &mut Option<BoardArrow>,
    ) -> Vec<BoardEvent> {
        let mut events = Vec::new();

        if mode == GameMode::Analysis {
            // 右键开始
            if response.drag_started_by(PointerButton::Secondary)
                && let Some(pos) = response.interact_pointer_pos()
                && let Some(from) = layout.pos_to_square(pos, flipped)
            {
                *arrow_drag = Some(BoardArrow {
                    from,
                    to: from,
                    color: egui::Color32::from_rgba_unmultiplied(0, 200, 0, 100),
                });

                ctx.request_repaint();
            }

            // 右键移动
            if response.dragged_by(PointerButton::Secondary)
                && let Some(pos) = response.interact_pointer_pos()
                && let Some(arrow) = arrow_drag
                && let Some(to) = layout.pos_to_square(pos, flipped)
            {
                arrow.to = to;

                events.push(BoardEvent::ArrowPreview {
                    arrow: Some(arrow.clone()),
                });

                ctx.request_repaint();
            }

            // 右键释放
            if response.drag_stopped_by(PointerButton::Secondary) {
                if let Some(arrow) = arrow_drag.take() {
                    if arrow.from != arrow.to {
                        events.push(BoardEvent::ArrowToggle {
                            from: arrow.from,
                            to: arrow.to,
                        });
                    }
                }
            }
        }

        // 拖拽开始
        if response.drag_started_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
            && let Some(sq) = layout.pos_to_square(pos, flipped)
        {
            let piece = controller.current_position().piece_at(sq);
            if let Some(p) = piece {
                let side = controller.current_position().side_to_move();
                let can_move = mode == GameMode::Analysis || p.color == side;

                if can_move {
                    let result = controller.select_square(sq);
                    if matches!(result, SelectionResult::Selected { .. }) {
                        *drag = Some((p, sq, pos));
                    }
                }
            }
        }

        // 拖拽移动
        if response.dragged_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
            && let Some((_piece, _from, drag_pos)) = drag
        {
            *drag_pos = pos;
            ctx.request_repaint();
        }

        // 拖拽释放
        if response.drag_stopped_by(PointerButton::Primary)
            && let Some((_piece, _from, pos)) = drag.take()
        {
            if let Some(target) = layout.pos_to_square(pos, flipped) {
                if let Some(event) = Self::execute_drag_drop(controller, target) {
                    events.push(event);
                }
            } else {
                controller.clear_selection();
            }
        }

        // 点击
        if response.clicked_by(PointerButton::Primary)
            && let Some(local) = response.interact_pointer_pos()
            && let Some(sq) = layout.pos_to_square(local, flipped)
        {
            match controller.select_square(sq) {
                SelectionResult::MoveMade { mv } => {
                    events.push(BoardEvent::MoveMade(mv));
                }
                SelectionResult::NeedsPromotion { from, to } => {
                    events.push(BoardEvent::PromotionNeeded { from, to });
                }
                _ => {}
            }
        }

        events
    }

    /// 执行拖拽释放：检查目标格子是否有合法走法，处理升变判断
    fn execute_drag_drop(controller: &mut GameController, target: Square) -> Option<BoardEvent> {
        let legal_moves = controller.legal_moves_for_selected();
        let matching: Vec<_> = legal_moves
            .iter()
            .filter(|m| m.to() == target)
            .copied()
            .collect();

        if matching.is_empty() {
            controller.clear_selection();
            return None;
        }

        // 检查是否需要升变选择
        let has_promotion = matching.iter().any(|m| m.is_promotion());
        if has_promotion && matching.len() > 1 {
            let from = matching[0].from();
            return Some(BoardEvent::PromotionNeeded { from, to: target });
        }

        // 直接执行
        let mv = matching[0];
        controller.make_move(mv);
        Some(BoardEvent::MoveMade(mv))
    }
}
