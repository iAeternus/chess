use arrayvec::ArrayVec;

use crate::{
    BitBoard, Color, Move, MoveFlag, PieceKind, Position, Promotion, Square, attack::PAWN_ATTACKS,
};

/// 生成兵的所有伪合法走法
///
/// 包含:
/// - 单步推进
/// - 双步推进
/// - 吃子
/// - 吃过路兵
/// - 升变
pub(crate) fn generate(position: &Position, color: Color, moves: &mut ArrayVec<Move, 256>) {
    let board = position.board();
    let pawns = board.piece_kind(color, PieceKind::Pawn);
    let occupied = board.occupied();
    let enemy = board.pieces(color.flip());
    for from in pawns {
        generate_push(from, color, occupied, moves);
        generate_capture(from, color, enemy, moves);
        generate_en_passant(position, from, color, moves);
    }
}

/// 推进
fn generate_push(from: Square, color: Color, occupied: BitBoard, moves: &mut ArrayVec<Move, 256>) {
    let direction: i8 = match color {
        Color::White => 1,
        Color::Black => -1,
    };
    let rank = from.rank() as i8;
    let file = from.file();
    let target_rank = rank + direction;
    if !(0..8).contains(&target_rank) {
        return;
    }

    let to = Square::from_coord(file, target_rank as u8).unwrap();

    // 前方有棋子不能走
    if occupied.contains(to) {
        return;
    }

    // 升变
    if is_promotion_rank(to, color) {
        add_promotions(from, to, false, moves);
        return;
    }

    moves.push(Move::new(from, to, MoveFlag::Quiet));

    // 双步推进
    // 双步推进依赖单步推进的空格检查
    // 中间格等于单步目标格，因此不会重复检查
    if is_start_rank(from, color) {
        let target = Square::from_coord(file, (rank + direction * 2) as u8).unwrap();
        if !occupied.contains(target) {
            moves.push(Move::new(from, target, MoveFlag::DoublePawnPush));
        }
    }
}

/// 吃子
fn generate_capture(from: Square, color: Color, enemy: BitBoard, moves: &mut ArrayVec<Move, 256>) {
    let attacks = PAWN_ATTACKS[color as usize][from.index()];
    let captures = attacks & enemy;
    for to in captures {
        if is_promotion_rank(to, color) {
            add_promotions(from, to, true, moves);
        } else {
            moves.push(Move::new(from, to, MoveFlag::Capture));
        }
    }
}

/// 吃过路兵
fn generate_en_passant(
    position: &Position,
    from: Square,
    color: Color,
    moves: &mut ArrayVec<Move, 256>,
) {
    let Some(ep) = position.en_passant() else {
        return;
    };

    let attacks = PAWN_ATTACKS[color as usize][from.index()];
    if attacks.contains(ep) {
        moves.push(Move::new(from, ep, MoveFlag::EnPassant));
    }
}

/// 升变
fn add_promotions(from: Square, to: Square, capture: bool, moves: &mut ArrayVec<Move, 256>) {
    for promotion in [
        Promotion::Queen,
        Promotion::Rook,
        Promotion::Bishop,
        Promotion::Knight,
    ] {
        moves.push(Move::new_promotion(from, to, promotion, capture));
    }
}

fn is_start_rank(sq: Square, color: Color) -> bool {
    match color {
        Color::White => sq.rank() == 1,
        Color::Black => sq.rank() == 6,
    }
}

fn is_promotion_rank(sq: Square, color: Color) -> bool {
    match color {
        Color::White => sq.rank() == 7,
        Color::Black => sq.rank() == 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrayvec::ArrayVec;

    use crate::{Color, Move, MoveFlag, Position, Promotion, Square};

    #[test]
    fn test_white_pawn_single_push() {
        // 白兵 e4 -> e5
        let position = Position::from_fen("8/8/8/8/4P3/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert_eq!(moves.len(), 1);

        let mv = &moves[0];
        assert_eq!(mv.from(), Square::E4);
        assert_eq!(mv.to(), Square::E5);
        assert_eq!(mv.flag(), MoveFlag::Quiet);
    }

    #[test]
    fn test_white_pawn_double_push() {
        // e2 -> e3
        // e2 -> e4
        let position = Position::from_fen("8/8/8/8/8/8/4P3/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert_eq!(moves.len(), 2);
        assert!(moves.iter().any(|m| m.from() == Square::E2
            && m.to() == Square::E3
            && m.flag() == MoveFlag::Quiet));
        assert!(moves.iter().any(|m| m.from() == Square::E2
            && m.to() == Square::E4
            && m.flag() == MoveFlag::DoublePawnPush));
    }

    #[test]
    fn test_pawn_blocked() {
        // e2前有黑子e3
        let position = Position::from_fen("8/8/8/8/8/4p3/4P3/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn test_white_pawn_capture() {
        // e4 吃 d5/f5
        let position = Position::from_fen("8/8/8/3p1p2/4P3/8/8/8 w - - 0 1").unwrap();

        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        // e4-e5
        // e4xd5
        // e4xf5
        assert_eq!(moves.len(), 3);
        assert!(
            moves
                .iter()
                .any(|m| m.to() == Square::D5 && m.flag() == MoveFlag::Capture)
        );
        assert!(
            moves
                .iter()
                .any(|m| m.to() == Square::F5 && m.flag() == MoveFlag::Capture)
        );
        assert!(
            moves
                .iter()
                .any(|m| m.to() == Square::E5 && m.flag() == MoveFlag::Quiet)
        );
    }

    #[test]
    fn test_pawn_capture_promotion() {
        // 白兵 e7 吃 d8 升变
        let position = Position::from_fen("3p4/4P3/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert_eq!(moves.len(), 8);

        // 吃子升变
        for promotion in [
            Promotion::Queen,
            Promotion::Rook,
            Promotion::Bishop,
            Promotion::Knight,
        ] {
            assert!(moves.iter().any(|m| m.from() == Square::E7
                && m.to() == Square::D8
                && m.flag() == MoveFlag::PromotionCapture
                && m.promotion().unwrap() == promotion));
        }

        // 前进升变
        for promotion in [
            Promotion::Queen,
            Promotion::Rook,
            Promotion::Bishop,
            Promotion::Knight,
        ] {
            assert!(moves.iter().any(|m| m.from() == Square::E7
                && m.to() == Square::E8
                && m.flag() == MoveFlag::Promotion
                && m.promotion().unwrap() == promotion));
        }
    }

    #[test]
    fn test_pawn_promotion_push() {
        // e7-e8=Q/R/B/N
        let position = Position::from_fen("8/4P3/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert_eq!(moves.len(), 4);
        assert!(moves.iter().all(|m| m.from() == Square::E7
            && m.to() == Square::E8
            && m.flag() == MoveFlag::Promotion));
    }

    #[test]
    fn test_en_passant() {
        // 白兵 e5
        // 黑兵刚走 d7-d5
        // ep=d6
        // e5xd6 e.p.
        let position = Position::from_fen("8/8/8/3pP3/8/8/8/8 w - d6 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        assert!(moves.iter().any(|m| m.from() == Square::E5
            && m.to() == Square::D6
            && m.flag() == MoveFlag::EnPassant));
    }

    #[test]
    fn test_black_pawn_direction() {
        // 黑兵 e5 -> e4
        let position = Position::from_fen("8/8/8/4p3/8/8/8/8 b - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::Black, &mut moves);

        assert!(
            moves
                .iter()
                .any(|m| m.from() == Square::E5 && m.to() == Square::E4)
        );
    }

    #[test]
    fn test_multiple_pawns() {
        // 两个白兵
        let position = Position::from_fen("8/8/8/8/8/P6P/8/8 w - - 0 1").unwrap();
        let mut moves = ArrayVec::<Move, 256>::new();

        generate(&position, Color::White, &mut moves);

        // a3:a4
        // h3:h4
        assert_eq!(moves.len(), 2);
    }
}
