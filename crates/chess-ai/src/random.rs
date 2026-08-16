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
