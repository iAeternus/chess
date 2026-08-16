//! Standard Algebraic Notation (SAN) 解析与生成
//!
//! SAN 是 PGN 格式中使用的走法表示法，例如 `e4`、`Nf3`、`O-O`、`exd5`、`e8=Q`
//!
//! 核心函数：
//! - `parse_san`：将 SAN 字符串解析为 `Move`
//! - `move_to_san`：将 `Move` 转换为 SAN 字符串

use crate::{
    ChessError, Move, MoveFlag, Piece, PieceKind, Position, Promotion, Result, Square,
    attack::is_square_attacked, makemove::make_move, movegen,
};

/// 将 SAN 字符串解析为当前局面下的唯一合法走法
///
/// SAN 是上下文相关的——同一字符串在不同局面下可能对应不同走法
/// 此函数通过生成所有合法走法并逐一匹配来找到唯一对应的走法
pub fn parse_san(position: &Position, san: &str) -> Result<Move> {
    let original = san.trim();
    if original.is_empty() {
        return Err(ChessError::ParseError("empty SAN string".into()));
    }

    // 处理易位
    if original == "O-O" || original == "0-0" {
        return find_castle(position, MoveFlag::KingCastle, original);
    }
    if original == "O-O-O" || original == "0-0-0" {
        return find_castle(position, MoveFlag::QueenCastle, original);
    }

    // 去除将军/将杀后缀
    let mut san = original.to_string();
    while san.ends_with('+') || san.ends_with('#') {
        san.pop();
    }

    // 提取升变
    let promotion = if let Some(eq_pos) = san.find('=') {
        let promo_char = san.chars().nth(eq_pos + 1).ok_or_else(|| {
            ChessError::ParseError(format!("invalid promotion in SAN: {original}"))
        })?;
        let promo = promotion_from_char(promo_char).ok_or_else(|| {
            ChessError::ParseError(format!(
                "invalid promotion piece '{}' in SAN: {original}",
                promo_char
            ))
        })?;
        san = san[..eq_pos].to_string();
        Some(promo)
    } else {
        None
    };

    // 目标格为最后两个字符
    if san.len() < 2 {
        return Err(ChessError::ParseError(format!(
            "invalid SAN (too short): {original}"
        )));
    }
    let target_str = &san[san.len() - 2..];
    let target = parse_square(target_str).ok_or_else(|| {
        ChessError::ParseError(format!(
            "invalid target square '{}' in SAN: {original}",
            target_str
        ))
    })?;
    san.truncate(san.len() - 2);

    // 判断棋子类型和消歧义信息
    let (piece_kind, disambig) =
        if san.is_empty() || !san.starts_with(|c: char| c.is_ascii_uppercase()) {
            // 兵走法：如 e4, exd5, e8=Q
            (PieceKind::Pawn, san.as_str())
        } else {
            // 非兵走法：如 Nf3, R1e3, Nbd7
            let kind = piece_kind_from_char(san.chars().next().unwrap()).ok_or_else(|| {
                ChessError::ParseError(format!("invalid piece letter in SAN: {original}"))
            })?;
            let rest = &san[1..];
            (kind, rest)
        };

    // 解析消歧义信息和吃子标记
    let is_capture = disambig.contains('x');
    let disambig_clean: String = disambig.chars().filter(|c| *c != 'x').collect();

    let disambig_file: Option<u8> = disambig_clean
        .chars()
        .find(|c| ('a'..='h').contains(c))
        .map(|c| c as u8 - b'a');
    let disambig_rank: Option<u8> = disambig_clean
        .chars()
        .find(|c| ('1'..='8').contains(c))
        .map(|c| c as u8 - b'1');

    // 生成所有合法走法并匹配
    let legal_moves = movegen::generate_legal2(position);

    let candidates: Vec<Move> = legal_moves
        .into_iter()
        .filter(|mv| {
            // 匹配目标格
            if mv.to() != target {
                return false;
            }

            // 匹配棋子类型
            let piece = position.piece_at(mv.from());
            if piece.map(|p| p.kind) != Some(piece_kind) {
                return false;
            }

            // 匹配升变
            if mv.promotion() != promotion {
                return false;
            }

            // 匹配吃子标记
            let mv_is_capture = mv.is_capture() || mv.flag() == MoveFlag::EnPassant;
            if is_capture && !mv_is_capture {
                return false;
            }
            // 如果是非吃子兵走法，不应匹配吃子走法
            if !is_capture && piece_kind == PieceKind::Pawn && mv_is_capture {
                return false;
            }

            // 匹配消歧义
            if let Some(file) = disambig_file
                && mv.from().file() != file
            {
                return false;
            }
            if let Some(rank) = disambig_rank
                && mv.from().rank() != rank
            {
                return false;
            }

            true
        })
        .collect();

    match candidates.len() {
        1 => Ok(candidates[0]),
        0 => Err(ChessError::ParseError(format!(
            "illegal SAN '{original}': no matching legal move"
        ))),
        _ => Err(ChessError::ParseError(format!(
            "ambiguous SAN '{original}': {} matches",
            candidates.len()
        ))),
    }
}

/// 将走法转换为 SAN 字符串
///
/// 需要局面上下文以生成消歧义信息和将军/将杀后缀
pub fn move_to_san(position: &Position, mv: Move) -> Result<String> {
    let moving_piece = position
        .piece_at(mv.from())
        .ok_or_else(|| ChessError::ParseError("move from empty square".into()))?;

    // 易位特殊处理
    match mv.flag() {
        MoveFlag::KingCastle => return Ok("O-O".to_string()),
        MoveFlag::QueenCastle => return Ok("O-O-O".to_string()),
        _ => {}
    }

    // 生成所有合法走法以判断消歧义需求
    let legal_moves = movegen::generate_legal2(position);

    // 找到所有与当前走法有相同目标格和棋子类型的走法
    let ambiguous: Vec<Move> = legal_moves
        .into_iter()
        .filter(|other| {
            other.to() == mv.to()
                && position.piece_at(other.from()).map(|p| p.kind) == Some(moving_piece.kind)
                && other.from() != mv.from()
        })
        .collect();

    // 确定消歧义字符串
    let disambig = disambiguation_string(mv.from(), &ambiguous, moving_piece);

    // 构建基础 SAN
    let mut result = String::new();

    if moving_piece.kind == PieceKind::Pawn {
        // 兵走法
        if mv.is_capture() || mv.flag() == MoveFlag::EnPassant {
            // 兵吃子：exd5 形式
            result.push(file_char(mv.from().file()));
            result.push('x');
        }
        // 目标格
        result.push_str(&format_square(mv.to()));
        // 升变
        if let Some(promotion) = mv.promotion() {
            result.push('=');
            result.push(promotion_to_char(promotion));
        }
    } else {
        // 非兵走法
        result.push(piece_kind_to_char(moving_piece.kind));
        result.push_str(&disambig);
        if mv.is_capture() {
            result.push('x');
        }
        result.push_str(&format_square(mv.to()));
    }

    // 判断将军/将杀后缀
    let mut check_pos = position.clone();
    make_move(&mut check_pos, mv);
    let opponent = check_pos.side_to_move();
    let king_sq = check_pos.board().king_square(opponent);
    if is_square_attacked(check_pos.board(), king_sq, opponent.flip()) {
        // 检测将杀：对方是否有合法走法
        let opponent_moves = movegen::generate_legal(&mut check_pos);
        if opponent_moves.is_empty() {
            result.push('#');
        } else {
            result.push('+');
        }
    }

    Ok(result)
}

/// 寻找与指定易位类型匹配的走法
fn find_castle(position: &Position, flag: MoveFlag, original: &str) -> Result<Move> {
    let legal_moves = movegen::generate_legal2(position);
    for mv in legal_moves {
        if mv.flag() == flag {
            return Ok(mv);
        }
    }
    Err(ChessError::ParseError(format!(
        "illegal SAN '{original}': castling not allowed"
    )))
}

/// 确定消歧义字符串
fn disambiguation_string(from: Square, ambiguous: &[Move], piece: Piece) -> String {
    let style = disambiguation_style(from, ambiguous, piece);

    match style {
        Disambiguation::None => String::new(),
        Disambiguation::File => file_char(from.file()).to_string(),
        Disambiguation::Rank => rank_char(from.rank()).to_string(),
        Disambiguation::Both => format_square(from),
    }
}

enum Disambiguation {
    None,
    File,
    Rank,
    Both,
}

fn disambiguation_style(from: Square, ambiguous: &[Move], _piece: Piece) -> Disambiguation {
    if ambiguous.is_empty() {
        return Disambiguation::None;
    }

    let same_file = ambiguous.iter().any(|mv| mv.from().file() == from.file());
    let same_rank = ambiguous.iter().any(|mv| mv.from().rank() == from.rank());

    if same_file && same_rank {
        // 同时存在同列和同行的歧义走法 -> 需要完整的起始格
        Disambiguation::Both
    } else if same_file {
        // 存在同列的歧义走法 -> 用行消歧义
        Disambiguation::Rank
    } else {
        // 所有歧义走法都在不同列 -> 用列消歧义即可
        Disambiguation::File
    }
}

fn parse_square(s: &str) -> Option<Square> {
    let bytes = s.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let file = bytes[0];
    let rank = bytes[1];
    if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
        return None;
    }
    Square::from_coord(file - b'a', rank - b'1')
}

fn format_square(sq: Square) -> String {
    sq.to_string()
}

fn file_char(file: u8) -> char {
    (b'a' + file) as char
}

fn rank_char(rank: u8) -> char {
    (b'1' + rank) as char
}

fn piece_kind_from_char(c: char) -> Option<PieceKind> {
    match c {
        'K' => Some(PieceKind::King),
        'Q' => Some(PieceKind::Queen),
        'R' => Some(PieceKind::Rook),
        'B' => Some(PieceKind::Bishop),
        'N' => Some(PieceKind::Knight),
        _ => None,
    }
}

fn piece_kind_to_char(kind: PieceKind) -> char {
    match kind {
        PieceKind::King => 'K',
        PieceKind::Queen => 'Q',
        PieceKind::Rook => 'R',
        PieceKind::Bishop => 'B',
        PieceKind::Knight => 'N',
        PieceKind::Pawn => 'P', // 不应直接使用，兵有特殊处理
    }
}

fn promotion_from_char(c: char) -> Option<Promotion> {
    match c.to_ascii_uppercase() {
        'Q' => Some(Promotion::Queen),
        'R' => Some(Promotion::Rook),
        'B' => Some(Promotion::Bishop),
        'N' => Some(Promotion::Knight),
        _ => None,
    }
}

fn promotion_to_char(promo: Promotion) -> char {
    match promo {
        Promotion::Queen => 'Q',
        Promotion::Rook => 'R',
        Promotion::Bishop => 'B',
        Promotion::Knight => 'N',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MoveFlag, Promotion, Square};

    fn startpos() -> Position {
        Position::startpos()
    }

    #[test]
    fn test_pawn_push_e4() {
        let pos = startpos();
        let mv = parse_san(&pos, "e4").unwrap();
        assert_eq!(mv.from(), Square::E2);
        assert_eq!(mv.to(), Square::E4);
        assert_eq!(mv.flag(), MoveFlag::DoublePawnPush);
    }

    #[test]
    fn test_pawn_push_d5() {
        // 黑方 d5
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1")
            .unwrap();
        let mv = parse_san(&pos, "d5").unwrap();
        assert_eq!(mv.from(), Square::D7);
        assert_eq!(mv.to(), Square::D5);
    }

    #[test]
    fn test_knight_move_nf3() {
        let pos = startpos();
        let mv = parse_san(&pos, "Nf3").unwrap();
        assert_eq!(mv.from(), Square::G1);
        assert_eq!(mv.to(), Square::F3);
        assert_eq!(mv.flag(), MoveFlag::Quiet);
    }

    #[test]
    fn test_knight_move_nc3() {
        let pos = startpos();
        let mv = parse_san(&pos, "Nc3").unwrap();
        assert_eq!(mv.from(), Square::B1);
        assert_eq!(mv.to(), Square::C3);
    }

    #[test]
    fn test_pawn_capture_exd5() {
        let pos =
            Position::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1")
                .unwrap();
        let mv = parse_san(&pos, "exd5").unwrap();
        assert_eq!(mv.from(), Square::E4);
        assert_eq!(mv.to(), Square::D5);
        assert!(mv.is_capture());
    }

    #[test]
    fn test_castle_kingside() {
        let pos = Position::from_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPPBPPP/R1BQK2R w KQkq - 4 4",
        )
        .unwrap();
        let mv = parse_san(&pos, "O-O").unwrap();
        assert_eq!(mv.flag(), MoveFlag::KingCastle);
        assert_eq!(mv.from(), Square::E1);
        assert_eq!(mv.to(), Square::G1);
    }

    #[test]
    fn test_castle_kingside_zero() {
        let pos = Position::from_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPPBPPP/R1BQK2R w KQkq - 4 4",
        )
        .unwrap();
        let mv = parse_san(&pos, "0-0").unwrap();
        assert_eq!(mv.flag(), MoveFlag::KingCastle);
    }

    #[test]
    fn test_castle_queenside() {
        let pos =
            Position::from_fen("r3kb1r/pppq1ppp/2n1bn2/4p3/4P3/2N1BN2/PPPQBPPP/R3K2R w KQkq - 6 6")
                .unwrap();
        let mv = parse_san(&pos, "O-O-O").unwrap();
        assert_eq!(mv.flag(), MoveFlag::QueenCastle);
        assert_eq!(mv.from(), Square::E1);
        assert_eq!(mv.to(), Square::C1);
    }

    #[test]
    fn test_promotion_e8q() {
        // 黑王在 h8，e8 为空，兵可以安静升变
        let pos = Position::from_fen("7k/4P3/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mv = parse_san(&pos, "e8=Q").unwrap();
        assert_eq!(mv.from(), Square::E7);
        assert_eq!(mv.to(), Square::E8);
        assert_eq!(mv.promotion().unwrap(), Promotion::Queen);
    }

    #[test]
    fn test_promotion_capture_dxe8q() {
        // 黑: Ke7 Re8
        // 白: Pd7
        // 测试 dxe8=Q 吃车升变
        let pos = Position::from_fen("4r3/3P4/8/8/8/8/8/4K2k w - - 0 1").unwrap();

        let mv = parse_san(&pos, "dxe8=Q").unwrap();

        assert_eq!(mv.from(), Square::D7);
        assert_eq!(mv.to(), Square::E8);
        assert_eq!(mv.promotion(), Some(Promotion::Queen));
        assert!(mv.is_capture());
    }

    #[test]
    fn test_disambig_file_nc3() {
        // b1 马到 c3，无歧义
        let pos = startpos();
        let mv = parse_san(&pos, "Nc3").unwrap();
        assert_eq!(mv.from(), Square::B1);
        assert_eq!(mv.to(), Square::C3);
    }

    #[test]
    fn test_disambig_file_nbd2() {
        // 白方两个马（b1 和 f3）都能到 d2，用列消歧义
        // 构造：只有白马在 b1/f3，王在 e1，黑王在 e8，d2 为空
        let pos = Position::from_fen("4k3/8/8/8/8/5N2/8/1N2K3 w - - 0 1").unwrap();
        let mv = parse_san(&pos, "Nbd2").unwrap();
        assert_eq!(mv.from(), Square::B1);
        assert_eq!(mv.to(), Square::D2);
    }

    #[test]
    fn test_disambig_rank_rooks() {
        // 两个车在 a1 和 a8 都可以到 a4，指定 rank 5 的车
        let pos = Position::from_fen("R3k3/8/8/R7/8/8/8/4K3 w Q - 0 1").unwrap();
        // R5a4: rank 5 (= index 4) 的车到 a4
        let mv = parse_san(&pos, "R5a4").unwrap();
        assert_eq!(mv.from().rank(), 4);
        assert_eq!(mv.to(), Square::A4);
    }

    #[test]
    fn test_ignore_check_suffix() {
        // 黑车在 e2 将军白王 e1，白王可走 d1/f1
        let pos = Position::from_fen("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1").unwrap();
        let mv = parse_san(&pos, "Kd1+").unwrap();
        assert_eq!(mv.from(), Square::E1);
        assert_eq!(mv.to(), Square::D1);
    }

    #[test]
    fn test_ignore_checkmate_suffix() {
        // 白车 g1 将杀黑王 h8：Rg8#
        let pos = Position::from_fen("7k/8/4K3/8/8/8/8/6R1 w - - 0 1").unwrap();
        let mv = parse_san(&pos, "Rg8#").unwrap();
        assert_eq!(mv.from(), Square::G1);
        assert_eq!(mv.to(), Square::G8);
    }

    #[test]
    fn test_invalid_san_returns_error() {
        let pos = startpos();
        assert!(parse_san(&pos, "Xf3").is_err());
        assert!(parse_san(&pos, "").is_err());
        assert!(parse_san(&pos, "Nx9").is_err());
    }

    #[test]
    fn test_ambiguous_san_returns_error() {
        // 两个车在 a1 和 a8，都能到 a4，没有消歧义 -> 歧义错误
        let pos = Position::from_fen("R3k3/8/8/8/8/8/8/R3K3 w Q - 0 1").unwrap();
        assert!(parse_san(&pos, "Ra4").is_err());
    }

    // ── move_to_san 测试 ──

    #[test]
    fn test_move_to_san_pawn_push() {
        let pos = startpos();
        let mv = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        assert_eq!(move_to_san(&pos, mv).unwrap(), "e4");
    }

    #[test]
    fn test_move_to_san_knight() {
        let pos = startpos();
        let mv = Move::new(Square::G1, Square::F3, MoveFlag::Quiet);
        assert_eq!(move_to_san(&pos, mv).unwrap(), "Nf3");
    }

    #[test]
    fn test_move_to_san_castle() {
        let pos = Position::from_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPPBPPP/R1BQK2R w KQkq - 4 4",
        )
        .unwrap();
        let mv = Move::new(Square::E1, Square::G1, MoveFlag::KingCastle);
        assert_eq!(move_to_san(&pos, mv).unwrap(), "O-O");
    }

    #[test]
    fn test_move_to_san_capture() {
        let pos =
            Position::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1")
                .unwrap();
        let mv = Move::new(Square::E4, Square::D5, MoveFlag::Capture);
        assert_eq!(move_to_san(&pos, mv).unwrap(), "exd5");
    }

    #[test]
    fn test_move_to_san_promotion() {
        // 黑王在 h8，e8 为空，兵安静升变 e8=Q+
        // 注意：升变后的后在 e8 攻击 h8 的黑王，产生将军
        let pos = Position::from_fen("7k/4P3/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mv = Move::new_promotion(Square::E7, Square::E8, Promotion::Queen, false);
        assert_eq!(move_to_san(&pos, mv).unwrap(), "e8=Q+");
    }

    #[test]
    fn test_move_to_san_check() {
        // 白车 a1 到 a8，攻击 e8 黑王 -> 将军 (+)
        let pos = Position::from_fen("4k3/8/8/8/8/8/8/R3K3 w Q - 0 1").unwrap();
        let mv = Move::new(Square::A1, Square::A8, MoveFlag::Quiet);
        let result = move_to_san(&pos, mv).unwrap();
        assert!(
            result.contains('+'),
            "expected check suffix, got '{result}'"
        );
    }

    #[test]
    fn test_move_to_san_checkmate() {
        // 黑王 h8，白王 f6，白车 a8 -> Ra8 本身就是已经在那？不对
        // 构造将杀：白车从 a1 到 a8#，黑王在 h8，白王控制 g7/g8
        let pos = Position::from_fen("7k/6R1/5K2/8/8/8/8/R7 w - - 0 1").unwrap();
        // a8=空，白车从 a1 到 a8 将杀
        let mv = Move::new(Square::A1, Square::A8, MoveFlag::Quiet);
        let result = move_to_san(&pos, mv).unwrap();
        assert!(
            result.contains('#'),
            "expected checkmate suffix, got '{result}'"
        );
    }

    #[test]
    fn test_move_to_san_disambig_file() {
        // 白马在 b1 和 f3，都能到 d2，d2 为空
        let pos = Position::from_fen("4k3/8/8/8/8/5N2/8/1N2K3 w - - 0 1").unwrap();
        let mv = Move::new(Square::B1, Square::D2, MoveFlag::Quiet);
        let result = move_to_san(&pos, mv).unwrap();
        assert_eq!(result, "Nbd2");
    }

    #[test]
    fn test_round_trip() {
        // 对初始局面的 20 个合法走法进行往返测试
        let pos = startpos();
        let mut pos_mut = pos.clone();
        let legal = movegen::generate_legal(&mut pos_mut);
        for mv in legal {
            let san = move_to_san(&pos, mv).unwrap();
            let parsed = parse_san(&pos, &san).unwrap();
            assert_eq!(
                parsed, mv,
                "round-trip failed: {mv:?} -> '{san}' -> {parsed:?}"
            );
        }
    }
}
