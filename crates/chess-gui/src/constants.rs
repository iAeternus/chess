use chess_core::{PieceKind, Promotion};

/// (棋子类型, 黑方icon, 白方icon)
pub const PIECE_ICONS: &[(PieceKind, &str, &str)] = &[
    (PieceKind::King, "♔", "♚"),
    (PieceKind::Queen, "♕", "♛"),
    (PieceKind::Rook, "♖", "♜"),
    (PieceKind::Bishop, "♗", "♝"),
    (PieceKind::Knight, "♘", "♞"),
    (PieceKind::Pawn, "♙", "♟"),
];

/// 升变棋子选项
pub const PROMOTION_PIECES: &[(Promotion, &str)] = &[
    (Promotion::Queen, "♛"),
    (Promotion::Rook, "♜"),
    (Promotion::Bishop, "♝"),
    (Promotion::Knight, "♞"),
];
