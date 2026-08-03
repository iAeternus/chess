mod parser;
mod san;
mod writer;

pub use parser::parse_pgn;
pub use writer::write_pgn;
pub use san::{move_to_san, parse_san};

use crate::{Game, Result};

/// 从 PGN 文本解析对局
///
/// # 示例
///
/// ```ignore
/// let pgn = r#"[Event "Test"]
/// [Site "Nowhere"]
/// [Date "2024.01.01"]
/// [Round "1"]
/// [White "A"]
/// [Black "B"]
/// [Result "1-0"]
///
/// 1. e4 e5 2. Nf3 Nc6 3. Bb5 1-0"#;
/// let game = chess_core::from_pgn(pgn).unwrap();
/// ```
pub fn from_pgn(pgn: &str) -> Result<Game> {
    parse_pgn(pgn)
}

/// 将对局序列化为 PGN 文本
pub fn to_pgn(game: &Game) -> String {
    write_pgn(game)
}
