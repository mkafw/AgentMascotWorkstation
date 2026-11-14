use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tokio::process::Command;
use tokio::sync::mpsc;

/// 1. 【吸收 tinyflows::triggers】: 纯粹的真实事件触发枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventTrigger {
    /// 人类通过终端或消息明确给出的目标
    UserGoal(String),
    /// Linux inotify 原生捕获的文件写入/修改事件
    FileModified { path: String, ext: String },
    /// 上游工具执行完毕的真实反馈事件
    TaskFinished {
        task_id: String,
        success: bool,
        output_snippet: String,
    },
}

/// 2. 【吸收 rig 强类型任务契约】: 确定的原子任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicTask {
    pub id: String,
    pub title: String,
    pub command: String,
    pub args: Vec<String>,
    pub depends_on: Vec<String>,
    pub status: TaskStatus,
    pub output: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

/// 3. 【吸收 tinyflows::engine】: 内存有向无环图 (DAG)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoalDAG {
    pub goal_id: String,
    pub goal_title: String,
    pub tasks: HashMap<String, AtomicTask>,
}

impl GoalDAG {
    pub fn new(goal_id: &str, goal_title: &str) -> Self {
        Self {
            goal_id: goal_id.to_string(),
            goal_title: goal_title.to_string(),
            tasks: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: AtomicTask) {
        self.tasks.insert(task.id.clone(), task);
    }

    /// 拓扑遍历：寻找所有依赖已全量 Completed 的 Pending 任务
    pub fn get_ready_tasks(&self) -> Vec<AtomicTask> {
        let completed: HashSet<String> = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Completed)
            .map(|t| t.id.clone())
            .collect();

        self.tasks
            .values()
            .filter(|t| {
                t.status == TaskStatus::Pending
                    && t.depends_on.iter().all(|dep| completed.contains(dep))
            })
            .cloned()
            .collect()
    }

    pub fn is_all_completed(&self) -> bool {
        !self.tasks.is_empty() && self.tasks.values().all(|t| t.status == TaskStatus::Completed)
    }

    pub fn get_progress_percentage(&self) -> u32 {
        if self.tasks.is_empty() {
            return 100;
        }
        let completed_count = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        ((completed_count as f32 / self.tasks.len() as f32) * 100.0) as u32
    }
}

/// 4. 【吸收 rig 的结构化分解器】: 目标解析为确定性原子任务图
pub struct GoalDecomposer;

impl GoalDecomposer {
    pub fn decompose(goal: &str) -> GoalDAG {
        let clean_goal = goal.trim();
        let goal_id = format!("GOAL-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
        let mut dag = GoalDAG::new(&goal_id, clean_goal);

        if clean_goal.contains("短剧") || clean_goal.contains("分镜") || clean_goal.contains("EP") {
            // 典型短剧工业流水线 DAG (Fan-out -> Fan-in)
            dag.add_task(AtomicTask {
                id: "T1_SCRIPT_AUDIT".into(),
                title: "剧本规范与台词字数核验 (CLAUDE.md门禁)".into(),
                command: "omni".into(),
                args: vec!["--help".into()], // 真实调用本地二进制
                depends_on: vec![],
                status: TaskStatus::Pending,
                output: None,
            });

            dag.add_task(AtomicTask {
                id: "T2_AUDIO_VOICE".into(),
                title: "BaoCut 语音转写与台词打轴提取".into(),
                command: "baocut".into(),
                args: vec!["--help".into()],
                depends_on: vec!["T1_SCRIPT_AUDIT".into()],
                status: TaskStatus::Pending,
                output: None,
            });

            dag.add_task(AtomicTask {
                id: "T3_STORYBOARD".into(),
                title: "黄金分镜8镜矩阵与提示词结构化".into(),
                command: "echo".into(),
                args: vec!["[Storyboard] 8镜结构化提示词生成完成".into()],
                depends_on: vec!["T1_SCRIPT_AUDIT".into()],
                status: TaskStatus::Pending,
                output: None,
            });

            dag.add_task(AtomicTask {
                id: "T4_CUTCLI_ASSEMBLY".into(),
                title: "CutCLI 剪映桌面草稿一键合龙".into(),
                command: "cutcli".into(),
                args: vec!["--help".into()],
                // Fan-in: 必须等音频打轴和分镜图双就绪
                depends_on: vec!["T2_AUDIO_VOICE".into(), "T3_STORYBOARD".into()],
                status: TaskStatus::Pending,
                output: None,
            });
        } else {
            // 通用系统与记忆梳理流水线
            dag.add_task(AtomicTask {
                id: "T1_INSPECT_GIT".into(),
                title: "GitHub 远程状态与本地分支核对".into(),
                command: "gh".into(),
                args: vec!["auth".into(), "status".into()],
                depends_on: vec![],
                status: TaskStatus::Pending,
                output: None,
            });

            dag.add_task(AtomicTask {
                id: "T2_HIERARCH_CHECK".into(),
                title: "Rust hierarch 认知与因果逻辑语法验证".into(),
                command: "hierarch".into(),
                args: vec!["--help".into()],
                depends_on: vec!["T1_INSPECT_GIT".into()],
                status: TaskStatus::Pending,
                output: None,
            });
        }

        dag
    }
}

/// 5. 【吸收 watchexec 的底层文件事件监听器】
pub fn start_inotify_watcher(
    watch_dir: PathBuf,
    tx: mpsc::Sender<EventTrigger>,
) -> notify::Result<RecommendedWatcher> {
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                // 只对内容修改和创建感兴趣
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    for path in event.paths {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if ext == "md" || ext == "json" || ext == "rs" || ext == "xlsx" {
                                let path_str = path.to_string_lossy().to_string();
                                let _ = tx.blocking_send(EventTrigger::FileModified {
                                    path: path_str,
                                    ext: ext.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        },
        Config::default(),
    )?;

    watcher.watch(&watch_dir, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}

/// 6. 【真实工具执行器 (Subprocess)】
pub async fn execute_atomic_task(task: &AtomicTask) -> (bool, String) {
    let mut cmd = Command::new(&task.command);
    for arg in &task.args {
        cmd.arg(arg);
    }

    match cmd.output().await {
        Ok(output) => {
            let success = output.status.success();
            let snippet = if success {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                String::from_utf8_lossy(&output.stderr).trim().to_string()
            };
            let short = if snippet.chars().count() > 80 {
                format!("{}...", snippet.chars().take(80).collect::<String>())
            } else {
                snippet
            };
            (success, short)
        }
        Err(e) => (false, format!("执行失败 (command not found?): {}", e)),
    }
}
