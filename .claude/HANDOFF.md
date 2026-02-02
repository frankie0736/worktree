# Handoff 文档 - wt 开发进度

## Session 10 完成的工作 (2026-02-02)

### Bug 修复：TUI merged/archive 花屏问题

**问题**：在 TUI 中按 `m` (merged) 或 `a` (archive) 时屏幕花屏

**原因**：`app.mark_merged()` 和 `app.archive()` 调用的命令会 `println!` 到 stdout，污染 TUI 的 alternate screen

**解决方案**：为 `merged::execute()` 和 `archive::execute()` 添加 `silent` 参数

| 调用场景 | silent 值 | 行为 |
|----------|-----------|------|
| CLI (`wt merged/archive`) | false | 正常输出提示 |
| TUI (按 m/a 键) | true | 静默执行 |
| Action API (`--action`) | true | 只输出 JSON |

**修改文件**：
- `src/commands/merged.rs` - 添加 `silent: bool` 参数
- `src/commands/archive.rs` - 添加 `silent: bool` 参数
- `src/main.rs` - CLI 调用传 `false`
- `src/tui/app.rs` - TUI 调用传 `true`

---

## Session 9 完成的工作

### 代码重构（8 个任务）

**DRY 改进**：
- `branch_pattern()` - 统一分支名模式 (`src/constants.rs`)
- `ensure_exists()` / `validate_transition()` - TaskStore 辅助方法
- `kill_window_if_exists()` - tmux 窗口管理
- `find_transcript_for_instance()` - transcript 路径查找
- `auto_mark_done_if_needed()` - 自动标记完成
- `GitMetrics` 结构体 - 统一 git 统计

**SRP 改进**：
- 拆分 `status.rs` (800+ 行) 为 `status/` 模块

**新功能**：
- `wt start --all` - 一键启动所有就绪任务
- init_script 在 tmux 窗口内并行执行

---

## 历史 Session 摘要

| Session | 主要工作 |
|---------|----------|
| 8 | `--action` API、done 统一关闭 tmux |
| 7 | TUI 优化、transcript 查找、冲突检测 |
| 6 | Archived 状态、备份功能、分支后缀 |
| 5 | tail/logs 命令 |
| 1-4 | 初始实现 |

---

## 已知问题

1. **旧任务无法 tail**：无 session_id（已通过 find_latest_transcript 缓解）
2. **context_percent**：使用固定 200k

---

## 相关文件

| 文件 | 说明 |
|------|------|
| `src/commands/merged.rs` | merged 命令，支持 silent 模式 |
| `src/commands/archive.rs` | archive 命令，支持 silent 模式 |
| `src/commands/status/` | status 命令模块 |
| `src/tui/app.rs` | TUI 状态和操作 |
| `src/models/store.rs` | TaskStore 及辅助方法 |
| `src/services/git.rs` | GitMetrics 和 git 操作 |
| `src/services/tmux.rs` | tmux 操作 |
