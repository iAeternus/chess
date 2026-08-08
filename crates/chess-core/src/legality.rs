use crate::{
    Color, Move, MoveFlag, PieceKind, Position, Square,
    attack::is_square_attacked,
    makemove::{make_move, unmake_move},
};

/// 判断一个走法是否合法
/// 条件：
/// 1. move本身是伪合法
/// 2. 移动后己方King不能被攻击
pub fn is_legal(position: &mut Position, mv: Move) -> bool {
    if let Some(piece) = position.piece_at(mv.to()) {
        if piece.kind == PieceKind::King {
            return false;
        }
    }

    if !special_move_legal(position, mv) {
        return false;
    }

    let side = position.side_to_move();

    // 执行走法，这里会修改 position，后面须 unmake
    let undo = make_move(position, mv);

    // side_to_move 已经切换，检查原来的 side 的 King
    let king_bb = position.board().piece_kind(side, PieceKind::King);
    let king_square = king_bb.lsb().expect("position without king");

    // 判断王是否被攻击，攻击方是当前行动方(对手)
    let legal = !is_square_attacked(position.board(), king_square, side.flip());

    // 恢复原局面
    unmake_move(position, undo);

    legal
}

fn special_move_legal(position: &Position, mv: Move) -> bool {
    match mv.flag() {
        MoveFlag::KingCastle => check_castle(position, true, mv),
        MoveFlag::QueenCastle => check_castle(position, false, mv),
        _ => true,
    }
}

fn check_castle(position: &Position, king_side: bool, mv: Move) -> bool {
    let color = position.side_to_move();
    let (king_from, pass, king_to) = match (color, king_side) {
        (Color::White, true) => (Square::E1, Square::F1, Square::G1),
        (Color::White, false) => (Square::E1, Square::D1, Square::C1),
        (Color::Black, true) => (Square::E8, Square::F8, Square::G8),
        (Color::Black, false) => (Square::E8, Square::D8, Square::C8),
    };

    if mv.from() != king_from || mv.to() != king_to {
        return false;
    }

    let enemy = color.flip();

    // 王当前位置不能被攻击
    if is_square_attacked(position.board(), king_from, enemy) {
        return false;
    }

    // 中间经过格不能被攻击
    if is_square_attacked(position.board(), pass, enemy) {
        return false;
    }

    // 目标格不能被攻击
    if is_square_attacked(position.board(), king_to, enemy) {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Move, MoveFlag, Position, Square};

    #[test]
    fn test_legal_normal_move() {
        // 初始局面 e2-e4
        // 白兵双步推进: e2 -> e4
        // 不暴露白王，合法
        let mut position = Position::startpos();
        let mv = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        assert!(is_legal(&mut position, mv));
    }

    #[test]
    fn test_illegal_expose_king() {
        // 黑车 e8 白车 e2 白王 e1
        // 白车离开 e file后:
        // 黑车直接攻击白王，非法
        let mut position = Position::from_fen("4r3/8/8/8/8/8/4R3/4K3 w - - 0 1").unwrap();
        let mv = Move::new(Square::E2, Square::A2, MoveFlag::Quiet);
        assert!(!is_legal(&mut position, mv));
    }

    #[test]
    fn test_capture_attacker() {
        // 黑车 e8 白后 e2 白王 e1
        // 白后:  e2 x e8
        // 吃掉攻击白王的黑车，合法
        let mut position = Position::from_fen("4r3/8/8/8/8/8/4Q3/4K3 w - - 0 1").unwrap();
        let mv = Move::new(Square::E2, Square::E8, MoveFlag::Capture);
        assert!(is_legal(&mut position, mv));
    }

    #[test]
    fn test_king_move_into_attack() {
        // 黑车 e8 白王 e1
        // 白王: e1 -> e2
        // e2仍处于黑车攻击范围，王不能移动到被攻击格
        let mut position = Position::from_fen("4r3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mv = Move::new(Square::E1, Square::E2, MoveFlag::Quiet);
        assert!(!is_legal(&mut position, mv));
    }

    #[test]
    fn test_position_restore_after_check() {
        // 测试 make_move / unmake_move 对称性
        // 调用 is_legal 后:
        // 1. 棋盘恢复
        // 2. 行棋方恢复
        // 3. Zobrist恢复
        let mut position = Position::startpos();
        let before_fen = position.to_fen();
        let before_hash = position.zobrist_key();
        let mv = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        let _ = is_legal(&mut position, mv);

        assert_eq!(position.to_fen(), before_fen);
        assert_eq!(position.zobrist_key(), before_hash);
    }

    #[test]
    fn test_castle_through_attack() {
        // 白王车易位: e1 -> g1
        // 黑车 f8
        // f1 被攻击
        // 王不能经过被攻击格
        let mut position = Position::from_fen("4k3/5r2/8/8/8/8/8/R3K2R w K - 0 1").unwrap();
        let mv = Move::new(Square::E1, Square::G1, MoveFlag::KingCastle);
        assert!(!is_legal(&mut position, mv));
    }

    #[test]
    fn test_illegal_move_does_not_change_position() {
        // 黑车 e8 白车 e2 白王 e1
        // 白车移动导致将军，检查非法走法不会修改局面
        let mut position = Position::from_fen("4r3/8/8/8/8/8/4R3/4K3 w - - 0 1").unwrap();
        let before = position.to_fen();
        let mv = Move::new(Square::E2, Square::A2, MoveFlag::Quiet);

        assert!(!is_legal(&mut position, mv));
        assert_eq!(position.to_fen(), before);
    }

    #[test]
    fn test_castle_when_in_check() {
        let mut position = Position::from_fen("4r1k1/8/8/8/8/8/8/4K2R w K - 0 1").unwrap();
        let mv = Move::new(Square::E1, Square::G1, MoveFlag::KingCastle);
        assert!(!is_legal(&mut position, mv));
    }
}
