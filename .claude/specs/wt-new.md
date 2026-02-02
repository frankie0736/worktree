# Spec: wt new - 快速创建隔离开发环境

## 背景

用户有时想快速创建一个隔离的 worktree 环境进行探索性开发，不需要预先定义 task 文件。

## 命令

```bash
wt new              # 自动生成名称 new-YYYYMMDD-HHMMSS
wt new <name>       # 指定名称
```

## 行为

### 创建流程

1. **名称处理**
   - 无参数：生成 `new-YYYYMMDD-HHMMSS`（如 `new-20260202-143052`）
   - 有参数：使用用户指定的名称

2. **冲突检查**
   - 检查 `.wt/tasks/<name>.md` 是否存在 → 报错
   - 检查 `status.json` 中是否已有同名条目 → 报错
   - 检查分支 `wt/<name>-*` 是否存在 → 报错

3. **创建资源**（复用 start 逻辑）
   - 创建 git worktree 和分支 `wt/<name>-<uuid>`
   - 执行 `init_script`（如果配置了）
   - 复制 `copy_files`（如果配置了）
   - 创建 tmux 窗口并 cd 到 worktree 目录

4. **记录状态**
   - 在 `status.json` 中记录，状态为 `Running`
   - 添加 `scratch: true` 标记区分

5. **不启动 claude**
   - 与 `wt start` 不同，tmux 窗口只打开 shell，不执行 claude 命令

### 输出

```
Created scratch environment 'new-20260202-143052'
  Worktree: .wt/worktrees/new-20260202-143052
  Branch:   wt/new-20260202-143052-abc123
  Tmux:     project:new-20260202-143052
```

## 与其他命令的交互

| 命令 | 对 scratch 环境的行为 |
|------|----------------------|
| `list` | 不显示（只显示有 task 文件的） |
| `status` | 正常显示（跟普通 task 一样） |
| `done` | 禁止，报错 "Scratch 环境请直接使用 wt archive" |
| `merged` | 禁止，报错 "Scratch 环境请直接使用 wt archive" |
| `archive` | 允许从 Running 直接 archive（跳过 Merged 检查） |
| `reset` | 等同于 archive（直接清理资源） |
| `validate` | 不检查（没有 task 文件） |
| `next` | 不显示（没有 task 文件） |
| `tail/logs` | 无 transcript，报错（预期行为） |

## 数据结构变更

### status.json

```json
{
  "tasks": {
    "new-20260202-143052": {
      "status": "running",
      "scratch": true,
      "instance": {
        "branch": "wt/new-20260202-143052-abc123",
        "worktree_path": ".wt/worktrees/new-20260202-143052",
        "tmux_session": "project",
        "tmux_window": "new-20260202-143052",
        "session_id": null
      }
    }
  }
}
```

## 实现清单

### 新增文件

- [ ] `src/commands/new.rs` - new 命令实现

### 修改文件

- [ ] `src/cli.rs` - 添加 New 命令定义
- [ ] `src/commands/mod.rs` - 注册 new 模块
- [ ] `src/main.rs` - 路由 new 命令
- [ ] `src/models/status.rs` - TaskState 添加 `scratch: Option<bool>` 字段
- [ ] `src/models/store.rs` - 添加 `is_scratch()` 辅助方法
- [ ] `src/commands/done.rs` - 检查 scratch，禁止并报错
- [ ] `src/commands/merged.rs` - 检查 scratch，禁止并报错
- [ ] `src/commands/archive.rs` - 允许 scratch 从 Running 直接 archive
- [ ] `src/commands/reset.rs` - scratch 时等同于 archive

### 测试

- [ ] `tests/cli/new.rs` - CLI 测试
  - `wt new` 自动生成名称
  - `wt new xxx` 指定名称
  - 名称冲突检测
  - scratch 环境的 done/merged 禁止
  - scratch 环境的 archive 允许
  - scratch 环境的 reset 等同 archive
