use chess_ai::{ChessEngine, EngineKind};
use chess_core::{Color, Piece, Square};
use egui::{Align2, Pos2, ViewportBuilder};
use std::sync::mpsc::{Receiver, Sender};

use crate::constants::PROMOTION_PIECES;
use crate::game::{GameController, GameMode};
use crate::gui::{
    board::{BoardArrow, BoardEvent, BoardRenderer, BoardState, ChessBoard},
    panel::{EngineInfo, EngineInfoPanel, MoveListAction, MoveListPanel, Toolbar, ToolbarAction},
    piece::texture::PieceTextureManager,
    theme::AppTheme,
};

pub struct ViewEgui {
    controller: GameController,
    chess_board: ChessBoard,
    piece_textures: Option<PieceTextureManager>,
    theme: AppTheme,

    /// 走法列表面板
    move_list_panel: MoveListPanel,
    /// 引擎信息面板
    engine_info_panel: EngineInfoPanel,
    /// 工具栏
    toolbar: Toolbar,

    /// 状态
    status_message: String,

    /// 拖拽：(棋子, 来源格, painter-local 鼠标位置)
    drag: Option<(Piece, Square, Pos2)>,

    /// 分析模式箭头
    arrows: Vec<BoardArrow>,
    /// 当前右键拖动预览
    arrow_preview: Option<BoardArrow>,

    /// 升变待选
    pending_promotion: Option<(Square, Square)>,

    /// HumanVsAI: 引擎选择
    selected_engine_kind: EngineKind,

    /// actor mailbox（占位：后续重构为真实消息类型）
    #[allow(dead_code)] // 占位：actor 重构后使用
    tx: Sender<()>,
    rx: Receiver<()>,
}

impl ViewEgui {
    pub fn new(tx: Sender<()>, rx: Receiver<()>) -> Self {
        let theme = AppTheme::default();
        let controller = GameController::new(); // 默认 HumanVsHuman

        let colors = theme.colors();
        Self {
            controller,
            chess_board: ChessBoard::new(BoardRenderer::new(colors)),
            piece_textures: None,
            theme,
            move_list_panel: MoveListPanel::new(),
            engine_info_panel: EngineInfoPanel::new(),
            toolbar: Toolbar::new(),
            status_message: String::from("White to move"),
            drag: None,
            arrows: Vec::new(),
            arrow_preview: None,
            pending_promotion: None,
            selected_engine_kind: EngineKind::Random,
            tx,
            rx,
        }
    }

    /// 在获得 egui Context 后初始化依赖上下文的资源
    fn init(&mut self, ctx: &egui::Context) {
        self.theme.apply_egui_theme(ctx);
        self.piece_textures = Some(PieceTextureManager::new(ctx, 128));
    }

    /// 以原生窗口方式启动事件循环
    pub fn run(mut view: Self) -> eframe::Result<()> {
        let options = eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_resizable(false)
                .with_inner_size([(1920.0 / 4.0) * 3.0, (1080.0 / 4.0) * 3.0])
                .with_active(false)
                .with_title("Chess — Professional Analysis Board"),
            ..Default::default()
        };

        eframe::run_native(
            "Chess",
            options,
            Box::new(move |cc| {
                view.init(&cc.egui_ctx);
                Ok(Box::new(view))
            }),
        )
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
            view_from: self.controller.view_from(),
            selected_square: self.controller.selected_square(),
            legal_moves: self.controller.legal_moves_for_selected(),
            last_move: self.controller.last_move(),
            king_in_check,
            drag: self.drag,
            arrows: self.arrows.clone(),
            arrow_preview: self.arrow_preview.clone(),
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
                self.controller.flip_view();
            }
            if i.key_pressed(egui::Key::N) && self.controller.mode() != GameMode::Replay {
                self.controller.new_game();
            }
            if i.key_pressed(egui::Key::Escape) {
                self.arrows.clear();
                self.arrow_preview = None;
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
                        self.controller.flip_view();
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
                let colors = self.theme.colors();
                let engine_info = EngineInfo {
                    // TODO: 未填充
                    name: Some(self.selected_engine_kind.short_name()),
                    depth: self.selected_engine_kind.depth(),
                    ..Default::default()
                };
                self.engine_info_panel.show(ui, &engine_info, &colors);
                ui.separator();

                // 模式与位置
                ui.label(self.controller.mode().to_string());
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
                        let mut changed = false;
                        egui::ComboBox::from_id_salt("engine_select")
                            .selected_text(self.selected_engine_kind.to_string())
                            .show_ui(ui, |ui| {
                                for &kind in EngineKind::ALL.iter() {
                                    if ui
                                        .selectable_value(
                                            &mut self.selected_engine_kind,
                                            kind,
                                            kind.to_string(),
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
                        ToolbarAction::FlipBoard => self.controller.flip_view(),
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
                    &colors,
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

    /// 棋盘区域 TODO: 第一次绘制的箭头没有拖动实时预览，保存第一个箭头后续绘制箭头才会有预览
    fn show_board(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mode = self.controller.mode();
            let board_state = self.build_board_state();

            let response = self.chess_board.show(
                ui,
                &board_state,
                self.piece_textures
                    .as_ref()
                    .expect("ViewEgui::init must be called before show_board"),
                &mut self.controller,
                &mut self.drag,
                ctx,
                mode,
            );

            let colors = self.theme.colors();

            for event in response.events {
                match event {
                    BoardEvent::MoveMade(_) => {}
                    BoardEvent::PromotionNeeded { from, to } => {
                        self.pending_promotion = Some((from, to));
                    }
                    BoardEvent::ArrowToggle { from, to } => {
                        // 松开鼠标后，预览消失
                        self.arrow_preview = None;
                        let index = self.arrows.iter().position(|a| {
                            // 正向相同，反向也认为相同
                            (a.from == from && a.to == to) || (a.from == to && a.to == from)
                        });

                        if let Some(index) = index {
                            // 已存在 -> 删除
                            self.arrows.remove(index);
                        } else {
                            // 不存在 -> 添加
                            self.arrows.push(BoardArrow {
                                from,
                                to,
                                color: colors.arrow_color,
                            });
                        }
                    }
                    BoardEvent::ArrowPreview { arrow } => {
                        self.arrow_preview = arrow.map(|mut arrow| {
                            arrow.color = colors.arrow_preview_color;
                            arrow
                        });
                    }
                    BoardEvent::ClearArrows => {
                        self.arrows.clear();
                        self.arrow_preview = None;
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
        self.selected_engine_kind.create()
    }

    fn handle_engine(&mut self, ctx: &egui::Context) {
        if self.controller.is_engine_turn() && self.controller.request_engine_move().is_some() {
            ctx.request_repaint(); // 引擎走了一步，请求重绘
        }
    }
}

impl eframe::App for ViewEgui {
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

        // 8. actor mailbox 轮询（占位）
        self.rx.try_recv().ok();
    }
}
