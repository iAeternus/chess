use crate::{
    CastlingRights, Color, Move, MoveFlag, Piece, PieceKind, Position, Square, zobrist::Zobrist,
};

pub struct Undo {
    /// 执行的走法
    pub mv: Move,
    /// 被吃掉的棋子
    captured: Option<Piece>,
    /// 原行动方
    prev_side: Color,
    /// 原王车易位权限
    prev_castling: CastlingRights,
    /// 原en passant
    prev_en_passant: Option<Square>,
    /// 原50步计数
    prev_halfmove: u32,
    /// 原回合数
    prev_fullmove: u32,
    /// 原Zobrist
    prev_zobrist: u64,
}

/// 执行走法，返回 Undo 信息
pub(crate) fn make_move(position: &mut Position, mv: Move) -> Undo {
    let mut hash = position.zobrist_key();

    let captured = match mv.flag() {
        MoveFlag::EnPassant => {
            let sq = Square::from_coord(mv.to().file(), mv.from().rank()).unwrap();
            position.piece_at(sq)
        }
        _ => position.piece_at(mv.to()),
    };

    let undo = Undo {
        mv,
        captured,
        prev_side: position.side_to_move(),
        prev_castling: position.castling(),
        prev_en_passant: position.en_passant(),
        prev_halfmove: position.halfmove_clock(),
        prev_fullmove: position.fullmove_number(),
        prev_zobrist: position.zobrist_key(),
    };

    // 删除旧 ep hash
    Zobrist::update_en_passant(&mut hash, position.en_passant(), None);
    position.set_en_passant(None);

    let old_castling = position.castling();

    match mv.flag() {
        MoveFlag::Quiet | MoveFlag::DoublePawnPush => {
            move_piece(position, mv, &mut hash);
            if mv.flag() == MoveFlag::DoublePawnPush {
                let ep = Square::between(mv.from(), mv.to());
                Zobrist::update_en_passant(&mut hash, None, ep);
                position.set_en_passant(ep);
            }
        }
        MoveFlag::Capture => capture_piece(position, mv, &mut hash),
        MoveFlag::Promotion => promote(position, mv, false, &mut hash),
        MoveFlag::PromotionCapture => promote(position, mv, true, &mut hash),
        MoveFlag::EnPassant => en_passant(position, mv, &mut hash),
        MoveFlag::KingCastle => castle_kingside(position, &mut hash),
        MoveFlag::QueenCastle => castle_queenside(position, &mut hash),
    }

    update_castling(position, mv);
    let new_castling = position.castling();
    Zobrist::update_castling(&mut hash, old_castling, new_castling);

    let moving_piece = position.piece_at(mv.to());
    update_clock(position, mv, moving_piece);

    position.switch_side();
    Zobrist::update_side(&mut hash);

    position.set_zobrist(hash);

    undo
}

/// 撤销走法
pub(crate) fn unmake_move(position: &mut Position, undo: Undo) {
    let mv = undo.mv;
    position.set_side_to_move(undo.prev_side);
    match mv.flag() {
        MoveFlag::Quiet | MoveFlag::DoublePawnPush => move_piece_reverse(position, mv),
        MoveFlag::Capture => {
            move_piece_reverse(position, mv);
            restore_captured(position, mv.to(), undo.captured);
        }
        MoveFlag::Promotion => undo_promotion(position, mv),
        MoveFlag::PromotionCapture => {
            undo_promotion(position, mv);
            restore_captured(position, mv.to(), undo.captured);
        }
        MoveFlag::EnPassant => undo_en_passant(position, mv),
        MoveFlag::KingCastle => undo_kingside_castle(position),
        MoveFlag::QueenCastle => undo_queenside_castle(position),
    }

    // 恢复局面状态
    position.set_castling(undo.prev_castling);
    position.set_en_passant(undo.prev_en_passant);
    position.set_halfmove_clock(undo.prev_halfmove);
    position.set_fullmove_number(undo.prev_fullmove);
    position.set_side_to_move(undo.prev_side);

    // 恢复hash
    position.set_zobrist(undo.prev_zobrist);
}

fn restore_captured(position: &mut Position, square: Square, captured: Option<Piece>) {
    if let Some(piece) = captured {
        position.board_mut().add_piece(square, piece);
    }
}

fn move_piece(position: &mut Position, mv: Move, hash: &mut u64) {
    let piece = position.piece_at(mv.from()).expect("no piece");
    Zobrist::update_piece(hash, piece, mv.from(), mv.to());
    let board = position.board_mut();
    board.remove_piece(mv.from());
    board.add_piece(mv.to(), piece);
}

fn move_piece_reverse(position: &mut Position, mv: Move) {
    let piece = position.piece_at(mv.to()).expect("no piece on destination");
    let board = position.board_mut();
    board.remove_piece(mv.to());
    board.add_piece(mv.from(), piece);
}

fn capture_piece(position: &mut Position, mv: Move, hash: &mut u64) {
    let captured = position.piece_at(mv.to()).expect("capture without piece");
    Zobrist::update_remove(hash, captured, mv.to());
    position.board_mut().remove_piece(mv.to());
    move_piece(position, mv, hash);
}

fn promote(position: &mut Position, mv: Move, capture: bool, hash: &mut u64) {
    let pawn = position
        .piece_at(mv.from())
        .expect("promotion without pawn");
    Zobrist::update_remove(hash, pawn, mv.from());

    if let Some(captured) = position.piece_at(mv.to()) {
        Zobrist::update_remove(hash, captured, mv.to());
    }
    let promoted_kind = Option::<PieceKind>::from(mv.promotion()).expect("invalid promotion");
    let promoted = Piece::new(pawn.color, promoted_kind);
    Zobrist::update_add(hash, promoted, mv.to());

    let board = position.board_mut();

    // 删除兵
    board.remove_piece(mv.from());
    // 删除被吃棋子
    if capture {
        board.remove_piece(mv.to());
    }
    // 添加升变棋子
    board.add_piece(mv.to(), promoted);
}

fn undo_promotion(position: &mut Position, mv: Move) {
    let promoted = position.piece_at(mv.to()).expect("missing promoted piece");
    let board = position.board_mut();
    board.remove_piece(mv.to());

    let pawn = Piece {
        color: promoted.color,
        kind: PieceKind::Pawn,
    };

    board.add_piece(mv.from(), pawn);
}

fn en_passant(position: &mut Position, mv: Move, hash: &mut u64) {
    let pawn = position.piece_at(mv.from()).unwrap(); // SAFETY: sq is valid here
    let captured_sq = Square::from_coord(mv.to().file(), mv.from().rank()).unwrap(); // SAFETY: file and rank is valid here
    let captured = position.piece_at(captured_sq).unwrap(); // SAFETY: sq is valid here

    Zobrist::update_remove(hash, captured, captured_sq);
    Zobrist::update_piece(hash, pawn, mv.from(), mv.to());

    let board = position.board_mut();

    board.remove_piece(captured_sq);
    board.remove_piece(mv.from());
    board.add_piece(mv.to(), pawn);
}

fn undo_en_passant(position: &mut Position, mv: Move) {
    let pawn = position.piece_at(mv.to()).unwrap();
    let board = position.board_mut();

    board.remove_piece(mv.to());
    board.add_piece(mv.from(), pawn);

    let captured_sq = Square::from_coord(mv.to().file(), mv.from().rank()).unwrap();
    board.add_piece(
        captured_sq,
        Piece {
            color: pawn.color.flip(),
            kind: PieceKind::Pawn,
        },
    );
}

fn castle_kingside(position: &mut Position, hash: &mut u64) {
    let color = position.side_to_move();
    let (king_from, king_to, rook_from, rook_to) = match color {
        Color::White => (Square::E1, Square::G1, Square::H1, Square::F1),
        Color::Black => (Square::E8, Square::G8, Square::H8, Square::F8),
    };
    debug_assert!(position.piece_at(king_from).is_some());
    debug_assert!(position.piece_at(rook_from).is_some());
    debug_assert!(position.piece_at(king_to).is_none());
    debug_assert!(position.piece_at(rook_to).is_none());

    move_specific_piece(position, king_from, king_to, hash);
    move_specific_piece(position, rook_from, rook_to, hash);
}

fn castle_queenside(position: &mut Position, hash: &mut u64) {
    let color = position.side_to_move();
    let (king_from, king_to, rook_from, rook_to) = match color {
        Color::White => (Square::E1, Square::C1, Square::A1, Square::D1),
        Color::Black => (Square::E8, Square::C8, Square::A8, Square::D8),
    };

    move_specific_piece(position, king_from, king_to, hash);
    move_specific_piece(position, rook_from, rook_to, hash);
}

fn undo_kingside_castle(position: &mut Position) {
    let color = position.side_to_move();

    match color {
        Color::White => {
            move_specific_piece_reverse(position, Square::G1, Square::E1);
            move_specific_piece_reverse(position, Square::F1, Square::H1);
        }
        Color::Black => {
            move_specific_piece_reverse(position, Square::G8, Square::E8);
            move_specific_piece_reverse(position, Square::F8, Square::H8);
        }
    }
}

fn undo_queenside_castle(position: &mut Position) {
    let color = position.side_to_move();
    match color {
        Color::White => {
            move_specific_piece_reverse(position, Square::C1, Square::E1);
            move_specific_piece_reverse(position, Square::D1, Square::A1);
        }
        Color::Black => {
            move_specific_piece_reverse(position, Square::C8, Square::E8);
            move_specific_piece_reverse(position, Square::D8, Square::A8);
        }
    }
}

fn move_specific_piece(position: &mut Position, from: Square, to: Square, hash: &mut u64) {
    let mv = Move::new(from, to, MoveFlag::Quiet);
    move_piece(position, mv, hash);
}

fn move_specific_piece_reverse(position: &mut Position, from: Square, to: Square) {
    let piece = position.piece_at(from).expect("missing piece");
    let board = position.board_mut();
    board.remove_piece(from);
    board.add_piece(to, piece);
}

fn update_castling(position: &mut Position, mv: Move) {
    let mut rights = position.castling();

    // 王或车移动
    remove_castling_by_square(&mut rights, mv.from());

    // 吃掉王车初始位置上的车
    if mv.is_capture() {
        remove_castling_by_square(&mut rights, mv.to());
    }

    position.set_castling(rights);
}

fn remove_castling_by_square(rights: &mut CastlingRights, sq: Square) {
    let remove = match sq {
        Square::E1 => CastlingRights::WHITE_KING_SIDE | CastlingRights::WHITE_QUEEN_SIDE,
        Square::E8 => CastlingRights::BLACK_KING_SIDE | CastlingRights::BLACK_QUEEN_SIDE,
        Square::H1 => CastlingRights::WHITE_KING_SIDE,
        Square::A1 => CastlingRights::WHITE_QUEEN_SIDE,
        Square::H8 => CastlingRights::BLACK_KING_SIDE,
        Square::A8 => CastlingRights::BLACK_QUEEN_SIDE,
        _ => CastlingRights::empty(),
    };

    rights.remove(remove);
}

fn update_clock(position: &mut Position, mv: Move, moving_piece: Option<Piece>) {
    let reset = mv.is_capture()
        || moving_piece
            .map(|p| p.kind == PieceKind::Pawn)
            .unwrap_or(false);

    if reset {
        position.set_halfmove_clock(0);
    } else {
        position.set_halfmove_clock(position.halfmove_clock() + 1);
    }

    if position.side_to_move() == Color::Black {
        position.set_fullmove_number(position.fullmove_number() + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Promotion;

    #[test]
    fn test_make_quiet_move() {
        let mut pos = Position::startpos();
        let mv = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let undo = make_move(&mut pos, mv);

        assert!(pos.piece_at(Square::E4).is_some());
        assert!(pos.piece_at(Square::E2).is_none());

        unmake_move(&mut pos, undo);
        assert!(pos.piece_at(Square::E2).is_some());
    }

    #[test]
    fn test_capture() {
        let mut pos = Position::from_fen("8/8/8/3p4/4P3/8/8/8 w - - 0 1").unwrap();
        let mv = Move::new(Square::E4, Square::D5, MoveFlag::Capture);
        let undo = make_move(&mut pos, mv);

        assert!(matches!(
            pos.piece_at(Square::D5),
            Some(Piece {
                kind: PieceKind::Pawn,
                color: Color::White
            })
        ));

        unmake_move(&mut pos, undo);
        assert_eq!(
            pos.piece_at(Square::D5),
            Some(Piece::new(Color::Black, PieceKind::Pawn))
        );
    }

    #[test]
    fn test_double_push_ep() {
        let mut pos = Position::startpos();
        let mv = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        make_move(&mut pos, mv);

        assert_eq!(pos.en_passant(), Some(Square::E3));
    }

    #[test]
    fn test_en_passant() {
        let mut pos = Position::from_fen("8/8/8/3pP3/8/8/8/8 w - d6 0 1").unwrap();
        let mv = Move::new(Square::E5, Square::D6, MoveFlag::EnPassant);
        let undo = make_move(&mut pos, mv);

        assert!(pos.piece_at(Square::D6).is_some());
        assert!(pos.piece_at(Square::D5).is_none());

        unmake_move(&mut pos, undo);
        assert!(pos.piece_at(Square::D5).is_some());
    }

    #[test]
    fn test_white_kingside_castle() {
        let mut pos = Position::from_fen("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
        let mv = Move::new(Square::E1, Square::G1, MoveFlag::KingCastle);
        let undo = make_move(&mut pos, mv);

        assert!(pos.piece_at(Square::G1).is_some());
        assert!(pos.piece_at(Square::F1).is_some());

        unmake_move(&mut pos, undo);
        assert!(pos.piece_at(Square::E1).is_some());
    }

    #[test]
    fn test_promotion() {
        let mut pos = Position::from_fen("8/4P3/8/8/8/8/8/7K w - - 0 1").unwrap();
        let mv = Move::new_promotion(Square::E7, Square::E8, Promotion::Queen, false);
        let undo = make_move(&mut pos, mv);

        assert_eq!(pos.piece_at(Square::E8).unwrap().kind, PieceKind::Queen);

        unmake_move(&mut pos, undo);
        assert_eq!(pos.piece_at(Square::E7).unwrap().kind, PieceKind::Pawn);
    }

    #[test]
    fn test_zobrist_restore() {
        let mut pos = Position::startpos();
        let original = pos.zobrist_key();
        let mv = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        let undo = make_move(&mut pos, mv);
        unmake_move(&mut pos, undo);

        assert_eq!(pos.zobrist_key(), original);
    }

    #[test]
    fn test_incremental_hash() {
        let mut pos = Position::startpos();
        let mv = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        make_move(&mut pos, mv);

        assert_eq!(pos.zobrist_key(), Zobrist::compute(&pos));
    }
}
