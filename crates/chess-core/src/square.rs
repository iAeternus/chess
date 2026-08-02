use std::{fmt::Display};

/// 棋盘格
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(u8);

impl Square {
    pub fn new(index: u32) -> Option<Self> {
        if index >= 64 {
            return None;
        }
        Some(Self(index as u8))
    }

    /// SAFETY: index in valid range [0, 63]
    pub unsafe fn new_unchecked(index: u32) -> Self {
        Self(index as u8)
    }

    /// 从坐标构造
    pub fn from_coord(file: u8, rank: u8) -> Option<Self> {
        if file >= 8 || rank >= 8 {
            return None;
        }
        Some(Self(rank * 8 + file))
    }

    /// 数组索引
    pub fn index(&self) -> usize {
        self.0 as usize
    }

    pub fn bit(&self) -> u64 {
        1u64 << self.0
    }

    /// 行
    pub fn rank(&self) -> u8 {
        self.0 / 8
    }

    /// 列
    pub fn file(&self) -> u8 {
        self.0 % 8
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file = (b'a' + self.file()) as char;
        let rank = self.rank() + 1;
        write!(f, "{}{}", file, rank)
    }
}

macro_rules! define_squares {
    ($($name:ident = $idx:expr),*) => {
        impl Square {
            $(pub const $name: Square = Square($idx);)*
        }
    };
}

define_squares! {
    A1 = 0,
    B1 = 1,
    C1 = 2,
    D1 = 3,
    E1 = 4,
    F1 = 5,
    G1 = 6,
    H1 = 7,

    A2 = 8,
    B2 = 9,
    C2 = 10,
    D2 = 11,
    E2 = 12,
    F2 = 13,
    G2 = 14,
    H2 = 15,

    A3 = 16,
    B3 = 17,
    C3 = 18,
    D3 = 19,
    E3 = 20,
    F3 = 21,
    G3 = 22,
    H3 = 23,

    A4 = 24,
    B4 = 25,
    C4 = 26,
    D4 = 27,
    E4 = 28,
    F4 = 29,
    G4 = 30,
    H4 = 31,

    A5 = 32,
    B5 = 33,
    C5 = 34,
    D5 = 35,
    E5 = 36,
    F5 = 37,
    G5 = 38,
    H5 = 39,

    A6 = 40,
    B6 = 41,
    C6 = 42,
    D6 = 43,
    E6 = 44,
    F6 = 45,
    G6 = 46,
    H6 = 47,

    A7 = 48,
    B7 = 49,
    C7 = 50,
    D7 = 51,
    E7 = 52,
    F7 = 53,
    G7 = 54,
    H7 = 55,

    A8 = 56,
    B8 = 57,
    C8 = 58,
    D8 = 59,
    E8 = 60,
    F8 = 61,
    G8 = 62,
    H8 = 63
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(Square::new(0), Some(Square::A1));
        assert_eq!(Square::new(28), Some(Square::E4));
        assert_eq!(Square::new(63), Some(Square::H8));

        assert_eq!(Square::new(64), None);
        assert_eq!(Square::new(u32::MAX), None);
    }

    #[test]
    fn test_coord_mapping() {
        let cases = [(0, 0, Square::A1), (4, 3, Square::E4), (7, 7, Square::H8)];

        for (file, rank, expected) in cases {
            let sq = Square::from_coord(file, rank).unwrap();

            assert_eq!(sq, expected);
            assert_eq!(sq.file(), file);
            assert_eq!(sq.rank(), rank);
            assert_eq!(sq.index(), (rank * 8 + file) as usize);
        }

        assert_eq!(Square::from_coord(8, 0), None);
        assert_eq!(Square::from_coord(0, 8), None);
    }

    #[test]
    fn test_round_trip_all_squares() {
        for rank in 0..8 {
            for file in 0..8 {
                let sq = Square::from_coord(file, rank).unwrap();

                assert_eq!(sq.file(), file);
                assert_eq!(sq.rank(), rank);

                let rebuilt = Square::new(sq.index() as u32).unwrap();

                assert_eq!(sq, rebuilt);
            }
        }
    }

    #[test]
    fn test_bitboard_mask() {
        assert_eq!(Square::A1.bit(), 1u64 << 0);
        assert_eq!(Square::E4.bit(), 1u64 << 28);
        assert_eq!(Square::H8.bit(), 1u64 << 63);

        assert_eq!(Square::A1.bit().count_ones(), 1);
    }

    #[test]
    fn test_display() {
        assert_eq!(Square::A1.to_string(), "a1");
        assert_eq!(Square::E4.to_string(), "e4");
        assert_eq!(Square::H8.to_string(), "h8");
    }
}
