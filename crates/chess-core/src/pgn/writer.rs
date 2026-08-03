//! PGN 输出：将 [`Game`] 序列化为 PGN 文本

use super::san::move_to_san;
use crate::{Game, Position, makemove::make_move};

/// 将对局写为完整的 PGN 文本
///
/// 包含：头信息、空行、走法文本（带编号和结果）
/// 走法文本以 80 字符为上限换行
pub fn write_pgn(game: &Game) -> String {
    let mut result = String::new();

    // 头信息
    for (key, value) in game.headers() {
        result.push_str(&format!("[{} \"{}\"]\n", key, value));
    }

    // 空行分隔
    result.push('\n');

    // 走法文本
    let move_text = format_move_text(game);
    result.push_str(&wrap_text(&move_text, 80));
    result.push('\n');

    result
}

/// 生成走法文本（不含换行）
fn format_move_text(game: &Game) -> String {
    let history = game.history();
    if history.is_empty() {
        return String::from("*");
    }

    let mut result = String::new();
    let mut position = Position::startpos();

    // 需要从头回放走法以获取每个走法前的局面
    // 但 history 中的 Moves 是已经执行过的，我们需要"撤销"或"重建"
    // 简单方案：从头开始，对每个走法调用 move_to_san，然后执行

    for (i, (mv, _undo)) in history.iter().enumerate() {
        // 白方走法前加编号
        if i % 2 == 0 {
            let move_number = i / 2 + 1;
            result.push_str(&format!("{}. ", move_number));
        }

        // 生成 SAN
        let san = move_to_san(&position, *mv).unwrap_or_else(|_| "?".to_string());
        result.push_str(&san);

        // 执行走法以更新局面
        make_move(&mut position, *mv);

        // 空格分隔
        if i + 1 < history.len() {
            result.push(' ');
        }
    }

    // 附加结果
    let game_result = game.result();
    result.push(' ');
    result.push_str(game_result);

    result
}

/// 将文本按指定宽度换行
///
/// 不在单词中间断行，以空格为换行点
fn wrap_text(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        return text.to_string();
    }

    let mut result = String::new();
    let mut line_start = 0;
    let bytes = text.as_bytes();

    while line_start < text.len() {
        let mut line_end = (line_start + max_width).min(text.len());

        // 如果还没到末尾，回退到最近的空格
        if line_end < text.len() {
            // 在 line_start..line_end 范围内找最后一个空格
            let slice = &text[line_start..line_end];
            if let Some(last_space) = slice.rfind(' ') {
                line_end = line_start + last_space;
            }
        }

        result.push_str(&text[line_start..line_end]);
        result.push('\n');

        // 跳过空格继续
        line_start = line_end;
        while line_start < bytes.len() && bytes[line_start] == b' ' {
            line_start += 1;
        }
    }

    // 移除末尾多余的换行
    result.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    #[test]
    fn test_write_empty_game() {
        let game = Game::new();
        let pgn = write_pgn(&game);
        assert!(pgn.contains('*'), "empty game should have * result");
    }

    #[test]
    fn test_write_with_headers() {
        let mut game = Game::new();
        game.set_header("Event", "Test");
        game.set_header("White", "Player A");
        game.set_header("Result", "*");

        let pgn = write_pgn(&game);
        assert!(pgn.contains("[Event \"Test\"]"));
        assert!(pgn.contains("[White \"Player A\"]"));
    }

    #[test]
    fn test_write_single_move() {
        let mut game = Game::new();
        // e2-e4
        let mv = crate::Move::new(
            crate::Square::E2,
            crate::Square::E4,
            crate::MoveFlag::DoublePawnPush,
        );
        game.play(mv).unwrap();

        let pgn = write_pgn(&game);
        assert!(
            pgn.contains("1. e4"),
            "expected '1. e4' in PGN output, got: {pgn}"
        );
    }

    #[test]
    fn test_wrap_text() {
        let text = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3";
        let wrapped = wrap_text(text, 50);
        for line in wrapped.lines() {
            assert!(line.len() <= 50, "line too long: {line}");
        }
    }
}
