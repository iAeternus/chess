use arrayvec::ArrayVec;

use crate::{
    ChessError, Move, PieceKind, Position, Result,
    attack::is_square_attacked,
    makemove::{Undo, make_move, unmake_move},
    movegen,
};

pub struct Game {
    position: Position,
    history: Vec<(Move, Undo)>,
    /// PGN 头信息，保持插入顺序
    headers: Vec<(String, String)>,
}

impl Game {
    /// 从标准起始局面创建
    pub fn new() -> Self {
        Self {
            position: Position::startpos(),
            history: Vec::new(),
            headers: Vec::new(),
        }
    }

    /// 从 FEN 创建
    pub fn from_fen(fen: &str) -> Result<Self> {
        Ok(Self {
            position: Position::from_fen(fen)?,
            history: Vec::new(),
            headers: Vec::new(),
        })
    }

    /// 获取当前局面（只读引用）
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// 获取当前所有合法走法
    pub fn legal_moves(&mut self) -> ArrayVec<Move, 256> {
        movegen::generate_legal(&mut self.position)
    }

    /// 执行走法
    pub fn play(&mut self, mv: Move) -> Result<()> {
        if !self.legal_moves().contains(&mv) {
            return Err(ChessError::InvalidMove(mv));
        }

        let undo = make_move(&mut self.position, mv);
        self.history.push((mv, undo));

        Ok(())
    }

    /// 撤销上一步走法
    pub fn undo(&mut self) -> Result<()> {
        let (_, undo) = self
            .history
            .pop()
            .ok_or_else(|| ChessError::NothingToUndo)?;

        unmake_move(&mut self.position, undo);

        Ok(())
    }

    /// 检测游戏是否结束
    pub fn is_game_over(&mut self) -> Result<bool> {
        Ok(self.is_checkmate()? || self.is_stalemate()?)
    }

    /// 检测是否将军
    pub fn is_check(&self) -> Result<bool> {
        let side = self.position.side_to_move();
        let king = self
            .position
            .board()
            .piece_kind(side, PieceKind::King)
            .lsb()
            .ok_or(ChessError::NoKing(side))?;

        Ok(is_square_attacked(
            &self.position.board(),
            king,
            side.flip(),
        ))
    }

    /// 检测是否将杀
    pub fn is_checkmate(&mut self) -> Result<bool> {
        Ok(self.is_check()? && self.legal_moves().is_empty())
    }

    /// 检测是否逼和
    pub fn is_stalemate(&mut self) -> Result<bool> {
        Ok(!self.is_check()? && self.legal_moves().is_empty())
    }

    /// 走法历史
    pub fn history(&self) -> &[(Move, Undo)] {
        &self.history
    }

    /// 重置
    pub fn reset(&mut self) {
        self.position = Position::startpos();
        self.history.clear();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CastlingRights, MoveFlag, Promotion, Square};

    #[test]
    fn initial_position_20_moves() {
        let mut game = Game::new();
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
        let mut game = Game::from_fen("4k3/8/8/8/8/5r2/8/4K2R w K - 0 1").unwrap();
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
        let mut game = Game::from_fen("4k3/8/8/3pP3/8/8/4K3/8 w - - 0 1").unwrap();
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
            let mut game = Game::from_fen("8/4P3/8/8/8/8/8/4K3 w - - 0 1").unwrap();

            // 白王 e1
            // 白兵 e7，即将进入第8排升变
            // 黑方无子阻挡
            // 可以升变为后、车、象、马四种棋子
            let mv = Move::new_promotion(Square::E7, Square::E8, promotion, false);
            assert!(
                game.legal_moves().contains(&mv),
                "promotion {:?} missing",
                promotion
            );

            game.play(mv).unwrap();

            let promoted_kind =
                Option::<PieceKind>::from(mv.promotion()).expect("invalid promotion");
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
        assert!(game.is_check().unwrap());
    }

    #[test]
    fn checkmate_detection() {
        // 黑王 h8
        // 白王 f6，保护 g7 周围区域
        // 白后 g7，占据黑王附近并攻击 h8
        // 黑王被将军，且无合法逃脱格
        // 形成将杀
        let mut game = Game::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(game.is_checkmate().unwrap());
        assert!(game.is_game_over().unwrap());
    }

    #[test]
    fn stalemate_detection() {
        // 黑王 h8
        // 白王 f7，控制黑王可移动区域
        // 白后 g6，限制黑王活动范围
        // 黑王没有合法走法
        // 但当前没有被将军，因此为逼和
        let mut game = Game::from_fen("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(game.is_stalemate().unwrap());
        assert!(game.is_game_over().unwrap());
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
}
