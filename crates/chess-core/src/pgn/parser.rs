//! PGN 解析器
//!
//! 将 PGN 文本解析为 [`ParsedPgn`]，包含头信息和对局走法

use super::san::parse_san;
use crate::{ChessError, Move, Position, Result, make_move};

#[derive(Debug, Clone)]
pub struct ParsedPgn {
    pub start_position: Position,
    pub moves: Vec<Move>,
    pub headers: Vec<(String, String)>,
}

/// 解析完整的 PGN 文本，返回包含所有走法的 ParsedPgn
///
/// 解析过程：
/// 1. 提取头信息 `[Key "Value"]`
/// 2. 提取走法文本，跳过注释、变着、NAG
/// 3. 逐着解析 SAN 并执行
pub fn parse_pgn(pgn: &str) -> Result<ParsedPgn> {
    let pgn = pgn.trim();
    if pgn.is_empty() {
        return Err(ChessError::ParseError("empty PGN".into()));
    }

    // 解析头信息
    let (headers, rest) = parse_headers(pgn)?;

    // 提取走法文本
    let move_text = rest.trim();
    if move_text.is_empty() {
        return Err(ChessError::ParseError("PGN has no move text".into()));
    }

    // 提取 SAN 走法列表
    let sans = extract_san_moves(move_text)?;
    if sans.is_empty() {
        return Err(ChessError::ParseError("PGN has no moves".into()));
    }

    // 确定起始局面
    let start_position = determine_start_position(&headers)?;

    // 逐着解析 SAN，但不执行，仅生成 Move 列表
    let mut moves = Vec::with_capacity(sans.len());
    // 使用一个临时的 Position 来解析 SAN（SAN 解析需要当前局面）
    let mut current_pos = start_position.clone();

    for san in &sans {
        let mv = parse_san(&current_pos, san)
            .map_err(|e| ChessError::ParseError(format!("illegal SAN '{}': {}", san, e)))?;
        // 注意：这里需要用底层 make_move 更新 current_pos，不检查合法性（因为 parse_san 已保证合法）
        make_move(&mut current_pos, mv);
        moves.push(mv);
    }

    Ok(ParsedPgn {
        start_position,
        moves,
        headers,
    })
}

/// 从 PGN 头信息中提取起始局面
fn determine_start_position(headers: &[(String, String)]) -> Result<Position> {
    // 查找 FEN 头信息
    let fen = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("FEN"))
        .map(|(_, v)| v.as_str());

    if let Some(fen) = fen {
        // 如果有 FEN，必须同时有 SetUp "1"
        let setup = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("SetUp"))
            .map(|(_, v)| v.as_str());

        if setup != Some("1") {
            // 根据 PGN 规范，有 FEN 就应有 SetUp "1"，但有些宽松实现可忽略此检查
            // 这里选择宽松处理：只要有 FEN 就使用它
        }

        Position::from_fen(fen)
            .map_err(|e| ChessError::ParseError(format!("invalid FEN in PGN: {}", e)))
    } else {
        // 没有 FEN，使用标准起始局面
        Ok(Position::startpos())
    }
}

/// 解析 PGN 头信息
///
/// 格式：`[Key "Value"]`，每行一个
/// 遇到空行或非 `[` 开头的行时停止
fn parse_headers(pgn: &str) -> Result<(Vec<(String, String)>, &str)> {
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut remaining = pgn;

    loop {
        let line = remaining.trim_start();
        if !line.starts_with('[') {
            break;
        }

        // 找到对应的 ]
        let close_bracket = line
            .find(']')
            .ok_or_else(|| ChessError::ParseError("unclosed header bracket".into()))?;

        let header_content = &line[1..close_bracket];
        remaining = &line[close_bracket + 1..];

        // 解析 Key "Value"
        let (key, value) = parse_header_pair(header_content)?;
        headers.push((key, value));
    }

    Ok((headers, remaining))
}

/// 解析单个头信息键值对
fn parse_header_pair(content: &str) -> Result<(String, String)> {
    let content = content.trim();

    // 找到 Key 和 Value 的分界（第一个空格或第一个 `"` 之前）
    let key_end = content
        .find('"')
        .map(|i| content[..i].trim_end().len())
        .unwrap_or_else(|| content.find(' ').unwrap_or(content.len()));

    let key = content[..key_end].trim().to_string();
    let value_part = content[key_end..].trim();

    // 解析引号内的值
    if !value_part.starts_with('"') {
        return Err(ChessError::ParseError(format!(
            "invalid header value: {content}"
        )));
    }

    let value = parse_quoted_string(&value_part[1..])
        .ok_or_else(|| ChessError::ParseError(format!("unclosed quote in header: {content}")))?;

    Ok((key, value.0.to_string()))
}

/// 解析引号内的字符串（处理转义 `\"`）
/// 返回 (值, 剩余文本)
fn parse_quoted_string(s: &str) -> Option<(&str, &str)> {
    let mut chars = s.char_indices();
    let mut result_end = None;

    while let Some((i, c)) = chars.next() {
        match c {
            '"' => {
                result_end = Some(i);
                break;
            }
            '\\' => {
                // 跳过下一个字符（转义）
                chars.next();
            }
            _ => {}
        }
    }

    result_end.map(|i| (&s[..i], &s[i + 1..]))
}

/// 从走法文本中提取 SAN 走法
///
/// 跳过：
/// - 走法编号（如 `1.`、`1...`）
/// - 花括号注释 `{...}`
/// - 括号变着 `(...)`
/// - NAG `$数字`
///
/// 遇到结果标记（`1-0`、`0-1`、`1/2-1/2`、`*`）时停止
fn extract_san_moves(text: &str) -> Result<Vec<String>> {
    let mut sans: Vec<String> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // 跳过空白
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // 跳过花括号注释
        if c == '{' {
            i += 1;
            let mut depth = 1;
            while i < len && depth > 0 {
                if chars[i] == '{' {
                    depth += 1;
                } else if chars[i] == '}' {
                    depth -= 1;
                }
                i += 1;
            }
            continue;
        }

        // 跳过括号变着
        if c == '(' {
            i += 1;
            let mut depth = 1;
            while i < len && depth > 0 {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                }
                i += 1;
            }
            continue;
        }

        // 跳过 NAG
        if c == '$' {
            i += 1;
            while i < len && chars[i].is_ascii_digit() {
                i += 1;
            }
            continue;
        }

        // 跳过走法编号（如 "3." 或 "3..."）
        if c.is_ascii_digit() {
            let start = i;
            while i < len && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i < len && chars[i] == '.' {
                i += 1;
                // 可能是 "3..."
                while i < len && chars[i] == '.' {
                    i += 1;
                }
                continue;
            }
            // 不是走法编号，回退
            i = start;
        }

        // 检查结果标记
        if i + 2 < len {
            let slice: String = chars[i..i + 3].iter().collect();
            if slice == "1-0" || slice == "0-1" {
                break;
            }
        }
        if i + 6 < len {
            let slice: String = chars[i..i + 7].iter().collect();
            if slice == "1/2-1/2" {
                break;
            }
        }
        if c == '*' {
            break;
        }

        // 读取一个 SAN 单词
        let start = i;
        while i < len
            && !chars[i].is_whitespace()
            && chars[i] != '{'
            && chars[i] != '('
            && chars[i] != ')'
            && chars[i] != '$'
        {
            // 检查是否遇到走法编号
            if chars[i].is_ascii_digit()
                && i + 1 < len
                && (chars[i + 1] == '.' || chars[i + 1].is_ascii_digit())
                && i > start
            {
                // 在单词中间遇到可能的走法编号（如 "e5 2.Nf3"）
                // 简单处理：非起始位置的数字.组合交给主循环处理
                break;
            }
            i += 1;
        }

        let word: String = chars[start..i].iter().collect();
        let word = word.trim();

        if !word.is_empty() {
            sans.push(word.to_string());
        }
    }

    Ok(sans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_header() {
        let input = r#"[Event "Test Event"]
        [Site "Test Site"]

        1. e4 1-0"#;
        let (headers, rest) = parse_headers(input).unwrap();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].0, "Event");
        assert_eq!(headers[0].1, "Test Event");
        assert_eq!(headers[1].0, "Site");
        assert_eq!(headers[1].1, "Test Site");
        assert!(rest.trim().starts_with("1. e4"));
    }

    #[test]
    fn test_parse_headers_escaped_quote() {
        let input = r#"[Event "He said \"hello\""]

        1. e4"#;
        let (headers, _) = parse_headers(input).unwrap();
        assert_eq!(headers[0].1, r#"He said \"hello\""#);
    }

    #[test]
    fn test_extract_san_moves_simple() {
        let text = "1. e4 e5 2. Nf3 Nc6 3. Bb5 1-0";
        let sans = extract_san_moves(text).unwrap();
        assert_eq!(sans, vec!["e4", "e5", "Nf3", "Nc6", "Bb5"]);
    }

    #[test]
    fn test_extract_san_skips_comments() {
        let text = "1. e4 { good move } e5 2. Nf3 1-0";
        let sans = extract_san_moves(text).unwrap();
        assert_eq!(sans, vec!["e4", "e5", "Nf3"]);
    }

    #[test]
    fn test_extract_san_skips_variations() {
        let text = "1. e4 ( 1. d4 d5 ) e5 2. Nf3 1-0";
        let sans = extract_san_moves(text).unwrap();
        assert_eq!(sans, vec!["e4", "e5", "Nf3"]);
    }

    #[test]
    fn test_extract_san_skips_nag() {
        let text = "1. e4 $1 e5 2. Nf3 $6 1-0";
        let sans = extract_san_moves(text).unwrap();
        assert_eq!(sans, vec!["e4", "e5", "Nf3"]);
    }

    #[test]
    fn test_extract_san_draw_result() {
        let text = "1. e4 e5 2. Nf3 Nf6 1/2-1/2";
        let sans = extract_san_moves(text).unwrap();
        assert_eq!(sans, vec!["e4", "e5", "Nf3", "Nf6"]);
    }

    #[test]
    fn test_extract_san_star_result() {
        let text = "1. e4 e5 *";
        let sans = extract_san_moves(text).unwrap();
        assert_eq!(sans, vec!["e4", "e5"]);
    }

    #[test]
    fn test_parse_pgn_returns_headers_and_moves() {
        let pgn = r#"[Event "Test"]
        [Result "1-0"]

        1. e4 e5 1-0"#;

        let parsed = parse_pgn(pgn).unwrap();

        assert_eq!(
            parsed
                .headers
                .iter()
                .find(|(k, _)| k == "Event")
                .map(|(_, v)| v.as_str()),
            Some("Test")
        );
        assert_eq!(parsed.moves.len(), 2);
        assert_eq!(parsed.start_position, Position::startpos());
    }

    #[test]
    fn test_empty_pgn_error() {
        assert!(parse_pgn("").is_err());
    }

    #[test]
    fn test_only_headers_error() {
        assert!(parse_pgn("[Event \"Test\"]\n").is_err());
    }
}
