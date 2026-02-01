# Handoff 文档 - wt 开发进度

## Session 6 完成的工作 (2026-02-02)

### 1. 新增 Archived 状态

**状态流变更**：
```
Pending → Running → Done → Merged → Archived
            ↑________|________|________|
                       (reset)
```

- `Archived` 是新的终态，表示任务已彻底完成并清理
- `reset` 现在可从 Running/Done/Merged/Archived 任一状态回到 Pending
- `Archived` 和 `Merged` 都视为"已完成"，允许依赖它们的任务启动

### 2. 重构 `wt merged` 命令

**行为变更**：
- 旧：merged 会删除 worktree、分支、tmux window
- 新：merged 只关闭 tmux window，保留 worktree 和分支供查看

### 3. 新增 `wt archive` 命令

**功能**：将 Merged 任务归档，执行完整清理

```bash
wt archive auth    # 归档 auth 任务
```

**执行步骤**：
1. 验证状态必须是 Merged
2. 执行 `archive_script`（如配置）
3. 删除 worktree、分支
4. 状态变为 Archived，清除 instance

### 4. 增强 `wt reset` 命令

**新功能**：reset 前自动备份代码到 `.wt/backups/`

```bash
wt reset auth    # 备份后重置
ls .wt/backups/  # 查看备份
```

**备份流程**：
1. 执行 `archive_script` 瘦身（删除 node_modules 等）
2. 复制代码到 `.wt/backups/{task}-{timestamp}/`
3. 跳过 `.git` 目录

### 5. 分支名添加唯一后缀

**格式**：`wt/{task}-{session_id前4位}`

示例：`wt/auth-3e20`、`wt/ui-a1b2`

**目的**：避免同名任务重复 start 时分支冲突

### 6. 新增配置项 `archive_script`

```yaml
# .wt/config.yaml
archive_script: |
  rm -rf node_modules/
  rm -rf dist/
  rm -rf target/
```

用于 archive 和 reset 前清理大文件。

### 7. 删除 `wt cleanup` 命令

不再需要，用户可手动删除 `.wt/backups/` 目录。

### 8. TUI 更新

| 按键 | 功能 |
|------|------|
| `t` | tail |
| `d` | 标记 done |
| `m` | 标记 merged |
| `a` | **新增：archive（仅 Merged 状态）** |

TUI 现在也显示 Merged 状态的任务，方便执行 archive。

---

## 历史 Session 摘要

### Session 5 (2026-02-02)
- `wt review` 重构为 `wt tail`
- 新增 `wt logs` 命令
- TUI 快捷键 `r` 改为 `t`

### Session 4 (2026-02-01)
- 目录结构重构：`.wt.yaml` → `.wt/config.yaml`
- 配置项：`claude_command` + `start_args`
- TUI Enter 智能切换 tmux 窗口

### Session 1-3 (2026-02-01)
- 初始实现：review、status TUI、transcript 集成
- 删除 `wt enter`，功能合并到 TUI Enter 键

---

## 待实现功能

（暂无）

---

## 已知问题

1. **旧任务无法 tail**：没有 session_id 的任务无法 tail
2. **context_percent 计算**：使用固定 200k context window

---

## 相关文件索引

| 文件 | 说明 |
|------|------|
| `src/commands/archive.rs` | archive 命令实现 |
| `src/commands/merged.rs` | merged 命令（只关 tmux） |
| `src/commands/reset.rs` | reset 命令（含备份逻辑） |
| `src/commands/tail.rs` | tail 命令实现 |
| `src/commands/logs.rs` | logs 命令实现 |
| `src/commands/status.rs` | status 命令（含 TUI 调用）|
| `src/models/task.rs` | TaskStatus 枚举（含 Archived）|
| `src/models/config.rs` | 配置解析（含 archive_script）|
| `src/constants.rs` | 常量（BACKUPS_DIR、branch_name）|
| `src/services/dependency.rs` | 依赖检查（Archived 视同 Merged）|
| `src/tui/app.rs` | TUI 应用状态（含 archive 操作）|
| `src/tui/ui.rs` | TUI 渲染（Merged 状态显示）|
| `src/tui/mod.rs` | TUI 入口（'a' 键处理）|
