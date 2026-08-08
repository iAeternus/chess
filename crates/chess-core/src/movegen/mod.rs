mod bishop;
mod king;
mod knight;
mod pawn;
mod queen;
mod rook;

use arrayvec::ArrayVec;

use crate::{Move, Position, legality};

/// 生成所有伪合法走法
pub fn generate_pseudo_legal(position: &Position) -> ArrayVec<Move, 256> {
    let mut moves = ArrayVec::<Move, 256>::new();
    let color = position.side_to_move();

    pawn::generate(position, color, &mut moves);
    knight::generate(position, color, &mut moves);
    bishop::generate(position, color, &mut moves);
    rook::generate(position, color, &mut moves);
    queen::generate(position, color, &mut moves);
    king::generate(position, color, &mut moves);

    moves
}

/// 生成所有合法走法（`&mut Position` 版本，零 clone）
///
/// 内部通过 make/unmake 检查合法性，会临时修改局面但调用后恢复。
/// 适合已持有 `&mut Position` 的场景（如 minimax 搜索、perft）。
/// 流程：pseudo legal -> make_move -> 检查自己的King
pub fn generate_legal(position: &mut Position) -> ArrayVec<Move, 256> {
    // SAFETY: 一个局面最多约218步 < 256
    generate_pseudo_legal(position)
        .into_iter()
        .filter(|mv| legality::is_legal(position, *mv))
        .collect()
}

/// 生成所有合法走法（`&Position` 便捷版本，内部 clone 一次）
///
/// 适合只有共享引用（`&Position`）的场景，调用方无需手动 clone。
pub fn legal_moves_of(position: &Position) -> ArrayVec<Move, 256> {
    let mut pos = position.clone();
    generate_legal(&mut pos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Position, Square};

    #[test]
    fn test_generate_legal_no_capture_king() {
        let mut position = Position::from_fen("7k/6Q1/8/8/8/8/8/6K1 w - - 0 1").unwrap();
        let moves = generate_legal(&mut position);

        assert!(
            !moves
                .iter()
                .any(|m| m.from() == Square::G7 && m.to() == Square::H8)
        );
    }
}
