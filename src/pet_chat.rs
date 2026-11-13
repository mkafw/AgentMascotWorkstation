use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 1. 对应 xiyan1314/daifeiyu-desktop-pet 的 memory.json 持久化对话记忆
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: String,
}

/// 2. 对应 xiyan1314/daifeiyu-desktop-pet 的 ai_emote 表情联动机
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AiEmote {
    Normal,     // 平和眨眼
    Thinking,   // 思考中
    Happy,      // 欢呼雀跃
    Angry,      // 傲娇生气
    FocusTyping,// 敲键盘
}

/// 3. 四合一对话调度核心 (Chat Engine)
pub struct PetChatEngine {
    pub history_file: String,
    pub max_rounds: usize,
    pub memory: Arc<Mutex<Vec<ChatMessage>>>,
}

impl PetChatEngine {
    pub fn new(history_path: &str, max_rounds: usize) -> Self {
        let history = if Path::new(history_path).exists() {
            let data = fs::read_to_string(history_path).unwrap_or_else(|_| "[]".to_string());
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            history_file: history_path.to_string(),
            max_rounds,
            memory: Arc::new(Mutex::new(history)),
        }
    }

    /// 核心交互：输入自然语言，驱动创作工作流，并返回 (回复内容, 触发的表情, 是否派发任务)
    pub async fn process_user_input(&self, input: &str) -> (String, AiEmote, Option<String>) {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return ("主人有什么吩咐嘛？星婉在听哦~".into(), AiEmote::Normal, None);
        }

        // 保存用户发言到记忆库
        let mut mem = self.memory.lock().await;
        mem.push(ChatMessage {
            role: "user".into(),
            content: trimmed.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        // 智能意图路由 (Jev/Mu + daifeiyu 指令分流)
        let (reply, emote, task_type) = if trimmed.contains("去AI味") || trimmed.contains("审校") {
            (
                "遵命！已激活 story-deslop 深度去AI味工种，正在严格剔除一切模板套话！".into(),
                AiEmote::FocusTyping,
                Some("task_deslop".into()),
            )
        } else if trimmed.contains("写") || trimmed.contains("续写") || trimmed.contains("开书") {
            (
                "四翅全开！自心心法五感八触已注入，正在为您爆更黄金剧情！".into(),
                AiEmote::Happy,
                Some("task_zixin_write".into()),
            )
        } else if trimmed.contains("分镜") || trimmed.contains("短剧") {
            (
                "短剧工业管线就绪！正在根据 8 词对白标准生成分镜与剪映工程！".into(),
                AiEmote::FocusTyping,
                Some("task_short_drama".into()),
            )
        } else if trimmed.contains("你是谁") || trimmed.contains("名字") {
            (
                "我是星婉呀！您的专属上古四翅应龙娘，也是这套创作系统的总导演~ 💖".into(),
                AiEmote::Happy,
                None,
            )
        } else {
            (
                format!("法旨收到：「{}」！星婉已派发至后台 Rust 任务队列！", trimmed),
                AiEmote::Thinking,
                Some("task_custom".into()),
            )
        };

        // 保存助手回答到记忆库
        mem.push(ChatMessage {
            role: "assistant".into(),
            content: reply.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        // 保持记忆在 max_rounds 轮数以内 (来自原版 xiyan1314 设计)
        if mem.len() > self.max_rounds * 2 {
            let start = mem.len() - (self.max_rounds * 2);
            *mem = mem[start..].to_vec();
        }

        // 原子落盘
        let _ = fs::write(&self.history_file, serde_json::to_string_pretty(&*mem).unwrap());

        (reply, emote, task_type)
    }
}
