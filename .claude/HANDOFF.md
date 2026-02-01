# Handoff 文档 - wt 开发进度

## Session 4 完成的工作 (2026-02-01)

### 1. 目录结构重构

- `.wt.yaml` → `.wt/config.yaml`
- `.wt-worktrees/` → `.wt/worktrees/` (默认)
- `.gitignore` 只需一条 `.wt/` 规则

### 2. 配置项重构

- 删除 `agent_command`
- 新增 `claude_command`：基础命令，默认 `claude`，允许 `ccc` 或 `claude --yolo`
- 新增 `start_args`：start 命令的参数

**命令变更**：
- `wt start`: 执行 `${claude_command} ${start_args}`
- `wt review`: 使用 `${claude_command}` 替代硬编码的 `claude`

### 3. TUI Enter 行为重构

| 环境 | tmux 窗口状态 | Enter 行为 |
|------|--------------|-----------|
| tmux 内 | 存在 | 切换到目标窗口 |
| tmux 内 | 已关 | 输出 resume 命令 |
| tmux 外 | 存在 | 执行 `tmux attach` |
| tmux 外 | 已关 | 输出 resume 命令 |

### 4. 文档整理

- 删除已实现的 specs 文件
- 更新 README.md、CLAUDE.md、SKILL.md
- 移除 `wt enter` 命令相关提示

---

## Session 3 完成的工作 (2026-02-01)

### 1. 简化 `wt review` 设计

直接输出两个可复制的命令，不需要 TUI。

### 2. 删除 `wt enter` 命令

功能已被 `wt status` TUI 的 Enter 键替代。

### 3. 修改 `wt status` 默认行为

- `wt status` → TUI（默认）
- `wt status --json` → JSON 输出
- 非 TTY 环境自动降级到 JSON

### 4. TUI 新增快捷键

| 按键 | 功能 |
|------|------|
| `Enter` | 进入 tmux 窗口 |
| `r` | review |
| `d` | 标记 done |
| `m` | 标记 merged |

---

## Session 2 完成的工作 (2026-02-01)

### 1. 修复 Duration 时间跳动 Bug

从 Claude Code transcript 读取时间戳，不再使用当前时间计算。

### 2. 简化 Instance 结构

移除 `started_at`, `finished_at`，保留 `session_id`。

---

## Session 1 完成的工作

### 1. 新增 `wt review <task>` 命令

### 2. 迁移到 Claude Code Transcript

从 `~/.claude/projects/<escaped_path>/<session_id>.jsonl` 读取。

### 3. Git diff 统计改进

使用 `git diff --shortstat main...HEAD` 显示整个分支变更。

---

## 待实现功能

（暂无）

---

## 已知问题

1. **旧任务无法 review**：没有 session_id 的任务无法 review
2. **context_percent 计算**：使用固定 200k context window

---

## 相关文件索引

| 文件 | 说明 |
|------|------|
| `src/commands/review.rs` | review 命令实现 |
| `src/commands/status.rs` | status 命令（含 TUI 调用）|
| `src/services/transcript.rs` | transcript 解析服务 |
| `src/tui/app.rs` | TUI 应用状态和操作 |
| `src/tui/ui.rs` | TUI 渲染 |
| `src/tui/mod.rs` | TUI 入口和事件处理 |
