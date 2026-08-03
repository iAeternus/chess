use crate::{
    Move, Position,
    makemove::{make_move, unmake_move},
    movegen,
};

/// Perft(Position Evaluation Function Test)
/// 统计从当前局面开始 depth 层所有合法节点数量
pub fn perft(position: &mut Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = movegen::generate_legal(position);
    let mut nodes = 0u64;
    for mv in moves {
        let undo = make_move(position, mv);
        nodes += perft(position, depth - 1);
        unmake_move(position, undo);
    }

    nodes
}

/// 带走法分解的 perft
///
/// 用于调试：
/// e2e4: 600
/// d2d4: 560
pub fn divide(position: &mut Position, depth: u32) -> Vec<(Move, u64)> {
    let moves = movegen::generate_legal(position);
    let mut result = Vec::with_capacity(moves.len());

    for mv in moves {
        let undo = make_move(position, mv);
        let nodes = perft(position, depth - 1);
        unmake_move(position, undo);
        result.push((mv, nodes));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;

    #[test]
    fn perft_startpos() {
        let mut pos = Position::startpos();

        assert_eq!(perft(&mut pos, 1), 20);
        assert_eq!(perft(&mut pos, 2), 400);
        assert_eq!(perft(&mut pos, 3), 8902);
        assert_eq!(perft(&mut pos, 4), 197281);
    }

    #[test]
    fn perft_kiwipete() {
        let mut pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        assert_eq!(perft(&mut pos, 1), 48);
        assert_eq!(perft(&mut pos, 2), 2039);
        assert_eq!(perft(&mut pos, 3), 97862);
    }
}
