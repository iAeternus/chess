use chess_ai::{EngineActor, EngineCommand, EngineKind, EngineResponse};
use std::error::Error;
use tokio::sync::mpsc;

use crate::gui::view::ViewEgui;

pub enum AppRequest {
    Engine(EngineCommand),
}

pub enum AppResponse {
    Engine(EngineResponse),
}

pub struct App;

impl App {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // channels
        let (engine_response_tx, engine_response_rx) = mpsc::channel(1024);
        let (engine_command_tx, engine_command_rx) = mpsc::channel(8);
        let (view_response_tx, view_response_rx) = flume::bounded(1024);
        let (view_command_tx, view_command_rx) = flume::bounded(1024);

        // actors
        let engine_actor = EngineActor::new(EngineKind::Random);
        let view = ViewEgui::new(view_response_rx, view_command_tx);

        // thread + tokio runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        runtime.spawn(async move {
            engine_actor
                .run(engine_command_rx, engine_response_tx)
                .await;
        });

        runtime.spawn(async move {
            Self::async_event_loop(
                view_command_rx,
                view_response_tx,
                engine_command_tx,
                engine_response_rx,
            )
            .await;
        });

        ViewEgui::run(view)?;

        println!("exiting App::run()");
        Ok(())
    }

    /// 事件循环，桥接 GUI(flume) 与引擎 Actor(tokio mpsc)
    async fn async_event_loop(
        view_command_rx: flume::Receiver<AppRequest>,
        view_response_tx: flume::Sender<AppResponse>,
        engine_command_tx: mpsc::Sender<EngineCommand>,
        mut engine_response_rx: mpsc::Receiver<EngineResponse>,
    ) {
        loop {
            tokio::select! {
                view_cmd_opt = view_command_rx.recv_async() => {
                    let Ok(command) = view_cmd_opt else { break; };
                    match command {
                        AppRequest::Engine(command) => {
                            // 转发引擎命令
                            if engine_command_tx.send(command).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                engine_res_opt = engine_response_rx.recv() => {
                    let Some(response) = engine_res_opt else { break; };
                    if view_response_tx.send(AppResponse::Engine(response)).is_err() {
                        break;
                    }
                }
            }
        }
    }
}
