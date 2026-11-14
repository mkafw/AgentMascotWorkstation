use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{SystemTime, UNIX_EPOCH};

/// MSP (Mascot State Protocol) v1.0 数据契约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MascotStateBeacon {
    pub version: String,
    pub source: String,
    pub updated_at: u64,
    pub task: TaskState,
    pub mascot: MascotHint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub step_name: String,
    pub step_index: usize,
    pub total_steps: usize,
    pub progress: f32, // 0.0 ~ 100.0
    pub detail: String,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Idle,
    Planning,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MascotHint {
    pub posture: String,
    pub mood: String,
    pub dialogue: String,
    pub preferred_avatar: Option<String>, // "chibi" 或 "goddess"
}

impl MascotStateBeacon {
    pub fn idle(source: &str, dialogue: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            version: "1.0.0".into(),
            source: source.into(),
            updated_at: now,
            task: TaskState {
                id: "IDLE".into(),
                title: "系统就绪待命".into(),
                status: TaskStatus::Idle,
                step_name: "WAITING".into(),
                step_index: 0,
                total_steps: 0,
                progress: 0.0,
                detail: "等待新任务接入...".into(),
                error_msg: None,
            },
            mascot: MascotHint {
                posture: "IdleBreath".into(),
                mood: "peaceful".into(),
                dialogue: dialogue.into(),
                preferred_avatar: Some("chibi".into()),
            },
        }
    }
}

/// 标准系统信标路径定位器 (优先遵循 Linux XDG 规范)
pub fn default_beacon_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".config")
            .join("agent-mascot")
            .join("runtime_state.json")
    } else {
        PathBuf::from("tasks/runtime_state.json")
    }
}

/// 生产者：内核级原子信标发射器
pub struct BeaconEmitter {
    primary_path: PathBuf,
    secondary_path: Option<PathBuf>,
}

impl BeaconEmitter {
    pub fn new() -> Self {
        let primary = default_beacon_path();
        // 兼容原有的 workspace 内部路径
        let secondary = PathBuf::from("tasks/runtime_state.json");
        Self {
            primary_path: primary,
            secondary_path: if secondary.parent().map(|p| p.exists()).unwrap_or(false) {
                Some(secondary)
            } else {
                None
            },
        }
    }

    pub fn with_path(primary: PathBuf, secondary: Option<PathBuf>) -> Self {
        Self {
            primary_path: primary,
            secondary_path: secondary,
        }
    }

    /// 内核级原子写入：先写临时文件再 rename，保证读取端绝无脏读
    pub fn emit(&self, beacon: &MascotStateBeacon) -> Result<(), std::io::Error> {
        self.atomic_write(&self.primary_path, beacon)?;
        if let Some(ref sec) = self.secondary_path {
            let _ = self.atomic_write(sec, beacon);
        }
        Ok(())
    }

    fn atomic_write(&self, target: &Path, beacon: &MascotStateBeacon) -> Result<(), std::io::Error> {
        let dir = target.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(dir)?;

        let json_data = serde_json::to_string_pretty(beacon)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let temp_path = dir.join(format!(".tmp_{}_{}", std::process::id(), fastrand()));
        {
            let mut file = fs::File::create(&temp_path)?;
            file.write_all(json_data.as_bytes())?;
            file.sync_all()?;
        }

        fs::rename(temp_path, target)?;
        Ok(())
    }
}

/// 消费者：零开销 Inotify 信标监听器
pub struct BeaconListener {
    target_path: PathBuf,
}

impl BeaconListener {
    pub fn new() -> Self {
        Self {
            target_path: default_beacon_path(),
        }
    }

    pub fn with_path(target_path: PathBuf) -> Self {
        Self { target_path }
    }

    pub fn read_current(&self) -> Result<Option<MascotStateBeacon>, std::io::Error> {
        if !self.target_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&self.target_path)?;
        let beacon: MascotStateBeacon = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some(beacon))
    }

    /// 阻塞式监听（微秒级内核事件通知，0% CPU 占用）
    pub fn listen<F>(&self, mut callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(MascotStateBeacon),
    {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        let watch_dir = self
            .target_path
            .parent()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Parent dir not found"))?;
        fs::create_dir_all(watch_dir)?;
        watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;

        for res in rx {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    let filename = self.target_path.file_name().unwrap_or_default();
                    if event.paths.iter().any(|p| p.file_name() == Some(filename)) {
                        if let Ok(Some(beacon)) = self.read_current() {
                            callback(beacon);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn fastrand() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
