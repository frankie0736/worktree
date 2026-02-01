# TUI 可测试操作

## 目标

为 `wt status` 添加 `--action` 参数，支持以非交互方式执行 TUI 操作，使 TUI 功能可被 CI、开发测试、AI Agent 自动化验证。

## 用户故事

1. **作为开发者**，我希望在 CI 中验证 TUI 的状态变更逻辑没被破坏
2. **作为开发者**，我希望开发时快速验证功能，不用进入交互模式
3. **作为 AI Agent**，我希望能执行 TUI 操作并获取结构化结果

## 命令行接口

```bash
# 查看任务可用操作
wt status --action list --task <name>

# 执行状态变更
wt status --action done --task <name>      # 标记完成 (d 键)
wt status --action merged --task <name>    # 标记已合并 (m 键)
wt status --action archive --task <name>   # 归档 (a 键)

# 获取导航信息 (Enter 键)
wt status --action enter --task <name>

# 查看任务详情 (t 键)
wt status --action tail --task <name>
```

## JSON 输出格式

### 成功响应

```json
{
  "action": "done",
  "success": true,
  "task": {
    "name": "ui",
    "status_before": "running",
    "status_after": "done"
  }
}
```

### 查看可用操作

```json
{
  "action": "list",
  "success": true,
  "task": {
    "name": "ui",
    "status": "running",
    "tmux_alive": true
  },
  "available_actions": ["tail", "enter"],
  "unavailable_actions": {
    "done": "task is running with tmux alive",
    "merged": "task is not done",
    "archive": "task is not merged"
  }
}
```

### Enter 操作响应

```json
{
  "action": "enter",
  "success": true,
  "task": {
    "name": "ui"
  },
  "command": {
    "type": "tmux_switch",
    "session": "try-wt",
    "window": "ui"
  }
}
```

或 tmux 窗口已关闭时：

```json
{
  "action": "enter",
  "success": true,
  "task": {
    "name": "ui"
  },
  "command": {
    "type": "resume",
    "worktree": "/path/to/worktree",
    "session_id": "xxx",
    "shell_command": "cd /path && claude -r xxx"
  }
}
```

### 错误响应

```json
{
  "action": "done",
  "success": false,
  "error": "Cannot mark as done: tmux window still alive",
  "task": {
    "name": "ui",
    "status": "running"
  }
}
```

退出码：非零

## 支持的操作

| 操作 | 对应按键 | 前置条件 | 效果 |
|------|----------|----------|------|
| `list` | - | 任务存在 | 返回可用操作列表 |
| `done` | d | Running + tmux 已退出 | 状态 → Done |
| `merged` | m | Done | 状态 → Merged，关闭 tmux |
| `archive` | a | Merged | 状态 → Archived，删除 worktree/分支 |
| `enter` | Enter | Running/Done | 返回 tmux 命令或 resume 信息 |
| `tail` | t | Running/Done | 调用 `wt tail` 并返回结果 |

## 错误处理

- 任务不存在：`{"success": false, "error": "Task 'xxx' not found"}`
- 操作不可用：`{"success": false, "error": "Cannot xxx: reason"}`
- 所有错误返回非零退出码

## 技术方案

1. **CLI 层** (`src/cli.rs`)
   - 添加 `--action <ACTION>` 参数，可选值：list, done, merged, archive, enter, tail
   - 添加 `--task <NAME>` 参数，指定目标任务

2. **命令层** (`src/commands/status.rs`)
   - 当有 `--action` 参数时，调用新的 `execute_action()` 函数
   - 复用 `App` 的逻辑判断（can_mark_done, can_mark_merged 等）

3. **复用现有逻辑**
   - 状态变更复用 `App::mark_done()`, `App::mark_merged()`, `App::archive()`
   - 操作可用性检查复用 `App::can_mark_done()` 等方法
   - Enter 操作复用 `App::enter_action()`

## 验收标准

- [ ] `wt status --action list --task ui` 返回可用操作
- [ ] `wt status --action done --task ui` 成功时返回 success=true，状态变更
- [ ] `wt status --action done --task ui` 条件不满足时返回 success=false + 错误信息
- [ ] 所有操作返回 JSON 格式
- [ ] 错误情况返回非零退出码
- [ ] 现有 `wt status` 和 `wt status --json` 行为不变

## 测试用例

```bash
# 1. 创建测试任务
wt create --json '{"name": "test", "description": "test"}'

# 2. 验证 pending 任务可用操作（应该没有）
wt status --action list --task test
# expect: available_actions = []

# 3. 启动任务，验证 running 任务可用操作
wt start test
wt status --action list --task test
# expect: available_actions = ["tail", "enter"]

# 4. 验证非法操作报错
wt status --action done --task test
# expect: success=false (tmux still alive)

# 5. 关闭 tmux 后验证 done 操作
tmux kill-window -t xxx:test
wt status --action done --task test
# expect: success=true, status_after="done"

# 6. 验证 merged 操作
wt status --action merged --task test
# expect: success=true, status_after="merged"
```
