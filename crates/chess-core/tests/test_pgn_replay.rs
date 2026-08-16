//! 经典对局重现测试
//!
//! 从 `tests/resources/famous_games/` 读取 PGN 文件，
//! 逐着重现经典对局并验证结果

use chess_core::Game;

/// 重现 Fischer vs Spassky 1972 年第 6 局（41 回合，Fischer 胜）
#[test]
fn replay_fischer_spassky_1972_game6() {
    let pgn = include_str!("resources/famous_games/fischer-spassky-1972-game6.pgn");
    let game = Game::from_pgn(pgn).expect("应成功解析 Fischer-Spassky 1972 Game 6");

    // 验证头信息
    assert_eq!(game.header("White").unwrap(), "Bobby Fischer");
    assert_eq!(game.header("Black").unwrap(), "Boris Spassky");
    assert_eq!(game.result(), "1-0");

    // 验证走法数：41 回合 = 40 对走法 + 最后一着白方 = 81 个半回合
    assert_eq!(game.move_history().len(), 81);

    // 验证结果头
    assert_eq!(game.result(), "1-0");
}

/// 重现 Kasparov vs Topalov 1999 年 Wijk aan Zee 对局（44 回合，Kasparov 胜）
#[test]
fn replay_kasparov_topalov_1999() {
    let pgn = include_str!("resources/famous_games/kasparov-topalov-1999.pgn");
    let game = Game::from_pgn(pgn).expect("应成功解析 Kasparov-Topalov 1999");

    // 验证头信息
    assert_eq!(game.header("White").unwrap(), "Garry Kasparov");
    assert_eq!(game.header("Black").unwrap(), "Veselin Topalov");
    assert_eq!(game.result(), "1-0");

    // 验证走法数：44 回合 = 43 对走法 + 最后一着白方 = 87 个半回合
    assert_eq!(game.move_history().len(), 87);
}

/// 往返测试：解析 -> 序列化 -> 再解析，确保走法一致
#[test]
fn pgn_round_trip() {
    let pgn = include_str!("resources/famous_games/fischer-spassky-1972-game6.pgn");
    let game1 = Game::from_pgn(pgn).unwrap();
    let exported = game1.export_pgn();
    let game2 = Game::from_pgn(&exported).unwrap();

    assert_eq!(game1.move_history().len(), game2.move_history().len());
    assert_eq!(game1.result(), game2.result());

    // 验证每步走法相同
    for (i, (mv1, mv2)) in game1
        .move_history()
        .iter()
        .zip(game2.move_history().iter())
        .enumerate()
    {
        assert_eq!(
            mv1, mv2,
            "round-trip move mismatch at ply {}: {:?} vs {:?}",
            i, mv1, mv2
        );
    }
}
