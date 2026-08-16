use chess_core::{Move, Position, generate_legal2};
use rand::seq::IndexedRandom;

use crate::ChessEngine;

/// 随机走法引擎
#[derive(Default)]
pub struct RandomEngine;

impl ChessEngine for RandomEngine {
    fn name(&self) -> &str {
        "Random Engine"
    }

    fn search(&mut self, position: &Position) -> Option<Move> {
        let moves = generate_legal2(position);
        moves.choose(&mut rand::rng()).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_engine_name() {
        let engine = RandomEngine::default();
        assert_eq!(engine.name(), "Random Engine");
    }

    #[test]
    fn random_engine_checkmate_returns_none() {
        // 黑方被将杀
        let position = Position::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
        let mut engine = RandomEngine::default();
        assert!(
            engine.search(&position).is_none(),
            "random engine should return None on checkmate"
        );
    }

    #[test]
    fn random_engine_stalemate_returns_none() {
        // 黑方被逼和
        let position = Position::from_fen("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1").unwrap();
        let mut engine = RandomEngine::default();
        assert!(
            engine.search(&position).is_none(),
            "random engine should return None on stalemate"
        );
    }
}
