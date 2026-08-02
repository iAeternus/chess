use crate::Color;

/// 棋子类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PieceKind {
    /// 兵
    Pawn = 0,
    /// 马
    Knight = 1,
    /// 象
    Bishop = 2,
    /// 车
    Rook = 3,
    /// 后
    Queen = 4,
    /// 王
    King = 5,
}

impl PieceKind {
    /// 棋子类型数
    const COUNT: usize = 6;

    /// 所有棋子类型
    pub const ALL: [PieceKind; Self::COUNT] = [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ];
}

impl From<PieceKind> for usize {
    fn from(value: PieceKind) -> Self {
        value as usize
    }
}

/// 棋子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    pub fn new(color: Color, kind: PieceKind) -> Self {
        Self { color, kind }
    }

    pub fn from_char(c: char) -> Option<Self> {
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };

        let kind = match c.to_ascii_lowercase() {
            'p' => PieceKind::Pawn,
            'n' => PieceKind::Knight,
            'b' => PieceKind::Bishop,
            'r' => PieceKind::Rook,
            'q' => PieceKind::Queen,
            'k' => PieceKind::King,
            _ => return None,
        };

        Some(Piece::new(color, kind))
    }

    pub fn to_char(&self) -> char {
        let c = match self.kind {
            PieceKind::Pawn => 'P',
            PieceKind::Knight => 'N',
            PieceKind::Bishop => 'B',
            PieceKind::Rook => 'R',
            PieceKind::Queen => 'Q',
            PieceKind::King => 'K',
        };

        match self.color {
            Color::White => c,
            Color::Black => c.to_ascii_lowercase(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_char() {
        let cases = [
            (Color::White, PieceKind::Pawn, 'P'),
            (Color::White, PieceKind::Knight, 'N'),
            (Color::White, PieceKind::Bishop, 'B'),
            (Color::White, PieceKind::Rook, 'R'),
            (Color::White, PieceKind::Queen, 'Q'),
            (Color::White, PieceKind::King, 'K'),
            (Color::Black, PieceKind::Pawn, 'p'),
            (Color::Black, PieceKind::Knight, 'n'),
            (Color::Black, PieceKind::Bishop, 'b'),
            (Color::Black, PieceKind::Rook, 'r'),
            (Color::Black, PieceKind::Queen, 'q'),
            (Color::Black, PieceKind::King, 'k'),
        ];

        for (color, kind, expected) in cases {
            let piece = Piece::new(color, kind);
            assert_eq!(piece.to_char(), expected);
        }
    }
}
