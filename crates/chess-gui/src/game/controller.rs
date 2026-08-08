//! 对局控制器：封装 Game、历史导航、选中状态、AI 引擎。
//!
//! # 状态设计
//!
//! - `move_history` 是**单一真相来源**：存储所有已执行的半移动。
//! - `current_ply` 表示当前位置（0 = 初始局面，N = 第 N 步之后）。
//! - 导航通过 `game.reset()` + 重放 `move_history[..target]` 实现，无 O(N) 复杂度问题
//!   （国际象棋对局最多 ~200 半移动，重放耗时可忽略）。
//! - 不再有 `cached_moves` 和 `game.history()` 之间的不一致。

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

    /// 单一真相来源：所有已执行的半移动（按顺序）
    move_history: Vec<Move>,
    /// 当前位置：0 = 初始局面，N = 第 N 个半移动之后
    current_ply: usize,

    /// SAN 格式走法文本缓存（与 move_history 同步）
    san_cache: Vec<String>,

    /// 当前选中的格子
    selected_square: Option<Square>,
    /// 当前局面的所有合法走法（缓存，避免每帧生成）
    legal_moves_cache: ArrayVec<Move, 256>,

    /// AI 引擎（仅在 HumanVsAI 模式下存在）
    engine: Option<Box<dyn ChessEngine>>,
}

impl GameController {
    // 构造

    /// 创建默认对局（标准初始局面，双人对战模式）
    pub fn new() -> Self {
        let mut controller = Self {
            game: Game::new(),
            mode: GameMode::HumanVsHuman,
            move_history: Vec::with_capacity(256),
            current_ply: 0,
            san_cache: Vec::with_capacity(256),
            selected_square: None,
            legal_moves_cache: ArrayVec::new(),
            engine: None,
        };
        controller.refresh_legal_moves();
        controller
    }

    /// 创建带引擎的对局
    #[allow(dead_code)]
    pub fn new_with_engine(mode: GameMode, engine: Box<dyn ChessEngine>) -> Self {
        let mut controller = Self {
            game: Game::new(),
            mode,
            move_history: Vec::with_capacity(256),
            current_ply: 0,
            san_cache: Vec::with_capacity(256),
            selected_square: None,
            legal_moves_cache: ArrayVec::new(),
            engine: Some(engine),
        };
        controller.refresh_legal_moves();
        controller
    }

    /// 从 PGN 字符串加载对局（自动进入 Replay 模式）
    pub fn from_pgn(pgn: &str) -> Result<Self> {
        let mut game = chess_core::from_pgn(pgn)?;
        let move_history: Vec<Move> = game.history().iter().map(|(mv, _)| *mv).collect();

        // 撤销所有走法回到初始局面（保留 PGN headers）
        while !game.history().is_empty() {
            game.undo().ok();
        }

        let mut controller = Self {
            game,
            mode: GameMode::Replay,
            move_history,
            current_ply: 0,
            san_cache: Vec::with_capacity(256),
            selected_square: None,
            legal_moves_cache: ArrayVec::new(),
            engine: None,
        };
        controller.refresh_san_cache();
        controller.refresh_legal_moves();
        Ok(controller)
    }

    // 局面访问

    /// 获取当前局面的不可变引用
    pub fn current_position(&self) -> &Position {
        self.game.position()
    }

    /// 获取当前局面的所有合法走法
    #[allow(dead_code)]
    pub fn legal_moves(&self) -> &ArrayVec<Move, 256> {
        &self.legal_moves_cache
    }

    /// 获取当前选中棋子可以走到的合法目标走法
    pub fn legal_moves_for_selected(&self) -> ArrayVec<Move, 256> {
        match self.selected_square {
            Some(sq) => self
                .legal_moves_cache
                .iter()
                .filter(|m| m.from() == sq)
                .copied()
                .collect(),
            None => ArrayVec::new(),
        }
    }

    /// 最后一步走法（用于高亮显示）
    pub fn last_move(&self) -> Option<Move> {
        if self.current_ply > 0 {
            self.move_history.get(self.current_ply - 1).copied()
        } else {
            None
        }
    }

    // 走法执行

    /// 点击格子：根据当前选中状态决定行为
    ///
    /// - 未选中 + 点击己方棋子 → 选中
    /// - 已选中 + 点击己方其他棋子 → 重新选中
    /// - 已选中 + 点击合法目标 → 执行走法（可能触发升变选择）
    /// - 已选中 + 点击非法目标 → 清除选中
    /// - 已选中 + 点击同一格子 → 清除选中
    pub fn select_square(&mut self, sq: Square) -> SelectionResult {
        let position = self.game.position();
        let piece_at_sq = position.piece_at(sq);
        let side_to_move = position.side_to_move();

        // 分析模式下可以移动任意一方的棋子；其他模式下只能移动当前方
        let can_select =
            self.mode == GameMode::Analysis || piece_at_sq.is_some_and(|p| p.color == side_to_move);

        if let Some(selected) = self.selected_square {
            // 已有选中棋子
            if let Some(_piece) = piece_at_sq
                && can_select
                && sq != selected
            {
                // 点击了另一个可选棋子 → 重新选中
                self.set_selected(sq);
                return SelectionResult::Selected { square: sq };
            }

            // 检查是否点击了合法目标
            let matching: ArrayVec<Move, 256> = self
                .legal_moves_cache
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
            let has_legal = self.legal_moves_cache.iter().any(|m| m.from() == sq);
            if has_legal {
                self.set_selected(sq);
                return SelectionResult::Selected { square: sq };
            }
        }

        SelectionResult::Cleared
    }

    /// 执行走法（不经选中流程，用于拖拽释放和引擎走法）
    pub fn make_move(&mut self, mv: Move) {
        // 如果不在最新位置，截断未来分支
        if self.current_ply < self.move_history.len() {
            self.move_history.truncate(self.current_ply);
            self.navigate_to(self.current_ply);
        }

        if self.game.play(mv).is_ok() {
            self.move_history.push(mv);
            self.current_ply += 1;
            self.clear_selection();
            self.refresh_legal_moves();
            self.refresh_san_cache();
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
            .legal_moves_cache
            .iter()
            .find(|m| m.from() == from && m.to() == to && m.promotion() == Some(promotion))
            .copied();

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

    // 历史导航

    /// 是否可以后退
    pub fn can_go_back(&self) -> bool {
        self.current_ply > 0
    }

    /// 是否可以前进
    pub fn can_go_forward(&self) -> bool {
        self.current_ply < self.move_history.len()
    }

    /// 跳转到第一步
    pub fn go_to_start(&mut self) {
        self.navigate_to(0);
    }

    /// 后退一步
    pub fn go_back(&mut self) {
        if self.current_ply > 0 {
            self.navigate_to(self.current_ply - 1);
        }
    }

    /// 前进一步
    pub fn go_forward(&mut self) {
        if self.current_ply < self.move_history.len() {
            self.navigate_to(self.current_ply + 1);
        }
    }

    /// 跳转到最后一步
    pub fn go_to_end(&mut self) {
        let end = self.move_history.len();
        self.navigate_to(end);
    }

    /// 跳转到指定 ply
    pub fn go_to_ply(&mut self, ply: usize) {
        let target = ply.min(self.move_history.len());
        self.navigate_to(target);
    }

    /// 当前 ply 编号（用于 UI 显示）
    pub fn current_ply(&self) -> usize {
        self.current_ply
    }

    /// 总半移动数
    pub fn total_moves(&self) -> usize {
        self.move_history.len()
    }

    /// 完整的走法历史（用于走法列表面板）
    pub fn move_history(&self) -> &[Move] {
        &self.move_history
    }

    /// SAN 格式走法列表（与 move_history 一一对应）
    pub fn san_list(&self) -> &[String] {
        &self.san_cache
    }

    /// 刷新 SAN 缓存
    fn refresh_san_cache(&mut self) {
        self.san_cache.clear();
        let mut game = Game::new(); // TODO: 支持非标准初始局面（FEN header）
        for mv in &self.move_history {
            match chess_core::move_to_san(game.position(), *mv) {
                Ok(san) => self.san_cache.push(san),
                Err(_) => {
                    // 回退到坐标表示
                    self.san_cache.push(format!("{}{}", mv.from(), mv.to()));
                }
            }
            game.play(*mv).ok();
        }
    }

    // 局面快照（用于 SAN 生成）

    /// 获取指定 ply 之后的局面。
    ///
    /// 注意：这会临时修改内部 game 状态，但在同一帧内恢复。
    /// 调用者应一次获取所有需要的快照，避免重复导航。
    #[allow(dead_code)]
    pub fn position_at_ply(&mut self, ply: usize) -> Option<Position> {
        if ply > self.move_history.len() {
            return None;
        }
        let saved = self.current_ply;
        self.navigate_to(ply);
        let pos = self.game.position().clone();
        self.navigate_to(saved);
        Some(pos)
    }

    // 状态查询

    pub fn mode(&self) -> GameMode {
        self.mode
    }

    /// 切换模式（会重置对局）
    pub fn set_mode(&mut self, mode: GameMode, engine: Option<Box<dyn ChessEngine>>) {
        self.mode = mode;
        self.engine = engine;
        self.new_game();
    }

    /// 当前是否将军
    pub fn is_check(&self) -> bool {
        self.game.is_check()
    }

    /// 对局是否结束（分析模式下永不结束）
    #[allow(dead_code)]
    pub fn is_game_over(&mut self) -> bool {
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
        self.game.position().side_to_move()
    }

    /// 选中的格子
    pub fn selected_square(&self) -> Option<Square> {
        self.selected_square
    }

    // PGN

    /// 导出当前对局为 PGN 字符串
    pub fn export_pgn(&mut self) -> String {
        let saved_ply = self.current_ply;
        // 导航到最终位置以生成完整走法文本
        self.navigate_to(self.move_history.len());
        let pgn = chess_core::to_pgn(&self.game);
        // 恢复到之前的位置
        self.navigate_to(saved_ply);
        pgn
    }

    /// PGN 头信息
    #[allow(dead_code)]
    pub fn header(&self, key: &str) -> Option<&str> {
        self.game.header(key)
    }

    /// 所有 PGN headers（用于显示对局信息）
    #[allow(dead_code)]
    pub fn headers(&self) -> &[(String, String)] {
        self.game.headers()
    }

    // 对局控制

    /// 开始新对局（保持当前模式）
    pub fn new_game(&mut self) {
        self.game.reset();
        self.move_history.clear();
        self.current_ply = 0;
        self.clear_selection();
        self.refresh_legal_moves();
        self.refresh_san_cache();
    }

    // 引擎接口

    /// 请求引擎走一步（仅在 HumanVsAI 模式下有效）
    pub fn request_engine_move(&mut self) -> Option<Move> {
        if self.current_ply < self.move_history.len() {
            return None; // 不在最新位置，引擎不应走棋
        }
        let pos = self.game.position().clone();
        let mv = self.engine.as_mut()?.search(&pos)?;
        self.make_move(mv);
        Some(mv)
    }

    /// 是否轮到引擎走棋
    pub fn is_engine_turn(&self) -> bool {
        self.mode == GameMode::HumanVsAI
            && self.engine.is_some()
            && self.game.position().side_to_move() == Color::Black
            && self.current_ply >= self.move_history.len()
    }

    /// 引擎名称
    pub fn engine_name(&self) -> Option<&str> {
        self.engine.as_ref().map(|e| e.name())
    }

    // 内部方法

    /// 设置选中格子
    fn set_selected(&mut self, sq: Square) {
        self.selected_square = Some(sq);
        // legal_moves_cache 已包含所有合法走法，UI 按 from 过滤即可
    }

    /// 刷新合法走法缓存
    fn refresh_legal_moves(&mut self) {
        self.legal_moves_cache = self.game.legal_moves();
    }

    /// 核心导航方法：reset + 重放到目标 ply
    fn navigate_to(&mut self, target_ply: usize) {
        let target = target_ply.min(self.move_history.len());
        if target == self.current_ply {
            return;
        }

        self.game.reset();
        for mv in &self.move_history[..target] {
            self.game.play(*mv).ok();
        }
        self.current_ply = target;
        self.clear_selection();
        self.refresh_legal_moves();
    }
}

impl Default for GameController {
    fn default() -> Self {
        Self::new()
    }
}
