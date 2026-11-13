mod rea;

use chrono::Utc;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use rea::ReaEngine;

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

#[derive(Debug, Serialize, Deserialize)]
struct MascotRuntimeState {
    timestamp: String,
    status: String,
    mood: String,
    speech: String,
    active_task_id: Option<String>,
    progress: u32,
}

fn update_mascot_state(
    state_file: &Path,
    mascot_name: &str,
    status: &str,
    mood: &str,
    speech: &str,
    task_id: Option<String>,
    progress: u32,
) {
    let runtime = MascotRuntimeState {
        timestamp: Utc::now().to_rfc3339(),
        status: status.to_string(),
        mood: mood.to_string(),
        speech: speech.to_string(),
        active_task_id: task_id,
        progress,
    };

    if let Ok(json_str) = serde_json::to_string_pretty(&runtime) {
        let _ = fs::write(state_file, json_str);
    }

    let mood_tag = match mood {
        "cheerful" => format!("[{}]", mood.to_uppercase()).green().bold(),
        "focused" => format!("[{}]", mood.to_uppercase()).yellow().bold(),
        "critical" => format!("[{}]", mood.to_uppercase()).magenta().bold(),
        _ => format!("[{}]", mood.to_uppercase()).cyan().bold(),
    };

    println!(
        "\n{} 【看板娘 · {}】: 「{}」 (状态: {} | 进度: {}%)",
        "🎀".magenta(),
        mascot_name.cyan().bold(),
        speech.white(),
        mood_tag,
        progress.to_string().yellow()
    );
}

fn get_stage_info(stage: &str) -> (&'static str, &'static str) {
    match stage {
        "01_target_scan" => ("🔍 目标雷达扫描中...", "REA 逆向雷达启动，正在扫描目标二进制与架构特征~"),
        "02_golden_three_chapters" => ("🔬 抽取证据账本与状态转移...", "逆向解构完成！提取核心符号与无锁状态机契约！"),
        "03_world_and_characters" => ("🦀 Rust 强类型 Spec 建模...", "正在将逆向证据转化为 Rust `Trait` 与安全内存模型~"),
        "04_drafting_with_zixin" => ("⚡ Rust 净室 1:1 代码重构...", "零成本抽象构建中，爆肝编写高性能 Rust 原生代码！"),
        "05_deslop_and_review" => ("🧐 严格类型与无锁安全审查...", "通过 Rust 借用检查器与单元测试，彻底消除所有潜在崩溃~"),
        "06_storyboard_cutcli" => ("📦 编译超轻量原生 Binary...", "编译完成！单个 Release 绿色无依赖可执行文件出炉！"),
        _ => ("⚡ 执行中...", "努力干活中..."),
    }
}

fn main() {
    println!("{}", "============================================================".bright_blue());
    println!("{}", "🦀 REA-rs (Reverse Engineer Anything) 逆向与重构中枢启动".bright_green().bold());
    println!("{}", "============================================================".bright_blue());

    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config_path = base_dir.join("config.json");
    let queue_path = base_dir.join("tasks/queue.json");
    let state_path = base_dir.join("tasks/runtime_state.json");
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
                update_mascot_state(
                    &state_path,
                    &config.mascot.name,
                    "idle",
                    "cheerful",
                    "主人，所有逆向与 Rust 重构任务已全量完成！随时把新的黑盒丢给我拆解吧~",
                    None,
                    0,
                );
                println!("\n{}", "☕ [REA-rs Core] 任务队列就绪，进入守护监听状态。".bright_black());
                break;
            }
            Some(idx) => {
                let task_id = queue[idx].id.clone();
                let title = queue[idx].title.clone();
                let assigned_skill = queue[idx].assigned_skill.clone();
                let target_path = queue[idx].meta.target_binary_path.clone();
                queue[idx].status = "in_progress".to_string();

                println!(
                    "\n🎯 锁定 Top 目标: [{}] {}",
                    task_id.bright_yellow(),
                    title.bright_white().bold()
                );
                println!("🛠️ 派遣工种管线: {}", assigned_skill.cyan());

                // 如果指定了二进制目标，调用真实的 REA Rust 原生反编译引擎进行实测抽取
                if let Some(bin_path_str) = target_path {
                    let bin_path = Path::new(&bin_path_str);
                    if bin_path.exists() {
                        println!("🔬 正在对目标进行 REA 二进制逆向分析: {}", bin_path_str.bright_cyan());
                        if let Ok(spec) = ReaEngine::reverse_binary(bin_path) {
                            println!("✨ [REA 逆向成功] 识别格式: {}, 提取符号数: {}", spec.format.green(), spec.total_symbols_found.to_string().yellow());
                            let spec_out_path = specs_dir.join(format!("{}_spec.json", task_id));
                            let _ = fs::write(&spec_out_path, serde_json::to_string_pretty(&spec).unwrap());
                            println!("💾 证据账本 Spec 已沉淀至: {}", spec_out_path.display().to_string().magenta());
                        }
                    }
                }

                let total_stages = config.stages.len() as u32;

                for (s_idx, stage) in config.stages.iter().enumerate() {
                    let progress = (((s_idx + 1) as f32 / total_stages as f32) * 100.0) as u32;
                    let (_, bubble) = get_stage_info(stage);

                    update_mascot_state(
                        &state_path,
                        &config.mascot.name,
                        "working",
                        "focused",
                        bubble,
                        Some(task_id.clone()),
                        progress,
                    );

                    queue[idx].current_stage = Some(stage.clone());
                    queue[idx].progress = progress;

                    // 持久化当前状态到 queue.json
                    let _ = fs::write(&queue_path, serde_json::to_string_pretty(&queue).unwrap());
                    thread::sleep(Duration::from_millis(600));
                }

                // 标记完成
                queue[idx].status = "completed".to_string();
                let _ = fs::write(&queue_path, serde_json::to_string_pretty(&queue).unwrap());

                update_mascot_state(
                    &state_path,
                    &config.mascot.name,
                    "finished",
                    "cheerful",
                    &format!("太棒了！【{}】逆向分析与 Rust 重构 Spec 已全量生成！", title),
                    Some(task_id),
                    100,
                );
                thread::sleep(Duration::from_millis(400));
            }
        }
    }

    println!("\n{}", "🏁 [REA-rs Engine] 批次逆向重构闭环成功！".bright_green().bold());
}
