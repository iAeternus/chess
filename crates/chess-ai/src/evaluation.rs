use chess_core::{Color, PieceKind, Position, generate_legal};

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

pub fn terminal_score(position: &mut Position, ply: i32) -> Option<i32> {
    let moves = generate_legal(position);
    if !moves.is_empty() {
        return None;
    }

    if position.is_check() {
        match position.side_to_move() {
            Color::White => Some(-100000 + ply),
            Color::Black => Some(100000 - ply),
        }
    } else {
        Some(0)
    }
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
        // 期望: 900+500+320+330+100+20000-900-20000 = 1250
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

    #[test]
    fn test_terminal_score_white_checkmated() {
        // 白方被将杀：白王 g1，黑车 e1，白兵 f2/g2/h2 堵住逃跑路线
        let mut pos = Position::from_fen("6k1/5ppp/8/8/8/8/5PPP/4r1K1 w - - 0 1").unwrap();
        let score = terminal_score(&mut pos, 1);
        assert_eq!(score, Some(-100000 + 1));
    }

    #[test]
    fn test_terminal_score_black_checkmated() {
        // 黑方被将杀：黑王 h8，白后 g7，白王 f6
        let mut pos = Position::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
        let score = terminal_score(&mut pos, 2);
        assert_eq!(score, Some(100000 - 2));
    }

    #[test]
    fn test_terminal_score_stalemate() {
        // 黑方被逼和：黑王 h8，白后 g6，白王 f7
        let mut pos = Position::from_fen("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1").unwrap();
        let score = terminal_score(&mut pos, 1);
        assert_eq!(score, Some(0));
    }

    #[test]
    fn test_terminal_score_non_terminal() {
        // 有合法走法的局面应返回 None
        let mut pos = Position::startpos();
        let score = terminal_score(&mut pos, 0);
        assert_eq!(score, None);
    }

    #[test]
    fn test_terminal_score_mate_ply_bonus() {
        // 验证 ply 加成：浅层将杀分数更接近 0，体现"更快将杀/更慢被将杀"偏好
        // 白方被将杀，ply=1 分数 < ply=5（更负 = 输得更快 = 应避免）
        let mut pos1 = Position::from_fen("6k1/5ppp/8/8/8/8/5PPP/4r1K1 w - - 0 1").unwrap();
        let score_ply1 = terminal_score(&mut pos1, 1).unwrap();
        let score_ply5 = terminal_score(&mut pos1, 5).unwrap();
        assert!(
            score_ply1 < score_ply5,
            "mate at ply=1 ({score_ply1}) should be worse (more negative) than ply=5 ({score_ply5})"
        );
    }
}
