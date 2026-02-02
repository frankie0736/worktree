# Handoff 文档 - wt 开发进度

## Session 10 完成的工作 (2026-02-02)

### 1. Bug 修复：TUI merged/archive 花屏

为 `merged::execute()` 和 `archive::execute()` 添加 `silent` 参数，TUI 调用时静默执行。

### 2. Bug 修复：start_args 空格截断

`tmux send-keys` 添加 `-l` (literal) 选项，正确处理带空格的中文 prompt。

### 3. 改进：默认交互模式

`wt init` 生成的 config 默认使用交互模式，注释中提供非交互模式示例。

```yaml
# 交互模式（默认）
start_args: '"@.wt/tasks/${task}.md 请完成这个任务"'

# 非交互模式（注释示例）
# start_args: --verbose --output-format=stream-json -p "..."
```

---

## 待实现功能

### wt new - 快速创建隔离环境

**Spec**: `.claude/specs/wt-new.md`

```bash
wt new              # 自动生成 new-YYYYMMDD-HHMMSS
wt new <name>       # 指定名称
```

- 创建 worktree/分支/tmux，但不启动 claude
- `status.json` 中标记 `scratch: true`
- done/merged 禁止，archive/reset 直接清理

---

## 历史 Session 摘要

| Session | 主要工作 |
|---------|----------|
| 9 | 代码重构（DRY/SRP）、`wt start --all`、init_script 并行化 |
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
| `src/commands/init.rs` | init 命令，默认交互模式 |
| `src/commands/merged.rs` | merged 命令，支持 silent 模式 |
| `src/commands/archive.rs` | archive 命令，支持 silent 模式 |
| `src/services/tmux.rs` | tmux 操作，send-keys -l |
| `src/commands/status/` | status 命令模块 |
| `src/tui/app.rs` | TUI 状态和操作 |
| `.claude/specs/wt-new.md` | wt new 功能 spec |
