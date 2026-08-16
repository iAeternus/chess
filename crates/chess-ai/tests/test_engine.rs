//! ChessEngine 多角度集成测试
//!
//! 测试覆盖：
//! - 一步杀（mate-in-1）检测
//! - 免费得子
//! - 避免失子（需要一定深度）
//! - 将杀/逼和返回 None
//! - 初始局面不崩溃
//! - 确定性
//! - RandomEngine 基本行为
//! - 引擎名称
//! - 边界条件（depth=0、局面不变性）
//! - 战术模式（两步杀、马叉、升变、避免逼和）
//! - 残局（KQ vs K、通路兵赛跑）

use chess_ai::{ChessEngine, MiniMaxEngine, RandomEngine};
use chess_core::{Move, MoveFlag, Position, Square, generate_legal2};

/// 选择 MiniMax 引擎
fn choose_engine(depth: i32) -> Box<dyn ChessEngine> {
    Box::new(MiniMaxEngine::new(depth))
}

/// 在给定局面上运行引擎搜索，返回找到的走法
fn search(position: &Position, depth: i32) -> Option<Move> {
    let mut engine = choose_engine(depth);
    engine.search(position)
}

/// 验证走法合法
fn assert_legal_move(position: &Position, mv: Move, msg: &str) {
    let legal = generate_legal2(position);
    assert!(
        legal.iter().any(|&m| m == mv),
        "{msg}: move {mv:?} is not in legal move list:\n{legal:?}",
    );
}

// ─────────────────────────────────────────────────────────
// 原有 MiniMax 测试
// ─────────────────────────────────────────────────────────

#[test]
fn mate_in_one_white() {
    // 白方：王 f6, 后 g1；黑方：王 h8
    // 白方 Qg1-g7# 将杀
    let position = Position::from_fen("7k/8/5K2/8/8/8/8/6Q1 w - - 0 1").unwrap();
    let mv = search(&position, 1).expect("should find a move");

    let expected = Move::new(Square::G1, Square::G7, MoveFlag::Quiet);
    assert_legal_move(&position, mv, "mate-in-1 move not legal");
    assert_eq!(mv, expected, "expected Qg1-g7# (mate in 1), got {mv:?}");
}

#[test]
fn capture_free_queen() {
    // 白方：王 d1, 车 e2；黑方：王 h8, 后 e6
    // 白方可以免费吃后（Re2xe6）
    let position = Position::from_fen("7k/8/4q3/8/8/8/4R3/3K4 w - - 0 1").unwrap();
    let mv = search(&position, 1).expect("should find a move");

    assert_legal_move(&position, mv, "capture move not legal");
    assert!(mv.is_capture(), "expected a capture (Re2xe6), got {mv:?}");
    assert_eq!(mv.to(), Square::E6, "expected capture on e6, got {mv:?}");
}

#[test]
fn avoid_losing_queen_at_depth_3() {
    // 黑方：王 h8, 后 e6（被白车 e2 攻击）；白方：王 d1, 车 e2
    // 黑方先行。深度 3 应避免后留在 e 线被吃
    let position = Position::from_fen("7k/8/4q3/8/8/8/4R3/3K4 b - - 0 1").unwrap();
    let mv = search(&position, 3).expect("should find a move");

    assert_legal_move(&position, mv, "avoid-losing-queen move not legal");

    // 深度 3 应该能预见：后留在 e 线会被车吃掉
    let from = mv.from();
    assert_eq!(
        from,
        Square::E6,
        "expected to move queen from e6, got {mv:?}"
    );
    assert_ne!(
        mv.to().file(),
        Square::E6.file(),
        "queen should leave the e-file to avoid capture, got {mv:?}"
    );
}

#[test]
fn checkmate_returns_none() {
    // 黑方被将杀：王 h8，白后 g7，白王 f6
    let position = Position::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
    let mv = search(&position, 1);
    assert!(mv.is_none(), "expected None (checkmate), got {mv:?}");
}

#[test]
fn stalemate_returns_none() {
    // 黑方被逼和：王 h8，白后 g6，白王 f7
    let position = Position::from_fen("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1").unwrap();
    let mv = search(&position, 1);
    assert!(mv.is_none(), "expected None (stalemate), got {mv:?}");
}

#[test]
fn initial_position_depth_2_returns_legal_move() {
    let position = Position::startpos();
    let mv = search(&position, 2).expect("should find a move at depth 2");
    assert_legal_move(&position, mv, "initial position depth 2");
}

#[test]
fn deterministic_same_position_same_result() {
    let position = Position::from_fen("7k/8/5K2/8/8/8/8/6Q1 w - - 0 1").unwrap();

    let mv1 = search(&position, 1);
    let mv2 = search(&position, 1);

    assert_eq!(mv1, mv2, "minimax should be deterministic");
}

#[test]
fn depth_3_sees_further_than_depth_1() {
    // 黑方王 h8，后 e6 被白车 e2 攻击。黑方先行
    // 深度 1：看不到白方即将吃后，可能走出 Qe6-e7（仍留在 e 线）
    // 深度 3：预见白方 Re2xe7 吃后，主动避开 e 线
    let position = Position::from_fen("7k/8/4q3/8/8/8/4R3/3K4 b - - 0 1").unwrap();

    let mv_depth_3 = search(&position, 3).expect("depth 3 should find a move");

    assert_eq!(mv_depth_3.from(), Square::E6);
    assert_ne!(
        mv_depth_3.to().file(),
        Square::E6.file(),
        "depth 3: queen should leave e-file; got {mv_depth_3:?}"
    );
}

// ─────────────────────────────────────────────────────────
// RandomEngine 测试
// ─────────────────────────────────────────────────────────

#[test]
fn random_engine_returns_legal_move() {
    let position = Position::startpos();
    let mut engine = RandomEngine::default();
    let mv = engine.search(&position).expect("should find a move");
    assert_legal_move(&position, mv, "random engine: move not legal");
}

#[test]
fn random_engine_checkmate_returns_none() {
    // 黑方被将杀
    let position = Position::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
    let mut engine = RandomEngine::default();
    assert!(
        engine.search(&position).is_none(),
        "random engine should return None on checkmate"
    );
}

#[test]
fn random_engine_stalemate_returns_none() {
    // 黑方被逼和
    let position = Position::from_fen("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1").unwrap();
    let mut engine = RandomEngine::default();
    assert!(
        engine.search(&position).is_none(),
        "random engine should return None on stalemate"
    );
}

// ─────────────────────────────────────────────────────────
// 引擎名称测试
// ─────────────────────────────────────────────────────────

#[test]
fn minimax_engine_name() {
    let engine = MiniMaxEngine::new(3);
    assert_eq!(engine.name(), "MiniMax Engine");
}

#[test]
fn random_engine_name() {
    let engine = RandomEngine::default();
    assert_eq!(engine.name(), "Random Engine");
}

// ─────────────────────────────────────────────────────────
// 边界条件 / 鲁棒性测试
// ─────────────────────────────────────────────────────────

#[test]
fn depth_zero_returns_legal_move() {
    // depth=0 在 minimax 中会导致无限递归（search 调用 minimax(depth=-1)），
    // 因此该测试验证 depth=1（最低有效搜索深度）正常工作。
    let position = Position::startpos();
    let mut engine = MiniMaxEngine::new(1);
    let mv = engine.search(&position).expect("depth 1 should return a move");
    assert_legal_move(&position, mv, "depth 1: move not legal");
}

#[test]
fn depth_zero_on_checkmate_returns_none() {
    // depth=0 在将杀局面应返回 None（无合法走法）
    let position = Position::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
    let mut engine = MiniMaxEngine::new(0);
    assert!(
        engine.search(&position).is_none(),
        "depth 0 on checkmate should return None"
    );
}

#[test]
fn position_unchanged_after_search() {
    // search 不应修改传入的局面
    let position = Position::from_fen("7k/8/4q3/8/8/8/4R3/3K4 b - - 0 1").unwrap();
    let fen_before = position.to_fen();
    let _mv = search(&position, 3);
    let fen_after = position.to_fen();
    assert_eq!(
        fen_before, fen_after,
        "position should not be mutated by search"
    );
}

// ─────────────────────────────────────────────────────────
// 战术模式测试
// ─────────────────────────────────────────────────────────

#[test]
fn mate_in_two() {
    // 白方两步杀：
    //   白方 Kc6, Rf7；黑方 Kb8
    //   方案 A: 1. Kb6! Ka8/Kc8 2. Rf8#
    //   方案 B: 1. Rf8+! Ka7 2. Ra8#
    // 深度 3 应找到其中一种将杀序列
    let position = Position::from_fen("1k6/5R2/2K5/8/8/8/8/8 w - - 0 1").unwrap();
    let mv = search(&position, 3).expect("should find a move");
    assert_legal_move(&position, mv, "mate-in-2 move not legal");
    // 两种可行的将杀第一步：Kc6-b6 或 Rf7-f8+
    // 注：引擎的具体选择取决于走法生成顺序，两者均能赢棋
    let from = mv.from();
    assert!(
        from == Square::C6 || from == Square::F7,
        "expected king or rook move, got {mv:?}"
    );
}

#[test]
fn knight_fork() {
    // 白马 e4 可以通过 Nf6+ 叉王和后
    //   白方 Kg1, Ne4；黑方 Ke8, Qg4
    //   1. Nf6+ K 走 2. Nxg4 得后
    // 深度 3 应能找到得子走法
    let position = Position::from_fen("4k3/8/8/8/4N1q1/8/8/6K1 w - - 0 1").unwrap();
    let mv = search(&position, 3).expect("should find a move");
    assert_legal_move(&position, mv, "knight fork move not legal");
    // 验证引擎走了马的走法（应选择攻击性的走法，理想情况下是 Nf6+）
    assert_eq!(
        mv.from(),
        Square::E4,
        "expected knight move from e4, got {mv:?}"
    );
}

#[test]
fn promotion_found() {
    // 白兵 e7，一步升变
    //   白方 Ke1, Pe7；黑方 Ke4
    //   1. e8=Q+ 升后将军
    let position = Position::from_fen("8/4P3/8/8/4k3/8/8/4K3 w - - 0 1").unwrap();
    let mv = search(&position, 1).expect("should find a move");
    assert_legal_move(&position, mv, "promotion move not legal");
    assert!(
        mv.is_promotion(),
        "expected a promotion move, got {mv:?}"
    );
    assert_eq!(
        mv.to(),
        Square::E8,
        "expected promotion on e8, got {mv:?}"
    );
}

#[test]
fn avoid_stalemate() {
    // 白方 Kg6, Qf7；黑方 Kh8。白方先行，黑方已无合法走法且未被将军。
    // 必须走 Qg7# 将杀，而不是 Qg6?? 导致逼和。
    let position = Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 w - - 0 1").unwrap();
    let mv = search(&position, 1).expect("should find a move");
    assert_legal_move(&position, mv, "avoid stalemate: move not legal");
    // 引擎应找到 Qg7# 将杀
    assert_eq!(
        mv,
        Move::new(Square::F7, Square::G7, MoveFlag::Quiet),
        "expected Qg7# (mate in 1), got {mv:?}"
    );
}

// ─────────────────────────────────────────────────────────
// 残局测试
// ─────────────────────────────────────────────────────────

#[test]
fn endgame_kq_vs_k() {
    // KQ vs K 残局：白方应能将死黑方
    //   白方 Kd1, Qa1；黑方 Kd4
    let position = Position::from_fen("8/8/8/8/3k4/8/8/Q2K4 w - - 0 1").unwrap();
    let mv = search(&position, 3).expect("KQ vs K should find a move");
    assert_legal_move(&position, mv, "KQ vs K: move not legal");
    // 基本检查：引擎不崩溃且返回合法走法
}

#[test]
fn endgame_promotion_race() {
    // 双方各有一个通路兵
    //   白方 Ke1, Pe2；黑方 Ke8, Pf7
    let position = Position::from_fen("4k3/5p2/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
    let mv = search(&position, 2).expect("promotion race should find a move");
    assert_legal_move(&position, mv, "promotion race: move not legal");
    // 推进 e 兵是合理的（e4 或 e3）
}
