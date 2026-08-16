//! 引擎信息面板：显示引擎名称、深度、评估、最佳走法。

use crate::theme::ThemeColors;

/// 引擎分析信息
#[derive(Debug, Clone, Default)]
pub struct EngineInfo {
    pub name: Option<String>,
    pub depth: Option<u32>,
    /// 评估值（厘兵，正数 = 白方优势）
    pub score_cp: Option<i32>,
    /// 将杀步数
    pub score_mate: Option<i32>,
    /// 最佳走法（SAN 格式）
    pub best_move: Option<String>,
    /// 主变走法列表
    pub pv: Vec<String>,
    /// 搜索节点数
    pub nodes: Option<u64>,
    /// 搜索速度（节点/秒）
    pub nps: Option<u64>,
}

pub struct EngineInfoPanel;

impl EngineInfoPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&self, ui: &mut egui::Ui, info: &EngineInfo, colors: &ThemeColors) {
        ui.heading("Engine");

        // 引擎名称
        if let Some(ref name) = info.name {
            ui.label(egui::RichText::new(format!("{name}")).size(15.0).strong());
        } else {
            ui.label(
                egui::RichText::new("No engine loaded")
                    .size(14.0)
                    .color(colors.panel_dim_text),
            );
            return;
        }

        // 评估分数（仅在有数据时显示）
        let has_eval = info.score_cp.is_some() || info.score_mate.is_some();
        if has_eval {
            let eval_text = match (info.score_cp, info.score_mate) {
                (_, Some(mate)) => {
                    if mate > 0 {
                        format!("#{mate}")
                    } else {
                        format!("#{}", -mate)
                    }
                }
                (Some(cp), _) => {
                    if cp > 0 {
                        format!("+{:.2}", cp as f64 / 100.0)
                    } else {
                        format!("{:.2}", cp as f64 / 100.0)
                    }
                }
                _ => unreachable!(),
            };
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(&eval_text).size(24.0).strong());
            });
        }

        // 最佳走法
        if let Some(ref bm) = info.best_move {
            ui.label(format!("Best: {bm}"));
        }

        // 深度 + 节点信息
        let mut meta_parts: Vec<String> = Vec::new();
        if let Some(d) = info.depth {
            meta_parts.push(format!("Depth {d}"));
        }
        if let Some(nodes) = info.nodes {
            if let Some(nps) = info.nps {
                meta_parts.push(format!("{nodes} nodes ({nps} nps)"));
            } else {
                meta_parts.push(format!("{nodes} nodes"));
            }
        }
        if !meta_parts.is_empty() {
            ui.label(
                egui::RichText::new(meta_parts.join(" · "))
                    .size(13.0)
                    .color(colors.panel_dim_text),
            );
        }

        // 主变
        if !info.pv.is_empty() {
            ui.separator();
            ui.label(
                egui::RichText::new("PV:")
                    .size(13.0)
                    .color(colors.panel_dim_text),
            );
            ui.label(
                egui::RichText::new(info.pv.join(" "))
                    .size(13.0)
                    .family(egui::FontFamily::Monospace),
            );
        }

        // 没有任何分析数据时给出提示
        if !has_eval && info.best_move.is_none() && meta_parts.is_empty() && info.pv.is_empty() {
            ui.label(
                egui::RichText::new("No analysis data available")
                    .size(13.0)
                    .color(colors.panel_dim_text),
            );
        }
    }
}
