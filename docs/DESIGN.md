# Rust 国际象棋引擎 — 详细开发设计文档

## 概述

本文档描述一个现代化、模块化、高性能的 Rust 国际象棋引擎的完整架构设计。设计参考 Stockfish 等现代棋类引擎，充分利用 Rust 类型系统和零成本抽象，以 Bitboard 为核心数据结构。

---

## 一、项目架构总览

### 1.1 Workspace 结构

```
chess-engine/
├── Cargo.toml                  # workspace 根配置
├── README.md
├── DESIGN.md                   # 本文档
├── rust-toolchain.toml         # 固定工具链版本
├── .rustfmt.toml               # 格式化配置
├── clippy.toml                 # Clippy 配置
│
├── chess-core/                 # 核心引擎 crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # crate 根，re-export 所有公开类型
│       ├── square.rs           # Square 类型 (0-63)
│       ├── color.rs            # Color 枚举 (White/Black)
│       ├── piece.rs            # PieceKind + Piece 组合类型
│       ├── bitboard.rs         # BitBoard(u64) + 运算
│       ├── board.rs            # 12 bitboard 棋盘表示
│       ├── position.rs         # 完整棋局状态
│       ├── mv.rs               # 压缩 Move(u32)
│       ├── movegen/            # 走法生成模块
│       │   ├── mod.rs          # MoveGenerator trait + 调度
│       │   ├── pawn.rs         # 兵走法生成
│       │   ├── knight.rs       # 马走法生成
│       │   ├── bishop.rs       # 象走法生成
│       │   ├── rook.rs         # 车走法生成
│       │   ├── queen.rs        # 后走法生成
│       │   └── king.rs         # 王走法生成
│       ├── attack.rs           # 攻击表 + is_attacked
│       ├── legality.rs         # 走法合法性检查
│       ├── makemove.rs         # MoveMaker: apply + undo
│       ├── zobrist.rs          # Zobrist 哈希
│       ├── game.rs             # Game 高级 API
│       ├── fen.rs              # FEN 解析与输出
│       ├── castling.rs         # 王车易位权限
│       ├── perft.rs            # Perft 测试工具
│       └── error.rs            # 错误类型
│
├── chess-ai/                   # AI 引擎 crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── engine.rs           # ChessEngine trait
│       ├── random.rs           # RandomEngine
│       ├── alphabeta.rs        # AlphaBetaEngine
│       ├── eval.rs             # 局面评估
│       └── search.rs           # 搜索框架
│
├── chess-cli/                  # CLI 界面 crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # 入口
│       ├── uci.rs              # UCI 协议
│       └── repl.rs             # 交互式 REPL
│
├── chess-gui/                  # GUI crate (后续)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── view.rs             # BoardView trait
│
└── benches/                    # 性能基准测试
    ├── Cargo.toml
    └── src/
        └── bench.rs
```

### 1.2 依赖关系图

```
┌──────────────┐     ┌──────────────┐
│  chess-gui   │     │  chess-cli   │
│  (BoardView) │     │  (UCI/REPL)  │
└──────┬───────┘     └──────┬───────┘
       │                    │
       └────────┬───────────┘
                │
         ┌──────┴──────┐
         │   chess-ai   │
         │ (Engine trait)│
         └──────┬──────┘
                │
         ┌──────┴──────┐
         │  chess-core  │
         │  (纯 Rust)   │
         └─────────────┘
```

**规则**：
- `chess-core` 零外部依赖（仅使用 `std`，可选 `bitflags`）
- `chess-ai` 仅依赖 `chess-core`
- `chess-gui` / `chess-cli` 依赖 `chess-core` + `chess-ai`
- 任何 crate 不得绕过 `chess-core` 的公共 API 直接操作内部状态

### 1.3 crate 类型与版本

| Crate | 类型 | Rust Edition | 主要依赖 |
|-------|------|-------------|---------|
| chess-core | lib | 2024 | `bitflags` (可选), `arrayvec` |
| chess-ai | lib | 2024 | `chess-core`, `rand` |
| chess-cli | bin | 2024 | `chess-core`, `chess-ai`, `clap` |
| chess-gui | lib | 2024 | `chess-core` |
| benches | bench | 2024 | `chess-core`, `chess-ai`, `criterion` |

---

## 二、chess-core 详细设计

### 2.1 Square（棋盘格）

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Square(u8);
```

**设计要点**：
- 内部 `u8`，范围 0-63，对应 a1=0, b1=1, ..., h8=63（Little-Endian Rank-File mapping）
- 构造函数 `Square::new(index: u32) -> Option<Square>` —— 返回 `Option`，非法值返回 `None`
- 提供 `unsafe Square::new_unchecked(index: u32) -> Square` 用于性能关键路径（需证明安全性）
- 方法：
  - `index(&self) -> usize` —— 数组索引
  - `bit(&self) -> u64` —— 返回 `1u64 << self.0`
  - `rank(&self) -> u8` —— 0-7 (rank 1 = 0)
  - `file(&self) -> u8` —— 0-7 (a file = 0)
  - `from_coord(file: u8, rank: u8) -> Option<Square>` —— 从坐标构造
  - `Display` trait —— 输出 "e4" 这样的代数表示

**常量**：提供 `Square::A1` ~ `Square::H8` 共 64 个命名常量。

### 2.2 Color（颜色）

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}
```

- `#[repr(u8)]` 保证可以安全 transmute 为 `usize` 用于数组索引
- 提供 `flip(&self) -> Color` —— 切换颜色
- 提供 `From<Color> for usize` 实现

### 2.3 Piece（棋子）

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum PieceKind {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}
```

- `PieceKind` 提供 `const COUNT: usize = 6`
- `Piece` 提供 `to_char(&self) -> char`（如白兵 = 'P'，黑后 = 'q'）
- 提供 `Piece::new(color: Color, kind: PieceKind) -> Piece`
- `From<PieceKind> for usize` 和 `From<Color> for usize` 实现

### 2.4 BitBoard（位棋盘）

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BitBoard(u64);
```

**核心方法**：

| 方法 | 签名 | 说明 |
|------|------|------|
| `empty()` | `-> Self` | 返回空位棋盘 |
| `full()` | `-> Self` | 返回全满位棋盘 |
| `set(&mut self, sq: Square)` | | 设置一位 |
| `clear(&mut self, sq: Square)` | | 清除一位 |
| `contains(&self, sq: Square) -> bool` | | 测试一位 |
| `is_empty(&self) -> bool` | | 是否为空 |
| `pop_count(&self) -> u32` | | 1 的个数（使用 `count_ones()`） |
| `lsb(&self) -> Option<Square>` | | 最低有效位（bitscan forward） |
| `pop_lsb(&mut self) -> Option<Square>` | | 弹出最低有效位 |
| `as_u64(&self) -> u64` | | 获取原始值 |

**Trait 实现**：

```rust
impl BitOr for BitBoard { type Output = Self; }   // 并集
impl BitAnd for BitBoard { type Output = Self; }   // 交集
impl Not for BitBoard { type Output = Self; }      // 补集
impl BitOrAssign for BitBoard { }                   // |=
impl BitAndAssign for BitBoard { }                  // &=
impl BitXor for BitBoard { type Output = Self; }   // 异或
```

**迭代器**：

```rust
pub struct BitBoardIter {
    bb: BitBoard,
}

impl Iterator for BitBoardIter {
    type Item = Square;
    // 每次 pop_lsb，返回 Square
}

impl IntoIterator for BitBoard {
    type Item = Square;
    type IntoIter = BitBoardIter;
}
```

这样支持 `for sq in bitboard { ... }` 语法。

**位运算优化**：`pop_lsb()` 使用 `bb.0.trailing_zeros()` 和 `bb.0 &= bb.0 - 1`（Brian Kernighan 算法）。

### 2.5 Board（棋盘）

```rust
pub struct Board {
    pieces: [[BitBoard; 6]; 2],  // [color][piece_kind]
    by_square: [Option<Piece>; 64], // 快速反向查找（cache-friendly）
}
```

**设计理由**：
- `pieces[color][kind]` 是主要的位棋盘存储，12 个 BitBoard
- `by_square` 提供 O(1) 的 `piece_at(sq)` 查询，避免遍历 12 个位棋盘
- 两个数据结构保持同步更新

**核心方法**：

```rust
impl Board {
    pub fn new() -> Self;
    pub fn piece_at(&self, sq: Square) -> Option<Piece>;
    pub fn pieces(&self, color: Color) -> BitBoard;       // 某颜色所有棋子
    pub fn pieces_kind(&self, color: Color, kind: PieceKind) -> BitBoard;
    pub fn occupied(&self) -> BitBoard;                    // 所有棋子
    pub fn occupied_by(&self, color: Color) -> BitBoard;   // 某颜色棋子
    pub fn empty(&self) -> BitBoard;                       // 空格
    pub fn add_piece(&mut self, sq: Square, piece: Piece);
    pub fn remove_piece(&mut self, sq: Square) -> Option<Piece>;
    pub fn move_piece(&mut self, from: Square, to: Square) -> Option<Piece>;
    pub fn king_square(&self, color: Color) -> Square;     // 王的位置
}
```

**不变量**：`by_square` 与 `pieces` 位棋盘始终一致。

### 2.6 CastlingRights（王车易位权限）

```rust
use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct CastlingRights: u8 {
        const NONE = 0;
        const WHITE_KING_SIDE = 1 << 0;
        const WHITE_QUEEN_SIDE = 1 << 1;
        const BLACK_KING_SIDE = 1 << 2;
        const BLACK_QUEEN_SIDE = 1 << 3;
        const ALL = WHITE_KING_SIDE | WHITE_QUEEN_SIDE
                  | BLACK_KING_SIDE | BLACK_QUEEN_SIDE;
    }
}
```

**相关常量**（关联到 Square）：
- 王车易位涉及的关键格：初始王/车位置、经过格、目标格
- 掩码：王翼空位掩码、后翼空位掩码、攻击检测掩码

### 2.7 Move（走法）

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Move(u32);
```

**位布局**：

```
bits 0-5:   from square (6 bits, 0-63)
bits 6-11:  to square   (6 bits, 0-63)
bits 12-15: promotion   (4 bits, PieceKind)
bits 16-19: flags       (4 bits, MoveFlag)
bits 20-31: reserved    (12 bits, for future: score, etc.)
```

**MoveFlag 枚举**：

```rust
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum MoveFlag {
    Quiet = 0,
    DoublePawnPush = 1,
    KingCastle = 2,
    QueenCastle = 3,
    Capture = 4,
    EnPassant = 5,
    Promotion = 8,        // bit 3 set = promotion
    // Promotion + piece kind
    KnightPromotion = 8,  // 1000
    BishopPromotion = 9,  // 1001
    RookPromotion = 10,   // 1010
    QueenPromotion = 11,  // 1011
    // Promotion capture variants
    KnightPromotionCapture = 12, // 1100
    BishopPromotionCapture = 13, // 1101
    RookPromotionCapture = 14,   // 1110
    QueenPromotionCapture = 15,  // 1111
}
```

**Move 方法**：

```rust
impl Move {
    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Self;
    pub fn new_promotion(from: Square, to: Square, kind: PieceKind, capture: bool) -> Self;
    pub const NULL: Self;  // 空走法 (0)
    pub fn from(&self) -> Square;
    pub fn to(&self) -> Square;
    pub fn flag(&self) -> MoveFlag;
    pub fn promotion(&self) -> Option<PieceKind>;
    pub fn is_capture(&self) -> bool;
    pub fn is_promotion(&self) -> bool;
    pub fn is_castle(&self) -> bool;
    pub fn is_quiet(&self) -> bool;
}
```

### 2.8 Position（棋局状态）

```rust
pub struct Position {
    board: Board,
    side_to_move: Color,
    castling: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u32,    // 50步规则计数器
    fullmove_number: u32,   // 从1开始
    zobrist_key: u64,       // Zobrist 哈希（增量更新）
    // 可选：局面历史哈希（用于三次重复检测）
}
```

**公开 API**（只读访问）：

```rust
impl Position {
    pub fn board(&self) -> &Board;
    pub fn side_to_move(&self) -> Color;
    pub fn castling(&self) -> CastlingRights;
    pub fn en_passant(&self) -> Option<Square>;
    pub fn halfmove_clock(&self) -> u32;
    pub fn fullmove_number(&self) -> u32;
    pub fn zobrist_key(&self) -> u64;
    pub fn piece_at(&self, sq: Square) -> Option<Piece>;
}
```

**FEN 支持**：
- `Position::from_fen(fen: &str) -> Result<Position, ChessError>` —— 解析 FEN
- `Position::to_fen(&self) -> String` —— 输出 FEN
- 支持标准起始局面：`startpos()` 快捷方法

### 2.9 攻击表模块（attack.rs）

**预计算查找表**：

```rust
// 64 个元素，每个预先计算好
pub static KNIGHT_ATTACKS: [BitBoard; 64] = /* 编译期/启动时初始化 */;
pub static KING_ATTACKS: [BitBoard; 64] = /* ... */;
pub static PAWN_ATTACKS: [[BitBoard; 64]; 2] = /* [color][square] */;
```

**滑动棋子射线**：

```rust
// 使用 const fn 或 lazy_static/OnceLock 初始化
pub fn bishop_rays(sq: Square, occupied: BitBoard) -> BitBoard;
pub fn rook_rays(sq: Square, occupied: BitBoard) -> BitBoard;
pub fn queen_rays(sq: Square, occupied: BitBoard) -> BitBoard {
    bishop_rays(sq, occupied) | rook_rays(sq, occupied)
}
```

**射线计算算法**（Phase 3 实现）：
- 经典方法：对 4 个方向（主教）/ 4 个方向（车），使用 `trailing_zeros` 扫描到第一个阻挡
- Phase 8 优化：升级为 Magic Bitboard

**核心检查函数**：

```rust
/// 判断 sq 是否被 attack_color 颜色的棋子攻击
pub fn is_square_attacked(
    board: &Board,
    sq: Square,
    by_color: Color,
) -> bool;
```

**检测顺序**（从便宜到贵）：
1. 兵攻击（查 PAWN_ATTACKS 表）
2. 马攻击（查 KNIGHT_ATTACKS 表）
3. 王攻击（查 KING_ATTACKS 表）
4. 滑动棋子（计算射线，检查对应棋子存在性）

### 2.10 走法生成模块（movegen/）

**MoveGenerator trait**：

```rust
pub trait MoveGenerator {
    /// 生成指定局面的所有伪合法走法
    fn generate(&self, position: &Position) -> Vec<Move>;
}
```

**各棋子生成器设计**：

| 模块 | 核心算法 |
|------|---------|
| `pawn.rs` | 查找表 + 位运算：前推、双推、攻击（吃子+过路兵）、升变 |
| `knight.rs` | 查 KNIGHT_ATTACKS 表，过滤己方棋子 |
| `bishop.rs` | 计算主教射线，过滤己方棋子 |
| `rook.rs` | 计算车射线，过滤己方棋子 |
| `queen.rs` | 主教射线 ∪ 车射线 |
| `king.rs` | 查 KING_ATTACKS 表 + 王车易位生成 |

**聚合生成器**：

```rust
pub struct LegalMoveGenerator;

impl LegalMoveGenerator {
    /// 生成所有合法走法（伪合法 + 合法性过滤）
    pub fn generate_legal(position: &Position) -> Vec<Move>;

    /// 生成所有伪合法走法
    pub fn generate_pseudo_legal(position: &Position) -> Vec<Move>;
}
```

**性能设计**：
- 使用栈分配的 `ArrayVec<Move, 256>` 避免堆分配（最大合法走法数约 218）
- 攻击检测短路：先检测简单攻击（兵、马），再检测滑动棋子
- 王车易位合法性：先检查空位，再检查攻击

### 2.11 走法执行模块（makemove.rs）

**Undo 结构**：

```rust
pub struct Undo {
    mv: Move,
    captured: Option<Piece>,
    prev_castling: CastlingRights,
    prev_en_passant: Option<Square>,
    prev_halfmove: u32,
    prev_zobrist: u64,
}
```

**MoveMaker**：

```rust
pub struct MoveMaker;

impl MoveMaker {
    /// 执行走法，返回 Undo 信息
    pub fn make_move(position: &mut Position, mv: Move) -> Undo;

    /// 撤销走法
    pub fn unmake_move(position: &mut Position, undo: Undo);
}
```

**make_move 执行流程**：

```
1. 保存 Undo 信息
2. 清除 en_passant（除非本次产生新的）
3. 根据 MoveFlag 分发：
   - Quiet: 简单移动棋子
   - DoublePawnPush: 移动棋子 + 设置 en_passant
   - KingCastle: 移动王 e->g + 移动车 h->f
   - QueenCastle: 移动王 e->c + 移动车 a->d
   - Capture: 移除目标格棋子 + 移动棋子
   - EnPassant: 移除过路兵 + 移动兵
   - Promotion*: 移除目标格棋子(如有) + 替换升变棋子
4. 更新 CastlingRights（如涉及王或车移动）
5. 更新 halfmove_clock / fullmove_number
6. 增量更新 zobrist_key
7. 切换 side_to_move
```

**unmake_move 执行流程**：完全逆向以上步骤，使用 Undo 恢复状态。

### 2.12 Zobrist 哈希（zobrist.rs）

```rust
pub struct Zobrist;

impl Zobrist {
    /// 初始化所有随机数（使用确定性种子或固定随机数）
    pub fn init();

    /// 棋子键值
    pub fn piece_key(piece: Piece, sq: Square) -> u64;

    /// 王车易位键值
    pub fn castling_key(castling: CastlingRights) -> u64;

    /// 过路兵键值
    pub fn en_passant_key(sq: Option<Square>) -> u64;

    /// 走子方键值（黑方回合的 XOR 常数）
    pub fn side_key() -> u64;
}
```

**增量更新**：Position 的 `zobrist_key` 在 `make_move`/`unmake_move` 中增量更新，避免每次重新计算。

### 2.13 Game API（game.rs）

```rust
pub struct Game {
    position: Position,
    history: Vec<(Move, Undo)>,
}

impl Game {
    /// 从标准起始局面创建
    pub fn new() -> Self;

    /// 从 FEN 创建
    pub fn from_fen(fen: &str) -> Result<Self, ChessError>;

    /// 获取当前局面（只读引用）
    pub fn position(&self) -> &Position;

    /// 获取当前所有合法走法
    pub fn legal_moves(&self) -> Vec<Move>;

    /// 执行走法
    pub fn play(&mut self, mv: Move) -> Result<(), ChessError>;

    /// 撤销上一步走法
    pub fn undo(&mut self) -> Result<(), ChessError>;

    /// 检测游戏是否结束
    pub fn is_game_over(&self) -> bool;

    /// 检测是否将军
    pub fn is_check(&self) -> bool;

    /// 检测是否将杀
    pub fn is_checkmate(&self) -> bool;

    /// 检测是否逼和
    pub fn is_stalemate(&self) -> bool;

    /// 走法历史
    pub fn history(&self) -> &[(Move, Undo)];

    /// 重置
    pub fn reset(&mut self);
}
```

**设计原则**：GUI 和 AI 只能通过 `Game` 类型操作棋局，无法访问内部 Bitboard。

### 2.14 错误类型（error.rs）

```rust
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ChessError {
    InvalidSquare(u32),
    InvalidFen(String),
    InvalidMove(Move),
    NoKing(Color),
    GameOver,
    NothingToUndo,
    InvalidPromotion,
    ParseError(String),
}

impl std::fmt::Display for ChessError { ... }
impl std::error::Error for ChessError { ... }
```

---

## 三、chess-ai 设计

### 3.1 ChessEngine trait

```rust
pub trait ChessEngine {
    /// 搜索并返回最佳走法
    fn search(&mut self, position: &Position) -> Option<Move>;

    /// 引擎名称
    fn name(&self) -> &str;

    /// 设置搜索时间限制（毫秒）
    fn set_time_limit(&mut self, ms: u64);

    /// 设置搜索深度限制
    fn set_depth_limit(&mut self, depth: u32);
}
```

### 3.2 RandomEngine

- 从合法走法中随机选择
- 用于测试和基准对比
- 实现 `ChessEngine` trait

### 3.3 AlphaBetaEngine

```rust
pub struct AlphaBetaEngine {
    max_depth: u32,
    time_limit_ms: Option<u64>,
    nodes_searched: u64,
    transposition_table: TranspositionTable,  // Phase 8
}

impl AlphaBetaEngine {
    pub fn new(max_depth: u32) -> Self;

    fn alpha_beta(
        &mut self,
        position: &mut Position,
        depth: u32,
        alpha: i32,
        beta: i32,
    ) -> i32;

    fn quiescence_search(
        &mut self,
        position: &mut Position,
        alpha: i32,
        beta: i32,
    ) -> i32;

    fn evaluate(&self, position: &Position) -> i32;
}
```

**搜索优化（Phase 8）**：
- 走法排序（MVV-LVA）
- 迭代加深
- 置换表
- 空着裁剪（Null Move Pruning）
- 杀手走法（Killer Moves）
- 历史启发（History Heuristic）

### 3.4 评估函数（eval.rs）

**Phase 7 基本评估**：

```
Score = Material + PieceSquareTable + Mobility
```

- **子力价值**：P=100, N=320, B=330, R=500, Q=900, K=20000
- **棋子位置表**（Piece-Square Tables）：从白方视角，黑方镜像
- **基本机动性**：合法走法数量的少量加分

**Phase 8 扩展**：
- 兵结构（孤兵、叠兵、通路兵）
- 王安全
- 为 NNUE 预留输入接口

---

## 四、chess-cli 设计

### 4.1 命令行接口

```
chess-cli 0.1.0
Rust Chess Engine

USAGE:
    chess-cli [OPTIONS] [COMMAND]

COMMANDS:
    play       启动交互式对局
    perft      运行 perft 测试
    bench      运行基准测试
    uci        启动 UCI 模式
    fen <FEN>  从 FEN 加载并分析局面

OPTIONS:
    -d, --depth <N>     搜索深度 [default: 6]
    -t, --time <MS>     时间限制 (ms)
    -e, --engine <NAME> 引擎选择 [default: alphabeta]
```

### 4.2 UCI 协议（uci.rs）

实现标准 UCI 协议子集：
- `uci` —— 引擎标识
- `isready` / `readyok` —— 就绪检查
- `position [fen <fen> | startpos] [moves ...]` —— 设置局面
- `go [depth <n> | movetime <ms>]` —— 开始搜索
- `stop` —— 停止搜索
- `quit` —— 退出

---

## 五、chess-gui 设计（后续阶段）

### 5.1 BoardView trait

```rust
pub trait BoardView {
    /// 返回所有棋子的位置和类型
    fn pieces(&self, position: &Position) -> Vec<(Square, Piece)>;

    /// 返回上次走法（用于高亮）
    fn last_move(&self) -> Option<Move>;

    /// 合法走法标记格
    fn legal_move_squares(&self, from: Square) -> Vec<Square>;
}
```

GUI 通过 `BoardView` 获得渲染所需信息，完全隔离内部实现。

---

## 六、测试策略

### 6.1 单元测试层级

| 层级 | 测试内容 | 目标覆盖率 |
|------|---------|-----------|
| L1 | Square, BitBoard, Piece 基础类型 | 100% |
| L2 | Board 操作 | 100% |
| L3 | Move 编解码 | 100% |
| L4 | 走法生成（每种棋子） | 95%+ |
| L5 | 特殊规则（易位/过路兵/升变） | 100% |
| L6 | FEN 解析/输出往返 | 100% |
| L7 | Game API | 90%+ |

### 6.2 棋规测试用例

必须通过的测试：

```rust
#[test]
fn initial_position_20_moves();      // 起始局面 20 个合法走法
#[test]
fn castling_through_check_blocked(); // 王经过被攻击格不能易位
#[test]
fn en_passant_capture();             // 吃过路兵
#[test]
fn en_passant_blocked();             // 过路兵被阻挡
#[test]
fn promotion_all_types();            // 四种升变
#[test]
fn check_detection();                // 将军检测
#[test]
fn checkmate_detection();            // 将杀检测
#[test]
fn stalemate_detection();            // 逼和检测
#[test]
fn castling_rights_update();         // 易位权限更新
```

### 6.3 Perft 测试

```rust
#[test]
fn perft_startpos() {
    let pos = Position::startpos();
    assert_eq!(perft(&mut pos, 1), 20);
    assert_eq!(perft(&mut pos, 2), 400);
    assert_eq!(perft(&mut pos, 3), 8902);
    assert_eq!(perft(&mut pos, 4), 197281);
}

#[test]
fn perft_kiwipete() {
    // Kiwipete 局面的 perft 值
    let pos = Position::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
    ).unwrap();
    assert_eq!(perft(&mut pos, 1), 48);
    assert_eq!(perft(&mut pos, 2), 2039);
    assert_eq!(perft(&mut pos, 3), 97862);
}
```

### 6.4 集成测试

- `tests/integration_test.rs` —— 完整对局模拟
- `tests/perft_suite.rs` —— 标准 perft 套件（10+ 个已知局面）

---

## 七、开发阶段规划

### Phase 1：基础类型

**交付物**：
- `square.rs` —— Square 类型 + 所有常量
- `color.rs` —— Color 枚举
- `piece.rs` —— PieceKind, Piece
- `bitboard.rs` —— BitBoard + 运算符 + 迭代器
- `board.rs` —— Board（12 bitboard + by_square 数组）
- `error.rs` —— ChessError

### Phase 2：局状态与走法

**交付物**：
- `mv.rs` —— 压缩 Move + MoveFlag
- `castling.rs` —— CastlingRights bitflags
- `zobrist.rs` —— Zobrist 哈希表
- `fen.rs` —— FEN 解析/生成
- `position.rs` —— Position（组装以上所有）

### Phase 3：攻击表与走法生成

**交付物**：
- `attack.rs` —— 预计算表 + is_square_attacked
- `movegen/mod.rs` —— MoveGenerator trait + LegalMoveGenerator
- `movegen/pawn.rs` —— 兵走法生成
- `movegen/knight.rs` —— 马走法生成
- `movegen/bishop.rs` —— 象走法生成
- `movegen/rook.rs` —— 车走法生成
- `movegen/queen.rs` —— 后走法生成
- `movegen/king.rs` —— 王走法生成

### Phase 4：合法性检查

**交付物**：
- `legality.rs` —— is_legal + is_attacked
- 走法生成器集成合法性过滤

### Phase 5：走法执行与特殊规则

**交付物**：
- `makemove.rs` —— MoveMaker + Undo

### Phase 6：Game API + Perft

**交付物**：
- `game.rs` —— Game 高级 API
- `perft.rs` —— Perft 实现

### Phase 7：AI 引擎

**交付物**：
- `chess-ai` crate 结构
- `ChessEngine` trait
- `RandomEngine`
- `AlphaBetaEngine`（基础 Alpha-Beta + 静态搜索）
- `eval.rs`（基础评估函数）

### Phase 8：优化

**计划项**：
- Magic Bitboard（替换经典射线算法）
- 置换表（Transposition Table）
- 走法排序优化
- 迭代加深
- 空着裁剪
- Killer/History 启发
- NNUE 集成点预留
- Criterion benchmarks
- SIMD 加速（可选）

---

## 八、性能设计决策

### 8.1 零分配走法生成

- 走法生成器使用固定大小的栈数组（最大容量 256）
- 返回 `Vec<Move>` 用于 API 简洁性，内部路径使用 `ArrayVec`
- 后续可改为回调模式 `fn generate(&self, pos: &Position, f: impl FnMut(Move))` 避免分配

### 8.2 增量更新

| 数据 | 更新策略 |
|------|---------|
| Zobrist Key | XOR 增量：每次 make/unmake 只 XOR 变化的部分 |
| Halfmove Clock | 吃子/兵移动时归零，否则+1 |
| Fullmove Number | 黑方移动后+1 |
| CastlingRights | 按掩码清除受影响的权限 |

### 8.3 缓存友好的数据结构

- `Board::by_square: [Option<Piece>; 64]` 连续内存，一次缓存行加载
- `Position` 大小控制在 2 个缓存行以内（~128 bytes）
- `Undo` 大小控制在 1 个缓存行（~64 bytes）

---

## 九、API 可见性规则

### 9.1 chess-core 导出策略

`lib.rs` 中 re-export：

```rust
// 公共类型
pub use square::Square;
pub use color::Color;
pub use piece::{Piece, PieceKind};
pub use bitboard::BitBoard;
pub use board::Board;
pub use position::Position;
pub use mv::{Move, MoveFlag};
pub use castling::CastlingRights;
pub use game::Game;
pub use error::ChessError;
pub use fen::Fen;
pub use perft::perft;

// 内部模块不公开
mod attack;
mod movegen;
mod makemove;
mod zobrist;
mod legality;
```

### 9.2 访问控制矩阵

| 类型/方法 | chess-core 内部 | chess-ai | chess-gui/cli |
|-----------|----------------|----------|---------------|
| `Square`, `Piece`, `Color` | pub | pub | pub |
| `BitBoard` | pub | pub(crate) | 不可见 |
| `Board` 内部 | pub(crate) | 不可见 | 不可见 |
| `Position` 只读方法 | pub | pub | pub |
| `Position` 可变方法 | pub(crate) | 不可见 | 不可见 |
| `Game` | pub | pub | pub |
| `make_move` / `unmake_move` | pub(crate) | 不可见 | 不可见 |
| `MoveGenerator` | pub(crate) | 不可见 | 不可见 |

### 9.3 封装意图

```
外界 (GUI/CLI/AI)
    │
    ├─ Game (唯一可变入口)
    │   ├─ Game::play(Move)
    │   ├─ Game::undo()
    │   └─ Game::position() -> &Position
    │
    ├─ Position (只读)
    │   ├─ piece_at(Square) -> Option<Piece>
    │   ├─ side_to_move() -> Color
    │   ├─ legal_moves (via Game)
    │   └─ FEN 导入/导出
    │
    └─ 基础类型 (Square, Piece, Color, Move)
        └─ 用于显示和交互
```

---

## 十、扩展接口预留

### 10.1 NNUE 接口预留

```rust
// chess-ai/src/eval.rs 中预留
pub struct NnueEvaluator {
    // Phase 8+ 实现
    _reserved: (),
}

impl NnueEvaluator {
    pub fn load_weights(path: &str) -> Result<Self, ChessError>;
    pub fn evaluate(&self, position: &Position) -> i32;
}
```

### 10.2 搜索扩展点

```rust
// search.rs 中预留
pub trait SearchExtension {
    fn on_node_enter(&mut self, position: &Position, depth: u32);
    fn on_node_exit(&mut self, position: &Position, score: i32);
    fn prune(&self, position: &Position, depth: u32) -> Option<i32>;
}
```

### 10.3 UCI 扩展

```rust
// UCI 引擎可以注册自定义选项
pub trait UciOption {
    fn name(&self) -> &str;
    fn set_value(&mut self, value: &str) -> Result<(), ChessError>;
    fn get_value(&self) -> String;
}
```

---

## 十一、代码质量规范

### 11.1 Rust 工具链配置

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

### 11.2 必须遵守的规则

- **Edition 2024**：所有 crate 使用 `edition = "2024"`
- **零 unsafe**：整个项目不使用 `unsafe` 代码（Phase 8 如需 SIMD 优化时再评估）
- **禁止裸 usize**：所有棋盘位置使用 `Square` 类型
- **禁止 8x8 数组**：内部表示使用位棋盘，不兜底到 `[[Option<Piece>; 8]; 8]`
- **文档注释**：所有公开 API 必须有 `///` 注释
- **clippy clean**：`cargo clippy --all-targets -- -D warnings`
- **rustfmt**：所有代码通过 `cargo fmt --check`

### 11.3 命名约定

| 范畴 | 约定 | 示例 |
|------|------|------|
| 类型 | PascalCase | `AlphaBetaEngine` |
| 函数/方法 | snake_case | `is_square_attacked` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_PLY` |
| 模块 | snake_case | `movegen` |
| Crate | kebab-case | `chess-core` |

---

## 十二、文件清单

### chess-core/src/ 文件总览

| 文件 | 行数估算 | 核心职责 |
|------|---------|---------|
| `lib.rs` | ~30 | Re-export + crate 文档 |
| `square.rs` | ~150 | Square 类型 + 64 个常量 + Display |
| `color.rs` | ~30 | Color 枚举 + flip |
| `piece.rs` | ~60 | PieceKind + Piece |
| `bitboard.rs` | ~200 | BitBoard + 运算 + 迭代器 |
| `board.rs` | ~150 | Board: 12 bitboard + by_square |
| `position.rs` | ~120 | Position 结构 + 只读 API |
| `mv.rs` | ~200 | Move(u32) 压缩 + MoveFlag |
| `castling.rs` | ~30 | CastlingRights bitflags |
| `zobrist.rs` | ~80 | Zobrist 键表 |
| `fen.rs` | ~120 | FEN 解析/生成 |
| `attack.rs` | ~150 | 攻击表 + is_square_attacked |
| `movegen/mod.rs` | ~80 | MoveGenerator trait + 聚合 |
| `movegen/pawn.rs` | ~120 | 兵走法生成 |
| `movegen/knight.rs` | ~40 | 马走法生成 |
| `movegen/bishop.rs` | ~50 | 象走法生成 |
| `movegen/rook.rs` | ~50 | 车走法生成 |
| `movegen/queen.rs` | ~20 | 后走法生成 |
| `movegen/king.rs` | ~80 | 王走法生成 + 易位 |
| `legality.rs` | ~60 | is_legal 过滤 |
| `makemove.rs` | ~200 | make/unmake + Undo |
| `game.rs` | ~150 | Game 高级 API |
| `error.rs` | ~40 | ChessError |
| `perft.rs` | ~40 | perft 实现 |
| **总计** | ~2260 | |

---

## 十三、验证方案

### 端到端验证流程

```bash
# 1. 编译整个 workspace
cargo build --workspace

# 2. 运行所有测试
cargo test --workspace

# 3. 代码格式检查
cargo fmt --all -- --check

# 4. Clippy 检查
cargo clippy --workspace --all-targets -- -D warnings

# 5. 运行 perft 基准
cargo run -p chess-cli -- perft --depth 4

# 6. 运行性能基准
cargo bench -p benches

# 7. 交互式测试
cargo run -p chess-cli -- play --depth 4
```

### 持续集成检查项

- `cargo test --workspace` 全部通过
- `cargo fmt --check` 通过
- `cargo clippy -- -D warnings` 通过
- `cargo doc --no-deps` 无警告
- Perft(4) 值正确
- 零 `unsafe` 代码块
