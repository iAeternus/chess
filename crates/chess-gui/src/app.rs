//! ChessApp — 应用主结构体。
//!
//! 负责协调各个子系统：GameController、BoardRenderer、面板、纹理、交互。

use chess_ai::{ChessEngine, MiniMaxEngine, RandomEngine};
use chess_core::{Color, Piece, Square};
use egui::{Align2, Pos2};

use crate::board::chess_board::{BoardEvent, ChessBoard};
use crate::board::renderer::BoardRenderer;
use crate::board::state::BoardState;
use crate::constants::PROMOTION_PIECES;
use crate::game::controller::{GameController, GameMode};
use crate::panel::engine_info::{EngineInfo, EngineInfoPanel};
use crate::panel::move_list::{MoveListAction, MoveListPanel};
use crate::panel::toolbar::{Toolbar, ToolbarAction};
use crate::piece::texture::PieceTextureManager;
use crate::theme::AppTheme;

pub struct ChessApp {
    controller: GameController,
    chess_board: ChessBoard,
    piece_textures: PieceTextureManager,
    theme: AppTheme,

    // 面板
    move_list_panel: MoveListPanel,
    engine_info_panel: EngineInfoPanel,
    toolbar: Toolbar,

    // 状态
    status_message: String,

    // 棋盘翻转（黑方视角）
    flipped: bool,

    // 拖拽：(棋子, 来源格, painter-local 鼠标位置)
    drag: Option<(Piece, Square, Pos2)>,

    // 升变待选
    pending_promotion: Option<(Square, Square)>,

    // HumanVsAI: 引擎选择
    selected_engine_index: usize,
}

impl ChessApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = AppTheme::default();
        theme.apply_egui_theme(&cc.egui_ctx);

        let piece_textures = PieceTextureManager::new(&cc.egui_ctx, 128);
        let controller = GameController::new(); // 默认 HumanVsHuman

        let colors = theme.colors();
        Self {
            controller,
            chess_board: ChessBoard::new(BoardRenderer::new(colors)),
            piece_textures,
            theme,
            move_list_panel: MoveListPanel::new(),
            engine_info_panel: EngineInfoPanel::new(),
            toolbar: Toolbar::new(),
            status_message: String::from("White to move"),
            flipped: false,
            drag: None,
            pending_promotion: None,
            selected_engine_index: 0,
        }
    }

    /// 状态同步
    fn update_status(&mut self) {
        let at_end = self.controller.current_ply() >= self.controller.total_moves()
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

    /// 构建当前帧的 BoardState
    fn build_board_state(&self) -> BoardState {
        let king_in_check = if self.controller.is_check() {
            Some(
                self.controller
                    .current_position()
                    .board()
                    .king_square(self.controller.side_to_move()),
            )
        } else {
            None
        };

        BoardState {
            position: self.controller.current_position().clone(),
            selected_square: self.controller.selected_square(),
            legal_moves: self.controller.legal_moves_for_selected(),
            last_move: self.controller.last_move(),
            king_in_check,
            drag: self.drag,
            arrows: Vec::new(), // TODO: 分析模式箭头
        }
    }

    /// 键盘处理
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.controller.go_back();
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                self.controller.go_forward();
            }
            if i.key_pressed(egui::Key::Home) {
                self.controller.go_to_start();
            }
            if i.key_pressed(egui::Key::End) {
                self.controller.go_to_end();
            }
            if i.key_pressed(egui::Key::R) {
                self.flipped = !self.flipped;
            }
            if i.key_pressed(egui::Key::N) && self.controller.mode() != GameMode::Replay {
                self.controller.new_game();
            }
        });
    }

    /// 菜单栏
    fn show_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File
                ui.menu_button("File", |ui| {
                    let is_replay = self.controller.mode() == GameMode::Replay;
                    if ui
                        .add_enabled(!is_replay, egui::Button::new("New (N)"))
                        .clicked()
                    {
                        self.controller.new_game();
                        self.drag = None;
                        self.pending_promotion = None;
                        ui.close_menu();
                    }
                    if ui.button("Open PGN...").clicked() {
                        self.open_pgn();
                        ui.close_menu();
                    }
                    if ui.button("Save PGN...").clicked() {
                        self.save_pgn();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // View
                ui.menu_button("View", |ui| {
                    if ui.button("Flip Board (R)").clicked() {
                        self.flipped = !self.flipped;
                        ui.close_menu();
                    }
                    ui.menu_button("Theme", |ui| {
                        if ui
                            .radio_value(&mut self.theme, AppTheme::Dark, "Dark")
                            .clicked()
                        {
                            self.theme.apply_egui_theme(ctx);
                            self.chess_board.set_colors(self.theme.colors());
                        }
                        if ui
                            .radio_value(&mut self.theme, AppTheme::Light, "Light")
                            .clicked()
                        {
                            self.theme.apply_egui_theme(ctx);
                            self.chess_board.set_colors(self.theme.colors());
                        }
                    });
                });

                // Mode
                ui.menu_button("Mode", |ui| {
                    if ui.button("Human vs Human").clicked() {
                        self.controller.set_mode(GameMode::HumanVsHuman, None);
                        self.drag = None;
                        self.pending_promotion = None;
                        ui.close_menu();
                    }
                    if ui.button("Human vs AI").clicked() {
                        let engine = self.create_engine();
                        self.controller.set_mode(GameMode::HumanVsAI, Some(engine));
                        self.drag = None;
                        self.pending_promotion = None;
                        ui.close_menu();
                    }
                    if ui.button("Analysis").clicked() {
                        self.controller.set_mode(GameMode::Analysis, None);
                        self.drag = None;
                        self.pending_promotion = None;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    /// 侧边面板
    fn show_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("side_panel")
            .resizable(false)
            .default_width(340.0)
            .min_width(340.0)
            .show(ctx, |ui| {
                // 侧边栏字体（放大以提升可读性）
                let mut style = (*ui.ctx().style()).clone();
                style.text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::new(15.0, egui::FontFamily::Proportional),
                );
                style.text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::new(15.0, egui::FontFamily::Proportional),
                );
                style.text_styles.insert(
                    egui::TextStyle::Heading,
                    egui::FontId::new(18.0, egui::FontFamily::Proportional),
                );
                ui.ctx().set_style(style);

                // 引擎信息
                let engine_info = EngineInfo {
                    name: self.controller.engine_name().map(|s| s.to_string()),
                    ..Default::default()
                };
                self.engine_info_panel.show(ui, &engine_info);
                ui.separator();

                // 模式与位置
                let mode_text = match self.controller.mode() {
                    GameMode::HumanVsHuman => "Mode: Human vs Human",
                    GameMode::HumanVsAI => "Mode: Human vs AI",
                    GameMode::Analysis => "Mode: Analysis",
                    GameMode::Replay => "Mode: Replay",
                };
                ui.label(mode_text);
                ui.label(format!(
                    "Ply: {}/{}",
                    self.controller.current_ply(),
                    self.controller.total_moves()
                ));

                // HumanVsAI 模式下的引擎/执棋方选择
                if self.controller.mode() == GameMode::HumanVsAI {
                    ui.separator();

                    // 引擎选择
                    ui.horizontal(|ui| {
                        ui.label("Engine:");
                        let engine_options = [
                            "Random",
                            "Minimax (depth 3)",
                            "Minimax (depth 4)",
                            "Minimax (depth 5)",
                        ];
                        let mut changed = false;
                        egui::ComboBox::from_id_salt("engine_select")
                            .selected_text(engine_options[self.selected_engine_index])
                            .show_ui(ui, |ui| {
                                for (i, label) in engine_options.iter().enumerate() {
                                    if ui
                                        .selectable_value(
                                            &mut self.selected_engine_index,
                                            i,
                                            *label,
                                        )
                                        .clicked()
                                    {
                                        changed = true;
                                    }
                                }
                            });
                        if changed {
                            let engine = self.create_engine();
                            self.controller.set_engine(engine);
                        }
                    });

                    // 执棋方选择
                    ui.horizontal(|ui| {
                        ui.label("Play as:");
                        let player_color = self.controller.player_color();
                        let mut selected = player_color == Color::White;
                        if ui.selectable_label(selected, "White").clicked() && !selected {
                            self.controller.set_player_color(Color::White);
                        }
                        selected = player_color == Color::Black;
                        if ui.selectable_label(selected, "Black").clicked() && !selected {
                            self.controller.set_player_color(Color::Black);
                        }
                    });
                }

                ui.separator();

                // 工具栏
                let actions = self
                    .toolbar
                    .show(ui, self.controller.mode() == GameMode::Replay);
                for action in actions {
                    match action {
                        ToolbarAction::FlipBoard => self.flipped = !self.flipped,
                        ToolbarAction::NewGame => self.controller.new_game(),
                        ToolbarAction::OpenPgn => self.open_pgn(),
                        ToolbarAction::SavePgn => self.save_pgn(),
                    }
                }
                ui.separator();

                // 走法列表（含导航按钮）
                let moves = self.controller.move_history().to_vec();
                let san_list = self.controller.san_list().to_vec();
                let current_ply = self.controller.current_ply();
                let can_back = self.controller.can_go_back();
                let can_forward = self.controller.can_go_forward();
                self.move_list_panel.show(
                    ui,
                    &moves,
                    &san_list,
                    current_ply,
                    ui.available_height(),
                    can_back,
                    can_forward,
                    |action| match action {
                        MoveListAction::JumpToPly(ply) => self.controller.go_to_ply(ply),
                        MoveListAction::GoToStart => self.controller.go_to_start(),
                        MoveListAction::GoBack => self.controller.go_back(),
                        MoveListAction::GoForward => self.controller.go_forward(),
                        MoveListAction::GoToEnd => self.controller.go_to_end(),
                    },
                );
            });
    }

    /// 棋盘区域
    fn show_board(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mode = self.controller.mode();
            let board_state = self.build_board_state();

            self.chess_board.set_flipped(self.flipped);
            let response = self.chess_board.show(
                ui,
                &board_state,
                &self.piece_textures,
                &mut self.controller,
                &mut self.drag,
                ctx,
                mode,
            );

            for event in response.events {
                match event {
                    BoardEvent::MoveMade(_) => {}
                    BoardEvent::PromotionNeeded { from, to } => {
                        self.pending_promotion = Some((from, to));
                    }
                }
            }
        });
    }

    /// 升变弹窗
    fn show_promotion_dialog(&mut self, ctx: &egui::Context) {
        let (from, to) = match self.pending_promotion {
            Some(p) => p,
            None => return,
        };

        egui::Window::new("##promotion")
            .title_bar(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([220.0, 70.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (promo, label) in PROMOTION_PIECES {
                        if ui.button(*label).clicked() {
                            self.controller.complete_promotion(from, to, *promo);
                            self.pending_promotion = None;
                        }
                    }
                });
            });
    }

    /// PGN 导入
    fn open_pgn(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PGN Files", &["pgn"])
            .add_filter("All Files", &["*"])
            .pick_file()
            && let Ok(content) = std::fs::read_to_string(&path)
        {
            match GameController::from_pgn(&content) {
                Ok(controller) => {
                    self.controller = controller;
                    self.flipped = false;
                    self.drag = None;
                    self.pending_promotion = None;
                    self.status_message = format!(
                        "Loaded: {}",
                        path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default()
                    );
                }
                Err(e) => {
                    self.status_message = format!("Failed to load PGN: {e}");
                }
            }
        }
    }

    /// PGN 导出
    fn save_pgn(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PGN Files", &["pgn"])
            .add_filter("All Files", &["*"])
            .save_file()
        {
            let pgn = self.controller.export_pgn();
            if let Err(e) = std::fs::write(&path, pgn) {
                self.status_message = format!("Failed to save: {e}");
            } else {
                self.status_message = format!(
                    "Saved: {}",
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default()
                );
            }
        }
    }

    /// 根据当前选中的引擎索引创建对应的引擎实例
    fn create_engine(&self) -> Box<dyn ChessEngine> {
        match self.selected_engine_index {
            0 => Box::new(RandomEngine::default()),
            1 => Box::new(MiniMaxEngine::new(3)),
            2 => Box::new(MiniMaxEngine::new(4)),
            _ => Box::new(MiniMaxEngine::new(5)),
        }
    }

    fn handle_engine(&mut self, ctx: &egui::Context) {
        if self.controller.is_engine_turn() && self.controller.request_engine_move().is_some() {
            ctx.request_repaint(); // 引擎走了一步，请求重绘
        }
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. 键盘
        self.handle_keyboard(ctx);

        // 2. 菜单栏
        self.show_menu(ctx);

        // 3. 侧边面板（引擎、工具栏、走法列表）
        self.show_side_panel(ctx);

        // 4. 棋盘渲染 + 交互
        self.show_board(ctx);

        // 5. 升变弹窗（如果待选）
        self.show_promotion_dialog(ctx);

        // 6. 状态栏
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.update_status();
            ui.label(self.status_message.clone());
        });

        // 7. 引擎自动走棋
        self.handle_engine(ctx);
    }
}
