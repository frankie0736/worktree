# Handoff 文档 - wt 开发进度

## Session 7 完成的工作 (2026-02-02)

### 1. TUI 显示优化

**布局精简**：
```
旧: ▸ ● auth   12:34  ▰▰▰▰▰▱▱▱▱▱  45%   +123 -45
新: ▸ ● auth   12:34  45%  3c  +128/-12  [Edit]
```

- 去掉 context 进度条，只保留百分比（按使用量着色）
- 精简代码增删显示：`+128/-12`
- 新增 commit 数：`3c` 表示 3 个 commit
- 新增冲突检测：有冲突显示红色 `⚡CONFLICT`
- 新增当前操作：显示最后使用的工具名

### 2. 修复 transcript 查找逻辑

**问题**：Claude CLI 的 `--session-id` 参数不会强制使用我们指定的 ID，它只用于恢复已有 session。新 session 时 Claude 会自己生成 ID，导致我们保存的 session_id 找不到对应的 transcript。

**修复**：新增 `find_latest_transcript()` 函数，当保存的 session_id 找不到对应 transcript 时，自动查找该 worktree 最新的 transcript 文件。

影响的命令：`status`、`tail`、`logs`

### 3. 增强 `status --json` 输出

新增字段用于调试：
- `context_percent` - context 使用百分比
- `current_tool` - 当前使用的工具
- `has_conflict` - 是否有 merge 冲突
- `session_id` - 保存的 session ID
- `transcript_exists` - transcript 文件是否存在

### 4. 修复 `reset` 清理孤立资源

**问题**：`wt start` 过程中 init_script 失败时，worktree/分支已创建但状态还是 pending，导致 `wt reset` 无法清理。

**修复**：`reset` 现在即使是 pending 状态也会检查并清理可能存在的残留 worktree 和分支。

### 5. 配置模板更新

`wt init` 生成的配置现在包含 `archive_script` 示例。

### 6. 代码清理

- 删除未使用的 `tmux::kill_session` 函数
- 删除未使用的 `WtError::NoSessionId` 变体

---

## 历史 Session 摘要

### Session 6 (2026-02-02)
- 新增 Archived 状态和 `wt archive` 命令
- 重构 `wt merged` 只关 tmux，保留 worktree
- `wt reset` 增加自动备份功能
- 分支名添加唯一后缀避免冲突

### Session 5 (2026-02-02)
- `wt review` 重构为 `wt tail`
- 新增 `wt logs` 命令

### Session 1-4 (2026-02-01)
- 初始实现：所有基础命令、TUI、transcript 集成
- 目录结构重构：`.wt.yaml` → `.wt/config.yaml`

---

## 待实现功能

### TUI 可测试操作 (spec 已写)

位置：`.claude/specs/tui-testable-actions.md`

为 `wt status` 添加 `--action` 参数，支持非交互方式执行 TUI 操作：

```bash
wt status --action list --task ui      # 查看可用操作
wt status --action done --task ui      # 执行 done
wt status --action merged --task ui    # 执行 merged
wt status --action archive --task ui   # 执行 archive
wt status --action enter --task ui     # 获取 tmux 命令
wt status --action tail --task ui      # 查看输出
```

---

## 已知问题

1. **旧任务无法 tail**：没有 session_id 的任务无法 tail（已通过 find_latest_transcript 缓解）
2. **context_percent 计算**：使用固定 200k context window

---

## 相关文件索引

| 文件 | 说明 |
|------|------|
| `src/commands/status.rs` | status 命令（含 --json 增强）|
| `src/commands/tail.rs` | tail 命令（使用 find_latest_transcript）|
| `src/commands/logs.rs` | logs 命令（使用 find_latest_transcript）|
| `src/commands/reset.rs` | reset 命令（清理孤立资源）|
| `src/commands/init.rs` | init 命令（配置模板含 archive_script）|
| `src/services/transcript.rs` | transcript 服务（新增 find_latest_transcript）|
| `src/services/git.rs` | git 服务（新增 has_conflicts, find_branches）|
| `src/tui/ui.rs` | TUI 渲染（新布局）|
| `src/tui/app.rs` | TUI 应用状态（新增字段）|
| `.claude/specs/tui-testable-actions.md` | TUI 可测试操作 spec |
