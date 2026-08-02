use std::sync::LazyLock;

use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::{Board, CastlingRights, Color, PieceKind, Square};

/// Zobrist Hash 随机表
///
/// 国际象棋局面 Hash 由以下部分异或得到:
///
/// ```text
/// hash =
///     piece
///     XOR side_to_move
///     XOR castling
///     XOR en_passant
/// ```
///
/// halfmove_clock、fullmove_number 不参与 Zobrist Hash
#[derive(Debug)]
pub struct ZobristKeys {
    /// 棋子位置随机数，piece[color][piece_kind][square]
    piece: [[[u64; 64]; 6]; 2],

    /// 行棋方随机数
    /// 约定：白方走，不异或；黑方走，XOR 此值
    side_to_move: u64,

    /// 王车易位权限随机数
    /// CastlingRights 占4b: K Q k q
    /// 共16种状态
    castling: [u64; 16],

    /// 吃过路兵随机数
    /// 只需要记录file: a b c d e f g h
    /// 共8种
    en_passant: [u64; 8],
}

impl ZobristKeys {
    /// 全局共享随机表
    /// 整个程序生命周期只创建一次
    fn global() -> &'static Self {
        static INSTANCE: LazyLock<ZobristKeys> = LazyLock::new(ZobristKeys::new);

        &INSTANCE
    }

    /// 创建随机表，使用固定 Seed
    fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(0x9E3779B97F4A7C15);

        // 棋子随机表
        let mut piece = [[[0u64; 64]; 6]; 2];
        for color in 0..2 {
            for kind in 0..6 {
                for square in 0..64 {
                    piece[color][kind][square] = rng.random();
                }
            }
        }

        // 行棋方
        let side_to_move = rng.random();

        // 王车易位
        let mut castling = [0u64; 16];
        for value in &mut castling {
            *value = rng.random();
        }

        // 吃过路兵
        let mut en_passant = [0u64; 8];
        for value in &mut en_passant {
            *value = rng.random();
        }

        Self {
            piece,
            side_to_move,
            castling,
            en_passant,
        }
    }

    /// 计算局面 Zobrist Hash
    ///
    /// # 参数
    /// - board: 当前棋盘
    /// - side_to_move: 当前行动方
    /// - castling: 王车易位权限
    /// - en_passant: 吃过路兵目标格
    ///
    /// # 返回
    /// 64 bit Zobrist Hash
    pub fn compute(
        board: &Board,
        side_to_move: Color,
        castling: CastlingRights,
        en_passant: Option<Square>,
    ) -> u64 {
        let keys = Self::global();
        let mut hash = 0u64;

        // 棋子位置
        for color in [Color::White, Color::Black] {
            for kind in PieceKind::ALL {
                let mut bb = board.piece_kind(color, kind);
                while let Some(square) = bb.pop_lsb() {
                    hash ^= keys.piece[color as usize][kind as usize][square.index()];
                }
            }
        }

        // 行棋方
        if side_to_move == Color::Black {
            hash ^= keys.side_to_move;
        }

        // 王车易位权限
        hash ^= keys.castling[castling.bits() as usize];

        // 吃过路兵
        if let Some(square) = en_passant {
            hash ^= keys.en_passant[square.file() as usize];
        }

        hash
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Position;

    #[test]
    fn same_position_same_hash() {
        let a = Position::startpos();
        let b = Position::startpos();

        assert_eq!(a.zobrist_key(), b.zobrist_key());
    }

    #[test]
    fn different_side_different_hash() {
        let white = Position::from_fen("8/8/8/8/8/8/8/K6k w - - 0 1").unwrap();
        let black = Position::from_fen("8/8/8/8/8/8/8/K6k b - - 0 1").unwrap();

        assert_ne!(white.zobrist_key(), black.zobrist_key());
    }

    #[test]
    fn recompute_hash_equal() {
        let pos = Position::startpos();
        let hash = ZobristKeys::compute(
            pos.board(),
            pos.side_to_move(),
            pos.castling(),
            pos.en_passant(),
        );

        assert_eq!(hash, pos.zobrist_key());
    }
}
