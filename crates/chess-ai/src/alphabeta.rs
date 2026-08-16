use chess_core::{Move, Position};

use crate::ChessEngine;

pub struct AlphaBetaEngine {
    depth: i32,
}

impl AlphaBetaEngine {
    pub fn new(depth: i32) -> Self {
        Self { depth }
    }
}

impl ChessEngine for AlphaBetaEngine {
    fn name(&self) -> &str {
        "AlphaBeta Engine"
    }

    fn search(&mut self, position: &Position) -> Option<Move> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphabeta_engine_name() {
        let engine = AlphaBetaEngine::new(3);
        assert_eq!(engine.name(), "AlphaBeta Engine");
    }
}
