# wt - Worktree Task Manager

通过 git worktree 隔离工作区，tmux 管理 agent 进程，依赖关系控制任务执行顺序。

## 依赖

- Git (支持 worktree)
- tmux
- Rust (编译安装)

## 安装

```bash
cargo install --path .
```

## 快速开始

```bash
wt init                                    # 初始化
wt create --json '{"name": "auth", "depends": [], "description": "实现认证"}'
wt start auth                              # 启动任务
wt done auth                               # 标记完成
wt merged auth                             # PR 合并后
```

## 命令

| 命令 | 说明 |
|------|------|
| `wt init` | 初始化配置 |
| `wt create --json '{...}'` | 创建任务 |
| `wt validate [name]` | 验证任务 |
| `wt list [--tree] [--json]` | 列出任务 |
| `wt next [--json]` | 显示可启动任务 |
| `wt start <name>` | 启动任务 |
| `wt done <name>` | 标记完成 |
| `wt merged <name>` | 标记已合并 |
| `wt reset <name>` | 重置任务到 pending |
| `wt status [--watch] [--json]` | 查看运行中任务状态 |
| `wt cleanup [--all]` | 清理资源 |
| `wt enter [task]` | 进入 tmux |

## 任务状态

```
○ Pending  →  ● Running  →  ◉ Done  →  ✓ Merged
```

## License

MIT
