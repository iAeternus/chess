//! Chess GUI — lichess 风格专业国际象棋桌面软件。

mod board;
mod game;
mod panel;
mod piece;
mod theme;

use board::renderer::{BoardRenderer, DragState};
use game::controller::{GameController, GameMode, SelectionResult};
use panel::engine_info::EngineInfoPanel;
use panel::move_list::MoveListPanel;
use panel::toolbar::{Toolbar, ToolbarAction};
use piece::texture::PieceTextureManager;
use theme::AppTheme;

use chess_ai::RandomEngine;
use chess_core::Square;

struct ChessApp {
    controller: GameController,
    board_renderer: BoardRenderer,
    piece_textures: PieceTextureManager,
    theme: AppTheme,
    move_list_panel: MoveListPanel,
    engine_info_panel: EngineInfoPanel,
    toolbar: Toolbar,
    status_message: String,
    engine_pending: bool,
    /// 拖拽状态：(棋子信息, 当前鼠标在 board_rect 内的位置)
    drag: Option<(DragState, egui::Pos2)>,
}

impl ChessApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = AppTheme::default();
        theme.apply_egui_theme(&cc.egui_ctx);
        let piece_textures = PieceTextureManager::new(&cc.egui_ctx, 128);
        let engine = Box::new(RandomEngine::default());
        let controller = GameController::new_with_engine(GameMode::HumanVsAI, engine);
        let colors = theme.colors();
        Self {
            controller,
            board_renderer: BoardRenderer::new(colors),
            piece_textures,
            theme,
            move_list_panel: MoveListPanel::new(),
            engine_info_panel: EngineInfoPanel::new(),
            toolbar: Toolbar::new(),
            status_message: String::from("White to move"),
            engine_pending: false,
            drag: None,
        }
    }

    fn update_status(&mut self) {
        let at_end = self.controller.view_index() >= self.controller.total_moves()
            && self.controller.total_moves() > 0;
        let result = self.controller.game_result();
        if at_end && result != "*" {
            self.status_message = format!("Game over: {result}");
        } else if self.controller.is_check() {
            self.status_message = format!("Check! {:?} to move", self.controller.side_to_move());
        } else {
            self.status_message = format!("{:?} to move", self.controller.side_to_move());
        }
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Keyboard ──
        ctx.input(|i| {
            if i.key_pressed(egui::Key::ArrowLeft) { self.controller.go_back(); }
            if i.key_pressed(egui::Key::ArrowRight) { self.controller.go_forward(); }
            if i.key_pressed(egui::Key::Home) { self.controller.go_to_start(); }
            if i.key_pressed(egui::Key::End) { self.controller.go_to_end(); }
            if i.key_pressed(egui::Key::R) { self.board_renderer.flipped = !self.board_renderer.flipped; }
            if i.key_pressed(egui::Key::N) { self.controller.new_game(); }
        });

        // ── Menu ──
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New (N)").clicked() { self.controller.new_game(); ui.close_menu(); }
                    if ui.button("Open PGN...").clicked() { self.open_pgn(); ui.close_menu(); }
                    ui.separator();
                    if ui.button("Quit").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                });
                ui.menu_button("View", |ui| {
                    if ui.button(if self.board_renderer.flipped { "Flip Board (R)" } else { "Flip Board (R)" }).clicked() {
                        self.board_renderer.flipped = !self.board_renderer.flipped; ui.close_menu();
                    }
                    ui.menu_button("Theme", |ui| {
                        if ui.radio_value(&mut self.theme, AppTheme::Dark, "Dark").clicked() {
                            self.theme.apply_egui_theme(ctx);
                            self.board_renderer.set_colors(self.theme.colors());
                        }
                        if ui.radio_value(&mut self.theme, AppTheme::Light, "Light").clicked() {
                            self.theme.apply_egui_theme(ctx);
                            self.board_renderer.set_colors(self.theme.colors());
                        }
                    });
                    ui.menu_button("Game Mode", |ui| {
                        if ui.button("Human vs Human").clicked() {
                            self.controller.set_mode(GameMode::HumanVsHuman, None); ui.close_menu();
                        }
                        if ui.button("Human vs AI").clicked() {
                            self.controller.set_mode(GameMode::HumanVsAI, Some(Box::new(RandomEngine::default()))); ui.close_menu();
                        }
                    });
                });
            });
        });

        // ── Side panel ──
        egui::SidePanel::right("side_panel")
            .resizable(false)
            .default_width(260.0)
            .min_width(260.0)
            .show(ctx, |ui| {
                let engine_name = if self.controller.mode() == GameMode::HumanVsAI {
                    Some("Random Engine")
                } else { None };
                self.engine_info_panel.show(ui, engine_name);
                ui.separator();

                let mode_text = match self.controller.mode() {
                    GameMode::HumanVsHuman => "Mode: Human vs Human",
                    GameMode::HumanVsAI => "Mode: Human vs AI",
                    GameMode::Replay => "Mode: Replay",
                };
                ui.label(mode_text);
                ui.label(format!("Pos: {}/{}", self.controller.view_index(), self.controller.total_moves()));
                ui.separator();

                // 所有模式下按钮都可用
                let actions = self.toolbar.show(ui, self.controller.can_go_back(), self.controller.can_go_forward());
                for action in actions {
                    match action {
                        ToolbarAction::GoToStart => self.controller.go_to_start(),
                        ToolbarAction::GoBack => self.controller.go_back(),
                        ToolbarAction::GoForward => self.controller.go_forward(),
                        ToolbarAction::GoToEnd => self.controller.go_to_end(),
                        ToolbarAction::FlipBoard => self.board_renderer.flipped = !self.board_renderer.flipped,
                        ToolbarAction::NewGame => self.controller.new_game(),
                        ToolbarAction::OpenPgn => self.open_pgn(),
                    }
                }
                ui.separator();

                let moves = self.controller.move_history();
                self.move_list_panel.show_with_height(ui, moves, self.controller.view_index(), ui.available_height());
            });

        // ── Board ──
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::default()
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    let board_rect = BoardRenderer::board_rect(ui);

                    // Render
                    self.board_renderer.render(
                        ui, board_rect,
                        self.controller.current_position(),
                        &self.piece_textures,
                        self.controller.selected_square,
                        &self.controller.legal_moves_for_selected,
                        self.controller.last_move,
                        self.controller.is_check(),
                        self.drag.as_ref().map(|(ds, pos)| (DragState { piece: ds.piece, from: ds.from }, *pos)),
                    );

                    // Interaction: drag for play mode, nothing for replay
                    let is_replay = self.controller.mode() == GameMode::Replay;
                    let sense = if is_replay { egui::Sense::hover() } else { egui::Sense::drag() };
                    let response = ui.interact(board_rect, ui.next_auto_id(), sense);

                    if is_replay { return; }

                    // Drag started
                    if response.drag_started() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let abs = egui::Pos2::new(board_rect.min.x + pos.x, board_rect.min.y + pos.y);
                            if let Some(sq) = self.board_renderer.pos_to_square(board_rect, abs) {
                                let piece = self.controller.current_position().piece_at(sq);
                                if let Some(p) = piece {
                                    if p.color == self.controller.current_position().side_to_move() {
                                        // Select the piece first (shows legal moves)
                                        self.controller.select_square(sq);
                                        self.drag = Some((DragState { piece: p, from: sq }, pos));
                                    }
                                }
                            }
                        }
                    }

                    // Dragging
                    if response.dragged() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            if let Some((ref mut ds, ref mut drag_pos)) = self.drag {
                                *drag_pos = pos;
                            }
                        }
                        ctx.request_repaint();
                    }

                    // Drag released
                    if response.drag_released() {
                        if let Some((ds, pos)) = self.drag.take() {
                            let abs = egui::Pos2::new(board_rect.min.x + pos.x, board_rect.min.y + pos.y);
                            if let Some(target) = self.board_renderer.pos_to_square(board_rect, abs) {
                                // Try to find matching legal move
                                if let Some(mv) = self.controller.legal_moves_for_selected.iter().find(|m| m.to() == target) {
                                    self.controller.make_move(*mv);
                                    self.engine_pending = true;
                                    return;
                                }
                            }
                            // Invalid drop — clear selection
                            self.controller.clear_selection();
                        }
                    }

                    // Click (no drag)
                    if response.clicked() {
                        if let Some(local) = response.interact_pointer_pos() {
                            let abs = egui::Pos2::new(board_rect.min.x + local.x, board_rect.min.y + local.y);
                            if let Some(sq) = self.board_renderer.pos_to_square(board_rect, abs) {
                                let result = self.controller.select_square(sq);
                                if let SelectionResult::MoveMade { .. } = result {
                                    self.engine_pending = true;
                                }
                            }
                        }
                    }
                });
        });

        // ── Status bar ──
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.update_status();
            ui.label(self.status_message.clone());
        });

        // ── Engine auto-move ──
        if self.engine_pending || self.controller.is_engine_turn() {
            self.engine_pending = false;
            if self.controller.request_engine_move().is_some() {
                ctx.request_repaint();
            }
        }
        if self.controller.is_engine_turn() {
            ctx.request_repaint();
        }
    }
}

impl ChessApp {
    fn open_pgn(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PGN Files", &["pgn"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            if let Ok(content) = std::fs::read_to_string(&path) {
                match GameController::from_pgn(&content) {
                    Ok(controller) => {
                        self.controller = controller;
                        self.board_renderer.flipped = false;
                        self.status_message = format!("Loaded PGN: {}",
                            path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
                    }
                    Err(e) => { self.status_message = format!("Failed to load PGN: {e}"); }
                }
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Chess",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_min_inner_size([820.0, 650.0])
                .with_title("Chess — Professional Analysis Board"),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(ChessApp::new(cc)))),
    )
}
