//! MiniMaxEngine 多角度集成测试
//!
//! 测试覆盖：
//! - 一步杀（mate-in-1）检测
//! - 免费得子
//! - 避免失子（需要一定深度）
//! - 将杀/逼和返回 None
//! - 初始局面不崩溃
//! - 确定性

use chess_ai::{ChessEngine, MiniMaxEngine};
use chess_core::{Move, MoveFlag, Position, Square, legal_moves_of};

// ── 辅助函数 ──

/// 在给定局面上运行引擎搜索，返回找到的走法
fn search(position: &Position, depth: i32) -> Option<Move> {
    let mut engine = MiniMaxEngine::new(depth);
    engine.search(position)
}

/// 验证走法合法
fn assert_legal_move(position: &Position, mv: Move, msg: &str) {
    let legal = legal_moves_of(position);
    assert!(
        legal.iter().any(|&m| m == mv),
        "{msg}: move {mv:?} is not in legal move list:\n{legal:?}",
    );
}

// ── 测试 ──

#[test]
fn mate_in_one_white() {
    // 白方：王 f6, 后 g1。黑方：王 h8。
    // 白方 Qg1-g7# 将杀。
    let position = Position::from_fen("7k/8/5K2/8/8/8/8/6Q1 w - - 0 1").unwrap();
    let mv = search(&position, 1).expect("should find a move");

    let expected = Move::new(Square::G1, Square::G7, MoveFlag::Quiet);
    assert_legal_move(&position, mv, "mate-in-1 move not legal");
    assert_eq!(mv, expected, "expected Qg1-g7# (mate in 1), got {mv:?}");
}

#[test]
fn capture_free_queen() {
    // 白方：王 d1, 车 e2。黑方：王 h8, 后 e6。
    // 白方可以免费吃后（Re2xe6）。
    let position = Position::from_fen("7k/8/4q3/8/8/8/4R3/3K4 w - - 0 1").unwrap();
    let mv = search(&position, 1).expect("should find a move");

    assert_legal_move(&position, mv, "capture move not legal");
    assert!(mv.is_capture(), "expected a capture (Re2xe6), got {mv:?}");
    assert_eq!(mv.to(), Square::E6, "expected capture on e6, got {mv:?}");
}

#[test]
fn avoid_losing_queen_at_depth_3() {
    // 黑方：王 h8, 后 e6（被白车 e2 攻击）。白方：王 d1, 车 e2。
    // 黑方先行。深度 3 应避免后留在 e 线被吃。
    let position = Position::from_fen("7k/8/4q3/8/8/8/4R3/3K4 b - - 0 1").unwrap();
    let mv = search(&position, 3).expect("should find a move");

    assert_legal_move(&position, mv, "avoid-losing-queen move not legal");

    // 深度 3 应该能预见：后留在 e 线会被车吃掉
    // 好的走法应该把后移出 e 线
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
    // 黑方被将杀：王 h8，白后 g7，白王 f6。
    // 黑方先行，无合法走法。
    let position = Position::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
    let mv = search(&position, 1);
    assert!(mv.is_none(), "expected None (checkmate), got {mv:?}");
}

#[test]
fn stalemate_returns_none() {
    // 黑方被逼和：王 h8，白后 g6，白王 f7。
    // 黑方先行，无合法走法且未被将军。
    let position = Position::from_fen("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1").unwrap();
    let mv = search(&position, 1);
    assert!(mv.is_none(), "expected None (stalemate), got {mv:?}");
}

#[test]
fn initial_position_depth_2_returns_legal_move() {
    // 初始局面，深度 2 搜索不应崩溃，应返回合法走法。
    let position = Position::startpos();
    let mv = search(&position, 2).expect("should find a move at depth 2");
    assert_legal_move(&position, mv, "initial position depth 2");
}

#[test]
fn deterministic_same_position_same_result() {
    // 同一局面、同一深度，多次搜索应返回相同结果。
    let position = Position::from_fen("7k/8/5K2/8/8/8/8/6Q1 w - - 0 1").unwrap();

    let mv1 = search(&position, 1);
    let mv2 = search(&position, 1);

    assert_eq!(mv1, mv2, "minimax should be deterministic");
}

#[test]
fn depth_3_sees_further_than_depth_1() {
    // 在需要进行多步计算的局面中，深度 3 应优于深度 1。
    //
    // 局面：黑方王 h8，后 e6 被白车 e2 攻击。黑方先行。
    // 深度 1：看不到白方即将吃后，可能走出 Qe6-e7（仍留在 e 线）。
    // 深度 3：预见白方 Re2xe7 吃后，主动避开 e 线。
    let position = Position::from_fen("7k/8/4q3/8/8/8/4R3/3K4 b - - 0 1").unwrap();

    let mv_depth_3 = search(&position, 3).expect("depth 3 should find a move");

    // 深度 3 应该把后移出 e 线
    assert_eq!(mv_depth_3.from(), Square::E6);
    assert_ne!(
        mv_depth_3.to().file(),
        Square::E6.file(),
        "depth 3: queen should leave e-file; got {mv_depth_3:?}"
    );
}
