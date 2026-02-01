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
wt status                                  # 查看状态 (TUI)
wt tail auth                               # 查看最后输出
wt logs                                    # 生成调试日志
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
| `wt status [--json]` | 查看状态 (默认 TUI) |
| `wt tail <name> [-n N]` | 查看最后 N 条输出 (JSON) |
| `wt logs` | 生成所有任务的过滤日志 |
| `wt done <name>` | 标记完成 |
| `wt merged <name>` | 标记已合并 |
| `wt reset <name>` | 重置任务到 pending |
| `wt cleanup [--all]` | 清理资源 |

## Status TUI 快捷键

| 按键 | 功能 |
|------|------|
| `↑↓` / `jk` | 导航 |
| `Enter` | 进入 tmux 窗口 |
| `t` | tail (查看输出) |
| `d` | 标记 done (agent 已退出) |
| `m` | 标记 merged |
| `q` | 退出 |

**Enter 行为**：
- tmux 内 + 窗口存在 → 切换到目标窗口
- tmux 内 + 窗口已关 → 输出 resume 命令
- tmux 外 + 窗口存在 → 执行 `tmux attach`
- tmux 外 + 窗口已关 → 输出 resume 命令

## 配置

配置文件位于 `.wt/config.yaml`：

```yaml
# Claude CLI 命令（默认: claude）
# 如果你使用别名，在这里配置
# claude_command: ccc

# wt start 执行的参数
start_args: --verbose --output-format=stream-json -p "@.wt/tasks/${task}.md 请完成任务"

# tmux session 名称
tmux_session: my-project

# 其他可选配置
# worktree_dir: .wt/worktrees
# init_script: npm install
# copy_files:
#   - .env

# 日志过滤 (wt logs)
# logs:
#   exclude_types: [system, progress]
#   exclude_fields: [signature, uuid]
```

## 任务状态

```
○ Pending  →  ● Running  →  ◉ Done  →  ✓ Merged
```

## License

MIT
