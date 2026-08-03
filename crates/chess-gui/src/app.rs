//! ChessApp — 应用主结构体。
//!
//! 负责协调各个子系统：GameController、BoardRenderer、面板、纹理、交互。

use chess_ai::RandomEngine;
use chess_core::{Piece, Promotion, Square};
use egui::{Align2, Pos2, Rect, Sense, Vec2};

use crate::board::renderer::BoardRenderer;
use crate::board::state::BoardState;
use crate::game::controller::{GameController, GameMode, SelectionResult};
use crate::panel::engine_info::{EngineInfo, EngineInfoPanel};
use crate::panel::move_list::{MoveListAction, MoveListPanel};
use crate::panel::toolbar::{Toolbar, ToolbarAction};
use crate::piece::texture::PieceTextureManager;
use crate::theme::AppTheme;

/// 升变棋子选项
const PROMOTION_PIECES: &[(Promotion, &str)] = &[
    (Promotion::Queen, "♛"),
    (Promotion::Rook, "♜"),
    (Promotion::Bishop, "♝"),
    (Promotion::Knight, "♞"),
];

pub struct ChessApp {
    controller: GameController,
    board_renderer: BoardRenderer,
    piece_textures: PieceTextureManager,
    theme: AppTheme,

    // 面板
    move_list_panel: MoveListPanel,
    engine_info_panel: EngineInfoPanel,
    toolbar: Toolbar,

    // 状态
    status_message: String,

    // 拖拽：(棋子, 来源格, painter-local 鼠标位置)
    drag: Option<(Piece, Square, Pos2)>,

    // 升变待选
    pending_promotion: Option<(Square, Square)>,

    // 引擎走棋触发
    engine_pending: bool,
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
            board_renderer: BoardRenderer::new(colors),
            piece_textures,
            theme,
            move_list_panel: MoveListPanel::new(),
            engine_info_panel: EngineInfoPanel::new(),
            toolbar: Toolbar::new(),
            status_message: String::from("White to move"),
            drag: None,
            pending_promotion: None,
            engine_pending: false,
        }
    }

    // 状态同步

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

    // 键盘处理

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
                self.board_renderer.flipped = !self.board_renderer.flipped;
            }
            if i.key_pressed(egui::Key::N) && self.controller.mode() != GameMode::Replay {
                self.controller.new_game();
            }
        });
    }

    // 菜单栏

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
                        self.board_renderer.flipped = !self.board_renderer.flipped;
                        ui.close_menu();
                    }
                    ui.menu_button("Theme", |ui| {
                        if ui
                            .radio_value(&mut self.theme, AppTheme::Dark, "Dark")
                            .clicked()
                        {
                            self.theme.apply_egui_theme(ctx);
                            self.board_renderer.set_colors(self.theme.colors());
                        }
                        if ui
                            .radio_value(&mut self.theme, AppTheme::Light, "Light")
                            .clicked()
                        {
                            self.theme.apply_egui_theme(ctx);
                            self.board_renderer.set_colors(self.theme.colors());
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
                        self.controller
                            .set_mode(GameMode::HumanVsAI, Some(Box::new(RandomEngine::default())));
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

    // 侧边面板

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
                ui.separator();

                // 工具栏
                let actions = self
                    .toolbar
                    .show(ui, self.controller.mode() == GameMode::Replay);
                for action in actions {
                    match action {
                        ToolbarAction::FlipBoard => {
                            self.board_renderer.flipped = !self.board_renderer.flipped
                        }
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

    // 棋盘区域

    fn show_board(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();

            // 取较小维度作为基准，不强制放大（防止窗口缩小时溢出）
            let max_side = available.x.min(available.y);
            const MIN_BOARD_SIDE: f32 = 400.0;
            let side = if max_side < MIN_BOARD_SIDE {
                max_side
            } else {
                max_side * self.board_renderer.board_scale()
            };

            // 棋盘在 available 区域内居中
            let board_pos = Pos2::new(
                ui.cursor().min.x + (available.x - side) / 2.0,
                ui.cursor().min.y + (available.y - side) / 2.0,
            );

            // 在父布局中占位
            ui.allocate_space(Vec2::new(available.x, available.y));

            // 在精确位置创建子 UI，避免 horizontal layout 的 clip_rect 偏移
            let board_alloc = Rect::from_min_size(board_pos, Vec2::new(side, side));
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(board_alloc), |ui| {
                let is_replay = self.controller.mode() == GameMode::Replay;
                let sense = if is_replay {
                    Sense::hover()
                } else {
                    Sense::click_and_drag()
                };

                let (response, painter) = ui.allocate_painter(Vec2::new(side, side), sense);

                // board_rect 来自 egui 实际分配，非手动计算
                let board_rect = response.rect;

                let board_state = self.build_board_state();

                // 渲染（使用已分配的 painter）
                self.board_renderer
                    .paint(&painter, board_rect, &board_state, &self.piece_textures);

                // 交互（使用同一个 response，无需额外 ui.interact）
                if !is_replay {
                    self.handle_board_interaction(response, board_rect, ctx);
                }
            });
        });
    }

    /// 处理棋盘上的点击/拖拽事件
    fn handle_board_interaction(
        &mut self,
        response: egui::Response,
        board_rect: Rect,
        ctx: &egui::Context,
    ) {
        // 拖拽开始
        if response.drag_started()
            && let Some(pos) = response.interact_pointer_pos()
        {
            // pos 是相对于 board_rect 的坐标
            let abs = Pos2::new(board_rect.min.x + pos.x, board_rect.min.y + pos.y);
            if let Some(sq) = self.board_renderer.pos_to_square(board_rect, abs) {
                let piece = self.controller.current_position().piece_at(sq);
                if let Some(p) = piece {
                    let side = self.controller.current_position().side_to_move();
                    let can_move = self.controller.mode() == GameMode::Analysis || p.color == side;

                    if can_move {
                        // 选中棋子（仅在棋子有合法走法时生效）
                        let result = self.controller.select_square(sq);
                        if matches!(result, SelectionResult::Selected { .. }) {
                            self.drag = Some((p, sq, pos));
                        }
                    }
                }
            }
        }

        // 拖拽移动
        if response.dragged()
            && let Some(pos) = response.interact_pointer_pos()
            && let Some((_piece, _from, ref mut drag_pos)) = self.drag
        {
            *drag_pos = pos;
            ctx.request_repaint();
        }

        // 拖拽释放
        if response.drag_stopped()
            && let Some((_piece, _from, pos)) = self.drag.take()
        {
            let abs = Pos2::new(board_rect.min.x + pos.x, board_rect.min.y + pos.y);
            if let Some(target) = self.board_renderer.pos_to_square(board_rect, abs) {
                self.execute_drag_drop(target);
            } else {
                self.controller.clear_selection();
            }
        }

        // 点击
        if response.clicked()
            && let Some(local) = response.interact_pointer_pos()
        {
            let abs = Pos2::new(board_rect.min.x + local.x, board_rect.min.y + local.y);
            if let Some(sq) = self.board_renderer.pos_to_square(board_rect, abs) {
                self.execute_click(sq);
            }
        }
    }

    /// 执行拖拽释放
    fn execute_drag_drop(&mut self, target: Square) {
        let legal_moves = self.controller.legal_moves_for_selected();
        let matching: Vec<_> = legal_moves
            .iter()
            .filter(|m| m.to() == target)
            .copied()
            .collect();

        if matching.is_empty() {
            self.controller.clear_selection();
            return;
        }

        // 检查是否需要升变选择
        let has_promotion = matching.iter().any(|m| m.is_promotion());
        if has_promotion && matching.len() > 1 {
            let from = matching[0].from();
            self.pending_promotion = Some((from, target));
            return;
        }

        // 直接执行
        self.controller.make_move(matching[0]);
        self.engine_pending = true;
    }

    /// 执行点击
    fn execute_click(&mut self, sq: Square) {
        let result = self.controller.select_square(sq);
        match result {
            SelectionResult::MoveMade { .. } => {
                self.engine_pending = true;
            }
            SelectionResult::NeedsPromotion { from, to } => {
                self.pending_promotion = Some((from, to));
            }
            _ => {}
        }
    }

    // 升变弹窗

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
                            self.engine_pending = true;
                        }
                    }
                });
            });
    }

    // PGN 导入/导出

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
                    self.board_renderer.flipped = false;
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

    // 引擎

    fn handle_engine(&mut self, ctx: &egui::Context) {
        if self.engine_pending || self.controller.is_engine_turn() {
            self.engine_pending = false;
            if self.controller.request_engine_move().is_some() {
                // 引擎走了一步，请求重绘
            }
        }
        if self.controller.is_engine_turn() {
            ctx.request_repaint();
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
