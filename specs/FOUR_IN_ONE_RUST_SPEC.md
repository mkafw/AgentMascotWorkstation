# 🦀 星婉 · 四合一桌宠终端 Rust 架构实现全景指南

> 本指南拆解：如何通过 GitHub (gh) 逆向四大开源项目，并用纯 Rust (rs) 将它们 1:1 合成为单二进制应用。

---

## 一、 第一步：通过 `gh` (GitHub CLI) 穿透了解开源项目代码

不用在浏览器里繁琐翻找，直接在终端用 `gh` 提取这 4 个项目的关键模块：

```bash
# 1. 查看仓库 README 与核心设计
gh repo view 1190fasheqi/daifeiyu-pet
gh repo view YunYueSama/codex-deepseek-pet

# 2. 搜索特定的实现逻辑（例如：找他们是怎么实现透明窗口和动作循环的）
gh search code "frameless" --repo 1190fasheqi/daifeiyu-pet
gh search code "paintEvent" --repo YunYueSama/codex-deepseek-pet

# 3. 快速克隆或下载单文件
gh repo clone 1190fasheqi/daifeiyu-pet /tmp/daifeiyu-pet
```

---

## 二、 四大核心模块的 Rust 技术选型与映射

| 功能模块 | 原 Python/JS 项目来源 | 对应 Rust (Cargo) 最佳工业库 | 核心职责 |
| :--- | :--- | :--- | :--- |
| **透明置顶窗口 (GUI)** | `daifeiyu-desktop-pet` (PySide6) | `tauri` 或 `eframe` (egui) / `winit` | 开启 `transparent: true`, `decorations: false`, `always_on_top: true` |
| **60 FPS 动作帧机** | `YunYueSama/codex-deepseek-pet` | `tokio::time::interval` + Rust 状态机 | 控制呼吸、眨眼、行走、四羽煽动的帧索引切换 |
| **思维链气泡与对话** | `1190fasheqi/daifeiyu-pet` (CoT) | `reqwest` + `tokio` (流式推送) | 接收用户输入，流式打字机吐出思考过程与最终结果 |
| **任务看板驱动中枢** | `dsh-pet-indesktop` / `loopx` | `serde_json` + 本地 SQLite/文件管道 | 轮询 Top 创作目标，毫秒级推进进度条（0% ➔ 100%） |

---

## 三、 四合一 Rust 核心代码架构 (Rust 纯代码骨架)

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use std::time::Duration;

// 1. 状态定义：四合一的状态机
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PetAction {
    IdleBreathing,       // 摸鱼呼吸 (YunYueSama)
    Walking,             // 散步移动
    Thinking(String),    // 思维链输出中 (1190fasheqi CoT)
    Working(u32),        // 推进任务中, 进度 0~100 (DSH / loopx)
}

// 2. 任务看板条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub progress: u32,
    pub status: String,
}

// 3. 对话与语言调度中枢
pub struct MascotCommander {
    pub api_endpoint: String,
}

impl MascotCommander {
    // 语言交流：解析自然语言并自动路由给创作流水线
    pub async fn handle_user_command(&self, input: &str) -> String {
        if input.contains("去AI味") {
            "已为您启动 story-deslop 深度去AI味审校工种！正在对齐人情味！".into()
        } else if input.contains("写") || input.contains("续写") {
            "收到！正在调用自心写作总控，五感八触心法全开爆更中！".into()
        } else {
            format!("星婉收到法旨：「{}」，正在分配调度！", input)
        }
    }
}

// 4. Rust 主事件驱动闭环 (AutoLoop)
pub async fn run_mascot_loop(
    tx_ui: mpsc::Sender<PetAction>,
    commander: Arc<MascotCommander>,
) {
    let mut ticker = tokio::time::interval(Duration::from_millis(16)); // 60 FPS 刷新率
    let mut frame = 0;

    loop {
        ticker.tick().await;
        frame = (frame + 1) % 60;

        // 每隔特定周期轮询任务看板并更新思维链
        if frame == 0 {
            let action = PetAction::Working(45);
            let _ = tx_ui.send(action).await;
        }
    }
}
```
