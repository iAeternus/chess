use std::sync::LazyLock;

use crate::{BitBoard, Board, Color, PieceKind, Square};

/// 马攻击表，`KNIGHT_ATTACKS[sq]`
pub static KNIGHT_ATTACKS: LazyLock<[BitBoard; 64]> = LazyLock::new(|| {
    let mut table = [BitBoard::empty(); 64];
    for index in 0..64 {
        let sq = Square::new(index as u32).unwrap(); // SAFETY: index is valid here
        table[index] = generate_knight_attacks(sq);
    }
    table
});

/// 王攻击表，`KING_ATTACKS[sq]`
pub static KING_ATTACKS: LazyLock<[BitBoard; 64]> = LazyLock::new(|| {
    let mut table = [BitBoard::empty(); 64];
    for index in 0..64 {
        let sq = Square::new(index as u32).unwrap(); // SAFETY: index is valid here
        table[index] = generate_king_attacks(sq);
    }
    table
});

/// 兵攻击表，`PAWN_ATTACKS[color][sq]`
pub static PAWN_ATTACKS: LazyLock<[[BitBoard; 64]; 2]> = LazyLock::new(|| {
    let mut table = [[BitBoard::empty(); 64]; 2];
    for index in 0..64 {
        let sq = Square::new(index as u32).unwrap(); // SAFETY: index is valid here
        table[Color::White as usize][index] = generate_pawn_attacks(sq, Color::White);
        table[Color::Black as usize][index] = generate_pawn_attacks(sq, Color::Black);
    }
    table
});

/// 象攻击表，对角线4个方向
pub fn bishop_rays(sq: Square, occupied: BitBoard) -> BitBoard {
    sliding_attack(sq, occupied, &[(1, 1), (-1, 1), (1, -1), (-1, -1)])
}

/// 车攻击表，上下左右4个方向
pub fn rook_rays(sq: Square, occupied: BitBoard) -> BitBoard {
    sliding_attack(sq, occupied, &[(0, 1), (0, -1), (1, 0), (-1, 0)])
}

/// 后攻击表，对角线+上下左右8个方向
pub fn queen_rays(sq: Square, occupied: BitBoard) -> BitBoard {
    bishop_rays(sq, occupied) | rook_rays(sq, occupied)
}

/// 判断一个格子是否被某方攻击
/// 检查顺序：pawn/knight/king/bishop/rook/queen
///
/// # 返回
/// - true: 是
/// - false: 否
pub fn is_square_attacked(board: &Board, sq: Square, by_color: Color) -> bool {
    let sq_idx = sq.index();

    // 是否被pawn攻击
    let pawn_src = PAWN_ATTACKS[by_color.flip() as usize][sq_idx]
        & board.piece_kind(by_color, PieceKind::Pawn);
    if !pawn_src.is_empty() {
        return true;
    }

    // 是否被knight攻击
    let knight_src = KNIGHT_ATTACKS[sq_idx] & board.piece_kind(by_color, PieceKind::Knight);
    if !knight_src.is_empty() {
        return true;
    }

    // 是否被king攻击
    let king_src = KING_ATTACKS[sq_idx] & board.piece_kind(by_color, PieceKind::King);
    if !king_src.is_empty() {
        return true;
    }

    let occupied = board.occupied();

    // 是否被bishop queen攻击
    let bishop_queen_src = bishop_rays(sq, occupied)
        & (board.piece_kind(by_color, PieceKind::Bishop)
            | board.piece_kind(by_color, PieceKind::Queen));
    if !bishop_queen_src.is_empty() {
        return true;
    }

    // 是否被rook queen攻击
    let rook_queen_src = rook_rays(sq, occupied)
        & (board.piece_kind(by_color, PieceKind::Rook)
            | board.piece_kind(by_color, PieceKind::Queen));
    if !rook_queen_src.is_empty() {
        return true;
    }

    false
}

fn generate_knight_attacks(sq: Square) -> BitBoard {
    let mut result = BitBoard::empty();
    let file = sq.file() as i8;
    let rank = sq.rank() as i8;

    const OFFSETS: [(i8, i8); 8] = [
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];

    for (df, dr) in OFFSETS {
        let f = file + df;
        let r = rank + dr;
        if in_board(f, r) {
            set_bit_board_from_coord(&mut result, f as u8, r as u8);
        }
    }

    result
}

fn generate_king_attacks(sq: Square) -> BitBoard {
    let mut result = BitBoard::empty();
    let file = sq.file() as i8;
    let rank = sq.rank() as i8;

    for df in -1..=1 {
        for dr in -1..=1 {
            if df == 0 && dr == 0 {
                continue;
            }
            let f = file + df;
            let r = rank + dr;
            if in_board(f, r) {
                set_bit_board_from_coord(&mut result, f as u8, r as u8);
            }
        }
    }

    result
}

fn generate_pawn_attacks(sq: Square, color: Color) -> BitBoard {
    let mut result = BitBoard::empty();
    let file = sq.file() as i8;
    let rank = sq.rank() as i8;
    let direction = match color {
        Color::White => 1,
        Color::Black => -1,
    };

    for df in [-1, 1] {
        let f = file + df;
        let r = rank + direction;
        if in_board(f, r) {
            set_bit_board_from_coord(&mut result, f as u8, r as u8);
        }
    }

    result
}

fn in_board(file: i8, rank: i8) -> bool {
    (0..8).contains(&file) && (0..8).contains(&rank)
}

fn set_bit_board_from_coord(bb: &mut BitBoard, f: u8, r: u8) -> Square {
    // SAFETY: f and r is valid here
    let target = Square::from_coord(f as u8, r as u8).unwrap();
    bb.set(target);
    target
}

fn sliding_attack(sq: Square, occupied: BitBoard, directions: &[(i8, i8)]) -> BitBoard {
    let mut result = BitBoard::empty();
    let file = sq.file() as i8;
    let rank = sq.rank() as i8;

    for &(df, dr) in directions {
        let mut f = file + df;
        let mut r = rank + dr;
        while in_board(f, r) {
            let target = set_bit_board_from_coord(&mut result, f as u8, r as u8);
            // 遇到棋子停止
            if occupied.contains(target) {
                break;
            }
            f += df;
            r += dr;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Board, Piece};

    #[test]
    fn test_knight_center_attack() {
        // 马e4，攻击: c3 c5 d2 d6 f2 f6 g3 g5
        let attacks = KNIGHT_ATTACKS[Square::E4.index()];
        let expected = [
            Square::C3,
            Square::C5,
            Square::D2,
            Square::D6,
            Square::F2,
            Square::F6,
            Square::G3,
            Square::G5,
        ];

        for sq in expected {
            assert!(attacks.contains(sq), "Knight should attack {}", sq);
        }
        assert_eq!(attacks.pop_count(), 8);
    }

    #[test]
    fn test_knight_corner_attack() {
        // a1 的马只能攻击 b3 c2
        let attacks = KNIGHT_ATTACKS[Square::A1.index()];

        assert!(attacks.contains(Square::B3));
        assert!(attacks.contains(Square::C2));
        assert_eq!(attacks.pop_count(), 2);
    }

    #[test]
    fn test_king_center_attack() {
        let attacks = KING_ATTACKS[Square::E4.index()];
        let expected = [
            Square::D3,
            Square::E3,
            Square::F3,
            Square::D4,
            Square::F4,
            Square::D5,
            Square::E5,
            Square::F5,
        ];

        for sq in expected {
            assert!(attacks.contains(sq));
        }
        assert_eq!(attacks.pop_count(), 8);
    }

    #[test]
    fn test_king_corner_attack() {
        let attacks = KING_ATTACKS[Square::A1.index()];

        assert!(attacks.contains(Square::A2));
        assert!(attacks.contains(Square::B1));
        assert!(attacks.contains(Square::B2));
        assert_eq!(attacks.pop_count(), 3);
    }

    #[test]
    fn test_white_pawn_attack() {
        // 白兵e4: 攻击 d5 f5
        let attacks = PAWN_ATTACKS[Color::White as usize][Square::E4.index()];

        assert!(attacks.contains(Square::D5));
        assert!(attacks.contains(Square::F5));
        assert_eq!(attacks.pop_count(), 2);
    }

    #[test]
    fn test_black_pawn_attack() {
        // 黑兵 e5: 攻击 d4 f4
        let attacks = PAWN_ATTACKS[Color::Black as usize][Square::E5.index()];

        assert!(attacks.contains(Square::D4));
        assert!(attacks.contains(Square::F4));
    }

    #[test]
    fn test_rook_empty_board() {
        let attacks = rook_rays(Square::E4, BitBoard::empty());
        assert_eq!(attacks.pop_count(), 14); // 横向7 + 纵向7
    }

    #[test]
    fn test_rook_blocked() {
        let mut occupied = BitBoard::empty();
        occupied.set(Square::E6); // e6阻挡

        let attacks = rook_rays(Square::E4, occupied);

        assert!(attacks.contains(Square::E5));
        assert!(attacks.contains(Square::E6)); // 可以攻击阻挡棋子所在位置
        assert!(!attacks.contains(Square::E7)); // 不能穿过
    }

    #[test]
    fn test_bishop_empty_board() {
        let attacks = bishop_rays(Square::E4, BitBoard::empty());
        assert_eq!(attacks.pop_count(), 13);
    }

    #[test]
    fn test_queen_empty_board() {
        let attacks = queen_rays(Square::E4, BitBoard::empty());
        assert_eq!(attacks.pop_count(), 27); // rook 14 + bishop 13
    }

    #[test]
    fn test_attacked_by_knight() {
        let mut board = Board::default();
        board.add_piece(Square::E4, Piece::new(Color::Black, PieceKind::Knight));

        assert!(is_square_attacked(&board, Square::F6, Color::Black));
    }

    #[test]
    fn test_attacked_by_rook() {
        let mut board = Board::default();
        board.add_piece(Square::A1, Piece::new(Color::White, PieceKind::Rook));

        assert!(is_square_attacked(&board, Square::A8, Color::White));
    }

    #[test]
    fn test_rook_blocked_not_attacked() {
        let mut board = Board::default();
        board.add_piece(Square::A1, Piece::new(Color::White, PieceKind::Rook));
        board.add_piece(Square::A4, Piece::new(Color::Black, PieceKind::Pawn));

        assert!(!is_square_attacked(&board, Square::A8, Color::White));
    }

    #[test]
    fn test_attacked_by_queen() {
        let mut board = Board::default();
        board.add_piece(Square::D4, Piece::new(Color::Black, PieceKind::Queen));

        assert!(is_square_attacked(&board, Square::H8, Color::Black));
    }
}
