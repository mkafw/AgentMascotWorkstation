mod live2d;
mod rea;
mod router;

use chrono::Utc;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use live2d::{MascotLive2dPayload, MascotMotion};
use rea::ReaEngine;
use router::JevMuDecisionRouter;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MascotConfig {
    name: String,
    persona: String,
    theme: String,
    voice_mood: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WorkstationConfig {
    project: String,
    version: String,
    mascot: MascotConfig,
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

fn sync_live2d_state(
    state_file: &Path,
    payload: &MascotLive2dPayload,
) {
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

fn main() {
    println!("{}", "============================================================".bright_blue());
    println!("{}", "🦀 AgentMascotWorkstation 终极全域融合中枢启动".bright_green().bold());
    println!("{}", "   [Live2D具身桌宠 × AutoLoop自治循环 × REA逆向 × Rust决策分流]".bright_cyan());
    println!("{}", "============================================================".bright_blue());

    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config_path = base_dir.join("config.json");
    let queue_path = base_dir.join("tasks/queue.json");
    let live2d_state_path = base_dir.join("tasks/runtime_state.json");
    let specs_dir = base_dir.join("specs");
    let _ = fs::create_dir_all(&specs_dir);

    let config_data = fs::read_to_string(&config_path).expect("读取 config.json 失败");
    let config: WorkstationConfig = serde_json::from_str(&config_data).expect("解析 config.json 失败");

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
                let payload = MascotLive2dPayload::new(
                    &config.mascot.name,
                    MascotMotion::IdleBreathing,
                    "主人，队列全部消化完毕啦！我先稍微趴一会儿，随时叫醒我哦~",
                    None,
                    0,
                    false,
                );
                sync_live2d_state(&live2d_state_path, &payload);
                println!("\n{}", "☕ [Autoloop Core] 零成本守护待机，等待新 Top 目标。".bright_black());
                break;
            }
            Some(idx) => {
                let task_id = queue[idx].id.clone();
                let title = queue[idx].title.clone();
                let assigned_skill = queue[idx].assigned_skill.clone();
                let target_path = queue[idx].meta.target_binary_path.clone();
                queue[idx].status = "in_progress".to_string();

                println!(
                    "\n🎯 [LoopX 语义锁定] Top 目标: [{}] {}",
                    task_id.bright_yellow(),
                    title.bright_white().bold()
                );

                // 1. 若有二进制/工程靶体，调用 REA 原生引擎提取证据账本
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
                    
                    // 2. 通过 Jev / Mu 极速判定当前阶段分流动作
                    let action = JevMuDecisionRouter::route_task_stage(stage, 0.88);

                    let (motion, bubble) = match s_idx {
                        0 => (MascotMotion::InspectMagnifier, "REA 逆向雷达穿透中，正在扫描架构与设定骨架~"),
                        1 => (MascotMotion::TypingBlazing, "重构黄金三章，前三秒留存钩子直接锁死！"),
                        2 => (MascotMotion::TypingBlazing, "注入八触五感与自心心法人物阵营设定~"),
                        3 => (MascotMotion::TypingBlazing, "爆更正文中！调动七情六欲与白描赋比兴手法！"),
                        4 => (MascotMotion::InspectMagnifier, "去AI味审校中！逐句清理模板词与机械抒情！"),
                        _ => (MascotMotion::ExcitedCheering, "分镜与剪映工程一键成片导出！"),
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

    println!("\n{}", "🏁 [AgentMascotWorkstation] 批次闭环完成，全域融合成功！".bright_green().bold());
}
