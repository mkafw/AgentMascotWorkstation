use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// 桥接层：实现 ScreenBlueprint 与 AgentMascotWorkstation 跨项目单一事实源对接
pub struct BlueprintBridge;

impl BlueprintBridge {
    /// 自动从影视蓝图中读取当前判定的生产状态与下一步任务
    pub fn read_blueprint_state(blueprint_path: &Path) -> Option<String> {
        let state_file = blueprint_path.join("PROJECT_STATE.md");
        if state_file.exists() {
            fs::read_to_string(state_file).ok()
        } else {
            None
        }
    }
}
