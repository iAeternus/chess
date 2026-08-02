use arrayvec::ArrayVec;

use crate::{Color, Move, MoveFlag, PieceKind, Position, attack::rook_rays};

pub(crate) fn generate(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();
    let own = board.pieces(color);
    let occupied = board.occupied();
    let rooks = board.piece_kind(color, PieceKind::Rook);
    for from in rooks {
        let targets = rook_rays(from, occupied) & !own;
        for to in targets {
            if board.piece_at(to).is_some() {
                moves.push(Move::new(from, to, MoveFlag::Capture));
            } else {
                moves.push(Move::new(from, to, MoveFlag::Quiet));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrayvec::ArrayVec;

    use crate::{Color, Move, MoveFlag, Position, Square};

    #[test]
    fn test_rook_empty_center() {
        // 白车 e4
        // 横向:
        // a4 b4 c4 d4 f4 g4 h4
        // 纵向:
        // e1 e2 e3 e5 e6 e7 e8
        // 共14
        let position = Position::from_fen("8/8/8/8/4R3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert_eq!(moves.len(), 14);
        let targets = [
            Square::A4,
            Square::B4,
            Square::C4,
            Square::D4,
            Square::F4,
            Square::G4,
            Square::H4,
            Square::E1,
            Square::E2,
            Square::E3,
            Square::E5,
            Square::E6,
            Square::E7,
            Square::E8,
        ];
        for target in targets {
            assert!(
                moves.iter().any(|m| m.from() == Square::E4
                    && m.to() == target
                    && m.flag() == MoveFlag::Quiet),
                "rook missing e4 -> {}",
                target
            );
        }
    }

    #[test]
    fn test_rook_blocked_by_own_piece() {
        // 白车 e4
        // 白兵 e6 阻挡
        let position = Position::from_fen("8/8/4P3/8/4R3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        // e6不能走
        assert!(!moves.iter().any(|m| m.to() == Square::E6));
        // e7/e8不能穿透
        assert!(
            !moves
                .iter()
                .any(|m| m.to() == Square::E7 || m.to() == Square::E8)
        );
        // 横向正常
        assert!(moves.iter().any(|m| m.to() == Square::H4));
    }

    #[test]
    fn test_rook_capture_enemy() {
        // 白车 e4
        // 黑子 e6
        let position = Position::from_fen("8/8/4p3/8/4R3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        let capture = moves
            .iter()
            .find(|m| m.from() == Square::E4 && m.to() == Square::E6)
            .unwrap();
        assert_eq!(capture.flag(), MoveFlag::Capture);
        // 不能穿过
        assert!(!moves.iter().any(|m| m.to() == Square::E7));
    }

    #[test]
    fn test_multiple_rooks() {
        // 两个白车
        // a1: 14
        // h8: 14
        // 共28
        let position = Position::from_fen("7R/8/8/8/8/8/8/R7 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);
        assert_eq!(moves.len(), 28);
    }

    #[test]
    fn test_black_rook() {
        let position = Position::from_fen("8/8/8/8/4r3/8/8/8 b - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::Black, &mut moves);

        assert_eq!(moves.len(), 14);
    }
}
