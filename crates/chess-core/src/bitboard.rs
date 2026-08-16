use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, Not};

use crate::Square;

/// 位棋盘
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitBoard(u64);

impl BitBoard {
    pub fn from_square(sq: Square) -> Self {
        Self(sq.bit())
    }

    /// 空棋盘
    pub fn empty() -> Self {
        Self(0)
    }

    /// 全满棋盘
    pub fn full() -> Self {
        Self(u64::MAX)
    }

    /// 设置一位
    pub fn set(&mut self, sq: Square) {
        self.0 |= sq.bit()
    }

    /// 清除一位
    pub fn clear(&mut self, sq: Square) {
        self.0 &= !sq.bit()
    }

    /// 测试一位
    pub fn contains(&self, sq: Square) -> bool {
        self.0 & sq.bit() != 0
    }

    /// 是否为空棋盘
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// 计算棋子数量
    pub fn pop_count(&self) -> u32 {
        self.0.count_ones()
    }

    /// 获取最低有效位对应的棋盘位置
    /// 返回包含最低位1的Square，空棋盘返回None
    pub fn lsb(&self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            Square::new(self.0.trailing_zeros())
        }
    }

    /// 移除并返回最低有效位对应的棋盘位置
    /// 空棋盘返回None
    pub fn pop_lsb(&mut self) -> Option<Square> {
        let sq = self.lsb()?;
        self.0 &= self.0 - 1;
        Some(sq)
    }
}

impl BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitXor for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl From<BitBoard> for u64 {
    fn from(value: BitBoard) -> Self {
        value.0
    }
}

#[derive(Clone, Copy)]
pub struct BitBoardIter {
    bb: BitBoard,
}

impl Iterator for BitBoardIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.bb.pop_lsb()
    }
}

impl IntoIterator for BitBoard {
    type Item = Square;
    type IntoIter = BitBoardIter;

    fn into_iter(self) -> Self::IntoIter {
        BitBoardIter { bb: self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Square;

    #[test]
    fn test_empty_and_full() {
        let empty = BitBoard::empty();

        assert!(empty.is_empty());
        assert_eq!(empty.pop_count(), 0);

        let full = BitBoard::full();

        assert!(!full.is_empty());
        assert_eq!(full.pop_count(), 64);
    }

    #[test]
    fn test_set_clear_contains() {
        let mut bb = BitBoard::empty();

        let sq = Square::new(10).unwrap();

        assert!(!bb.contains(sq));

        bb.set(sq);

        assert!(bb.contains(sq));
        assert_eq!(bb.pop_count(), 1);

        bb.clear(sq);

        assert!(!bb.contains(sq));
        assert!(bb.is_empty());
    }

    #[test]
    fn test_lsb() {
        let mut bb = BitBoard::empty();

        let sq1 = Square::new(5).unwrap();
        let sq2 = Square::new(10).unwrap();

        bb.set(sq2);
        bb.set(sq1);

        assert_eq!(bb.lsb(), Some(sq1));

        assert_eq!(BitBoard::empty().lsb(), None);
    }

    #[test]
    fn test_pop_lsb() {
        let mut bb = BitBoard::empty();

        bb.set(Square::new(3).unwrap());
        bb.set(Square::new(8).unwrap());
        bb.set(Square::new(20).unwrap());

        assert_eq!(bb.pop_lsb(), Some(Square::new(3).unwrap()));

        assert_eq!(bb.pop_lsb(), Some(Square::new(8).unwrap()));

        assert_eq!(bb.pop_lsb(), Some(Square::new(20).unwrap()));

        assert_eq!(bb.pop_lsb(), None);
    }

    #[test]
    fn test_bit_operations() {
        let a = BitBoard::from_square(Square::new(1).unwrap());

        let b = BitBoard::from_square(Square::new(2).unwrap());

        assert_eq!((a | b).pop_count(), 2);

        assert_eq!((a & b).pop_count(), 0);

        assert_eq!((a ^ b).pop_count(), 2);
    }

    #[test]
    fn test_iterator() {
        let mut bb = BitBoard::empty();

        bb.set(Square::new(1).unwrap());
        bb.set(Square::new(5).unwrap());
        bb.set(Square::new(9).unwrap());

        let squares: Vec<_> = bb.into_iter().collect();

        assert_eq!(
            squares,
            vec![
                Square::new(1).unwrap(),
                Square::new(5).unwrap(),
                Square::new(9).unwrap()
            ]
        );
    }
}
