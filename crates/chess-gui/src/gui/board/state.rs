//! 棋盘渲染状态：封装渲染器所需的全部视觉信息

use arrayvec::ArrayVec;
use chess_core::{Color, Move, Piece, Position, Square};
use egui::Pos2;

/// 棋盘上的箭头（用于分析模式标注）
#[derive(Debug, Clone)]
pub struct BoardArrow {
    pub from: Square,
    pub to: Square,
    pub color: egui::Color32,
}

/// 棋盘渲染所需的完整状态
///
/// 由 `GameController` 提供数据，`ChessApp` 组装后传给 `BoardRenderer`
pub struct BoardState {
    /// 当前局面（棋盘格坐标）
    pub position: Position,

    /// 棋盘视角：底部棋子的颜色
    pub view_from: Color,

    /// 选中的格子（棋盘格坐标）
    pub selected_square: Option<Square>,

    /// 选中棋子的合法目标走法
    pub legal_moves: ArrayVec<Move, 256>,

    /// 最后一步走法（用于 from/to 高亮）
    pub last_move: Option<Move>,

    /// 被将军的王所在格子（用于光晕效果，None 表示未将军）
    pub king_in_check: Option<Square>,

    /// 拖拽状态：(棋子, 来源格子, 当前鼠标在 painter 坐标系下的位置)
    pub drag: Option<(Piece, Square, Pos2)>,

    /// 用户绘制的箭头（分析模式）
    pub arrows: Vec<BoardArrow>,

    /// 当前正在拖动的箭头预览
    pub arrow_preview: Option<BoardArrow>,
}

impl BoardState {
    /// 从当前位置创建渲染状态（不含拖拽和箭头）
    #[allow(dead_code)]
    pub fn from_position(position: Position) -> Self {
        Self {
            position,
            view_from: Color::White,
            selected_square: None,
            legal_moves: ArrayVec::new(),
            last_move: None,
            king_in_check: None,
            drag: None,
            arrows: Vec::new(),
            arrow_preview: None,
        }
    }
}
