use agent_mascot_workstation::event_engine::{
    execute_atomic_task, GoalDAG, GoalDecomposer, TaskStatus as EngineTaskStatus,
};
use agent_mascot_workstation::protocol::{
    BeaconEmitter, MascotHint, MascotStateBeacon, TaskState, TaskStatus,
};
use colored::*;
use std::env;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, Mutex};

enum WorkerEvent {
    TaskFinished {
        task_id: String,
        success: bool,
        output_snippet: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "============================================================".bright_blue());
    println!("{}", "🧊 [Cold Path] 纯 Rust 无头短剧流水线执行器 (Drama Worker)".bright_cyan().bold());
    println!("{}", "   [遵循 Goose 纯无头引擎 + Bellard 滤镜流隔离 + 细粒度遥测信标]".bright_black());
    println!("{}", "============================================================".bright_blue());

    let args: Vec<String> = env::args().collect();
    let goal_text = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "《被迫害妄想症》第一集短剧成片与分镜全流程调度".to_string()
    };

    let emitter = BeaconEmitter::new();
    let task_id = format!("DRAMA-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));

    // 1. 发射规划中状态信标
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let initial_beacon = MascotStateBeacon {
        version: "1.0.0".into(),
        source: "drama-worker".into(),
        updated_at: now,
        task: TaskState {
            id: task_id.clone(),
            title: goal_text.clone(),
            status: TaskStatus::Planning,
            step_name: "GRAPH_DECOMPOSE".into(),
            step_index: 0,
            total_steps: 4,
            progress: 5.0,
            detail: "正在利用 rig 强类型规则拆解全流程原子依赖图...".into(),
            error_msg: None,
        },
        mascot: MascotHint {
            posture: "ThinkingPondering".into(),
            mood: "focused".into(),
            dialogue: format!("收到目标【{}】，正在拆解工种依赖拓扑图！", goal_text),
            preferred_avatar: Some("chibi".into()),
        },
    };
    emitter.emit(&initial_beacon)?;

    // 2. 内存图构建
    let mut dag = GoalDecomposer::decompose(&goal_text);
    let total_tasks = dag.tasks.len();
    println!("📊 [拓扑图就绪] 总计 {} 个原子工种节点:", total_tasks.to_string().yellow());
    for (id, t) in &dag.tasks {
        println!("   ├─ [{}] {} (依赖: {:?})", id.bright_white(), t.title.bright_black(), t.depends_on);
    }

    let (tx, mut rx) = mpsc::channel::<WorkerEvent>(50);
    let current_dag = Arc::new(Mutex::new(dag));

    // 3. 触发首批无依赖任务
    {
        let dag_lock = current_dag.lock().await;
        let ready_tasks = dag_lock.get_ready_tasks();
        for mut t in ready_tasks {
            t.status = EngineTaskStatus::Running;
            let tx_clone = tx.clone();
            let t_id = t.id.clone();
            let t_title = t.title.clone();

            println!("🚀 [并发调度启动] -> [{}] {}", t_id.yellow(), t_title);
            tokio::spawn(async move {
                let (success, snippet) = execute_atomic_task(&t).await;
                let _ = tx_clone.send(WorkerEvent::TaskFinished {
                    task_id: t_id,
                    success,
                    output_snippet: snippet,
                }).await;
            });
        }
    }

    let mut step_counter = 0;

    // 4. 事件驱动循环推进拓扑
    while let Some(event) = rx.recv().await {
        match event {
            WorkerEvent::TaskFinished {
                task_id: completed_id,
                success,
                output_snippet,
            } => {
                step_counter += 1;
                let mut dag_lock = current_dag.lock().await;

                if let Some(task) = dag_lock.tasks.get_mut(&completed_id) {
                    task.status = if success {
                        EngineTaskStatus::Completed
                    } else {
                        EngineTaskStatus::Failed(output_snippet.clone())
                    };
                    task.output = Some(output_snippet.clone());

                    let icon = if success { "✅".green() } else { "❌".red() };
                    println!(
                        "{} [{}/{}] [{}] {} -> 输出: \"{}\"",
                        icon,
                        step_counter,
                        total_tasks,
                        completed_id.bright_yellow(),
                        task.title.bold(),
                        output_snippet.bright_black()
                    );
                }

                let progress = dag_lock.get_progress_percentage() as f32;
                let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

                if dag_lock.is_all_completed() {
                    // 全量竣工
                    let final_beacon = MascotStateBeacon {
                        version: "1.0.0".into(),
                        source: "drama-worker".into(),
                        updated_at: now,
                        task: TaskState {
                            id: task_id.clone(),
                            title: goal_text.clone(),
                            status: TaskStatus::Success,
                            step_name: "ALL_COMPLETED".into(),
                            step_index: total_tasks,
                            total_steps: total_tasks,
                            progress: 100.0,
                            detail: "所有工种已执行完毕，剪映草稿工程成功合龙！".into(),
                            error_msg: None,
                        },
                        mascot: MascotHint {
                            posture: "ExcitedCheering".into(),
                            mood: "happy".into(),
                            dialogue: format!("主人！全流程任务【{}】已全部打通交付！", goal_text),
                            preferred_avatar: Some("chibi".into()),
                        },
                    };
                    emitter.emit(&final_beacon)?;

                    println!("\n{}", "🎉 ========================================================".bright_green());
                    println!("🏁 [全流程交付成功] 目标【{}】已全部完成并通知桌宠信标！", goal_text.bright_white().bold());
                    println!("{}", "============================================================".bright_green());
                    break;
                } else if !success {
                    // 局部节点异常
                    let err_beacon = MascotStateBeacon {
                        version: "1.0.0".into(),
                        source: "drama-worker".into(),
                        updated_at: now,
                        task: TaskState {
                            id: task_id.clone(),
                            title: goal_text.clone(),
                            status: TaskStatus::Failed,
                            step_name: completed_id.clone(),
                            step_index: step_counter,
                            total_steps: total_tasks,
                            progress,
                            detail: format!("节点 [{}] 遇到异常: {}", completed_id, output_snippet),
                            error_msg: Some(output_snippet.clone()),
                        },
                        mascot: MascotHint {
                            posture: "PanickedError".into(),
                            mood: "sad".into(),
                            dialogue: format!("报告主人！节点【{}】出现异常，已保护现场！", completed_id),
                            preferred_avatar: Some("chibi".into()),
                        },
                    };
                    emitter.emit(&err_beacon)?;
                    println!("⚠️ [节点故障已隔离] 已发射异常信标，等待后续恢复或重试。");
                    break;
                } else {
                    // 推进中：发射进度遥测
                    let step_title = dag_lock
                        .tasks
                        .get(&completed_id)
                        .map(|t| t.title.as_str())
                        .unwrap_or("推进中");
                    let running_beacon = MascotStateBeacon {
                        version: "1.0.0".into(),
                        source: "drama-worker".into(),
                        updated_at: now,
                        task: TaskState {
                            id: task_id.clone(),
                            title: goal_text.clone(),
                            status: TaskStatus::Running,
                            step_name: completed_id.clone(),
                            step_index: step_counter,
                            total_steps: total_tasks,
                            progress,
                            detail: format!("已完成 [{}]，输出: {}", step_title, output_snippet),
                            error_msg: None,
                        },
                        mascot: MascotHint {
                            posture: "WorkingFocused".into(),
                            mood: "focused".into(),
                            dialogue: format!("节点【{}】搞定，当前进度 {:.0}%，正在推进下一环！", step_title, progress),
                            preferred_avatar: Some("chibi".into()),
                        },
                    };
                    emitter.emit(&running_beacon)?;

                    // 拓扑推进：调度新就绪的下游任务
                    let ready_tasks = dag_lock.get_ready_tasks();
                    for mut next_task in ready_tasks {
                        next_task.status = EngineTaskStatus::Running;
                        let tx_clone = tx.clone();
                        let n_id = next_task.id.clone();
                        let n_title = next_task.title.clone();

                        println!("⚡ [拓扑触发下游] -> [{}] {}", n_id.yellow(), n_title);
                        tokio::spawn(async move {
                            let (s, out) = execute_atomic_task(&next_task).await;
                            let _ = tx_clone.send(WorkerEvent::TaskFinished {
                                task_id: n_id,
                                success: s,
                                output_snippet: out,
                            }).await;
                        });
                    }
                }
            }
        }
    }

    Ok(())
}
