use arrayvec::ArrayVec;

use crate::{
    Board, CastlingRights, Color, Move, MoveFlag, Piece, PieceKind, Position, Square,
    attack::KING_ATTACKS,
};

pub(crate) fn generate(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
    generate_king_moves(position, color, moves);
    generate_castling(position, color, moves);
}

fn generate_king_moves(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();
    let own = board.pieces(color);
    let enemy_king = board.piece_kind(color.flip(), PieceKind::King);
    let kings = board.piece_kind(color, PieceKind::King);
    for from in kings {
        let targets = KING_ATTACKS[from.index()] & !own & !enemy_king;
        for to in targets {
            let flag = if board.piece_at(to).is_some() {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };
            moves.push(Move::new(from, to, flag));
        }
    }
}

/// 生成王车易位
///
/// 条件:
/// 1. CastlingRights允许
/// 2. 王和车未移动过
/// 3. 中间无棋子
///
/// 注意: 这里只生成伪合法走法，最终合法性由 make_move + is_square_attacked 再过滤；
/// 王是否被将军、经过攻击格等合法性检查，由 legality::is_legal 完成
fn generate_castling(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
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
fn try_white_kingside(position: &Position, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();

    // e1 必须有白王
    if !has_piece(board, Color::White, PieceKind::King, Square::E1) {
        return;
    }

    // h1 必须有白车
    if !has_piece(board, Color::White, PieceKind::Rook, Square::H1) {
        return;
    }

    // f1,g1必须为空
    if board.piece_at(Square::F1).is_some() || board.piece_at(Square::G1).is_some() {
        return;
    }

    moves.push(Move::new(Square::E1, Square::G1, MoveFlag::KingCastle));
}

/// 白方后翼易位
///
/// e1 -> c1
fn try_white_queenside(position: &Position, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();

    if !has_piece(board, Color::White, PieceKind::King, Square::E1) {
        return;
    }

    if !has_piece(board, Color::White, PieceKind::Rook, Square::A1) {
        return;
    }

    // b1,c1,d1必须为空
    if board.piece_at(Square::B1).is_some()
        || board.piece_at(Square::C1).is_some()
        || board.piece_at(Square::D1).is_some()
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

    if !has_piece(board, Color::Black, PieceKind::King, Square::E8) {
        return;
    }

    if !has_piece(board, Color::Black, PieceKind::Rook, Square::H8) {
        return;
    }

    if board.piece_at(Square::F8).is_some() || board.piece_at(Square::G8).is_some() {
        return;
    }

    moves.push(Move::new(Square::E8, Square::G8, MoveFlag::KingCastle));
}

/// 黑方后翼易位
///
/// e8 -> c8
fn try_black_queenside(position: &Position, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();

    if !has_piece(board, Color::Black, PieceKind::King, Square::E8) {
        return;
    }

    if !has_piece(board, Color::Black, PieceKind::Rook, Square::A8) {
        return;
    }

    if board.piece_at(Square::B8).is_some()
        || board.piece_at(Square::C8).is_some()
        || board.piece_at(Square::D8).is_some()
    {
        return;
    }

    moves.push(Move::new(Square::E8, Square::C8, MoveFlag::QueenCastle));
}

fn has_piece(board: &Board, color: Color, kind: PieceKind, square: Square) -> bool {
    matches!(
        board.piece_at(square),
        Some(piece) if piece.color ==color && piece.kind==kind
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrayvec::ArrayVec;

    use crate::{Color, Move, MoveFlag, Position, Square, legality::is_legal};

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
    fn test_no_castle_without_right() {
        // 没有K权限
        let position = Position::from_fen("4k3/8/8/8/8/8/8/4K2R w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert!(!moves.iter().any(|m| m.flag() == MoveFlag::KingCastle));
    }

    #[test]
    fn test_castle_without_king() {
        let position = Position::from_fen("4k3/8/8/8/8/8/8/7R w K - 0 1").unwrap();
        let mut moves = ArrayVec::new();

        generate(&position, Color::White, &mut moves);

        assert!(!moves.iter().any(|m| m.flag() == MoveFlag::KingCastle));
    }

    #[test]
    fn test_castle_requires_correct_rook() {
        let cases = [
            // 白王翼易位
            // h1 没有车
            ("4k3/8/8/8/8/8/8/4K3 w K - 0 1", Color::White),
            // h1 是黑车
            ("4k3/8/8/8/8/8/8/4K2r w K - 0 1", Color::White),
            // h1 是白象
            ("4k3/8/8/8/8/8/8/4K2B w K - 0 1", Color::White),
            // 黑王翼易位
            // h8 是白车
            ("4k2R/8/8/8/8/8/8/4K3 b k - 0 1", Color::Black),
            // h8 是黑马
            ("4k2n/8/8/8/8/8/8/4K3 b k - 0 1", Color::Black),
        ];

        for (fen, color) in cases {
            let position = Position::from_fen(fen).unwrap();
            let mut moves = ArrayVec::<Move, 256>::new();

            generate(&position, color, &mut moves);

            assert!(
                !moves.iter().any(|m| { m.flag() == MoveFlag::KingCastle }),
                "illegal castle generated for fen: {}",
                fen
            );
        }
    }

    #[test]
    fn test_queenside_b_square_attack_allowed() {
        let position = Position::from_fen("1r2k3/8/8/8/8/8/8/R3K3 w Q - 0 1").unwrap();
        let mut moves = ArrayVec::new();

        generate(&position, Color::White, &mut moves);

        assert!(moves.iter().any(|m| m.flag() == MoveFlag::QueenCastle));
    }

    #[test]
    fn test_castle_king_not_on_home_square() {
        let position = Position::from_fen("4k3/8/8/8/8/8/4K3/7R w K - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert!(!moves.iter().any(|m| m.flag() == MoveFlag::KingCastle));
    }

    #[test]
    fn test_both_castling_available() {
        let position = Position::from_fen("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert!(moves.iter().any(|m| m.flag() == MoveFlag::KingCastle));
        assert!(moves.iter().any(|m| m.flag() == MoveFlag::QueenCastle));
    }
}
