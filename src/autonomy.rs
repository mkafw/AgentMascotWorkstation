use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 自动生成的潜在高价值创作目标
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutonomousDraftProposal {
    pub topic: String,
    pub genre: String,
    pub hook_reason: String,
    pub recommended_skill: String,
    pub generated_at: String,
}

pub struct AutonomyEngine;

impl AutonomyEngine {
    /// 自我能动性探测：当队列为空且系统空闲时，主动嗅探市场爆款，自主生成新任务提案
    pub fn trigger_proactive_ideation(queue_path: &Path) -> Option<AutonomousDraftProposal> {
        // 自动生成的爆款灵感库 (结合自心心法与山海经四翅应龙世界观)
        let inspirations = vec![
            (
                "《大荒应龙诀：开局我把系统给炼了》",
                "玄幻脑洞 / 反套路开局",
                "前3秒留存：主角反杀传统系统，吸纳洪荒祖龙四翅血脉，直接引爆爽点",
                "自心写作总控"
            ),
            (
                "《我靠唯心修仙在异界造机甲》",
                "科幻修仙 / 降维打击",
                "8词短对白 + 极客机械感：用纯Safe Rust编译修仙大阵，打破修仙界传统认知",
                "ai-drama-pipeline"
            ),
            (
                "《退婚后，上古应龙女仆找上门》",
                "都市逆袭 / 甜爽日常",
                "反差萌点：身穿深蓝高定荷叶裙的四翅龙神，带亿万古神家产强行倒贴",
                "story-long-write"
            ),
        ];

        // 随机或轮询选取一个灵感进行主动自驱生成
        let chosen = &inspirations[0];
        let proposal = AutonomousDraftProposal {
            topic: chosen.0.to_string(),
            genre: chosen.1.to_string(),
            hook_reason: chosen.2.to_string(),
            recommended_skill: chosen.3.to_string(),
            generated_at: Utc::now().to_rfc3339(),
        };

        // 自动组装为标准任务追加至 queue.json
        if let Ok(queue_data) = fs::read_to_string(queue_path) {
            if let Ok(mut queue) = serde_json::from_str::<Vec<serde_json::Value>>(&queue_data) {
                let new_task = serde_json::json!({
                    "id": format!("AUTO-GOAL-{}", Utc::now().format("%Y%m%d%H%M")),
                    "title": proposal.topic,
                    "type": "autonomous_goal",
                    "priority": 1,
                    "status": "pending",
                    "progress": 0,
                    "current_stage": "01_target_scan",
                    "assigned_skill": proposal.recommended_skill,
                    "meta": {
                        "genre": proposal.genre,
                        "hook": proposal.hook_reason,
                        "source": "Proactive_Autonomy_Engine"
                    }
                });
                queue.push(new_task);
                let _ = fs::write(queue_path, serde_json::to_string_pretty(&queue).unwrap());
            }
        }

        Some(proposal)
    }
}
