use crate::{BitBoard, Color, Piece, PieceKind, Square};

/// 棋盘
#[derive(Clone)]
pub struct Board {
    pieces: [[BitBoard; 6]; 2],     // pieces[color][piece_kind]
    by_square: [Option<Piece>; 64], // 快速反向查找
}

impl Board {
    /// 查询指定位置的棋子
    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.by_square[sq.index()]
    }

    /// 查询指定颜色指定类型的棋子位置
    pub fn piece_kind(&self, color: Color, kind: PieceKind) -> BitBoard {
        self.pieces[color as usize][kind as usize]
    }

    /// 查询指定颜色所有棋子
    pub fn pieces(&self, color: Color) -> BitBoard {
        let mut bb = BitBoard::empty();
        for kind in PieceKind::ALL {
            bb |= self.piece_kind(color, kind)
        }
        bb
    }

    /// 查询所有棋子
    pub fn occupied(&self) -> BitBoard {
        self.pieces(Color::White) | self.pieces(Color::Black)
    }

    /// 查询所有空格
    pub fn empty(&self) -> BitBoard {
        !self.occupied()
    }

    /// 添加棋子
    pub fn add_piece(&mut self, sq: Square, piece: Piece) {
        debug_assert!(
            self.by_square[sq.index()].is_none(),
            "square already occupied"
        );
        self.pieces[piece.color as usize][piece.kind as usize].set(sq);
        self.by_square[sq.index()] = Some(piece);
    }

    /// 删除棋子，返回被删除的棋子，若该位置没有棋子，则返回None
    pub fn remove_piece(&mut self, sq: Square) -> Option<Piece> {
        let piece = self.by_square[sq.index()]?;
        self.pieces[piece.color as usize][piece.kind as usize].clear(sq);
        self.by_square[sq.index()] = None;
        Some(piece)
    }

    /// 移动棋子，不检查移动合法性，返回棋子
    pub fn move_piece(&mut self, from: Square, to: Square) -> Option<Piece> {
        debug_assert!(self.piece_at(to).is_none(), "destination occupied");
        let piece = self.remove_piece(from)?;
        self.add_piece(to, piece);
        Some(piece)
    }

    /// 查询王的位置
    pub fn king_square(&self, color: Color) -> Square {
        // SAFETY: 每方有且仅有一个King
        self.piece_kind(color, PieceKind::King).lsb().unwrap()
    }
}

impl Default for Board {
    /// 空棋盘
    fn default() -> Self {
        Self {
            pieces: [[BitBoard::empty(); 6]; 2],
            by_square: [None; 64],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_board() {
        let board = Board::default();

        assert!(board.occupied().is_empty());
        assert_eq!(board.piece_at(Square::new(0).unwrap()), None);
    }

    #[test]
    fn test_add_piece() {
        let mut board = Board::default();
        let sq = Square::new(12).unwrap();
        let pawn = Piece::new(Color::White, PieceKind::Pawn);

        board.add_piece(sq, pawn);

        // by_square
        assert_eq!(board.piece_at(sq), Some(pawn));

        // bitboard
        assert!(board.piece_kind(Color::White, PieceKind::Pawn).contains(sq));
        assert_eq!(board.occupied().pop_count(), 1);
    }

    #[test]
    fn test_remove_piece() {
        let mut board = Board::default();
        let sq = Square::new(20).unwrap();
        let queen = Piece::new(Color::Black, PieceKind::Queen);
        board.add_piece(sq, queen);

        let removed = board.remove_piece(sq);

        assert_eq!(removed, Some(queen));
        assert_eq!(board.piece_at(sq), None);
        assert!(
            !board
                .piece_kind(Color::Black, PieceKind::Queen)
                .contains(sq)
        );
    }

    #[test]
    fn test_move_piece() {
        let mut board = Board::default();
        let from = Square::new(8).unwrap();
        let to = Square::new(16).unwrap();
        let knight = Piece::new(Color::White, PieceKind::Knight);
        board.add_piece(from, knight);

        let moved = board.move_piece(from, to);

        assert_eq!(moved, Some(knight));
        assert_eq!(board.piece_at(from), None);
        assert_eq!(board.piece_at(to), Some(knight));
        assert!(
            board
                .piece_kind(Color::White, PieceKind::Knight)
                .contains(to)
        );
    }

    #[test]
    fn test_pieces_by_color() {
        let mut board = Board::default();
        board.add_piece(
            Square::new(0).unwrap(),
            Piece::new(Color::White, PieceKind::Rook),
        );
        board.add_piece(
            Square::new(1).unwrap(),
            Piece::new(Color::White, PieceKind::Knight),
        );
        board.add_piece(
            Square::new(63).unwrap(),
            Piece::new(Color::Black, PieceKind::King),
        );

        assert_eq!(board.pieces(Color::White).pop_count(), 2);
        assert_eq!(board.pieces(Color::Black).pop_count(), 1);
        assert_eq!(board.occupied().pop_count(), 3);
    }

    #[test]
    fn test_empty_board() {
        let mut board = Board::default();
        board.add_piece(
            Square::new(10).unwrap(),
            Piece::new(Color::White, PieceKind::Pawn),
        );

        let empty = board.empty();

        assert!(!empty.contains(Square::new(10).unwrap()));
        assert_eq!(empty.pop_count(), 63);
    }

    #[test]
    fn test_king_square() {
        let mut board = Board::default();
        let sq = Square::new(60).unwrap();
        board.add_piece(sq, Piece::new(Color::Black, PieceKind::King));

        assert_eq!(board.king_square(Color::Black), sq);
    }

    #[test]
    fn test_board_remove_consistency() {
        let mut board = Board::default();
        board.add_piece(Square::E4, Piece::new(Color::White, PieceKind::Pawn));
        board.remove_piece(Square::E4);
        assert!(board.piece_at(Square::E4).is_none());
    }
}
