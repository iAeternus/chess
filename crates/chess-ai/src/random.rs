use chess_core::{Move, Position, generate_legal};
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
        let mut position = position.clone();
        let moves = generate_legal(&mut position);
        moves.choose(&mut rand::rng()).copied()
    }
}
