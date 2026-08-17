use chess_core::{Move, Position};

use crate::EngineKind;

#[derive(Debug, Clone)]
pub enum EngineCommand {
    Search(Position),
    ChangeEngine(EngineKind),
    Terminate,
}

#[derive(Debug)]
pub enum EngineResponse {
    SearchComplete(Option<Move>),
    EngineChanged(EngineKind),
    Terminated,
}
