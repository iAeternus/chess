use std::cmp;

use chess_core::{Color, Move, Position, generate_legal, make_move, unmake_move};

use crate::{
    ChessEngine,
    evaluation::{evaluate, terminal_score, terminal_score_from_empty},
};

pub struct MiniMaxEngine {
    depth: i32,
}

impl Default for MiniMaxEngine {
    fn default() -> Self {
        Self::new(3)
    }
}

impl MiniMaxEngine {
    pub fn new(depth: i32) -> Self {
        Self { depth }
    }

    fn minimax(position: &mut Position, depth: i32, ply: i32) -> i32 {
        if depth == 0 {
            if let Some(score) = terminal_score(position, ply) {
                return score;
            }
            return evaluate(position);
        }

        let side = position.side_to_move();
        let moves = generate_legal(position);
        if moves.is_empty() {
            return terminal_score_from_empty(position, ply);
        }

        match side {
            Color::White => {
                let mut best = i32::MIN;
                for mv in moves {
                    let undo = make_move(position, mv);
                    let score = Self::minimax(position, depth - 1, ply + 1);
                    unmake_move(position, undo);
                    best = cmp::max(best, score);
                }
                best
            }
            Color::Black => {
                let mut best = i32::MAX;
                for mv in moves {
                    let undo = make_move(position, mv);
                    let score = Self::minimax(position, depth - 1, ply + 1);
                    unmake_move(position, undo);
                    best = cmp::min(best, score);
                }
                best
            }
        }
    }
}

impl ChessEngine for MiniMaxEngine {
    fn name(&self) -> &str {
        "MiniMax Engine"
    }

    fn search(&mut self, position: &Position) -> Option<Move> {
        let mut pos = position.clone();
        let moves = generate_legal(&mut pos);
        if moves.is_empty() {
            return None;
        }

        let mut best_move = None;
        match position.side_to_move() {
            Color::White => {
                let mut best_score = i32::MIN;
                for mv in moves {
                    let undo = make_move(&mut pos, mv);
                    let score = Self::minimax(&mut pos, self.depth - 1, 1);
                    unmake_move(&mut pos, undo);
                    if score > best_score {
                        best_score = score;
                        best_move = Some(mv);
                    }
                }
            }
            Color::Black => {
                let mut best_score = i32::MAX;
                for mv in moves {
                    let undo = make_move(&mut pos, mv);
                    let score = Self::minimax(&mut pos, self.depth - 1, 1);
                    unmake_move(&mut pos, undo);
                    if score < best_score {
                        best_score = score;
                        best_move = Some(mv);
                    }
                }
            }
        }

        best_move
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimax_engine_name() {
        let engine = MiniMaxEngine::new(3);
        assert_eq!(engine.name(), "MiniMax Engine");
    }
}
