# 目录结构重构规格

## 概述

将 wt 的配置和数据统一放入 `.wt/` 目录下，简化项目结构。

## 目标

1. 将 `.wt.yaml` 移动到 `.wt/config.yaml`
2. 将 `.wt-worktrees/` 默认移动到 `.wt/worktrees/`
3. 整个 `.wt/` 目录被 gitignore

## 新目录结构

```
project/
├── .wt/
│   ├── config.yaml    # 配置文件
│   ├── tasks/         # 任务定义 (*.md)
│   ├── status.json    # 运行时状态
│   └── worktrees/     # git worktrees (默认位置)
├── .gitignore         # 包含 .wt/ 条目
└── ...
```

### 对比旧结构

| 旧位置 | 新位置 |
|--------|--------|
| `.wt.yaml` | `.wt/config.yaml` |
| `.wt/tasks/` | `.wt/tasks/` (不变) |
| `.wt/status.json` | `.wt/status.json` (不变) |
| `.wt-worktrees/` | `.wt/worktrees/` (默认) |

## Git 追踪策略

**整个 `.wt/` 目录被 gitignore**

这意味着：
- 配置文件 (`config.yaml`) 不共享
- 任务定义 (`tasks/`) 不共享
- 运行时状态 (`status.json`) 不共享
- Worktrees (`worktrees/`) 不共享

每个开发者有自己独立的本地 wt 环境。

## config.yaml 变更

### 新的默认配置模板

`wt init` 生成全量带注释的配置文件：

```yaml
# wt 配置文件
# 文档: https://github.com/yansir/wt

# ============================================
# 必填配置
# ============================================

# 启动 agent 的命令
# 支持模板变量: ${task}, ${branch}, ${worktree}
agent_command: |
  claude --output-format stream-json -p "你是一个资深的软件工程师..."

# ============================================
# 可选配置
# ============================================

# tmux session 名称
# 默认: 项目目录名
tmux_session: "${project_name}"

# Worktree 存放目录
# 默认: .wt/worktrees
# 支持相对路径（相对于项目根目录）和绝对路径
# worktree_dir: .wt/worktrees

# 初始化脚本 (在每个新 worktree 中执行)
# 例如安装依赖、设置环境等
# init_script: |
#   npm install

# 需要复制到 worktree 的文件
# 这些文件不会被 git checkout 带过去
# copy_files:
#   - .env
#   - .env.local
```

### 配置项说明

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `agent_command` | string | 是 | - | 启动 agent 的命令 |
| `tmux_session` | string | 否 | 项目目录名 | tmux session 名称 |
| `worktree_dir` | string | 否 | `.wt/worktrees` | worktree 存放目录 |
| `init_script` | string | 否 | - | worktree 初始化脚本 |
| `copy_files` | list | 否 | - | 复制到 worktree 的文件列表 |

### 模板变量

| 变量 | 说明 | 示例 |
|------|------|------|
| `${task}` | 任务名 | `auth` |
| `${branch}` | 分支名 | `wt/auth` |
| `${worktree}` | worktree 绝对路径 | `/path/to/project/.wt/worktrees/auth` |

## 代码变更

### constants.rs

```rust
// 旧
pub const CONFIG_FILE: &str = ".wt.yaml";
pub const DEFAULT_WORKTREE_DIR: &str = ".wt-worktrees";

// 新
pub const CONFIG_FILE: &str = ".wt/config.yaml";
pub const DEFAULT_WORKTREE_DIR: &str = ".wt/worktrees";
```

### init.rs

1. 创建 `.wt/` 目录
2. 创建 `.wt/config.yaml` (全量带注释)
3. 创建 `.wt/tasks/` 目录
4. 更新 `.gitignore` 添加 `.wt/` (单条规则替代多条)

### .gitignore 变更

旧版本添加的条目：
```
# wt worktree manager
.wt-worktrees/
.wt/status.json
```

新版本只需一条：
```
# wt worktree manager
.wt/
```

## wt init 行为

```bash
$ wt init

# 创建目录
mkdir -p .wt/tasks

# 生成配置文件
cat > .wt/config.yaml << 'EOF'
# ... 全量带注释的配置 ...
EOF

# 更新 .gitignore
echo ".wt/" >> .gitignore  # (如果不存在)

# 输出提示
echo "Initialized wt in current directory."
echo ""
echo "Next steps:"
echo "  1. Edit .wt/config.yaml to customize settings"
echo "  2. Create tasks: wt create --json '{...}'"
echo "  3. Start working: wt start <task>"
```

## 迁移策略

**不需要向后兼容**

这是本地玩具项目，直接使用新结构。用户需要：
1. 删除旧的 `.wt.yaml` 和 `.wt-worktrees/`
2. 运行 `wt init` 重新初始化

## 测试计划

### 单元测试

- `constants::CONFIG_FILE` 返回新路径
- `constants::DEFAULT_WORKTREE_DIR` 返回新路径

### 集成测试

- `wt init` 创建正确的目录结构
- `wt init` 生成包含注释的完整配置文件
- `.gitignore` 只添加 `.wt/` 一条规则
- `wt start` 在 `.wt/worktrees/` 下创建 worktree
- 自定义 `worktree_dir` 仍然生效

## 文件变更清单

| 文件 | 变更 |
|------|------|
| `src/constants.rs` | 更新 `CONFIG_FILE` 和 `DEFAULT_WORKTREE_DIR` |
| `src/commands/init.rs` | 更新目录创建和配置生成逻辑 |
| `src/models/config.rs` | 可能需要调整默认值处理 |
| `tests/cli/init.rs` | 更新测试用例 |
| `.claude/CLAUDE.md` | 更新文档 |
