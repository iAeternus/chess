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
    todo!()
}

/// 车攻击表，上下左右4个方向
pub fn rook_rays(sq: Square, occupied: BitBoard) -> BitBoard {
    todo!()
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
    let color = by_color as usize;
    let sq_idx = sq.index();

    // 是否被pawn攻击
    let pawn_src = PAWN_ATTACKS[!color][sq_idx] & board.piece_kind(by_color, PieceKind::Pawn);
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
