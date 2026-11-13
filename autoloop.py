#!/usr/bin/env python3
"""
AgentMascotWorkstation - 核心调度与自治循环引擎 (AutoLoop Core)
负责自动遍历 Top 目标、驱动自心写作心法与短剧管线、与看板娘状态实时同步
"""

import os
import sys
import json
import time
from datetime import datetime
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent
CONFIG_PATH = BASE_DIR / "config.json"
QUEUE_PATH = BASE_DIR / "tasks" / "queue.json"
STATE_PATH = BASE_DIR / "tasks" / "runtime_state.json"

def load_json(path, default=None):
    if not path.exists():
        return default or {}
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)

def save_json(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)

def update_mascot_state(status, mood, speech, task_id=None, progress=0):
    """更新看板娘与桌面组件的状态信标"""
    state = {
        "timestamp": datetime.now().isoformat(),
        "status": status,          # idle, working, reviewing, alert, finished
        "mood": mood,              # cheerful, focused, critical, surprised
        "speech": speech,          # 看板娘台词气泡
        "active_task_id": task_id,
        "progress": progress
    }
    save_json(STATE_PATH, state)
    # 控制台打印看板娘气泡
    print(f"\n🎀 【看板娘 · 自心灵姬】: 「{speech}」 (状态: {mood.upper()} | 进度: {progress}%)")

def get_next_top_task():
    queue = load_json(QUEUE_PATH, [])
    pending = [t for t in queue if t.get("status") in ["pending", "in_progress"]]
    if not pending:
        return None
    # 按优先级升序 (1 最高)
    pending.sort(key=lambda x: x.get("priority", 999))
    return pending[0]

def execute_stage(task, stage_name, skill):
    """模拟/驱动具体的技能执行"""
    task_id = task["id"]
    title = task["title"]
    
    stages_desc = {
        "01_target_scan": ("🔍 目标雷达扫描中...", "正在提取市场热点与竞品爽点节奏~"),
        "02_golden_three_chapters": ("✍️ 重构黄金三章...", "注入前三秒留存钩子与核心金手指设定！"),
        "03_world_and_characters": ("👥 设定人物与世界观...", "正在生成八触五感人物背景与动机矩阵~"),
        "04_drafting_with_zixin": ("🔥 自心心法全开正文...", "调动七情六欲与白描赋比兴手法爆肝中！"),
        "05_deslop_and_review": ("🧐 严苛审校与去AI味...", "正在清除模板化套路词与空洞抒情，准备交付~"),
        "06_storyboard_cutcli": ("🎬 工业分镜与成片导出...", "正在生成剪映草稿工程与首帧关键帧！")
    }

    action_name, bubble = stages_desc.get(stage_name, ("执行中", "努力干活中..."))
    update_mascot_state("working", "focused", bubble, task_id, task.get("progress", 0))
    time.sleep(1.5)  # 节奏步进

def run_loop():
    print("=" * 60)
    print("🚀 AgentMascotWorkstation 自治创作工作站启动")
    print("=" * 60)
    
    config = load_json(CONFIG_PATH)
    all_stages = config.get("stages", [])

    while True:
        task = get_next_top_task()
        if not task:
            update_mascot_state("idle", "cheerful", "主人，当前任务池全部搞定啦！随时把新脑洞丢给我吧~")
            print("\n☕ 队列已清空，看板娘进入待机守候状态...")
            break

        task_id = task["id"]
        title = task["title"]
        assigned_skill = task.get("assigned_skill", "自心写作总控")
        print(f"\n🎯 锁定 Top 目标: [{task_id}] {title}")
        print(f"🛠️ 派遣工种管线: {assigned_skill}")

        task["status"] = "in_progress"
        queue = load_json(QUEUE_PATH, [])
        for i, t in enumerate(queue):
            if t["id"] == task_id:
                queue[i] = task
                break
        save_json(QUEUE_PATH, queue)

        for idx, stage in enumerate(all_stages):
            progress = int(((idx + 1) / len(all_stages)) * 100)
            task["current_stage"] = stage
            task["progress"] = progress
            execute_stage(task, stage, assigned_skill)
            
            # 更新队列状态
            for i, t in enumerate(queue):
                if t["id"] == task_id:
                    queue[i] = task
                    break
            save_json(QUEUE_PATH, queue)

        # 任务完成
        task["status"] = "completed"
        task["finished_at"] = datetime.now().isoformat()
        for i, t in enumerate(queue):
            if t["id"] == task_id:
                queue[i] = task
                break
        save_json(QUEUE_PATH, queue)

        update_mascot_state("finished", "cheerful", f"太棒了！目标【{title}】已圆满交付，准备切入下一个！", task_id, 100)
        time.sleep(1)

    print("\n🏁 批次工作流全部圆满闭环！")

if __name__ == "__main__":
    run_loop()
