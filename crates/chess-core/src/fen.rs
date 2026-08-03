//! Forsyth-Edwards Notation (FEN)
//!
//! FEN 用于描述国际象棋某一时刻的完整局面状态。
//! 标准 FEN 由 6 个字段组成：
//!
//! ```text
//! board side_to_move castling en_passant halfmove fullmove
//! ```
//!
//! 字段说明：
//!
//! - `board`：棋盘布局，从黑方第8行到白方第1行；`/`分隔行，数字表示连续空格，大写表示白子，小写表示黑子
//! - `side_to_move`：当前行动方；`w`表示白方，`b`表示黑方
//! - `castling`：王车易位权限；`K/Q`表示白王翼/后翼，`k/q`表示黑王翼/后翼，`-`表示无权限
//! - `en_passant`：吃过路兵目标格；记录上一回合兵两格移动经过的格子，无则为`-`
//! - `halfmove`：50步规则计数器；记录自上次兵移动或吃子后的半回合数
//! - `fullmove`：完整回合数；从1开始，黑方走完后递增
//!
//! 示例：
//!
//! ```text
//! rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
//! ```

use crate::{Board, CastlingRights, ChessError, Color, Piece, Position, Result, Square};

pub fn fen2position(fen: &str) -> Result<Position> {
    let fields = fen.split_whitespace().collect::<Vec<_>>();
    if fields.len() != 6 {
        return Err(ChessError::InvalidFen(fen.into()));
    }

    let board = parse_board(fields[0], fen)?;
    let side_to_move = parse_side_to_move(fields[1], fen)?;
    let castling = parse_castling(fields[2], fen)?;
    let en_passant = parse_en_passant(fields[3], fen)?;
    let halfmove_clock = parse_number(fields[4], fen)?;
    let fullmove_number = parse_number(fields[5], fen)?;

    Ok(Position::new(
        board,
        side_to_move,
        castling,
        en_passant,
        halfmove_clock,
        fullmove_number,
    ))
}

pub fn position2fen(position: &Position) -> String {
    format!(
        "{} {} {} {} {} {}",
        board2fen(position.board()),
        match position.side_to_move() {
            Color::White => "w",
            Color::Black => "b",
        },
        castling2fen(position.castling()),
        position
            .en_passant()
            .map(|s| s.to_string())
            .unwrap_or("-".into()),
        position.halfmove_clock(),
        position.fullmove_number()
    )
}

fn parse_board(s: &str, fen: &str) -> Result<Board> {
    let mut board = Board::default();
    let mut rank: u8 = 7; // 行
    let mut file: u8 = 0; // 列

    for c in s.chars() {
        match c {
            '/' => {
                if rank == 0 {
                    return Err(ChessError::InvalidFen(fen.into()));
                }
                rank -= 1;
                file = 0;
            }
            '1'..='8' => {
                // SAFETY: c is valid here
                file += c.to_digit(10).unwrap() as u8;
            }
            _ => {
                if file >= 8 {
                    return Err(ChessError::InvalidFen(fen.into()));
                }
                let piece =
                    Piece::from_char(c).ok_or_else(|| ChessError::InvalidFen(fen.into()))?;
                let sq = Square::from_coord(file, rank).unwrap(); // SAFETY: file and rank is valid here
                board.add_piece(sq, piece);
                file += 1;
            }
        }
    }

    Ok(board)
}

fn parse_side_to_move(s: &str, fen: &str) -> Result<Color> {
    match s {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(ChessError::InvalidFen(fen.into())),
    }
}

fn parse_castling(s: &str, fen: &str) -> Result<CastlingRights> {
    let mut rights = CastlingRights::empty();

    for c in s.chars() {
        match c {
            'K' => rights.insert(CastlingRights::WHITE_KING_SIDE),
            'Q' => rights.insert(CastlingRights::WHITE_QUEEN_SIDE),
            'k' => rights.insert(CastlingRights::BLACK_KING_SIDE),
            'q' => rights.insert(CastlingRights::BLACK_QUEEN_SIDE),
            '-' => {}
            _ => return Err(ChessError::InvalidFen(fen.into())),
        }
    }

    Ok(rights)
}

fn parse_en_passant(s: &str, fen: &str) -> Result<Option<Square>> {
    if s == "-" {
        return Ok(None);
    }

    let bytes = s.as_bytes();

    let [file, rank] = bytes else {
        return Err(ChessError::InvalidFen(fen.into()));
    };

    if !(b'a'..=b'h').contains(file) || !(b'1'..=b'8').contains(rank) {
        return Err(ChessError::InvalidFen(fen.into()));
    }

    let sq =
        Square::from_coord(file - b'a', rank - b'1').ok_or(ChessError::InvalidFen(fen.into()))?;
    Ok(Some(sq))
}

fn parse_number(number: &str, fen: &str) -> Result<u32> {
    number
        .parse::<u32>()
        .map_err(|_| ChessError::InvalidFen(fen.into()))
}

fn board2fen(board: &Board) -> String {
    let mut result = String::new();

    for rank in (0..8).rev() {
        let mut empty = 0;
        for file in 0..8 {
            let sq = Square::from_coord(file, rank).unwrap();
            match board.piece_at(sq) {
                Some(piece) => {
                    if empty > 0 {
                        result.push(char::from_digit(empty, 10).unwrap());
                        empty = 0;
                    }
                    result.push(piece.to_char());
                }
                None => {
                    empty += 1;
                }
            }
        }

        if empty > 0 {
            result.push(char::from_digit(empty, 10).unwrap());
        }
        if rank != 0 {
            result.push('/');
        }
    }

    result
}

fn castling2fen(rights: CastlingRights) -> String {
    let mut result = String::new();

    if rights.contains(CastlingRights::WHITE_KING_SIDE) {
        result.push('K');
    }

    if rights.contains(CastlingRights::WHITE_QUEEN_SIDE) {
        result.push('Q');
    }

    if rights.contains(CastlingRights::BLACK_KING_SIDE) {
        result.push('k');
    }

    if rights.contains(CastlingRights::BLACK_QUEEN_SIDE) {
        result.push('q');
    }

    if result.is_empty() {
        "-".to_string()
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, PieceKind};

    const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    #[test]
    fn test_start_position_round_trip() {
        let position = fen2position(START_FEN).unwrap();
        let fen = position2fen(&position);

        assert_eq!(fen, START_FEN);
    }

    #[test]
    fn test_empty_board() {
        let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
        let position = fen2position(fen).unwrap();

        assert_eq!(position2fen(&position), fen);
        assert!(position.board().occupied().is_empty());
    }

    #[test]
    fn test_all_piece_types() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let position = fen2position(fen).unwrap();

        assert_eq!(
            position
                .piece_at(Square::from_coord(0, 0).unwrap())
                .unwrap()
                .kind,
            PieceKind::Rook
        );
        assert_eq!(
            position
                .piece_at(Square::from_coord(4, 0).unwrap())
                .unwrap()
                .kind,
            PieceKind::King
        );
        assert_eq!(
            position
                .piece_at(Square::from_coord(3, 7).unwrap())
                .unwrap()
                .kind,
            PieceKind::Queen
        );
    }

    #[test]
    fn test_side_to_move() {
        let white = fen2position("8/8/8/8/8/8/8/K6k w - - 0 1").unwrap();
        assert_eq!(white.side_to_move(), Color::White);

        let black = fen2position("8/8/8/8/8/8/8/K6k b - - 0 1").unwrap();
        assert_eq!(black.side_to_move(), Color::Black);
    }

    #[test]
    fn test_castling_rights() {
        let fen = "8/8/8/8/8/8/8/R3K2R w KQ - 0 1";
        let position = fen2position(fen).unwrap();

        assert!(
            position
                .castling()
                .contains(CastlingRights::WHITE_KING_SIDE)
        );
        assert!(
            position
                .castling()
                .contains(CastlingRights::WHITE_QUEEN_SIDE)
        );
        assert_eq!(position2fen(&position), fen);
    }

    #[test]
    fn test_en_passant() {
        let fen = "8/8/8/3pP3/8/8/8/8 w - d6 0 1";
        let position = fen2position(fen).unwrap();

        assert_eq!(position.en_passant().unwrap().to_string(), "d6");
        assert_eq!(position2fen(&position), fen);
    }

    #[test]
    fn test_white_double_push_ep() {
        let fen = "8/8/8/8/8/8/4P3/8 b - e3 0 1";
        let pos = fen2position(fen).unwrap();

        assert_eq!(pos.en_passant().unwrap().to_string(), "e3");
    }

    #[test]
    fn test_black_double_push_ep() {
        let fen = "8/8/8/3p4/8/8/8/8 w - d6 0 1";
        let pos = fen2position(fen).unwrap();

        assert_eq!(pos.en_passant().unwrap().to_string(), "d6");
    }

    #[test]
    fn test_move_counters() {
        let fen = "8/8/8/8/8/8/8/K6k w - - 50 100";
        let position = fen2position(fen).unwrap();

        assert_eq!(position.halfmove_clock(), 50);
        assert_eq!(position.fullmove_number(), 100);
        assert_eq!(position2fen(&position), fen);
    }

    #[test]
    fn test_complex_position() {
        let fen = "r3k2r/ppp2ppp/2n5/3q4/3P4/2N5/PPP2PPP/R3K2R b KQkq - 3 10";
        let position = fen2position(fen).unwrap();

        assert_eq!(position2fen(&position), fen);
        assert_eq!(position.side_to_move(), Color::Black);
    }

    #[test]
    fn test_invalid_fen_field_count() {
        let result = fen2position("8/8/8");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_side() {
        let result = fen2position("8/8/8/8/8/8/8/K6k x - - 0 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_piece() {
        let result = fen2position("8/8/8/8/8/8/8/K6z w - - 0 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_castling() {
        let result = fen2position("8/8/8/8/8/8/8/K6k w XYZ - 0 1");
        assert!(result.is_err());
    }
}
