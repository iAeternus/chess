use arrayvec::ArrayVec;

use crate::{Color, Move, MoveFlag, PieceKind, Position, attack::queen_rays};

pub(crate) fn generate(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();
    let own = board.pieces(color);
    let occupied = board.occupied();
    let queens = board.piece_kind(color, PieceKind::Queen);
    for from in queens {
        let targets = queen_rays(from, occupied) & !own;
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
    fn test_queen_empty_center() {
        // 白后 e4
        // rook: 14
        // bishop: 13
        // total: 27
        let position = Position::from_fen("8/8/8/8/4Q3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);
        assert_eq!(moves.len(), 27);
    }

    #[test]
    fn test_queen_capture_enemy() {
        // 白后 e4
        // 黑子 h4
        let position = Position::from_fen("8/8/8/8/4Q2p/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        let capture = moves
            .iter()
            .find(|m| m.from() == Square::E4 && m.to() == Square::H4)
            .unwrap();
        assert_eq!(capture.flag(), MoveFlag::Capture);
        // h4 后不能继续穿透
        assert!(!moves.iter().any(|m| m.to() == Square::H5));
    }

    #[test]
    fn test_queen_blocked() {
        // 白后 e4
        // 白兵 e6
        let position = Position::from_fen("8/8/4P3/8/4Q3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert!(
            !moves
                .iter()
                .any(|m| m.to() == Square::E7 || m.to() == Square::E8)
        );
        assert!(moves.iter().any(|m| m.to() == Square::A4));
    }

    #[test]
    fn test_multiple_queens() {
        // 两个白后
        // e4: 27
        // a1: 21
        // 注意: 两个后互相阻挡部分路线
        let position = Position::from_fen("8/8/8/8/4Q3/8/8/Q7 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);
        assert!(moves.len() > 40);
    }

    #[test]
    fn test_black_queen() {
        let position = Position::from_fen("8/8/8/8/4q3/8/8/8 b - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::Black, &mut moves);

        assert_eq!(moves.len(), 27);
    }
}
