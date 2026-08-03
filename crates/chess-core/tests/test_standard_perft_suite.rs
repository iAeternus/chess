use chess_core::{Position, perft};

struct PerftCase {
    name: &'static str,
    fen: &'static str,
    depth: u32,
    nodes: u64,
}

#[test]
#[ignore = "测试时长较长"]
fn standard_perft_suite() {
    let cases = [
        // 起始局面
        //
        // 覆盖：
        // - 基础兵移动
        // - 兵双步移动
        // - 马跳跃移动
        // - 基础合法性检查
        //
        // 标准结果：
        // depth 1: 20
        // depth 2: 400
        // depth 3: 8902
        // depth 4: 197281
        // depth 5: 4865609
        PerftCase {
            name: "startpos",
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            depth: 5,
            nodes: 4_865_609,
        },
        // Kiwipete 局面
        //
        // 经典综合测试局面
        //
        // 覆盖：
        // - 王车易位合法性
        // - 吃过路兵
        // - 升变
        // - 将军过滤
        // - 被钉住棋子
        // - King safety 检查
        //
        // 用于验证完整 move generator
        PerftCase {
            name: "kiwipete",
            fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            depth: 4,
            nodes: 4_085_603,
        },
        // CPW Position 3
        //
        // En passant pin 测试局面
        //
        // 覆盖：
        // - 吃过路兵合法性
        // - 被钉住兵处理
        // - 王安全检测
        // - 复杂残局走法生成
        PerftCase {
            name: "cpw_pos3",
            fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            depth: 5,
            nodes: 674_624,
        },
        // CPW Position 4
        //
        // 升变与特殊走法测试局面
        //
        // 覆盖：
        // - 兵升变
        // - 升变吃子
        // - 王车易位
        // - 将军状态变化
        // - 边界棋盘处理
        PerftCase {
            name: "cpw_pos4",
            fen: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            depth: 5,
            nodes: 15_833_292,
        },
        // CPW Position 5
        //
        // 复杂中局测试局面
        //
        // 覆盖：
        // - 吃子变化
        // - 棋子协同攻击
        // - 王安全判断
        // - make_move / unmake_move 状态恢复
        PerftCase {
            name: "cpw_pos5",
            fen: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            depth: 4,
            nodes: 2_103_487,
        },
        // CPW Position 6
        //
        // Steven Edwards 复杂中残局测试局面
        //
        // 覆盖：
        // - 多方向攻击
        // - 王移动合法性
        // - 滑动棋攻击范围
        // - 深层 make_move / unmake_move
        //
        // 用于检测长期搜索中的状态污染
        PerftCase {
            name: "cpw_pos6",
            fen: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            depth: 4,
            nodes: 3_894_594,
        },
    ];

    for case in cases {
        let mut position = Position::from_fen(case.fen).unwrap();
        let result = perft(&mut position, case.depth);
        assert_eq!(
            result, case.nodes,
            "perft failed for {}: expected {}, got {}",
            case.name, case.nodes, result
        );
    }
}
