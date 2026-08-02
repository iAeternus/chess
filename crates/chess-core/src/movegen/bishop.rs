use arrayvec::ArrayVec;

use crate::{Color, Move, MoveFlag, PieceKind, Position, attack::bishop_rays};

pub(crate) fn generate(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();
    let own = board.pieces(color);
    let occupied = board.occupied();
    let bishops = board.piece_kind(color, PieceKind::Bishop);
    for from in bishops {
        let targets = bishop_rays(from, occupied) & !own; // TODO: bishop_rays返回包含阻挡棋子所在格，可能穿透!own约束
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
    fn test_bishop_empty_diagonal() {
        // 白象 e4
        // 8/8/8/8/4B3/8/8/8 w - - 0 1
        // 左上: d5 c6 b7 a8
        // 右上: f5 g6 h7
        // 左下: d3 c2 b1
        // 右下: f3 g2 h1
        // 共13格
        let position = Position::from_fen("8/8/8/8/4B3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert_eq!(moves.len(), 13);
        let targets = [
            Square::D5,
            Square::C6,
            Square::B7,
            Square::A8,
            Square::F5,
            Square::G6,
            Square::H7,
            Square::D3,
            Square::C2,
            Square::B1,
            Square::F3,
            Square::G2,
            Square::H1,
        ];
        for target in targets {
            assert!(
                moves.iter().any(|m| m.from() == Square::E4
                    && m.to() == target
                    && m.flag() == MoveFlag::Quiet),
                "bishop missing move e4 -> {}",
                target
            );
        }
    }

    #[test]
    fn test_bishop_blocked_by_own_piece() {
        // 白象 e4
        // 白兵 c6 阻挡左上
        // 8/8/2P5/8/4B3/8/8/8 w - - 0 1
        let position = Position::from_fen("8/8/2P5/8/4B3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        // c6不能走
        assert!(!moves.iter().any(|m| m.to() == Square::C6));
        // b7/a8也不能穿过
        assert!(!moves.iter().any(|m| m.to() == Square::B7));
        // 其他方向仍然正常
        assert!(moves.iter().any(|m| m.to() == Square::F5));
    }

    #[test]
    fn test_bishop_capture_enemy() {
        // 黑子 d5
        // 8/8/8/3p4/4B3/8/8/8 w - - 0 1
        let position = Position::from_fen("8/8/8/3p4/4B3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        let capture = moves
            .iter()
            .find(|m| m.from() == Square::E4 && m.to() == Square::D5)
            .unwrap();
        assert_eq!(capture.flag(), MoveFlag::Capture);
        // 不能穿过 d5
        assert!(!moves.iter().any(|m| m.to() == Square::C6));
    }

    #[test]
    fn test_multiple_bishops() {
        // 两个白象
        // 8/8/8/8/2B5/8/8/4B3
        let position = Position::from_fen("8/8/8/8/2B5/8/8/4B3 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        // e1象:
        // d2 c3 b4 a5
        // f2 g3 h4
        //
        // c4象:
        // b5 a6
        // d5 e6 f7 g8
        // b3 a2
        // d3 e2 f1
        assert_eq!(moves.len(), 18);
    }

    #[test]
    fn test_black_bishop() {
        let position = Position::from_fen("8/8/8/8/4b3/8/8/8 b - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::Black, &mut moves);

        assert_eq!(moves.len(), 13);
    }
}
