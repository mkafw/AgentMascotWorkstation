use serde::{Deserialize, Serialize};

/// 决策动作：由轻量决策引擎决定当前任务交给哪个工种管线
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PipelineAction {
    ScanAndAnalyze,      // 扫榜与逆向拆解
    ReconstructGolden,   // 黄金三章重构
    DraftingWithZixin,   // 自心心法全开爆更 (五感八触+七情六欲+白描赋比兴)
    DeslopReview,        // 严苛审校与去AI味
    StoryboardCutcli,    // 生成分镜与剪映草稿
    HumanInTheLoopAlert, // 严重剧情分歧/敏感卡点，挂起等待人类卡片审批
}

pub struct JevMuDecisionRouter;

impl JevMuDecisionRouter {
    /// 模拟 jev-rs / mu 的单次 Prefill 极速轻量路由 (无需大模型慢速思考)
    pub fn route_task_stage(current_stage: &str, deslop_score: f32) -> PipelineAction {
        match current_stage {
            "01_target_scan" => PipelineAction::ScanAndAnalyze,
            "02_golden_three_chapters" => PipelineAction::ReconstructGolden,
            "03_world_and_characters" => PipelineAction::DraftingWithZixin,
            "04_drafting_with_zixin" => {
                if deslop_score < 0.85 {
                    // AI 味超标，自动路由给去 AI 味审查
                    PipelineAction::DeslopReview
                } else {
                    PipelineAction::StoryboardCutcli
                }
            }
            "05_deslop_and_review" => PipelineAction::StoryboardCutcli,
            _ => PipelineAction::DraftingWithZixin,
        }
    }
}
