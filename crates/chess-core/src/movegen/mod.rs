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

/// 生成所有合法走法
/// 流程：pseudo legal -> make_move -> 检查自己的King
pub fn generate_legal(position: &mut Position) -> ArrayVec<Move, 256> {
    // SAFETY: 一个局面最多约218步 < 256
    generate_pseudo_legal(position)
        .into_iter()
        .filter(|mv| legality::is_legal(position, *mv))
        .collect()
}
