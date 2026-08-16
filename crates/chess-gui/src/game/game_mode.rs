use std::fmt::Display;

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

impl Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            GameMode::HumanVsHuman => "Mode: Human vs Human",
            GameMode::HumanVsAI => "Mode: Human vs AI",
            GameMode::Analysis => "Mode: Analysis",
            GameMode::Replay => "Mode: Replay",
        };
        write!(f, "{}", content)
    }
}
