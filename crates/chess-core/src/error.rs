use thiserror::Error;

use crate::{Color, Move};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChessError {
    /// 无效棋盘位置
    #[error("invalid square index: {0}")]
    InvalidSquare(u32),

    /// 无效 FEN 字符串
    #[error("invalid FEN: {0}")]
    InvalidFen(String),

    /// 非法走法
    #[error("invalid move: {0:?}")]
    InvalidMove(Move),

    /// 找不到指定颜色的王
    #[error("missing king for {0:?}")]
    NoKing(Color),

    /// 游戏已经结束
    #[error("game is already over")]
    GameOver,

    /// 没有可撤销的走法
    #[error("nothing to undo")]
    NothingToUndo,

    /// 非法升变
    #[error("invalid promotion")]
    InvalidPromotion,

    /// 解析错误
    #[error("parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, ChessError>;
