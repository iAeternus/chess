//! 对局控制器：面向 GUI 的薄封装层
//!
//! # 设计原则
//!
//! - 所有对局状态、历史、导航、SAN、PGN 均存储在 [`chess_core::Game`] 中，Game 是唯一权威数据源
//! - 本控制器仅负责 GUI 相关的临时状态：棋子选中、对局模式、AI 引擎调度、人类执棋颜色
//! - 对局操作（走棋、撤销、前进、后退、导入导出）全部委托给 `Game`，避免重复状态
//! - 历史导航由 `Game::go_to_ply` 内部基于起始局面重放实现
//!
//! # 用法
//!
//! ```ignore
//! let mut controller = GameController::new();
//! controller.select_square(Square::E2); // 选中白方 e2 兵
//! controller.select_square(Square::E4); // 执行 e2-e4
//! ```

use arrayvec::ArrayVec;
use chess_ai::ChessEngine;
use chess_core::{Color, Game, Move, Position, Promotion, Result, Square};

/// 对局模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// 双人对战
    HumanVsHuman,
    /// 人类执白 vs AI 执黑
    HumanVsAI,
    /// 分析模式：自由走棋，无胜负判定（类似 Lichess Analysis Board）
    Analysis,
    /// 棋谱回放模式
    Replay,
}

/// 点击/选择结果
#[derive(Debug)]
#[allow(dead_code)]
pub enum SelectionResult {
    /// 选中了一个棋子
    Selected { square: Square },
    /// 移动已完成
    MoveMade { mv: Move },
    /// 需要选择升变棋子（兵到达底线）
    NeedsPromotion { from: Square, to: Square },
    /// 选择已清除或无操作
    Cleared,
}

/// 对局控制器
///
/// 封装 `chess_core::Game`，提供：
/// - 走法执行与历史管理
/// - 历史导航（前进/后退/跳转）
/// - 棋子选中与合法走法提示
/// - AI 引擎集成
/// - PGN 导入/导出
pub struct GameController {
    game: Game,
    mode: GameMode,

    /// 当前选中的格子
    selected_square: Option<Square>,

    /// AI 引擎（仅在 HumanVsAI 模式下存在）
    engine: Option<Box<dyn ChessEngine>>,

    /// 人类执什么颜色（默认 White，即引擎执黑）
    player_color: Color,
}

impl GameController {
    // 构造

    /// 创建默认对局（标准初始局面，双人对战模式）
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            mode: GameMode::HumanVsHuman,
            selected_square: None,
            engine: None,
            player_color: Color::White,
        }
    }

    /// 创建带引擎的对局
    #[allow(dead_code)]
    pub fn new_with_engine(mode: GameMode, engine: Box<dyn ChessEngine>) -> Self {
        Self {
            game: Game::new(),
            mode,
            selected_square: None,
            engine: Some(engine),
            player_color: Color::White,
        }
    }

    /// 从 PGN 字符串加载对局（自动进入 Replay 模式）
    pub fn from_pgn(pgn: &str) -> Result<Self> {
        let game = Game::from_pgn(pgn)?;
        Ok(Self {
            game,
            mode: GameMode::Replay,
            selected_square: None,
            engine: None,
            player_color: Color::White,
        })
    }

    /// 获取当前局面的不可变引用
    pub fn current_position(&self) -> &Position {
        self.game.position()
    }

    /// 获取当前选中棋子可以走到的合法目标走法
    pub fn legal_moves_for_selected(&self) -> ArrayVec<Move, 256> {
        match self.selected_square {
            Some(sq) => self.game.legal_moves_from(sq),
            None => ArrayVec::new(),
        }
    }

    /// 最后一步走法（用于高亮显示）
    pub fn last_move(&self) -> Option<Move> {
        self.game.last_move()
    }

    /// 是否可以后退
    pub fn can_go_back(&self) -> bool {
        self.game.can_go_back()
    }

    /// 是否可以前进
    pub fn can_go_forward(&self) -> bool {
        self.game.can_go_forward()
    }

    /// 跳转到第一步
    pub fn go_to_start(&mut self) {
        self.game.go_to_start();
        self.clear_selection();
    }

    /// 后退一步
    pub fn go_back(&mut self) {
        self.game.go_back();
        self.clear_selection();
    }

    /// 前进一步
    pub fn go_forward(&mut self) {
        self.game.go_forward();
        self.clear_selection();
    }

    /// 跳转到最后一步
    pub fn go_to_end(&mut self) {
        self.game.go_to_end();
        self.clear_selection();
    }

    /// 跳转到指定 ply
    pub fn go_to_ply(&mut self, ply: usize) {
        self.game.go_to_ply(ply);
        self.clear_selection();
    }

    /// 当前 ply 编号（用于 UI 显示）
    pub fn current_ply(&self) -> usize {
        self.game.current_ply()
    }

    /// 总半移动数
    pub fn total_moves(&self) -> usize {
        self.game.total_moves()
    }

    /// 完整的走法历史（用于走法列表面板）
    pub fn move_history(&self) -> &[Move] {
        self.game.move_history()
    }

    /// SAN 格式走法列表（与 move_history 一一对应）
    pub fn san_list(&self) -> &[String] {
        self.game.san_list()
    }

    /// 获取指定 ply 之后的局面
    ///
    /// 注意：这会临时修改内部 game 状态，但在同一帧内恢复
    /// 调用者应一次获取所有需要的快照，避免重复导航
    pub fn position_at_ply(&self, ply: usize) -> Option<Position> {
        self.game.position_at_ply(ply)
    }

    /// 当前是否将军
    pub fn is_check(&self) -> bool {
        self.game.is_check()
    }

    /// 对局是否结束（分析模式下永不结束）
    pub fn is_game_over(&self) -> bool {
        if self.mode == GameMode::Analysis {
            return false;
        }
        self.game.is_game_over()
    }

    /// 对局结果字符串（"1-0", "0-1", "1/2-1/2", "*"）
    pub fn game_result(&self) -> &str {
        self.game.result()
    }

    /// 当前轮到哪一方走棋
    pub fn side_to_move(&self) -> Color {
        self.game.side_to_move()
    }

    /// 导出当前对局为 PGN 字符串
    pub fn export_pgn(&self) -> String {
        self.game.export_pgn()
    }

    /// PGN 头信息
    pub fn header(&self, key: &str) -> Option<&str> {
        self.game.header(key)
    }

    /// 所有 PGN headers（用于显示对局信息）
    pub fn headers(&self) -> &[(String, String)] {
        self.game.headers()
    }

    // 走法执行

    /// 点击格子：根据当前选中状态决定行为
    ///
    /// - 未选中 + 点击己方棋子 -> 选中
    /// - 已选中 + 点击己方其他棋子 -> 重新选中
    /// - 已选中 + 点击合法目标 -> 执行走法（可能触发升变选择）
    /// - 已选中 + 点击非法目标 -> 清除选中
    /// - 已选中 + 点击同一格子 -> 清除选中
    pub fn select_square(&mut self, sq: Square) -> SelectionResult {
        let position = self.game.position();
        let piece_at_sq = position.piece_at(sq);
        let side_to_move = position.side_to_move();
        let legal_moves = self.game.legal_moves();

        // 分析模式下可以移动任意一方的棋子；其他模式下只能移动当前方
        let can_select =
            self.mode == GameMode::Analysis || piece_at_sq.is_some_and(|p| p.color == side_to_move);

        if let Some(selected) = self.selected_square {
            // 已有选中棋子
            if let Some(_piece) = piece_at_sq
                && can_select
                && sq != selected
            {
                // 点击了另一个可选棋子 -> 重新选中
                self.set_selected(sq);
                return SelectionResult::Selected { square: sq };
            }

            // 检查是否点击了合法目标
            let matching: ArrayVec<Move, 256> = legal_moves
                .iter()
                .filter(|m| m.from() == selected && m.to() == sq)
                .copied()
                .collect();

            if matching.is_empty() {
                self.clear_selection();
                return SelectionResult::Cleared;
            }

            // 检查是否需要升变选择
            if matching.iter().any(|m| m.is_promotion()) {
                return SelectionResult::NeedsPromotion {
                    from: selected,
                    to: sq,
                };
            }

            // 直接执行（只有一个匹配走法）
            let mv = matching[0];
            self.make_move(mv);
            return SelectionResult::MoveMade { mv };
        }

        // 无选中棋子：检查该棋子是否有合法走法，没有则不可选中
        if piece_at_sq.is_some() && can_select {
            let has_legal = legal_moves.iter().any(|m| m.from() == sq);
            if has_legal {
                self.set_selected(sq);
                return SelectionResult::Selected { square: sq };
            }
        }

        SelectionResult::Cleared
    }

    /// 执行走法（不经选中流程，用于拖拽释放和引擎走法）
    pub fn make_move(&mut self, mv: Move) {
        if self.game.play(mv).is_ok() {
            self.clear_selection();
        }
    }

    /// 完成升变选择
    pub fn complete_promotion(
        &mut self,
        from: Square,
        to: Square,
        promotion: Promotion,
    ) -> SelectionResult {
        let mv = self
            .game
            .legal_moves()
            .into_iter()
            .find(|m| m.from() == from && m.to() == to && m.promotion() == Some(promotion));

        if let Some(mv) = mv {
            self.make_move(mv);
            SelectionResult::MoveMade { mv }
        } else {
            self.clear_selection();
            SelectionResult::Cleared
        }
    }

    /// 清除选中状态
    pub fn clear_selection(&mut self) {
        self.selected_square = None;
    }

    /// 开始新对局（保持当前模式）
    pub fn new_game(&mut self) {
        self.game = Game::new(); // 完全新的标准对局
        self.selected_square = None;
    }

    /// 恢复当前对局的起始局面
    pub fn reset_to_start(&mut self) {
        self.game.go_to_start();
        self.selected_square = None;
    }

    /// 请求引擎走一步（仅在 HumanVsAI 模式下有效）
    pub fn request_engine_move(&mut self) -> Option<Move> {
        if !self.game.is_at_latest() {
            return None;
        }

        let pos = self.game.position().clone();
        let mv = self.engine.as_mut()?.search(&pos)?;

        self.game.play(mv).ok()?;
        self.clear_selection();

        Some(mv)
    }

    /// 是否轮到引擎走棋
    pub fn is_engine_turn(&self) -> bool {
        self.mode == GameMode::HumanVsAI
            && self.engine.is_some()
            && self.game.side_to_move() != self.player_color
            && self.game.is_at_latest()
    }

    /// 引擎名称
    pub fn engine_name(&self) -> Option<&str> {
        self.engine.as_ref().map(|e| e.name())
    }

    /// 获取人类执棋颜色
    pub fn player_color(&self) -> Color {
        self.player_color
    }

    /// 设置人类执棋颜色（会重置对局）
    pub fn set_player_color(&mut self, color: Color) {
        self.player_color = color;
        self.new_game();
    }

    /// 设置 AI 引擎（会重置对局）
    pub fn set_engine(&mut self, engine: Box<dyn ChessEngine>) {
        self.engine = Some(engine);
        self.new_game();
    }

    /// 切换模式（会重置对局）
    pub fn set_mode(&mut self, mode: GameMode, engine: Option<Box<dyn ChessEngine>>) {
        self.mode = mode;
        self.engine = engine;
        self.new_game();
    }

    /// 获取游戏模式
    pub fn mode(&self) -> GameMode {
        self.mode
    }

    /// 选中的格子
    pub fn selected_square(&self) -> Option<Square> {
        self.selected_square
    }

    /// 设置选中格子
    fn set_selected(&mut self, sq: Square) {
        self.selected_square = Some(sq);
    }
}

impl Default for GameController {
    fn default() -> Self {
        Self::new()
    }
}
