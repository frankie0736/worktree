# wt - Worktree Task Manager

多 agent 并行开发任务管理 CLI 工具。

## 项目概述

wt 通过 git worktree 隔离工作区、tmux 管理 agent 进程、依赖关系控制任务执行顺序，实现多个 AI agent 并行开发不同功能分支。

## 目录结构

```
src/
├── main.rs           # CLI 入口
├── lib.rs            # 库导出
├── cli.rs            # Clap 命令定义
├── constants.rs      # 路径常量 (TASKS_DIR, STATUS_FILE 等)
├── error.rs          # 错误类型 (WtError)
├── models/
│   ├── task.rs       # Task, TaskStatus, TaskInput, TaskFrontmatter
│   ├── status.rs     # StatusStore, TaskState (运行时状态)
│   ├── store.rs      # TaskStore (加载任务 + 状态)
│   └── config.rs     # WtConfig (.wt.yaml 解析)
├── commands/         # 各子命令实现
│   ├── init.rs
│   ├── create.rs
│   ├── validate.rs
│   ├── list.rs
│   ├── next.rs
│   ├── start.rs
│   ├── done.rs
│   ├── merged.rs
│   ├── cleanup.rs
│   └── enter.rs
└── services/
    ├── command.rs    # 命令执行辅助 (CommandRunner)
    ├── git.rs        # git worktree 操作
    ├── tmux.rs       # tmux session/window 操作
    ├── workspace.rs  # worktree 初始化 (WorkspaceInitializer)
    └── dependency.rs # 依赖检查
```

## 核心概念

### Task（任务）

**定义**存储在 `.wt/tasks/*.md`（Git 追踪）：

```yaml
name: auth          # 任务名（= 文件名，= git 分支名 wt/<name>）
depends:            # 依赖的任务列表
  - database
```

**状态**存储在 `.wt/status.json`（Git 忽略）：

```json
{
  "tasks": {
    "auth": { "status": "running", "instance": {...} },
    "database": { "status": "merged" }
  }
}
```

### TaskStatus 状态流转

```
○ Pending  →  ● Running  →  ◉ Done  →  ✓ Merged
   (wt start)    (wt done)    (wt merged)
```

### 依赖规则

- 任务只能在所有依赖都 `Merged` 后才能 `start`
- `validate` 会检测循环依赖

## 常用命令

```bash
cargo build --release    # 编译
cargo test               # 运行测试
cargo install --path .   # 安装到 ~/.cargo/bin
```

## 任务名验证规则

任务名必须是有效的 git 分支名：
- 不能为空
- 不能含空格、制表符
- 不能含 `~ ^ : ? * [ @ {`
- 不能以 `-` 或 `.` 开头
- 不能以 `.` 或 `.lock` 结尾
- 不能含 `..`

## 相关文件

- @README.md - 用户文档
- @.claude/rules/rust-style.md - Rust 编码规范
- @.claude/rules/testing.md - 测试指南
