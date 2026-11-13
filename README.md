# 🎯 AgentMascotWorkstation (自心灵姬 · 自治创作工作站)

> **融合体**：Live2D 具身看板娘 + Agent-AutoLoop 长程自治引擎 + 自心写作总控 / 短剧九大工业工种

---

## 一、 为什么将它们融为一体？
在传统的创作流中，工具、代码库与大模型是割裂的：
1. **割裂痛点**：拆文用一个脚本、写正文手动敲提示词、看进度要翻文件夹，没法实现“挂机长程跑完 Top 榜单”。
2. **融合收益**：
   - **皮囊（具身看板娘）**：赋予系统温度与状态透传（开工专注敲键盘、审稿推眼镜挑刺、交付元气欢呼）。
   - **骨架（AutoLoop 循环状态机）**：自动从任务池轮询 Top 目标，逐级打通阶段闭环，不达质检标准不交付。
   - **血肉（自心心法与短剧管线）**：沉淀五感八触、前三秒留存、剪映工程一键成片工业级能力。

---

## 二、 目录结构规范
```
/home/daoxuan/Projects/AgentMascotWorkstation/
├── config.json               # 看板娘人设、阈值与执行阶段配置
├── autoloop.py               # 核心调度与自治循环引擎 (可后台守护运行)
├── index.html                # 桌面/网页实时可视化看板 (双向状态同步)
└── tasks/
    ├── queue.json            # Top 目标任务队列 (长程遍历目标池)
    └── runtime_state.json    # 看板娘情绪、实时台词与产线进度信标
```

---

## 三、 使用与启动指令

### 1. 运行后台自动巡航引擎
```bash
python3 /home/daoxuan/Projects/AgentMascotWorkstation/autoloop.py
```

### 2. 打开看板娘可视化控制台
可在浏览器中直接打开：
`file:///home/daoxuan/Projects/AgentMascotWorkstation/index.html`

### 3. 添加新的 Top 目标
直接在 `tasks/queue.json` 中追加任务（或后续通过 TG 机器人推送）：
```json
{
  "id": "TASK-20261113-003",
  "title": "异界修仙开局：白描赋比兴手法正文爆更",
  "type": "novel_long",
  "priority": 1,
  "status": "pending",
  "assigned_skill": "自心写作总控",
  "meta": { "genre": "玄幻修仙 / 爽点前置" }
}
```
