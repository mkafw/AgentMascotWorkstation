mod autonomy;
mod bridge;
mod live2d;
mod pet_chat;
mod rea;
mod router;

use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use autonomy::AutonomyEngine;
use bridge::BlueprintBridge;
use live2d::{MascotLive2dPayload, MascotMotion};
use pet_chat::{AiEmote, PetChatEngine};
use rea::ReaEngine;
use router::{JevMuDecisionRouter, PipelineAction};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MascotConfig {
    name: String,
    persona: String,
    theme: String,
    voice_mood: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AutonomyConfig {
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default = "default_max_proactive")]
    max_proactive_tasks: usize,
}

fn default_true() -> bool {
    true
}

fn default_max_proactive() -> usize {
    2
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WorkstationConfig {
    project: String,
    version: String,
    mascot: MascotConfig,
    #[serde(default)]
    autonomy: Option<AutonomyConfig>,
    stages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TaskMeta {
    #[serde(default)]
    genre: String,
    #[serde(default)]
    hook: String,
    #[serde(default)]
    target_binary_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TaskItem {
    id: String,
    title: String,
    #[serde(rename = "type")]
    task_type: String,
    priority: u32,
    status: String,
    progress: u32,
    current_stage: Option<String>,
    assigned_skill: String,
    meta: TaskMeta,
}

fn sync_live2d_state(state_file: &Path, payload: &MascotLive2dPayload) {
    if let Ok(json_str) = serde_json::to_string_pretty(payload) {
        let _ = fs::write(state_file, json_str);
    }

    let motion_str = format!("[{:?}]", payload.motion).magenta().bold();
    println!(
        "\n{} 【具身看板娘 · {}】: 「{}」 (姿态: {} | 进度: {}%)",
        "🎀".magenta(),
        payload.mascot_name.cyan().bold(),
        payload.bubble_text.white(),
        motion_str,
        payload.progress.to_string().yellow()
    );
}

#[tokio::main]
async fn main() {
    println!("{}", "============================================================".bright_blue());
    println!("{}", "🦀 星婉 · 四合一桌宠终端与创作中枢 (Rust Native Core) 启动".bright_green().bold());
    println!("{}", "   [1190fasheqi思维链 + xiyan1314记忆交流 + YunYueSama动作 + DSH任务看板]".bright_cyan());
    println!("{}", "============================================================".bright_blue());

    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config_path = base_dir.join("config.json");
    let queue_path = base_dir.join("tasks/queue.json");
    let live2d_state_path = base_dir.join("tasks/runtime_state.json");
    let memory_path = base_dir.join("tasks/memory.json");
    let specs_dir = base_dir.join("specs");
    let _ = fs::create_dir_all(&specs_dir);

    let config_data = fs::read_to_string(&config_path).expect("读取 config.json 失败");
    let config: WorkstationConfig = serde_json::from_str(&config_data).expect("解析 config.json 失败");

    // 跨工程桥接测试
    let blueprint_path = Path::new("/home/daoxuan/GoogleDrive/ScreenBlueprint");
    if let Some(bp_state) = BlueprintBridge::read_blueprint_state(blueprint_path) {
        println!("🔗 [ScreenBlueprint 跨工程桥接连通] 提取项目蓝图: {:?}", bp_state.lines().next().unwrap_or(""));
    }

    // 1. 初始化对话与长短时记忆中枢 (xiyan1314 架构实现)
    let chat_engine = Arc::new(PetChatEngine::new(&memory_path.to_string_lossy(), 10));

    // 2. 模拟初次交互欢迎并更新思维链
    let (welcome_msg, welcome_emote, _) = chat_engine.process_user_input("你好星婉").await;
    let initial_motion = match welcome_emote {
        AiEmote::Happy => MascotMotion::ExcitedCheering,
        _ => MascotMotion::IdleBreathing,
    };
    let initial_payload = MascotLive2dPayload::new(
        &config.mascot.name,
        initial_motion,
        &welcome_msg,
        None,
        0,
        false,
    );
    sync_live2d_state(&live2d_state_path, &initial_payload);

    let max_proactive = config
        .autonomy
        .as_ref()
        .map(|a| a.max_proactive_tasks)
        .unwrap_or(1);
    let mut proactive_count = 0;

    // 3. 驱动任务看板与 AutoLoop 循环 (DSH + 1190fasheqi + YunYueSama 动作闭环)
    loop {
        let queue_data = fs::read_to_string(&queue_path).unwrap_or_else(|_| "[]".to_string());
        let mut queue: Vec<TaskItem> = serde_json::from_str(&queue_data).unwrap_or_default();

        let next_idx = queue
            .iter()
            .enumerate()
            .filter(|(_, t)| t.status == "pending" || t.status == "in_progress")
            .min_by_key(|(_, t)| t.priority)
            .map(|(idx, _)| idx);

        match next_idx {
            None => {
                let autonomy_enabled = config.autonomy.as_ref().map(|a| a.enabled).unwrap_or(true);
                if autonomy_enabled && proactive_count < max_proactive {
                    proactive_count += 1;
                    println!(
                        "\n{}",
                        format!(
                            "⚡ [自我能动性触发 #{}/{}] 任务池已空，星婉主动启动灵感雷达！",
                            proactive_count, max_proactive
                        )
                        .yellow()
                        .bold()
                    );
                    if let Some(proposal) = AutonomyEngine::trigger_proactive_ideation(&queue_path) {
                        let proactive_speech = format!(
                            "主人！看您在休息，星婉主动为您推演了一部【{}】（{}），前3秒爆点已就绪，已经帮您排入生产队列啦！快夸我~ ✨",
                            proposal.topic, proposal.genre
                        );
                        let payload = MascotLive2dPayload::new(
                            &config.mascot.name,
                            MascotMotion::ExcitedCheering,
                            &proactive_speech,
                            None,
                            0,
                            false,
                        );
                        sync_live2d_state(&live2d_state_path, &payload);
                        println!("{}", "✨ [Proactive Goal] 新目标已自驱生成，继续无休止挂机闭环！".green().bold());
                        thread::sleep(Duration::from_millis(1500));
                        continue; // 继续下一轮自驱循环！
                    }
                }

                let payload = MascotLive2dPayload::new(
                    &config.mascot.name,
                    MascotMotion::IdleBreathing,
                    "主人，任务池已全量推演完成！四羽已收拢待命，随时输入新指令哦~ (≧ω≦)",
                    None,
                    0,
                    false,
                );
                sync_live2d_state(&live2d_state_path, &payload);
                println!("\n{}", "☕ [Rust Autoloop] 零成本待机，守护监听就绪。".bright_black());
                break;
            }
            Some(idx) => {
                let task_id = queue[idx].id.clone();
                let title = queue[idx].title.clone();
                let assigned_skill = queue[idx].assigned_skill.clone();
                let target_path = queue[idx].meta.target_binary_path.clone();
                queue[idx].status = "in_progress".to_string();

                println!(
                    "\n🎯 [任务看板锁定] Top 目标: [{}] {}",
                    task_id.bright_yellow(),
                    title.bright_white().bold()
                );
                println!("🛠️ 调度工种管线: {}", assigned_skill.cyan());

                // 若有二进制/工程靶体，调用 REA 原生引擎提取证据账本
                if let Some(bin_path_str) = target_path {
                    let bin_path = Path::new(&bin_path_str);
                    if bin_path.exists() {
                        println!("🔬 [REA 逆向靶体接入]: {}", bin_path_str.bright_cyan());
                        if let Ok(spec) = ReaEngine::reverse_binary(bin_path) {
                            println!("✨ [REA 提取成功] 符号数: {}", spec.total_symbols_found.to_string().yellow());
                            let spec_out = specs_dir.join(format!("{}_spec.json", task_id));
                            let _ = fs::write(&spec_out, serde_json::to_string_pretty(&spec).unwrap());
                        }
                    }
                }

                let total_stages = config.stages.len() as u32;

                for (s_idx, stage) in config.stages.iter().enumerate() {
                    let progress = (((s_idx + 1) as f32 / total_stages as f32) * 100.0) as u32;

                    // 决策分流
                    let action = JevMuDecisionRouter::route_task_stage(stage, 0.88);
                    let (motion, bubble) = match action {
                        PipelineAction::ScanAndAnalyze => (
                            MascotMotion::InspectMagnifier,
                            "REA 逆向雷达穿透中，正在扫描架构与设定骨架~",
                        ),
                        PipelineAction::ReconstructGolden => (
                            MascotMotion::TypingBlazing,
                            "重构黄金三章，前三秒留存钩子直接锁死！",
                        ),
                        PipelineAction::DraftingWithZixin => (
                            MascotMotion::TypingBlazing,
                            "爆更正文中！调动七情六欲与白描赋比兴手法！",
                        ),
                        PipelineAction::DeslopReview => (
                            MascotMotion::InspectMagnifier,
                            "去AI味审校中！逐句清理模板词与机械抒情！",
                        ),
                        PipelineAction::StoryboardCutcli => (
                            MascotMotion::ExcitedCheering,
                            "分镜与剪映工程一键成片导出！",
                        ),
                        PipelineAction::HumanInTheLoopAlert => (
                            MascotMotion::ConfusedThinking,
                            "遇到关键分歧点，请主人拍板！",
                        ),
                    };

                    let payload = MascotLive2dPayload::new(
                        &config.mascot.name,
                        motion,
                        bubble,
                        Some(task_id.clone()),
                        progress,
                        false,
                    );
                    sync_live2d_state(&live2d_state_path, &payload);

                    queue[idx].current_stage = Some(stage.clone());
                    queue[idx].progress = progress;
                    let _ = fs::write(&queue_path, serde_json::to_string_pretty(&queue).unwrap());
                    thread::sleep(Duration::from_millis(500));
                }

                queue[idx].status = "completed".to_string();
                let _ = fs::write(&queue_path, serde_json::to_string_pretty(&queue).unwrap());

                let finish_payload = MascotLive2dPayload::new(
                    &config.mascot.name,
                    MascotMotion::ExcitedCheering,
                    &format!("太棒了！目标【{}】工业成片交付，无缝切入下一个！", title),
                    Some(task_id),
                    100,
                    false,
                );
                sync_live2d_state(&live2d_state_path, &finish_payload);
                thread::sleep(Duration::from_millis(400));
            }
        }
    }

    println!("\n{}", "🏁 [AgentMascotWorkstation] 批次闭环完成，四合一全量固化！".bright_green().bold());
}
