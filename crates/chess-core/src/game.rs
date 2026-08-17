//! 对局模型：管理局面、走法历史、导航和 PGN
//!
//! Game 是唯一权威数据源
//! 内部用起始局面 + 走法列表 + 游标表示当前位置，导航时从起始局面重放
//! san_cache 和 undos 与走法列表保持同步

use std::{fmt::Display, str::FromStr};

use arrayvec::ArrayVec;

use crate::{
    ChessError, Color, Move, Position, Promotion, Result, Square,
    makemove::{Undo, make_move},
    movegen,
    pgn::{move_to_san, parse_pgn, write_pgn},
};

pub struct Game {
    /// 起始局面，导航和 reset 都回到这里
    start_position: Position,

    /// 当前局面：始终等于 start_position 重放 moves[..cursor] 后的局面
    position: Position,

    /// 已执行的半移动
    moves: Vec<Move>,

    /// 与 moves 一一对应的撤销信息，只用于内部 undo
    undos: Vec<Undo>,

    /// 当前位置：0 表示初始局面，len 表示最新局面
    cursor: usize,

    /// SAN 缓存，与 moves 一一对应
    san_cache: Vec<String>,

    /// PGN 头信息，保持插入顺序
    headers: Vec<(String, String)>,

    /// 棋盘视角：底部棋子的颜色（White = 白方在下方，Black = 黑方在下方即翻转）
    /// 纯显示字段，不影响规则、FEN、PGN 或 Zobrist
    view_from: Color,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// 从标准起始局面创建
    pub fn new() -> Self {
        let start = Position::startpos();
        Self {
            start_position: start.clone(),
            position: start,
            moves: Vec::new(),
            undos: Vec::new(),
            cursor: 0,
            san_cache: Vec::new(),
            headers: Vec::new(),
            view_from: Color::White,
        }
    }

    /// 从 FEN 创建
    pub fn from_fen(fen: &str) -> Result<Self> {
        let start = Position::from_fen(fen)?;
        Ok(Self {
            start_position: start.clone(),
            position: start,
            moves: Vec::new(),
            undos: Vec::new(),
            cursor: 0,
            san_cache: Vec::new(),
            headers: Vec::new(),
            view_from: Color::White,
        })
    }

    /// 从 PGN 创建
    pub fn from_pgn(pgn: &str) -> Result<Self> {
        let parsed = parse_pgn(pgn)?;

        let mut game = Self {
            start_position: parsed.start_position.clone(),
            position: parsed.start_position,
            moves: Vec::new(),
            undos: Vec::new(),
            cursor: 0,
            san_cache: Vec::new(),
            headers: parsed.headers,
            view_from: Color::White,
        };

        // 逐着执行，填充 undos 和 san_cache
        for mv in parsed.moves {
            game.play(mv)?; // 此时会生成 SAN 并更新 position 等
        }

        // 加载完成后通常定位到起始局面
        game.go_to_start();

        Ok(game)
    }

    /// 执行走法
    pub fn play(&mut self, mv: Move) -> Result<()> {
        if !self.legal_moves().contains(&mv) {
            return Err(ChessError::InvalidMove(mv));
        }

        // 如果不在最新位置，截断未来分支
        if self.cursor < self.moves.len() {
            self.moves.truncate(self.cursor);
            self.undos.truncate(self.cursor);
            self.san_cache.truncate(self.cursor);
        }

        // 生成 SAN，需要走棋前的局面
        let san =
            move_to_san(&self.position, mv).unwrap_or_else(|_| format!("{}{}", mv.from(), mv.to()));

        // 执行走法，并保存 undo
        let undo = make_move(&mut self.position, mv);

        self.moves.push(mv);
        self.undos.push(undo);
        self.san_cache.push(san);
        self.cursor += 1;

        Ok(())
    }

    pub fn can_go_back(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.cursor < self.moves.len()
    }

    pub fn go_to_start(&mut self) {
        self.go_to_ply(0);
    }

    pub fn go_to_end(&mut self) {
        let end = self.moves.len();
        self.go_to_ply(end);
    }

    pub fn go_back(&mut self) {
        if self.cursor > 0 {
            self.go_to_ply(self.cursor - 1);
        }
    }

    pub fn go_forward(&mut self) {
        if self.cursor < self.moves.len() {
            self.go_to_ply(self.cursor + 1);
        }
    }

    pub fn go_to_ply(&mut self, ply: usize) {
        let target = ply.min(self.moves.len());

        if target == self.cursor {
            return;
        }

        // 从起始局面重放到目标 ply，O(N)但由于步数不多可接受
        self.position = self.start_position.clone();
        for mv in &self.moves[..target] {
            make_move(&mut self.position, *mv);
        }

        self.cursor = target;
    }

    pub fn current_ply(&self) -> usize {
        self.cursor
    }

    pub fn total_moves(&self) -> usize {
        self.moves.len()
    }

    pub fn is_at_latest(&self) -> bool {
        self.cursor == self.moves.len()
    }

    /// 后退一个半移动
    pub fn undo(&mut self) -> Result<()> {
        if self.cursor == 0 {
            return Err(ChessError::NothingToUndo);
        }

        self.go_back();
        Ok(())
    }

    /// 前进一个半移动
    pub fn redo(&mut self) -> Result<()> {
        if self.cursor >= self.moves.len() {
            return Err(ChessError::NothingToRedo);
        }

        self.go_forward();
        Ok(())
    }

    /// 获取起始局面
    pub fn start_position(&self) -> &Position {
        &self.start_position
    }

    /// 获取当前局面
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// 获取当前所有合法走法
    pub fn legal_moves(&self) -> ArrayVec<Move, 256> {
        movegen::generate_legal2(&self.position)
    }

    /// 获取从当前 sq 出发的所有合法走法
    pub fn legal_moves_from(&self, sq: Square) -> ArrayVec<Move, 256> {
        self.legal_moves()
            .into_iter()
            .filter(|m| m.from() == sq)
            .collect()
    }

    /// 获取从 from 出发到 to 的所有合法走法（升变时含多个变体）
    pub fn legal_moves_between(&self, from: Square, to: Square) -> ArrayVec<Move, 256> {
        self.legal_moves()
            .into_iter()
            .filter(|m| m.from() == from && m.to() == to)
            .collect()
    }

    /// 查找 from→to 且升变类型匹配的唯一合法走法
    pub fn find_legal_move(
        &self,
        from: Square,
        to: Square,
        promotion: Option<Promotion>,
    ) -> Option<Move> {
        self.legal_moves_between(from, to)
            .into_iter()
            .find(|m| m.promotion() == promotion)
    }

    /// 获取上一步走法
    pub fn last_move(&self) -> Option<Move> {
        self.cursor
            .checked_sub(1)
            .and_then(|i| self.moves.get(i))
            .copied()
    }

    /// 获取走法历史
    pub fn move_history(&self) -> &[Move] {
        &self.moves
    }

    /// 获取 SAN 缓存
    pub fn san_list(&self) -> &[String] {
        &self.san_cache
    }

    /// 是否将军
    pub fn is_check(&self) -> bool {
        self.position.is_check()
    }

    /// 是否将死
    pub fn is_checkmate(&self) -> bool {
        self.position.is_check() && self.legal_moves().is_empty()
    }

    /// 是否逼和
    pub fn is_stalemate(&self) -> bool {
        !self.position.is_check() && self.legal_moves().is_empty()
    }

    /// 是否游戏结束
    pub fn is_game_over(&self) -> bool {
        self.is_checkmate() || self.is_stalemate()
    }

    /// 获取走棋方
    pub fn side_to_move(&self) -> Color {
        self.position.side_to_move()
    }

    /// 当前棋盘视角（底部棋子的颜色）
    ///
    /// `Color::White` = 白方在下方（正常视角），`Color::Black` = 黑方在下方（翻转视角）
    pub fn view_from(&self) -> Color {
        self.view_from
    }

    /// 设置棋盘视角
    pub fn set_view_from(&mut self, view_from: Color) {
        self.view_from = view_from;
    }

    /// 切换棋盘视角
    pub fn flip_view(&mut self) {
        self.view_from = self.view_from.flip();
    }

    /// 棋盘格 → 视角格（渲染用）
    pub fn square_to_view(&self, sq: Square) -> Square {
        sq.view(self.view_from)
    }

    /// 视角格 → 棋盘格（输入用）
    pub fn view_to_square(&self, sq: Square) -> Square {
        sq.view(self.view_from)
    }

    /// 获取第 ply 着局面
    pub fn position_at_ply(&self, ply: usize) -> Option<Position> {
        if ply > self.moves.len() {
            return None;
        }

        let mut pos = self.start_position.clone();
        for mv in &self.moves[..ply] {
            make_move(&mut pos, *mv);
        }

        Some(pos)
    }

    /// 导出为 PGN
    pub fn export_pgn(&self) -> String {
        // 构造一个临时 Game，游标在末尾
        let mut tmp = Self {
            start_position: self.start_position.clone(),
            position: self.start_position.clone(),
            moves: self.moves.clone(),
            undos: Vec::new(), // 不需要
            cursor: self.moves.len(),
            san_cache: self.san_cache.clone(),
            headers: self.headers.clone(),
            view_from: Color::White,
        };

        // 重放到末尾
        for mv in &tmp.moves {
            make_move(&mut tmp.position, *mv);
        }

        write_pgn(&tmp)
    }

    /// 分析模式专用：尝试执行任意一方棋子的合法走法 TODO: 需要进一步检查
    pub fn play_any(&mut self, mv: Move) -> Result<()> {
        let side_to_move = self.position.side_to_move();

        // 如果本来轮到该方，直接走
        if self.position.piece_at(mv.from()).map(|p| p.color) == Some(side_to_move) {
            return self.play(mv);
        }

        // 否则需要临时切换行棋方来验证走法
        let moving_side = match self.position.piece_at(mv.from()) {
            Some(p) => p.color,
            None => return Err(ChessError::InvalidMove(mv)),
        };

        let saved_side = side_to_move;

        // 需要 Position 提供 setter，或通过 make_move/unmake_move 模拟
        self.position.set_side_to_move(moving_side);

        let legal = self.legal_moves();
        let valid = legal.contains(&mv);

        self.position.set_side_to_move(saved_side);

        if !valid {
            return Err(ChessError::InvalidMove(mv));
        }

        // 检查通过后执行
        self.play(mv)
    }

    /// 获取所有 PGN 头信息（保持插入顺序）
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }

    /// 大小写不敏感查找头信息值
    pub fn header(&self, key: &str) -> Option<&str> {
        let key_lower = key.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v.as_str())
    }

    /// 设置头信息（若已存在则更新值，否则追加）
    pub fn set_header(&mut self, key: &str, value: &str) {
        let key_lower = key.to_lowercase();
        if let Some((_, v)) = self
            .headers
            .iter_mut()
            .find(|(k, _)| k.to_lowercase() == key_lower)
        {
            *v = value.to_string();
        } else {
            self.headers.push((key.to_string(), value.to_string()));
        }
    }

    /// 获取对局结果（来自 Result 头信息，默认为 "*"）
    pub fn result(&self) -> &str {
        self.header("Result").unwrap_or("*")
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.export_pgn())
    }
}

impl FromStr for Game {
    type Err = ChessError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Game::from_pgn(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CastlingRights, Move, MoveFlag, Promotion, Square};

    #[test]
    fn initial_position_20_moves() {
        let game = Game::new();
        let moves = game.legal_moves();

        // 初始局面：
        // 白方：
        // 8个兵可前进一步，8个兵可前进两步，共16步
        // b1、g1马各有2种走法，共4步
        // 合计20个合法走法
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn castling_through_check_blocked() {
        // 白王 e1，白车 h1
        // 黑车 f3，攻击 f1 格
        // 王翼易位需要经过 f1
        // f1 被攻击，王不能经过危险格，因此禁止易位
        let game = Game::from_fen("4k3/8/8/8/8/5r2/8/4K2R w K - 0 1").unwrap();
        let moves = game.legal_moves();
        let castle = Move::new(Square::E1, Square::G1, MoveFlag::KingCastle);
        assert!(!moves.contains(&castle));
    }

    #[test]
    fn en_passant_capture() {
        // 白王 e1，黑王 e8
        // 白兵 e5，黑兵 d5
        // 当前 ep 目标格为 d6
        // 白兵可以 e5xd6 吃掉刚经过 d6 的黑兵
        let mut game = Game::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap();
        let mv = Move::new(Square::E5, Square::D6, MoveFlag::EnPassant);
        assert!(game.legal_moves().contains(&mv));

        game.play(mv).unwrap();

        // 白兵移动到 d6
        assert!(game.position().piece_at(Square::D6).is_some());
        // 原黑兵 d5 被吃除
        assert!(game.position().piece_at(Square::D5).is_none());
    }

    #[test]
    fn en_passant_blocked() {
        // 白王 e2，黑王 e8
        // 白兵 e5，黑兵 d5
        // 但是 FEN 中没有 ep 权限
        // 即使白兵可以攻击 d6，也不能执行吃过路兵
        let game = Game::from_fen("4k3/8/8/3pP3/8/8/4K3/8 w - - 0 1").unwrap();
        let mv = Move::new(Square::E5, Square::D6, MoveFlag::EnPassant);
        assert!(!game.legal_moves().contains(&mv));
    }

    #[test]
    fn promotion_all_types() {
        let promotions = [
            Promotion::Queen,
            Promotion::Rook,
            Promotion::Bishop,
            Promotion::Knight,
        ];

        for promotion in promotions {
            let mut game = Game::from_fen("3k4/4P3/8/8/8/8/8/4K3 w - - 0 1").unwrap();

            // 白王 e1
            // 白兵 e7，即将进入第8排升变
            // 黑王 d8，不在升变格 e8 上
            let mv = Move::new_promotion(Square::E7, Square::E8, promotion, false);
            assert!(
                game.legal_moves().contains(&mv),
                "promotion {:?} missing",
                promotion
            );

            game.play(mv).unwrap();

            let promoted_kind = mv
                .promotion()
                .expect("promotion move without promotion piece")
                .into();
            assert_eq!(
                game.position().piece_at(Square::E8).unwrap().kind,
                promoted_kind
            );
        }
    }

    #[test]
    fn check_detection() {
        // 白王 e1
        // 黑王 e8
        // 黑车 e2
        // 黑车沿 e 文件攻击白王
        // 白王处于将军状态
        let game = Game::from_fen("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1").unwrap();
        assert!(game.position().is_check());
    }

    #[test]
    fn checkmate_detection() {
        // 黑王 h8
        // 白王 f6，保护 g7 周围区域
        // 白后 g7，占据黑王附近并攻击 h8
        // 黑王被将军，且无合法逃脱格
        // 形成将杀
        let game = Game::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(game.is_checkmate());
        assert!(game.is_game_over());
    }

    #[test]
    fn stalemate_detection() {
        // 黑王 h8
        // 白王 f7，控制黑王可移动区域
        // 白后 g6，限制黑王活动范围
        // 黑王没有合法走法
        // 但当前没有被将军，因此为逼和
        let game = Game::from_fen("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(game.is_stalemate());
        assert!(game.is_game_over());
    }

    #[test]
    fn castling_rights_update() {
        // 白王 e1
        // 白车 a1、h1
        // 当前拥有白方王翼和后翼易位权限
        // 白王移动到 e2 后：
        // 王已经移动，永久失去两侧易位权限
        let mut game = Game::from_fen("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
        let mv = Move::new(Square::E1, Square::E2, MoveFlag::Quiet);

        game.play(mv).unwrap();

        let rights = game.position().castling();
        assert!(!rights.contains(CastlingRights::WHITE_KING_SIDE));
        assert!(!rights.contains(CastlingRights::WHITE_QUEEN_SIDE));
    }

    #[test]
    fn legal_moves_between_basic() {
        let game = Game::new();

        // e2-e4（双步兵）在初始局面中合法
        let pushes = game.legal_moves_between(Square::E2, Square::E4);
        assert_eq!(pushes.len(), 1);
        assert_eq!(pushes[0].flag(), MoveFlag::DoublePawnPush);

        // 目标格超出兵的可达范围 -> 空
        assert!(game.legal_moves_between(Square::E2, Square::E5).is_empty());
        // 起始格无棋子 -> 空
        assert!(game.legal_moves_between(Square::D4, Square::D5).is_empty());
    }

    #[test]
    fn legal_moves_between_promotion_variants() {
        // 白兵 a7 即将升变，黑王不在 a8/e8
        let game = Game::from_fen("6k1/P7/8/8/8/8/8/7K w - - 0 1").unwrap();

        let promotions = game.legal_moves_between(Square::A7, Square::A8);
        assert_eq!(promotions.len(), 4);

        let kinds: Vec<Promotion> = promotions.iter().filter_map(|m| m.promotion()).collect();
        assert!(kinds.contains(&Promotion::Queen));
        assert!(kinds.contains(&Promotion::Rook));
        assert!(kinds.contains(&Promotion::Bishop));
        assert!(kinds.contains(&Promotion::Knight));
    }

    #[test]
    fn find_legal_move_promotion() {
        let game = Game::from_fen("6k1/P7/8/8/8/8/8/7K w - - 0 1").unwrap();

        let mv = game.find_legal_move(Square::A7, Square::A8, Some(Promotion::Queen));
        assert!(mv.is_some());
        assert_eq!(mv.unwrap().promotion(), Some(Promotion::Queen));

        // 无升变的普通走法在升变格上找不到
        assert_eq!(game.find_legal_move(Square::A7, Square::A8, None), None);
    }

    #[test]
    fn find_legal_move_quiet() {
        let game = Game::new();

        let mv = game.find_legal_move(Square::E2, Square::E4, None);
        assert_eq!(
            mv,
            Some(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush,))
        );

        // 带升变类型查询普通走法 -> None
        assert_eq!(
            game.find_legal_move(Square::E2, Square::E4, Some(Promotion::Queen)),
            None
        );
    }

    #[test]
    fn view_from_defaults_to_white() {
        assert_eq!(Game::new().view_from(), Color::White);
        assert_eq!(
            Game::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1")
                .unwrap()
                .view_from(),
            Color::White
        );
        assert_eq!(
            Game::from_pgn("1. e4 e5").unwrap().view_from(),
            Color::White
        );
    }

    #[test]
    fn flip_view_toggles() {
        let mut game = Game::new();
        assert_eq!(game.view_from(), Color::White);
        game.flip_view();
        assert_eq!(game.view_from(), Color::Black);
        game.flip_view();
        assert_eq!(game.view_from(), Color::White);
        game.set_view_from(Color::Black);
        assert_eq!(game.view_from(), Color::Black);
    }

    #[test]
    fn view_square_mapping() {
        let mut game = Game::new();
        assert_eq!(game.view_from(), Color::White);
        assert_eq!(game.square_to_view(Square::E4), Square::E4);
        assert_eq!(game.view_to_square(Square::A1), Square::A1);

        game.set_view_from(Color::Black);
        assert_eq!(game.square_to_view(Square::E4), Square::D5);
        assert_eq!(game.view_to_square(Square::D5), Square::E4);
        assert_eq!(game.square_to_view(Square::A1), Square::H8);

        // 双向互逆
        for i in 0..64 {
            let sq = Square::new(i).unwrap();
            assert_eq!(game.view_to_square(game.square_to_view(sq)), sq);
        }
    }

    #[test]
    fn view_does_not_affect_rules() {
        let fen = "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let moves_before = game.legal_moves();
        game.set_view_from(Color::Black);
        let moves_after = game.legal_moves();
        assert_eq!(moves_before, moves_after);
        assert!(!game.export_pgn().contains("Black"));
    }
}
