use chess_core::{Move, Position, legal_moves_of};
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
        let moves = legal_moves_of(position);
        moves.choose(&mut rand::rng()).copied()
    }
}
