use arrayvec::ArrayVec;

use crate::{
    CastlingRights, Color, Move, MoveFlag, PieceKind, Position, Square,
    attack::{KING_ATTACKS, is_square_attacked},
};

pub(crate) fn generate(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
    generate_king_moves(position, color, moves);
    generate_castling(position, color, moves);
}

fn generate_king_moves(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();
    let own = board.pieces(color);
    let kings = board.piece_kind(color, PieceKind::King);
    for from in kings {
        let targets = KING_ATTACKS[from.index()] & !own;
        for to in targets {
            if board.piece_at(to).is_some() {
                moves.push(Move::new(from, to, MoveFlag::Capture));
            } else {
                moves.push(Move::new(from, to, MoveFlag::Quiet));
            }
        }
    }
}

/// 生成王车易位
///
/// 条件:
/// 1. CastlingRights允许
/// 2. 王和车未移动过
/// 3. 中间无棋子
/// 4. 王经过的格子没有被攻击
///
/// 注意: 这里只生成伪合法走法，最终合法性由 make_move + is_square_attacked 再过滤
pub fn generate_castling(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
    let rights = position.castling();

    match color {
        Color::White => {
            // 王翼易位 O-O
            if rights.contains(CastlingRights::WHITE_KING_SIDE) {
                try_white_kingside(position, moves);
            }

            // 后翼易位 O-O-O
            if rights.contains(CastlingRights::WHITE_QUEEN_SIDE) {
                try_white_queenside(position, moves);
            }
        }

        Color::Black => {
            // 王翼易位 O-O
            if rights.contains(CastlingRights::BLACK_KING_SIDE) {
                try_black_kingside(position, moves);
            }

            // 后翼易位 O-O-O
            if rights.contains(CastlingRights::BLACK_QUEEN_SIDE) {
                try_black_queenside(position, moves);
            }
        }
    }
}

/// 白方王翼易位
///
/// e1 -> g1
pub fn try_white_kingside(position: &Position, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();

    // f1,g1必须为空
    if board.piece_at(Square::F1).is_some() || board.piece_at(Square::G1).is_some() {
        return;
    }

    // 王不能经过攻击
    if is_square_attacked(board, Square::E1, Color::Black)
        || is_square_attacked(board, Square::F1, Color::Black)
        || is_square_attacked(board, Square::G1, Color::Black)
    {
        return;
    }

    moves.push(Move::new(Square::E1, Square::G1, MoveFlag::KingCastle));
}

/// 白方后翼易位
///
/// e1 -> c1
pub fn try_white_queenside(position: &Position, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();

    // b1,c1,d1必须为空
    if board.piece_at(Square::B1).is_some()
        || board.piece_at(Square::C1).is_some()
        || board.piece_at(Square::D1).is_some()
    {
        return;
    }

    // 王经过 e1,d1,c1
    if is_square_attacked(board, Square::E1, Color::Black)
        || is_square_attacked(board, Square::D1, Color::Black)
        || is_square_attacked(board, Square::C1, Color::Black)
    {
        return;
    }

    moves.push(Move::new(Square::E1, Square::C1, MoveFlag::QueenCastle));
}

/// 黑方王翼易位
///
/// e8 -> g8
fn try_black_kingside(position: &Position, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();

    if board.piece_at(Square::F8).is_some() || board.piece_at(Square::G8).is_some() {
        return;
    }

    if is_square_attacked(board, Square::E8, Color::White)
        || is_square_attacked(board, Square::F8, Color::White)
        || is_square_attacked(board, Square::G8, Color::White)
    {
        return;
    }

    moves.push(Move::new(Square::E8, Square::G8, MoveFlag::KingCastle));
}

/// 黑方后翼易位
///
/// e8 -> c8
fn try_black_queenside(position: &Position, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();

    if board.piece_at(Square::B8).is_some()
        || board.piece_at(Square::C8).is_some()
        || board.piece_at(Square::D8).is_some()
    {
        return;
    }

    if is_square_attacked(board, Square::E8, Color::White)
        || is_square_attacked(board, Square::D8, Color::White)
        || is_square_attacked(board, Square::C8, Color::White)
    {
        return;
    }

    moves.push(Move::new(Square::E8, Square::C8, MoveFlag::QueenCastle));
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrayvec::ArrayVec;

    use crate::{Color, Move, MoveFlag, Position, Square};

    #[test]
    fn test_king_empty_center() {
        // 白王 e4
        // 8个方向
        let position = Position::from_fen("8/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert_eq!(moves.len(), 8);
        let targets = [
            Square::D3,
            Square::E3,
            Square::F3,
            Square::D4,
            Square::F4,
            Square::D5,
            Square::E5,
            Square::F5,
        ];
        for target in targets {
            assert!(
                moves.iter().any(|m| m.from() == Square::E4
                    && m.to() == target
                    && m.flag() == MoveFlag::Quiet),
                "king missing e4 -> {}",
                target
            );
        }
    }

    #[test]
    fn test_king_capture_enemy() {
        // 白王 e4
        // 黑子 f5
        let position = Position::from_fen("8/8/8/5p2/4K3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        let capture = moves
            .iter()
            .find(|m| m.from() == Square::E4 && m.to() == Square::F5)
            .unwrap();
        assert_eq!(capture.flag(), MoveFlag::Capture);
    }

    #[test]
    fn test_king_blocked_by_own_piece() {
        // 白王 e4
        // 白兵 e5
        let position = Position::from_fen("8/8/8/4P3/4K3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);
        assert!(!moves.iter().any(|m| m.to() == Square::E5));
    }

    #[test]
    fn test_white_kingside_castle() {
        // 白王翼易位 e1 -> g1
        let position = Position::from_fen("4k3/8/8/8/8/8/8/4K2R w K - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert!(moves.iter().any(|m| m.from() == Square::E1
            && m.to() == Square::G1
            && m.flag() == MoveFlag::KingCastle));
    }

    #[test]
    fn test_white_queenside_castle() {
        // 白后翼易位  e1 -> c1
        let position = Position::from_fen("4k3/8/8/8/8/8/8/R3K3 w Q - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert!(moves.iter().any(|m| m.from() == Square::E1
            && m.to() == Square::C1
            && m.flag() == MoveFlag::QueenCastle));
    }

    #[test]
    fn test_black_kingside_castle() {
        let position = Position::from_fen("4k2r/8/8/8/8/8/8/4K3 b k - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::Black, &mut moves);

        assert!(moves.iter().any(|m| m.from() == Square::E8
            && m.to() == Square::G8
            && m.flag() == MoveFlag::KingCastle));
    }

    #[test]
    fn test_black_queenside_castle() {
        let position = Position::from_fen("r3k3/8/8/8/8/8/8/4K3 b q - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::Black, &mut moves);

        assert!(moves.iter().any(|m| m.from() == Square::E8
            && m.to() == Square::C8
            && m.flag() == MoveFlag::QueenCastle));
    }

    #[test]
    fn test_castle_blocked() {
        // f1 有棋子
        // 不允许王翼易位
        let position = Position::from_fen("4k3/8/8/8/8/8/8/4KN1R w K - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);
        assert!(!moves.iter().any(|m| m.flag() == MoveFlag::KingCastle));
    }

    #[test]
    fn test_castle_through_attack() {
        // f1 被攻击
        // 黑车 f8
        let position = Position::from_fen("4k3/5r2/8/8/8/8/8/4K2R w K - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert!(!moves.iter().any(|m| m.flag() == MoveFlag::KingCastle));
    }

    #[test]
    fn test_no_castle_without_right() {
        // 没有K权限
        let position = Position::from_fen("4k3/8/8/8/8/8/8/4K2R w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert!(!moves.iter().any(|m| m.flag() == MoveFlag::KingCastle));
    }
}
