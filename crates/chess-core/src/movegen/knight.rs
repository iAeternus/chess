use arrayvec::ArrayVec;

use crate::{Color, Move, MoveFlag, PieceKind, Position, attack::KNIGHT_ATTACKS};

pub(crate) fn generate(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();
    let own = board.pieces(color);
    let knights = board.piece_kind(color, PieceKind::Knight);
    for from in knights {
        let targets = KNIGHT_ATTACKS[from.index()] & !own;
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
    fn test_knight_empty_center() {
        // 白马 e4
        // 8/8/8/8/4N3/8/8/8 w - - 0 1
        // 攻击:
        // c3 c5
        // d2 d6
        // f2 f6
        // g3 g5
        let position = Position::from_fen("8/8/8/8/4N3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert_eq!(moves.len(), 8);
        let targets = [
            Square::C3,
            Square::C5,
            Square::D2,
            Square::D6,
            Square::F2,
            Square::F6,
            Square::G3,
            Square::G5,
        ];
        for target in targets {
            assert!(
                moves.iter().any(|m| m.from() == Square::E4
                    && m.to() == target
                    && m.flag() == MoveFlag::Quiet),
                "knight missing move e4 -> {}",
                target
            );
        }
    }

    #[test]
    fn test_knight_corner() {
        // 白马 a1
        // 只能攻击: b3 c2
        let position = Position::from_fen("8/8/8/8/8/8/8/N7 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert_eq!(moves.len(), 2);
        assert!(moves.iter().any(|m| m.to() == Square::B3));
        assert!(moves.iter().any(|m| m.to() == Square::C2));
    }

    #[test]
    fn test_knight_capture_enemy() {
        // 白马 e4
        // 黑兵 c5
        // 可以吃 c5
        let position = Position::from_fen("8/8/8/2p5/4N3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        let capture = moves
            .iter()
            .find(|m| m.from() == Square::E4 && m.to() == Square::C5)
            .unwrap();
        assert_eq!(capture.flag(), MoveFlag::Capture);
        assert!(!moves.iter().any(|m| m.to() == Square::B7));
    }

    #[test]
    fn test_knight_blocked_by_own_piece() {
        // 马不受阻挡影响
        // 但是不能走到己方棋子所在格
        // 白马 e4
        // 白兵 c5
        let position = Position::from_fen("8/8/8/2P5/4N3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        // c5 是己方棋子，不能生成
        assert!(!moves.iter().any(|m| m.to() == Square::C5));
        // 其他位置仍然存在
        assert!(moves.iter().any(|m| m.to() == Square::G5));
        assert_eq!(moves.len(), 7);
    }

    #[test]
    fn test_multiple_knights() {
        // 两个白马
        // e1: c2 d3 f3 g2
        // e4:
        // c3 c5 d2 d6
        // f2 f6 g3 g5
        let position = Position::from_fen("8/8/8/8/4N3/8/8/4N3 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        // e1 knight: c2 d3 f3 g2 = 4
        // e4 knight: 8
        assert_eq!(moves.len(), 12);
    }

    #[test]
    fn test_black_knight() {
        let position = Position::from_fen("8/8/8/8/4n3/8/8/8 b - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::Black, &mut moves);

        assert_eq!(moves.len(), 8);
        assert!(moves.iter().all(|m| m.flag() == MoveFlag::Quiet));
    }
}
