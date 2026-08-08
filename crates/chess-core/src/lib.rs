//! # chess-core — 国际象棋核心引擎
//!
//! ## 架构分层
//!
//! `chess-core` 采用分层设计，对外导出不同抽象级别的 API：
//!
//! ### Fundamental 层（基础操作）
//!
//! 棋盘状态与走法生成的低级 API，适合 AI 引擎和性能敏感的代码：
//!
//! - [`Position`] — 完整局面状态（棋盘、行棋方、易位权、过路兵、Zobrist 哈希）
//! - [`Board`] — 位棋盘棋子存储
//! - [`Move`] — 紧凑的走法编码（32-bit）
//! - [`generate_legal`] — 生成合法走法（`&mut Position`，零 clone，适合搜索树）
//! - [`generate_pseudo_legal`] — 生成伪合法走法（`&Position`）
//! - [`legal_moves_of`] — 生成合法走法（`&Position`，内部 clone，便捷版本）
//! - [`is_legal`] — 判断走法是否合法（内部 make/unmake 来回）
//! - [`make_move`] / [`unmake_move`] — 执行/撤销走法
//! - [`Color`], [`Square`], [`Piece`], [`PieceKind`], [`CastlingRights`], [`BitBoard`]
//!
//! ### Orchestration 层（对局管理）
//!
//! 高级 API，封装走法历史、PGN 管理、回合状态追踪：
//!
//! - [`Game`] — 完整对局管理器（位置 + 走法历史 + PGN 头信息）
//!
//! ### 建议使用方式
//!
//! - **编写 AI 引擎** → 使用 Fundamental 层：直接操作 [`Position`]，调用
//!   [`generate_legal`]、[`make_move`]/[`unmake_move`]，避免 [`Game`] 的开销
//! - **编写 GUI / 对局管理** → 使用 Orchestration 层：通过 [`Game`] 管理走法历史、
//!   导入/导出 PGN、查询对局状态
//! - **编写 PGN 工具** → 启用 `all` feature，使用 [`from_pgn`]、[`to_pgn`]、[`parse_san`]、
//!   [`move_to_san`]

mod attack;
mod bitboard;
mod board;
mod castling;
mod color;
mod error;
mod fen;
mod game;
mod legality;
mod makemove;
mod movegen;
mod mv;
mod perft;
mod pgn;
mod piece;
mod position;
mod square;
mod zobrist;

pub use bitboard::BitBoard;
pub use board::Board;
pub use castling::CastlingRights;
pub use color::Color;
pub use error::{ChessError, Result};
pub use game::Game;
pub use legality::is_legal;
pub use makemove::{Undo, make_move, unmake_move};
pub use movegen::{generate_legal, generate_pseudo_legal, legal_moves_of};
pub use mv::{Move, MoveFlag, Promotion};
pub use perft::{divide, perft};
pub use piece::{Piece, PieceKind};
pub use position::Position;
pub use square::Square;

#[cfg(feature = "all")]
pub use fen::{fen2position, position2fen};
#[cfg(feature = "all")]
pub use pgn::{from_pgn, move_to_san, parse_san, to_pgn};
#[cfg(feature = "all")]
pub use zobrist::Zobrist;
