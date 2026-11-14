use agent_mascot_workstation::protocol::{BeaconListener, MascotStateBeacon, TaskStatus};
use colored::*;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

fn render_mascot_board(beacon: &MascotStateBeacon) {
    let status_badge = match beacon.task.status {
        TaskStatus::Idle => " 💤 待命就绪 ".on_bright_black().white(),
        TaskStatus::Planning => " 🧠 目标拆解中 ".on_cyan().black().bold(),
        TaskStatus::Running => " ⚡ 任务执行中 ".on_yellow().black().bold(),
        TaskStatus::Success => " 🎉 交付成功 ".on_green().black().bold(),
        TaskStatus::Failed => " ❌ 异常告警 ".on_red().white().bold(),
    };

    let posture_badge = format!("[{}]", beacon.mascot.posture).magenta().bold();
    let progress_bar = make_progress_bar(beacon.task.progress);

    println!("\n{}", "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓".bright_purple());
    println!(
        "┃ 🐉 【星婉 · 具身看板娘】 {}  来源: {:<18} ┃",
        status_badge,
        beacon.source.bright_white()
    );
    println!("┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫");
    println!("┃ 💬 台词: 「{}」", beacon.mascot.dialogue.bright_white().bold());
    println!("┃ 🎭 姿态: {:<20} 情绪: {:<18} ┃", posture_badge, beacon.mascot.mood.cyan());
    println!("┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫");
    println!("┃ 🎯 任务: {:<52} ┃", beacon.task.title.bright_yellow());
    println!("┃ 进度: {} {:.1}%", progress_bar, beacon.task.progress);
    println!("┃ 环节: [{}/{}] {:<40} ┃", 
        beacon.task.step_index, 
        beacon.task.total_steps, 
        beacon.task.step_name.bright_cyan()
    );
    println!("┃ 详情: {:<52} ┃", beacon.task.detail.bright_black());

    if let Some(ref err) = beacon.task.error_msg {
        println!("┃ ⚠️ 错误: {:<52} ┃", err.bright_red());
    }

    println!("┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛");
    let _ = io::stdout().flush();
}

fn make_progress_bar(pct: f32) -> String {
    let total_blocks = 24;
    let filled = ((pct / 100.0) * total_blocks as f32).round() as usize;
    let filled = filled.min(total_blocks);
    let empty = total_blocks - filled;

    let bar_filled = "█".repeat(filled).bright_green();
    let bar_empty = "░".repeat(empty).bright_black();
    format!("[{}{}]", bar_filled, bar_empty)
}

fn sync_to_live2d_web(beacon: &MascotStateBeacon) {
    // 镜像同步到本地 tasks/runtime_state.json 供 HTML/Live2D 读取
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let web_state_path = base_dir.join("tasks/runtime_state.json");
    if let Ok(json_data) = serde_json::to_string_pretty(beacon) {
        let _ = fs::write(web_state_path, json_data);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "============================================================".bright_purple());
    println!("{}", "🔥 [Hot Path] 星婉 · 纯原生 Rust 看板娘中枢 (The Observer)".bright_magenta().bold());
    println!("{}", "   [0% CPU 占用 · Linux Inotify 零损耗监听 · 彻底与业务解耦]".bright_black());
    println!("{}", "============================================================".bright_purple());

    let args: Vec<String> = env::args().collect();
    let listener = BeaconListener::new();

    // 1. 启动即读取当前的最后持久化状态
    match listener.read_current()? {
        Some(beacon) => {
            render_mascot_board(&beacon);
            sync_to_live2d_web(&beacon);
        }
        None => {
            let default_idle = MascotStateBeacon::idle("agent-mascot", "主人好！星婉已就位，正在静候业务信号！");
            render_mascot_board(&default_idle);
            sync_to_live2d_web(&default_idle);
        }
    }

    if args.contains(&"--once".to_string()) {
        println!("{}", "✨ [--once] 单次状态读取完成，优雅退出。".bright_blue());
        return Ok(());
    }

    println!("\n👁️  [Inotify 内核监听已激活] 正在待命，等待冷端 Worker 发射状态信号...");
    println!("   (按 Ctrl+C 退出)\n");

    // 2. 阻塞式事件驱动监听（0% CPU，事件微秒级到达）
    listener.listen(|beacon| {
        render_mascot_board(&beacon);
        sync_to_live2d_web(&beacon);
    })?;

    Ok(())
}
