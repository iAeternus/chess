use std::fmt::Display;

use crate::{AlphaBetaEngine, ChessEngine, MiniMaxEngine, RandomEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineKind {
    Random,
    MiniMax3,
    MiniMax4,
    MiniMax5,
    AlphaBeta3,
    AlphaBeta4,
    AlphaBeta5,
}

impl EngineKind {
    pub const ALL: &[EngineKind] = &[
        EngineKind::Random,
        EngineKind::MiniMax3,
        EngineKind::MiniMax4,
        EngineKind::MiniMax5,
        EngineKind::AlphaBeta3,
        EngineKind::AlphaBeta4,
        EngineKind::AlphaBeta5,
    ];

    pub fn short_name(self) -> String {
        let name = match self {
            EngineKind::Random => "Random",
            EngineKind::MiniMax3 | EngineKind::MiniMax4 | EngineKind::MiniMax5 => "MiniMax",
            EngineKind::AlphaBeta3 | EngineKind::AlphaBeta4 | EngineKind::AlphaBeta5 => "AlphaBeta",
        };
        name.to_string()
    }

    pub fn create(self) -> Box<dyn ChessEngine> {
        match self {
            EngineKind::Random => Box::new(RandomEngine::default()),
            EngineKind::MiniMax3 => Box::new(MiniMaxEngine::new(3)),
            EngineKind::MiniMax4 => Box::new(MiniMaxEngine::new(4)),
            EngineKind::MiniMax5 => Box::new(MiniMaxEngine::new(5)),
            EngineKind::AlphaBeta3 => Box::new(AlphaBetaEngine::new(3)),
            EngineKind::AlphaBeta4 => Box::new(AlphaBetaEngine::new(4)),
            EngineKind::AlphaBeta5 => Box::new(AlphaBetaEngine::new(5)),
        }
    }

    pub fn depth(self) -> Option<u32> {
        match self {
            EngineKind::Random => None,
            EngineKind::MiniMax3 => Some(3),
            EngineKind::MiniMax4 => Some(4),
            EngineKind::MiniMax5 => Some(5),
            EngineKind::AlphaBeta3 => Some(3),
            EngineKind::AlphaBeta4 => Some(4),
            EngineKind::AlphaBeta5 => Some(5),
        }
    }
}

impl Display for EngineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            EngineKind::Random => "Random",
            EngineKind::MiniMax3 => "MiniMax (depth 3)",
            EngineKind::MiniMax4 => "MiniMax (depth 4)",
            EngineKind::MiniMax5 => "MiniMax (depth 5)",
            EngineKind::AlphaBeta3 => "AlphaBeta (depth 3)",
            EngineKind::AlphaBeta4 => "AlphaBeta (depth 4)",
            EngineKind::AlphaBeta5 => "AlphaBeta (depth 5)",
        };
        write!(f, "{}", content)
    }
}
