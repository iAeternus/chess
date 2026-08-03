//! 引擎信息面板：显示引擎名称、深度、评估、最佳走法。

/// 引擎分析信息（后续由 engine_bridge 填充）
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

    pub fn show(&self, ui: &mut egui::Ui, info: &EngineInfo) {
        ui.heading("Engine");
        ui.separator();

        if let Some(ref name) = info.name {
            ui.label(format!("Engine: {name}"));
        } else {
            ui.label("Engine: --");
        }

        // 评估
        let eval_text = match (info.score_cp, info.score_mate) {
            (_, Some(mate)) => {
                if mate > 0 {
                    format!("Mate in {} (White)", mate)
                } else {
                    format!("Mate in {} (Black)", -mate)
                }
            }
            (Some(cp), None) => {
                if cp > 0 {
                    format!("+{:.2}", cp as f64 / 100.0)
                } else {
                    format!("{:.2}", cp as f64 / 100.0)
                }
            }
            (None, None) => "--".to_string(),
        };
        ui.label(format!("Eval: {eval_text}"));

        // 深度
        if let Some(d) = info.depth {
            ui.label(format!("Depth: {d}"));
        } else {
            ui.label("Depth: --");
        }

        // 最佳走法
        if let Some(ref bm) = info.best_move {
            ui.label(format!("Best: {bm}"));
        } else {
            ui.label("Best: --");
        }

        // 搜索信息
        if let Some(nodes) = info.nodes {
            if let Some(nps) = info.nps {
                ui.label(format!("{nodes} nodes ({nps} nps)"));
            } else {
                ui.label(format!("{nodes} nodes"));
            }
        }

        // 主变
        if !info.pv.is_empty() {
            ui.separator();
            ui.label("PV:");
            ui.label(info.pv.join(" "));
        }
    }
}
