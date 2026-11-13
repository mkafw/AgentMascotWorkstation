use serde::{Deserialize, Serialize};

/// 具身看板娘 Live2D 情绪与动作姿态
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MascotMotion {
    IdleBreathing,       // 摸鱼待机 (呼吸微动)
    TypingBlazing,       // 双手敲键盘冒火 (码字中)
    InspectMagnifier,    // 推眼镜拿放大镜 (审查去AI味中)
    ExcitedCheering,     // 振臂欢呼 (交付完成)
    ConfusedThinking,    // 歪头疑问 (遇到卡点等待指示)
}

/// 具身桌宠通信结构 (可直接由 Live2D / 桌宠前端 WebSocket 接收)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MascotLive2dPayload {
    pub mascot_name: String,
    pub motion: MascotMotion,
    pub bubble_text: String,
    pub current_task: Option<String>,
    pub progress: u32,
    pub needs_human_card: bool,
}

impl MascotLive2dPayload {
    pub fn new(name: &str, motion: MascotMotion, text: &str, task: Option<String>, progress: u32, alert: bool) -> Self {
        Self {
            mascot_name: name.to_string(),
            motion,
            bubble_text: text.to_string(),
            current_task: task,
            progress,
            needs_human_card: alert,
        }
    }
}
