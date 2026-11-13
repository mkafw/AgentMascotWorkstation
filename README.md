# 🎯 星婉·全域创作工作站 (XingWan Autonomous Workstation)

> **全域统御中枢**：具身看板娘【星婉】 + REA 逆向工程一切 + AutoLoop 闭环自治引擎 + 自心写作总控 / 短剧九大工业工种

---

## 一、 系统定位：星婉 (XingWan)
「星婉」是统御整个影视蓝图、网文长篇与短剧工业管线的**具身制作人中枢与总导演看板娘**。
她不仅是一个桌宠或界面，更是整个创作生命周期的**神经中枢**：
1. **眼（REA 逆向雷达）**：穿透全网爆款与二进制靶体，生成不可篡改的证据账本 Spec。
2. **脑（Jev / Mu + Rust AutoLoop）**：毫秒级意图分流，无锁驱动长周期 Top 目标自治推进。
3. **手（自心心法 + 9 岗流水线）**：调动五感八触、白描赋比兴、去 AI 味审校与剪映一键成片。
4. **身（Live2D 具身桌宠）**：桌面上实时呈现情绪姿态（码字冒火、推眼镜挑刺、交付欢呼），遇重大分歧发卡片请人类拍板。

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
