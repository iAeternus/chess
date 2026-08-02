use crate::{PieceKind, Square};

/// 走法标志
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MoveFlag {
    /// 普通移动
    Quiet = 0,
    /// 兵两步移动
    DoublePawnPush = 1,
    /// 王翼易位
    KingCastle = 2,
    /// 后翼易位
    QueenCastle = 3,
    /// 吃子
    Capture = 4,
    /// 吃过路兵
    EnPassant = 5,
    /// 升变
    Promotion = 6,
    /// 吃子升变
    PromotionCapture = 7,
}

impl TryFrom<u8> for MoveFlag {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Quiet),
            1 => Ok(Self::DoublePawnPush),
            2 => Ok(Self::KingCastle),
            3 => Ok(Self::QueenCastle),
            4 => Ok(Self::Capture),
            5 => Ok(Self::EnPassant),
            6 => Ok(Self::Promotion),
            7 => Ok(Self::PromotionCapture),
            _ => Err(()),
        }
    }
}

/// 升变类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Promotion {
    /// 不升变
    None = 0,
    /// 升变为马
    Knight = 1,
    /// 升变为象
    Bishop = 2,
    /// 升变为车
    Rook = 3,
    /// 升变为后
    Queen = 4,
}

impl TryFrom<u8> for Promotion {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Knight),
            2 => Ok(Self::Bishop),
            3 => Ok(Self::Rook),
            4 => Ok(Self::Queen),
            _ => Err(()),
        }
    }
}

impl From<Promotion> for Option<PieceKind> {
    fn from(value: Promotion) -> Self {
        match value {
            Promotion::Knight => Some(PieceKind::Knight),
            Promotion::Bishop => Some(PieceKind::Bishop),
            Promotion::Rook => Some(PieceKind::Rook),
            Promotion::Queen => Some(PieceKind::Queen),
            Promotion::None => None,
        }
    }
}

/// 走法，使用一个u32压缩存储
/// ```text
/// 31                       20 19      16 15      12 11       6 5       0
/// +-------------------------+----------+----------+----------+---------+
/// |       reserved          |  flags   |promotion |    to    |  from   |
/// +-------------------------+----------+----------+----------+---------+
/// ```
/// 字段说明：
/// - from: 起始棋盘位置，6 bit表示64个棋盘格
/// - to: 目标棋盘位置
/// - promotion: 升变类型Queen/Rook/Bishop/Knight
/// - flags: MoveFlag，描述移动行为
/// - reserved: 保留字段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(u32);

impl Move {
    const FROM_SHIFT: u32 = 0;
    const TO_SHIFT: u32 = 6;
    const PROMOTION_SHIFT: u32 = 12;
    const FLAG_SHIFT: u32 = 16;

    const SQUARE_MASK: u32 = 0b111111;
    const PROMOTION_MASK: u32 = 0b1111;
    const FLAG_MASK: u32 = 0b1111;

    /// 空走法，用于搜索初始化
    pub const NULL: Self = Self(0);

    /// 创建普通走法
    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
        let value = (from.index() as u32) << Self::FROM_SHIFT
            | (to.index() as u32) << Self::TO_SHIFT
            | (flag as u32) << Self::FLAG_SHIFT;
        Self(value)
    }

    /// 创建升变走法
    pub fn new_promotion(from: Square, to: Square, promotion: Promotion, capture: bool) -> Self {
        let flag = if capture {
            MoveFlag::PromotionCapture
        } else {
            MoveFlag::Promotion
        };

        let value = (from.index() as u32)
            | ((to.index() as u32) << 6)
            | ((promotion as u32) << 12)
            | ((flag as u32) << 16);
        Self(value)
    }

    /// 获取起始位置
    pub fn from(&self) -> Square {
        let index = self.0 & Self::SQUARE_MASK;
        Square::new(index).expect("invalid square") // SAFETY: index in range [0,63]
    }

    /// 获取目标位置
    pub fn to(&self) -> Square {
        let index = (self.0 >> Self::TO_SHIFT) & Self::SQUARE_MASK;
        Square::new(index).expect("invalid square") // SAFETY: index in range [0,63]
    }

    /// 获取走法类型
    pub fn flag(&self) -> MoveFlag {
        let value = ((self.0 >> Self::FLAG_SHIFT) & Self::FLAG_MASK) as u8;
        MoveFlag::try_from(value).expect("invalid move flag") // SAFETY: value in range [0000,1111]
    }

    /// 获取升变类型
    pub fn promotion(&self) -> Promotion {
        let value = ((self.0 >> Self::PROMOTION_SHIFT) & Self::PROMOTION_MASK) as u8;
        Promotion::try_from(value).expect("invalid promotion") // SAFETY: value in range [0000,1111]
    }

    /// 是否吃子
    pub fn is_capture(&self) -> bool {
        matches!(
            self.flag(),
            MoveFlag::Capture | MoveFlag::EnPassant | MoveFlag::PromotionCapture,
        )
    }
    /// 是否升变
    pub fn is_promotion(&self) -> bool {
        self.promotion() != Promotion::None
    }

    /// 是否王车易位
    pub fn is_castle(&self) -> bool {
        matches!(self.flag(), MoveFlag::KingCastle | MoveFlag::QueenCastle)
    }

    /// 是否普通移动
    pub fn is_quiet(&self) -> bool {
        self.flag() == MoveFlag::Quiet
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Square;

    #[test]
    fn test_move_encode_decode() {
        let mv = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        assert_eq!(mv.from(), Square::E2);
        assert_eq!(mv.to(), Square::E4);
        assert_eq!(mv.flag(), MoveFlag::DoublePawnPush);
        assert_eq!(mv.promotion(), Promotion::None);
    }

    #[test]
    fn test_null_move() {
        let mv = Move::NULL;

        assert_eq!(mv.as_u32(), 0);
        assert_eq!(mv.from(), Square::A1);
        assert_eq!(mv.to(), Square::A1);
        assert_eq!(mv.flag(), MoveFlag::Quiet);
    }

    #[test]
    fn test_capture_move() {
        let mv = Move::new(Square::E4, Square::D5, MoveFlag::Capture);

        assert_eq!(mv.flag(), MoveFlag::Capture);
        assert!(mv.is_capture());
        assert!(!mv.is_promotion());
    }

    #[test]
    fn test_castle_move() {
        let king_side = Move::new(Square::E1, Square::G1, MoveFlag::KingCastle);

        assert!(king_side.is_castle());
        assert_eq!(king_side.flag(), MoveFlag::KingCastle);

        let queen_side = Move::new(Square::E1, Square::C1, MoveFlag::QueenCastle);

        assert!(queen_side.is_castle());
    }

    #[test]
    fn test_promotion() {
        let mv = Move::new_promotion(Square::E7, Square::E8, Promotion::Queen, false);

        assert_eq!(mv.flag(), MoveFlag::Promotion);
        assert!(mv.is_promotion());
        assert_eq!(mv.promotion(), Promotion::Queen);
        assert!(!mv.is_capture());
    }

    #[test]
    fn test_capture_promotion() {
        let mv = Move::new_promotion(Square::E7, Square::D8, Promotion::Queen, true);

        assert_eq!(mv.flag(), MoveFlag::PromotionCapture);
        assert!(mv.is_capture());
        assert!(mv.is_promotion());
        assert_eq!(mv.promotion(), Promotion::Queen);
    }

    #[test]
    fn test_all_promotions() {
        let promotions = [
            Promotion::Knight,
            Promotion::Bishop,
            Promotion::Rook,
            Promotion::Queen,
        ];

        for promotion in promotions {
            let mv = Move::new_promotion(Square::A7, Square::A8, promotion, false);
            assert!(mv.is_promotion());
            assert_eq!(mv.promotion(), promotion);
        }
    }

    #[test]
    fn test_move_size() {
        assert_eq!(std::mem::size_of::<Move>(), 4);
    }
}
