mod bridge;
mod event_engine;
mod live2d;
mod pet_chat;
mod rea;

use colored::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use bridge::BlueprintBridge;
use event_engine::{
    execute_atomic_task, start_inotify_watcher, EventTrigger, GoalDAG, GoalDecomposer,
    TaskStatus,
};
use live2d::{MascotLive2dPayload, MascotMotion};

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
}

fn sync_live2d_state(state_file: &Path, payload: &MascotLive2dPayload) {
    if let Ok(json_str) = serde_json::to_string_pretty(payload) {
        let _ = fs::write(state_file, json_str);
    }

    let motion_str = format!("[{:?}]", payload.motion).magenta().bold();
    println!(
        "{} 【具身看板娘 · {}】: 「{}」 (姿态: {} | 进度: {}%)",
        "🎀".magenta(),
        payload.mascot_name.cyan().bold(),
        payload.bubble_text.white(),
        motion_str,
        payload.progress.to_string().yellow()
    );
    let _ = io::stdout().flush();
}

#[tokio::main]
async fn main() {
    println!("{}", "============================================================".bright_blue());
    println!("{}", "🦀 星婉 · 纯原生 Rust 事件驱动工作站 (Event-Driven Core)".bright_green().bold());
    println!("{}", "   [吸收 tinyflows 拓扑 + watchexec 原生 inotify + rig 强类型拆解]".bright_cyan());
    println!("{}", "============================================================".bright_blue());

    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = base_dir.join("config.json");
    let queue_path = base_dir.join("tasks/queue.json");
    let live2d_state_path = base_dir.join("tasks/runtime_state.json");
    let _memory_path = base_dir.join("tasks/memory.json");
    let blueprint_path = PathBuf::from("/home/daoxuan/GoogleDrive/ScreenBlueprint");

    let config_data = fs::read_to_string(&config_path).unwrap_or_else(|_| "{}".to_string());
    let config: WorkstationConfig = serde_json::from_str(&config_data).unwrap_or_else(|_| WorkstationConfig {
        project: "AgentMascotWorkstation".into(),
        version: "1.0.0".into(),
        mascot: MascotConfig {
            name: "星婉".into(),
            persona: "制作人看板娘".into(),
            theme: "深蓝龙女".into(),
            voice_mood: "专注".into(),
        },
    });

    // 1. 跨工程蓝图连通检查
    if let Some(bp_state) = BlueprintBridge::read_blueprint_state(&blueprint_path) {
        println!("🔗 [ScreenBlueprint 跨工程桥接]: {:?}", bp_state.lines().next().unwrap_or(""));
    }

    // 2. 初始化事件总线信道 (Event Channel)
    let (tx, mut rx) = mpsc::channel::<EventTrigger>(100);

    // 3. 启动底层 Linux inotify 文件监视器 (watchexec 模式)
    let watch_target = if blueprint_path.exists() {
        blueprint_path.clone()
    } else {
        base_dir.clone()
    };

    let _watcher = match start_inotify_watcher(watch_target.clone(), tx.clone()) {
        Ok(w) => {
            println!("👁️  [inotify 触发器挂载成功]: 正在零开销监听 -> {}", watch_target.display().to_string().bright_green());
            Some(w)
        }
        Err(e) => {
            println!("⚠️  [inotify 挂载警告]: {}", e);
            None
        }
    };

    // 4. 解析用户入参或预置目标
    let args: Vec<String> = env::args().collect();
    let initial_goal = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "《被迫害妄想症》第一集短剧成片与分镜全流程调度".to_string()
    };

    println!("\n📥 [初始化事件注入]: 目标 -> \"{}\"", initial_goal.bright_yellow().bold());
    let _ = tx.send(EventTrigger::UserGoal(initial_goal)).await;

    // 5. 当前活跃的任务图 (内存 DAG)
    let current_dag: Arc<Mutex<Option<GoalDAG>>> = Arc::new(Mutex::new(None));

    // 6. 核心事件循环 (Event-Driven Loop)
    while let Some(event) = rx.recv().await {
        match event {
            EventTrigger::UserGoal(goal_text) => {
                println!("\n🎯 {}", "------------------------------------------------------------".bright_black());
                println!("⚡ [事件到达: 用户目标] 开始用 rig 强类型规约拆解: \"{}\"", goal_text.cyan().bold());

                let dag = GoalDecomposer::decompose(&goal_text);
                println!("📊 [tinyflows 任务拓扑图构建成功] 总计 {} 个原子节点:", dag.tasks.len().to_string().yellow());
                for (id, task) in &dag.tasks {
                    println!("   ├─ [{}] {} (依赖: {:?})", id.bright_white(), task.title.bright_black(), task.depends_on);
                }

                let payload = MascotLive2dPayload::new(
                    &config.mascot.name,
                    MascotMotion::TypingBlazing,
                    &format!("收到新目标【{}】，拓扑图已拆解，正在触发第一批原子任务！", dag.goal_title),
                    Some(dag.goal_id.clone()),
                    0,
                    false,
                );
                sync_live2d_state(&live2d_state_path, &payload);

                // 寻找所有无依赖就绪任务并并发执行 (Fan-out)
                let ready_tasks = dag.get_ready_tasks();
                *current_dag.lock().await = Some(dag);

                for mut task in ready_tasks {
                    task.status = TaskStatus::Running;
                    let tx_clone = tx.clone();
                    let t_id = task.id.clone();
                    let t_title = task.title.clone();

                    println!("🚀 [并发调度启动] -> [{}] {}", t_id.yellow(), t_title);
                    tokio::spawn(async move {
                        let (success, snippet) = execute_atomic_task(&task).await;
                        let _ = tx_clone.send(EventTrigger::TaskFinished {
                            task_id: t_id,
                            success,
                            output_snippet: snippet,
                        }).await;
                    });
                }
            }

            EventTrigger::TaskFinished { task_id, success, output_snippet } => {
                let mut dag_opt = current_dag.lock().await;
                if let Some(ref mut dag) = *dag_opt {
                    if let Some(task) = dag.tasks.get_mut(&task_id) {
                        task.status = if success {
                            TaskStatus::Completed
                        } else {
                            TaskStatus::Failed(output_snippet.clone())
                        };
                        task.output = Some(output_snippet.clone());

                        let icon = if success { "✅".green() } else { "❌".red() };
                        println!(
                            "{} [任务执行完成] [{}] {} -> 输出: \"{}\"",
                            icon,
                            task_id.bright_yellow(),
                            task.title.bold(),
                            output_snippet.bright_black()
                        );
                    }

                    let progress = dag.get_progress_percentage();

                    // 拓扑推进：寻找新产生的就绪任务 (Fan-in 汇聚或下游接力)
                    let ready_tasks = dag.get_ready_tasks();
                    if !ready_tasks.is_empty() {
                        for mut next_task in ready_tasks {
                            next_task.status = TaskStatus::Running;
                            let tx_clone = tx.clone();
                            let t_id = next_task.id.clone();
                            let t_title = next_task.title.clone();

                            println!("⚡ [拓扑触发下游] -> [{}] {}", t_id.yellow(), t_title);
                            tokio::spawn(async move {
                                let (s, out) = execute_atomic_task(&next_task).await;
                                let _ = tx_clone.send(EventTrigger::TaskFinished {
                                    task_id: t_id,
                                    success: s,
                                    output_snippet: out,
                                }).await;
                            });
                        }
                    }

                    if dag.is_all_completed() {
                        println!("\n{}", "🎉 ========================================================".bright_green());
                        println!("🏁 [全拓扑图交付完成] 目标【{}】所有原子工种执行完毕！", dag.goal_title.bright_white().bold());
                        println!("{}", "============================================================".bright_green());

                        // 将真实的拓扑结果同步给 queue.json，结束造假历史
                        let serialized_tasks: Vec<_> = dag.tasks.values().cloned().collect();
                        let _ = fs::write(&queue_path, serde_json::to_string_pretty(&serialized_tasks).unwrap());

                        let finish_payload = MascotLive2dPayload::new(
                            &config.mascot.name,
                            MascotMotion::ExcitedCheering,
                            &format!("主人！全流程真实任务【{}】已全部打通交付！", dag.goal_title),
                            Some(dag.goal_id.clone()),
                            100,
                            false,
                        );
                        sync_live2d_state(&live2d_state_path, &finish_payload);

                        // 如果非守护模式，直接优雅退出
                        if !args.contains(&"--daemon".to_string()) {
                            println!("\n{}", "☕ 一次性批次执行完毕，优雅收官。如需常驻监听文件变动，可附加 --daemon 参数。".bright_blue());
                            break;
                        }
                    } else {
                        let payload = MascotLive2dPayload::new(
                            &config.mascot.name,
                            MascotMotion::InspectMagnifier,
                            &format!("正在推进节点 [{}]，当前全流程进度 {}%...", task_id, progress),
                            Some(dag.goal_id.clone()),
                            progress,
                            false,
                        );
                        sync_live2d_state(&live2d_state_path, &payload);
                    }
                }
            }

            EventTrigger::FileModified { path, ext } => {
                let filename = Path::new(&path).file_name().unwrap_or_default().to_string_lossy();
                println!("📂 [inotify 捕获文件事件] {} (扩展名: {})", filename.bright_magenta(), ext.yellow());

                let payload = MascotLive2dPayload::new(
                    &config.mascot.name,
                    MascotMotion::InspectMagnifier,
                    &format!("察觉到文件修改 [{}]，已就绪待命！", filename),
                    None,
                    0,
                    false,
                );
                sync_live2d_state(&live2d_state_path, &payload);
            }
        }
    }
}
