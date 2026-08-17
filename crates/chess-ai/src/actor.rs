use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

use crate::{ChessEngine, EngineCommand, EngineKind, EngineResponse};

pub struct EngineActor {
    engine: Arc<Mutex<Box<dyn ChessEngine>>>,
}

impl EngineActor {
    pub fn new(kind: EngineKind) -> Self {
        Self {
            engine: Arc::new(Mutex::new(kind.create())),
        }
    }

    pub async fn run(
        self,
        command_rx: mpsc::Receiver<EngineCommand>,
        response_tx: mpsc::Sender<EngineResponse>,
    ) {
        Self::event_loop(self.engine, command_rx, response_tx).await;
    }

    async fn event_loop(
        engine: Arc<Mutex<Box<dyn ChessEngine>>>,
        mut command_rx: mpsc::Receiver<EngineCommand>,
        response_tx: mpsc::Sender<EngineResponse>,
    ) {
        while let Some(command) = command_rx.recv().await {
            match command {
                EngineCommand::Search(position) => {
                    let engine = engine.clone();
                    let response_tx = response_tx.clone();
                    tokio::spawn(async move {
                        let mv = tokio::task::spawn_blocking(move || {
                            engine.lock().unwrap().search(&position)
                        })
                        .await
                        .ok()
                        .flatten();
                        let _ = response_tx.send(EngineResponse::SearchComplete(mv)).await;
                    });
                }
                EngineCommand::ChangeEngine(new_kind) => {
                    *engine.lock().unwrap() = new_kind.create();
                    let _ = response_tx
                        .send(EngineResponse::EngineChanged(new_kind))
                        .await;
                }
                EngineCommand::Terminate => {
                    let _ = response_tx.send(EngineResponse::Terminated).await;
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::Position;
    use tokio::sync::mpsc;

    #[test]
    fn actor_handles_commands() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let (command_tx, command_rx) = mpsc::channel(8);
            let (response_tx, mut response_rx) = mpsc::channel(8);

            let actor = EngineActor::new(EngineKind::Random);
            tokio::spawn(async move {
                actor.run(command_rx, response_tx).await;
            });

            command_tx
                .send(EngineCommand::Search(Position::startpos()))
                .await
                .unwrap();
            match response_rx.recv().await {
                Some(EngineResponse::SearchComplete(Some(_))) => {}
                other => panic!("expected SearchComplete(Some), got {other:?}"),
            }

            command_tx
                .send(EngineCommand::ChangeEngine(EngineKind::MiniMax3))
                .await
                .unwrap();
            match response_rx.recv().await {
                Some(EngineResponse::EngineChanged(kind)) => {
                    assert_eq!(kind, EngineKind::MiniMax3)
                }
                other => panic!("expected EngineChanged, got {other:?}"),
            }

            command_tx.send(EngineCommand::Terminate).await.unwrap();
            match response_rx.recv().await {
                Some(EngineResponse::Terminated) => {}
                other => panic!("expected Terminated, got {other:?}"),
            }
        });
    }
}
