use chess_core::{Color, PieceKind, Position};

/// 子力评估
/// 约定：正数代表白方优势，负数代表黑方优势
pub fn evaluate(position: &Position) -> i32 {
    let mut score = 0;
    for (_, piece) in position.board() {
        let value = match piece.kind {
            PieceKind::Pawn => 100,
            PieceKind::Knight => 320,
            PieceKind::Bishop => 330,
            PieceKind::Rook => 500,
            PieceKind::Queen => 900,
            PieceKind::King => 20000,
        };
        if piece.color == Color::White {
            score += value;
        } else {
            score -= value;
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_startpos() {
        // 双方子力完全相同
        let position = Position::startpos();
        let score = evaluate(&position);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_evaluate_white_advantage() {
        // 白方多一个后
        let position = Position::from_fen("7k/8/8/8/8/8/8/Q6K w - - 0 1").unwrap();
        let score = evaluate(&position);
        assert_eq!(score, 900);
    }

    #[test]
    fn test_evaluate_black_advantage() {
        // 黑方多一个车
        let position = Position::from_fen("7k/8/8/8/8/8/8/r6K b - - 0 1").unwrap();
        let score = evaluate(&position);
        assert_eq!(score, -500);
    }

    #[test]
    fn test_evaluate_piece_values() {
        // 白:
        // Queen 900
        // Rook 500
        // Knight 320
        // Bishop 330
        // Pawn 100
        //
        // King 20000
        //
        // 黑:
        // Queen 900
        //
        // 期望:
        // 900+500+320+330+100+20000-900-20000
        // =1250
        let position = Position::from_fen("3qk3/8/8/8/8/8/P1N1B3/R2QK3 w - - 0 1").unwrap();
        let score = evaluate(&position);
        assert_eq!(score, 1250);
    }

    #[test]
    fn test_evaluate_empty_board() {
        let position = Position::from_fen("8/8/8/8/8/8/8/8 w - - 0 1").unwrap();
        let score = evaluate(&position);
        assert_eq!(score, 0);
    }
}
