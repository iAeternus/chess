use std::sync::LazyLock;

use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::{CastlingRights, Color, Piece, PieceKind, Position, Square};

/// Zobrist Hash
///
/// 国际象棋局面 Hash 由以下部分异或得到:
///
/// ```text
/// hash = piece XOR side_to_move XOR castling XOR en_passant
/// ```
///
/// halfmove_clock、fullmove_number 不参与 Zobrist Hash
///
/// 正式搜索过程中不重新计算 Hash，
/// 而是在 make_move / unmake_move 中进行增量更新
pub struct Zobrist;

/// Zobrist随机表
struct ZobristTable {
    /// 棋子位置随机数
    /// piece[color][piece_kind][square]
    piece: [[[u64; 64]; 6]; 2],

    /// 行棋方随机数
    /// 约定: White不异或，BlackXOR 此值
    side_to_move: u64,

    /// 王车易位权限随机数
    /// CastlingRights占4bit: K Q k q
    /// 共16种状态
    castling: [u64; 16],

    /// 吃过路兵随机数
    /// 只记录file: a b c d e f g h
    /// 共8种状态
    en_passant: [u64; 8],
}

impl ZobristTable {
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
}

static TABLE: LazyLock<ZobristTable> = LazyLock::new(ZobristTable::new);

impl Zobrist {
    /// 初始化所有随机数
    /// LazyLock会自动初始化，该函数用于提前触发初始化
    pub fn init() {
        LazyLock::force(&TABLE);
    }

    /// 棋子键值，用于增量更新
    /// 删除棋子:  hash ^= piece_key(piece, from)
    /// 添加棋子: hash ^= piece_key(piece, to)
    pub fn piece_key(piece: Piece, sq: Square) -> u64 {
        TABLE.piece[piece.color as usize][piece.kind as usize][sq.index()]
    }

    /// 王车易位键值
    /// 更新:
    /// hash ^= old_castling
    /// hash ^= new_castling
    pub fn castling_key(castling: CastlingRights) -> u64 {
        TABLE.castling[castling.bits() as usize]
    }

    /// 过路兵键值，只记录file
    /// e.g.
    /// ep=e6，使用: en_passant[e]
    pub fn en_passant_key(sq: Option<Square>) -> u64 {
        match sq {
            Some(square) => TABLE.en_passant[square.file() as usize],
            None => 0,
        }
    }

    /// 走子方键值，黑方回合 XOR 常数
    /// 每次换边: hash ^= side_key()
    pub fn side_key() -> u64 {
        TABLE.side_to_move
    }

    /// 重新计算完整局面 Hash
    /// 注意: 此函数不用于搜索
    /// 用途:
    /// - FEN初始化
    /// - 单元测试
    /// - Debug验证增量Hash
    pub fn compute(position: &Position) -> u64 {
        let board = position.board();
        let mut hash = 0;

        // 棋子位置
        for color in [Color::White, Color::Black] {
            for kind in PieceKind::ALL {
                let mut bb = board.piece_kind(color, kind);
                while let Some(square) = bb.pop_lsb() {
                    hash ^= Self::piece_key(Piece::new(color, kind), square);
                }
            }
        }

        // 行棋方
        if position.side_to_move() == Color::Black {
            hash ^= Self::side_key();
        }

        // 王车易位
        hash ^= Self::castling_key(position.castling());

        // 吃过路兵
        hash ^= Self::en_passant_key(position.en_passant());

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
        let hash = Zobrist::compute(&pos);

        assert_eq!(hash, pos.zobrist_key());
    }
}
