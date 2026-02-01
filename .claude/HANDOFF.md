# Handoff 文档 - wt 开发进度

## Session 8 完成的工作 (2026-02-02)

### 1. TUI 可测试操作

为 `wt status` 添加 `--action` 和 `--task` 参数：

```bash
wt status --action list --task ui      # 查看可用操作
wt status --action done --task ui      # 标记完成（自动关 tmux）
wt status --action merged --task ui    # 标记已合并
wt status --action archive --task ui   # 归档
wt status --action enter --task ui     # 获取 tmux 命令
wt status --action tail --task ui      # 查看输出
```

### 2. done 操作统一行为

所有 done 操作都会自动关闭 tmux：
- TUI `d` 键
- `--action done`
- `wt done`

### 3. TUI footer 提示修复

Running 状态现在正确显示 `d done` 快捷键。

---

## 历史 Session 摘要

| Session | 主要工作 |
|---------|----------|
| 7 | TUI 优化、transcript 查找修复、冲突检测 |
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
| `src/cli.rs` | CLI 参数 |
| `src/commands/status.rs` | status 命令（含 --action）|
| `src/commands/done.rs` | done 命令（自动关 tmux）|
| `src/tui/app.rs` | TUI 状态 |
| `src/tui/ui.rs` | TUI 渲染 |
