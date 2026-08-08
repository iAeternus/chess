use chess_core::{Move, Position, generate_legal};
use rand::seq::IndexedRandom;

use crate::ChessEngine;

/// 随机走法引擎
pub struct RandomEngine {
    name: String,
    time_limit: u64,
    depth_limit: u32,
}

impl RandomEngine {
    pub fn new() -> Self {
        Self {
            name: "Random Engine".to_string(),
            time_limit: 0,
            depth_limit: 0,
        }
    }
}

impl Default for RandomEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ChessEngine for RandomEngine {
    fn search(&mut self, position: &Position) -> Option<Move> {
        let mut position = position.clone();
        let moves = generate_legal(&mut position);
        moves.choose(&mut rand::rng()).copied()
    }

    fn name(&self) -> &str {
        &self.name
    }
}
