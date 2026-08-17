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

    /// 从 SAN 棋子字母解析类型（'K','Q','R','B','N'）；兵（无字母）返回 None。
    pub fn from_san_char(c: char) -> Option<Self> {
        match c {
            'K' => Some(Self::King),
            'Q' => Some(Self::Queen),
            'R' => Some(Self::Rook),
            'B' => Some(Self::Bishop),
            'N' => Some(Self::Knight),
            _ => None,
        }
    }
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

    #[test]
    fn test_from_san_char() {
        let cases = [
            ('K', PieceKind::King),
            ('Q', PieceKind::Queen),
            ('R', PieceKind::Rook),
            ('B', PieceKind::Bishop),
            ('N', PieceKind::Knight),
        ];

        for (c, expected) in cases {
            assert_eq!(PieceKind::from_san_char(c), Some(expected), "char '{c}'");
        }

        // 兵没有 SAN 字母，非棋子字母返回 None
        assert_eq!(PieceKind::from_san_char('e'), None);
        assert_eq!(PieceKind::from_san_char('x'), None);
        assert_eq!(PieceKind::from_san_char('p'), None);
        assert_eq!(PieceKind::from_san_char('P'), None);
    }
}
