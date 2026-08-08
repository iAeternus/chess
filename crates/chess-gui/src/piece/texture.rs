//! 棋子纹理管理器。
//!
//! 在应用启动时将 12 个 SVG 棋子素材光栅化为 egui 纹理，提供高效的渲染接口。

use chess_core::{Color, PieceKind};
use egui::{Color32, Pos2, Rect, TextureHandle, TextureOptions};
use std::collections::HashMap;

/// 棋子纹理管理器：持有所有 12 个棋子纹理
pub struct PieceTextureManager {
    textures: HashMap<(Color, PieceKind), TextureHandle>,
    piece_size: f32,
}

impl PieceTextureManager {
    /// 从内嵌 SVG 资源加载并光栅化所有棋子纹理。
    ///
    /// `texture_size` 指定光栅化尺寸（像素），建议 128 以保证高 DPI 显示质量。
    pub fn new(ctx: &egui::Context, texture_size: u32) -> Self {
        let mut textures = HashMap::new();

        let pieces: &[(Color, PieceKind, &[u8])] = &[
            (
                Color::White,
                PieceKind::King,
                include_bytes!("../../assets/w_king.svg"),
            ),
            (
                Color::White,
                PieceKind::Queen,
                include_bytes!("../../assets/w_queen.svg"),
            ),
            (
                Color::White,
                PieceKind::Rook,
                include_bytes!("../../assets/w_rook.svg"),
            ),
            (
                Color::White,
                PieceKind::Bishop,
                include_bytes!("../../assets/w_bishop.svg"),
            ),
            (
                Color::White,
                PieceKind::Knight,
                include_bytes!("../../assets/w_knight.svg"),
            ),
            (
                Color::White,
                PieceKind::Pawn,
                include_bytes!("../../assets/w_pawn.svg"),
            ),
            (
                Color::Black,
                PieceKind::King,
                include_bytes!("../../assets/king.svg"),
            ),
            (
                Color::Black,
                PieceKind::Queen,
                include_bytes!("../../assets/queen.svg"),
            ),
            (
                Color::Black,
                PieceKind::Rook,
                include_bytes!("../../assets/rook.svg"),
            ),
            (
                Color::Black,
                PieceKind::Bishop,
                include_bytes!("../../assets/bishop.svg"),
            ),
            (
                Color::Black,
                PieceKind::Knight,
                include_bytes!("../../assets/knight.svg"),
            ),
            (
                Color::Black,
                PieceKind::Pawn,
                include_bytes!("../../assets/pawn.svg"),
            ),
        ];

        for (color, kind, svg_bytes) in pieces {
            let name = format!("piece_{:?}_{:?}", color, kind);
            let texture = Self::load_svg_texture(ctx, svg_bytes, &name, texture_size);
            textures.insert((*color, *kind), texture);
        }

        Self {
            textures,
            piece_size: texture_size as f32,
        }
    }

    /// 获取指定棋子的纹理句柄
    pub fn get(&self, color: Color, kind: PieceKind) -> &TextureHandle {
        &self.textures[&(color, kind)]
    }

    /// 在指定位置渲染棋子
    pub fn render(
        &self,
        painter: &egui::Painter,
        color: Color,
        kind: PieceKind,
        center: Pos2,
        size: f32,
    ) {
        let texture = self.get(color, kind);

        let half = size / 2.0;
        let rect = Rect::from_min_max(
            Pos2::new(center.x - half, center.y - half),
            Pos2::new(center.x + half, center.y + half),
        );

        // 图像 UV：完整图像 (0,0) -> (1,1)
        let uv = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));

        painter.image(texture.id(), rect, uv, Color32::WHITE);
    }

    /// 获取棋子纹理的原始像素尺寸
    #[allow(dead_code)]
    pub fn texture_size(&self) -> f32 {
        self.piece_size
    }

    /// 将 SVG 字节光栅化为 egui 纹理
    fn load_svg_texture(
        ctx: &egui::Context,
        svg_bytes: &[u8],
        name: &str,
        size: u32,
    ) -> TextureHandle {
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(svg_bytes, &opt)
            .unwrap_or_else(|e| panic!("failed to parse SVG '{name}': {e}"));

        // 计算缩放比例
        let svg_size = tree.size();
        let scale_x = size as f32 / svg_size.width();
        let scale_y = size as f32 / svg_size.height();
        let scale = scale_x.min(scale_y);

        // 光栅化
        let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)
            .unwrap_or_else(|| panic!("failed to create pixmap for '{name}'"));
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // 转换为 egui::ColorImage
        let image =
            egui::ColorImage::from_rgba_unmultiplied([size as usize, size as usize], pixmap.data());

        // 加载为纹理
        ctx.load_texture(name.to_string(), image, TextureOptions::LINEAR)
    }
}
