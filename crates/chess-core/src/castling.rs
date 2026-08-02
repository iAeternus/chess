use bitflags::bitflags;

bitflags! {
    /// 王车易位权限
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CastlingRights: u8 {
        const WHITE_KING_SIDE = 1 << 0;
        const WHITE_QUEEN_SIDE = 1 << 1;
        const BLACK_KING_SIDE = 1 << 2;
        const BLACK_QUEEN_SIDE = 1 << 3;
        const ALL = Self::WHITE_KING_SIDE.bits()
            | Self::WHITE_QUEEN_SIDE.bits()
            | Self::BLACK_KING_SIDE.bits()
            | Self::BLACK_QUEEN_SIDE.bits();
    }
}
