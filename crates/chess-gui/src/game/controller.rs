//! 对局控制器：封装 Game、历史导航、选中状态、AI 引擎。

use arrayvec::ArrayVec;
use chess_ai::ChessEngine;
use chess_core::{Game, Move, Position, Result};

/// 对局模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    HumanVsHuman,
    HumanVsAI,
    Replay,
}

/// 点击结果
pub enum SelectionResult {
    Selected { square: chess_core::Square },
    MoveMade { mv: Move },
    Cleared,
}

/// 对局控制器：封装 Game + 导航状态 + AI
pub struct GameController {
    game: Game,
    mode: GameMode,
    engine: Option<Box<dyn ChessEngine>>,
    view_index: usize,
    pub selected_square: Option<chess_core::Square>,
    pub legal_moves_for_selected: ArrayVec<Move, 256>,
    pub last_move: Option<Move>,
    /// 缓存的走法历史（仅 Move），避免每次查询时分配 Vec
    cached_moves: ArrayVec<Move, 512>,
}

impl GameController {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            mode: GameMode::HumanVsHuman,
            engine: None,
            view_index: 0,
            selected_square: None,
            legal_moves_for_selected: ArrayVec::new(),
            last_move: None,
            cached_moves: ArrayVec::new(),
        }
    }

    pub fn new_with_engine(mode: GameMode, engine: Box<dyn ChessEngine>) -> Self {
        Self {
            game: Game::new(),
            mode,
            engine: Some(engine),
            view_index: 0,
            selected_square: None,
            legal_moves_for_selected: ArrayVec::new(),
            last_move: None,
            cached_moves: ArrayVec::new(),
        }
    }

    pub fn from_pgn(pgn: &str) -> Result<Self> {
        let mut game = chess_core::from_pgn(pgn)?;
        let cached: ArrayVec<Move, 512> =
            game.history().iter().map(|(mv, _)| *mv).collect();
        // Undo 所有走法回到开局，从第一回合开始回放
        while game.history().len() > 0 {
            game.undo().ok();
        }
        Ok(Self {
            game,
            mode: GameMode::Replay,
            engine: None,
            view_index: 0,
            selected_square: None,
            legal_moves_for_selected: ArrayVec::new(),
            last_move: None,
            cached_moves: cached,
        })
    }

    pub fn current_position(&self) -> &Position {
        self.game.position()
    }

    /// 获取当前局面下的所有合法走法（透传 chess-core 的 ArrayVec）
    pub fn legal_moves(&mut self) -> ArrayVec<Move, 256> {
        self.game.legal_moves()
    }

    pub fn select_square(&mut self, sq: chess_core::Square) -> SelectionResult {
        let position = self.game.position().clone();
        let piece_at_sq = position.piece_at(sq);
        let side_to_move = position.side_to_move();

        if let Some(selected) = self.selected_square {
            if let Some(piece) = piece_at_sq {
                if piece.color == side_to_move && sq != selected {
                    self.set_selected(sq, &position);
                    return SelectionResult::Selected { square: sq };
                }
            }

            if let Some(mv) = self
                .legal_moves_for_selected
                .iter()
                .find(|mv| mv.to() == sq)
            {
                let mv = *mv;
                self.make_move(mv);
                return SelectionResult::MoveMade { mv };
            }

            self.clear_selection();
            SelectionResult::Cleared
        } else {
            if let Some(piece) = piece_at_sq {
                if piece.color == side_to_move {
                    self.set_selected(sq, &position);
                    return SelectionResult::Selected { square: sq };
                }
            }
            SelectionResult::Cleared
        }
    }

    pub fn make_move(&mut self, mv: Move) {
        self.ensure_at_latest();

        if self.game.play(mv).is_ok() {
            self.view_index += 1;
            self.last_move = Some(mv);
            self.clear_selection();
            self.cached_moves.push(mv);
        }
    }

    pub fn request_engine_move(&mut self) -> Option<Move> {
        let pos = self.game.position().clone();
        let mv = self.engine.as_mut()?.search(&pos)?;
        self.make_move(mv);
        Some(mv)
    }

    pub fn is_engine_turn(&self) -> bool {
        self.mode == GameMode::HumanVsAI
            && self.engine.is_some()
            && self.game.position().side_to_move() == chess_core::Color::Black
            && self.view_index >= self.game.history().len()
    }

    // ── 历史导航 ──

    pub fn can_go_back(&self) -> bool {
        self.view_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.view_index < self.cached_moves.len()
    }

    pub fn go_to_start(&mut self) {
        while self.view_index > 0 {
            self.game.undo().ok();
            self.view_index -= 1;
        }
        self.clear_selection();
    }

    pub fn go_back(&mut self) {
        if self.view_index > 0 {
            self.game.undo().ok();
            self.view_index -= 1;
            self.update_last_move();
            self.clear_selection();
        }
    }

    pub fn go_forward(&mut self) {
        if self.view_index < self.cached_moves.len() {
            let target = self.view_index + 1;
            self.replay_to(target);
        }
    }

    pub fn go_to_end(&mut self) {
        let target = self.cached_moves.len();
        self.replay_to(target);
    }

    pub fn view_index(&self) -> usize {
        self.view_index
    }

    pub fn total_moves(&self) -> usize {
        self.game.history().len()
    }

    // ── 状态查询 ──

    pub fn mode(&self) -> GameMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: GameMode, engine: Option<Box<dyn ChessEngine>>) {
        self.mode = mode;
        self.engine = engine;
        // 重置游戏状态
        self.game.reset();
        self.view_index = 0;
        self.clear_selection();
        self.last_move = None;
        self.cached_moves.clear();
    }

    pub fn is_game_over(&mut self) -> bool {
        self.game.is_game_over().unwrap_or(false)
    }

    pub fn game_result(&self) -> &str {
        self.game.result()
    }

    pub fn is_check(&self) -> bool {
        self.game.is_check().unwrap_or(false)
    }

    pub fn side_to_move(&self) -> chess_core::Color {
        self.game.position().side_to_move()
    }

    /// 获取缓存的走法历史（仅 Move slice）
    pub fn move_history(&self) -> &[Move] {
        &self.cached_moves
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        self.game.header(key)
    }

    pub fn new_game(&mut self) {
        self.game.reset();
        self.view_index = 0;
        self.clear_selection();
        self.last_move = None;
        self.cached_moves.clear();
    }

    // ── 私有辅助 ──

    fn set_selected(&mut self, sq: chess_core::Square, position: &Position) {
        self.selected_square = Some(sq);
        let mut pos = position.clone();
        let all_moves = chess_core::generate_legal(&mut pos);
        self.legal_moves_for_selected = all_moves
            .into_iter()
            .filter(|mv| mv.from() == sq)
            .collect();
    }

    fn clear_selection(&mut self) {
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
    }

    fn ensure_at_latest(&mut self) {
        let total = self.cached_moves.len();
        if self.view_index < total {
            self.replay_to(total);
        }
    }

    fn replay_to(&mut self, target: usize) {
        if target == self.view_index {
            return;
        }

        // 后撤
        while self.view_index > target {
            self.game.undo().ok();
            self.view_index -= 1;
        }
        // 前进：从 cached_moves 重放
        while self.view_index < target && self.view_index < self.cached_moves.len() {
            let mv = self.cached_moves[self.view_index];
            self.game.play(mv).ok();
            self.view_index += 1;
        }

        self.update_last_move();
    }

    fn update_last_move(&mut self) {
        let history = self.game.history();
        self.last_move = if self.view_index > 0 && self.view_index <= history.len() {
            Some(history[self.view_index - 1].0)
        } else {
            None
        };
    }
}

impl Default for GameController {
    fn default() -> Self {
        Self::new()
    }
}
