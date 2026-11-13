# 🧬 Self-Improvement & Evolution Log (自我进化与经验沉淀库)

> 本记录遵循 `agent-self-improve` 规范，沉淀关键设计突破、踩坑修正与最佳工程实践。

---

## 📌 [L-20261113-001] 架构决策：REA 逆向工程与长程创作中枢的 Rust 全域融合

- **日期**: 2026-11-13
- **类别**: `best_practice` / `architecture`
- **问题背景**:
  用户前期积累了大量前沿开源资产（`morluto/rea` 逆向一切、`dsh-live2d-companion` 桌宠看板娘、`agent-autoloop` 后台自治循环、`loopx` 目标控制台、`jev-rs`/`mu` 极速分流）与本地创作工种（`自心写作总控`、`ai-drama-pipeline`），原本各组件独立存在，难以无人值守长周期跑通爆款目标。
- **演进与解法**:
  1. **放弃胶水脚本，全面拥抱 Rust 原生重构**：采用 Rust (2021 edition) + `goblin` + `serde` 构建单一二进制无依赖运行时，内存开销降低至 MB 级，杜绝长时间挂机内存泄漏与 GC 停顿。
  2. **证据账本（Evidence-Driven Spec）机制**：将 REA 原理内置化，通过对二进制/AST 的无损符号抽取，输出 `specs/*_spec.json` 强类型约束，彻底根除 LLM 创作/重构代码时的模板化幻觉。
  3. **具身看板娘（自心灵姬）状态机**：通过 `live2d.rs` 协议同步敲键盘、推眼镜、交付欢呼等 5 种微姿态，打破传统 CLI 工具冷冰冰的感知边界。
  4. **打通影视蓝图总控**：直接与 `ScreenBlueprint/PROJECT_STATE.md` 建立双向事实源桥接，统一生产调度标准。

---

## 📌 [L-20261113-002] Git 推送合规：GitHub Private Email 隐私防御触发

- **日期**: 2026-11-13
- **类别**: `correction` / `tool_gotcha`
- **现象**:
  首次推送到 GitHub 远程仓库时，被服务端拒绝：`GH007: Your push would publish a private email address`。
- **根因**:
  本地 Git 配置默认使用了含有物理主机特征的私有邮箱，触发了 GitHub 开启的 "Block command line pushes that expose my email" 保护机制。
- **进化解法**:
  在自动化提交流中，固化采用 GitHub 官方推荐的免泄密 noreply 签名：
  ```bash
  git config user.name "mkafw"
  git config user.email "mkafw@users.noreply.github.com"
  git commit --amend --reset-author --no-edit
  ```
- **沉淀策略**:
  后续所有本地创建的自动化仓库均默认注入该配置，不再产生二次拦截阻断。
