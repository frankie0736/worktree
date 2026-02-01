# wt review 命令设计规格

## 概述

`wt review <task>` 命令用于查看已结束任务的执行结果，并输出可复制的命令以便继续对话。

## 设计理念

**不需要 TUI**：review 的核心需求就是"继续对话"，输出可执行的命令是最直接的方式。

- **human 用户**：复制交互式命令，在 tmux 中执行
- **agent**：使用 `-p` 模式的命令，可程序化调用

## 核心流程

```
用户执行 wt review <task>
    │
    ├── 检查任务状态
    │   ├── Running? → 检查 tmux 窗口是否存在
    │   │   ├── 存在 → 拒绝，提示用 wt enter
    │   │   └── 不存在 → 自动标记为 Done，继续
    │   ├── Pending/Merged? → 拒绝，无结果可查看
    │   └── Done? → 继续
    │
    ├── 检查 worktree 存在
    │   └── 不存在? → 拒绝 review
    │
    ├── 检查 session_id 和 transcript 存在
    │   └── 不存在? → 拒绝 review
    │
    └── 输出结果
        ├── 显示统计信息（duration, tokens, turns）
        ├── 显示代码变更统计
        ├── 显示结果摘要
        └── 输出两个可复制的命令
```

## 数据源变更

### 移除 .wt/logs/ 和 tee 逻辑

**原方案**：`start` 时若 agent_command 包含 `--output-format=stream-json`，用 tee 输出到 `.wt/logs/<task>.jsonl`

**新方案**：直接读取 Claude Code 的 transcript 文件 `~/.claude/projects/<hash>/<session_id>.jsonl`

**优点**：
- 无论用什么模式启动 agent，transcript 都会生成
- 减少维护成本，不需要自己管理日志
- 数据更完整（transcript 包含所有工具调用、响应等）

### Instance 结构

```rust
// models/task.rs
pub struct Instance {
    pub branch: String,
    pub worktree_path: String,
    pub tmux_session: String,
    pub tmux_window: String,
    pub session_id: Option<String>,  // Claude Code session UUID
}
```

注意：`started_at/finished_at` 字段已移除，改用 transcript 中的时间戳计算 duration。

### start 命令变更

```rust
// commands/start.rs

// 生成 UUID
let session_id = uuid::Uuid::new_v4().to_string();

// 构建命令时添加 --session-id
let agent_cmd = format!(
    "{} --session-id {}",
    expanded_cmd,
    session_id
);

// 保存到 Instance
store.set_instance(&name, Some(Instance {
    // ...existing fields...
    session_id,
}));
```

### Transcript 路径定位

```rust
// services/transcript.rs

use sha2::{Sha256, Digest};

/// 计算 Claude Code 使用的项目路径 hash
pub fn project_hash(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let result = hasher.finalize();
    // Claude Code 使用前 16 个字符
    hex::encode(&result[..8])
}

/// 获取 transcript 文件路径
pub fn transcript_path(worktree_path: &str, session_id: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap();
    let hash = project_hash(worktree_path);
    home.join(".claude/projects")
        .join(hash)
        .join(format!("{}.jsonl", session_id))
}
```

## 输出格式

```
$ wt review auth

Task: auth (Done)

Duration: 45m 23s

## Statistics
  Input: 45,230 tokens | Output: 12,450 tokens | Turns: 23
  Context usage: 28%

## Code Changes
  +156 -23

## Result
Successfully implemented authentication module with JWT tokens...

---
# Resume conversation (interactive, for human):
cd ../wt-auth && claude -r 01jgxyz...

# Continue with prompt (for agent):
cd ../wt-auth && claude --output-format stream-json -r 01jgxyz... -p "继续完成任务"
```

### 命令说明

| 命令 | 用途 | 使用者 |
|------|------|--------|
| `claude -r <session_id>` | 交互式恢复对话 | human（通常在 tmux 中执行）|
| `claude --output-format stream-json -r <session_id> -p "..."` | 非交互式继续 | agent（程序化调用）|

### 为什么用 `-r` 而不是 `-c`

- `-c` (continue) 恢复最近的对话
- `-r` (resume) 恢复指定 session_id 的对话

使用 `-r` 更精确，避免在多任务环境下恢复错误的 session。

## Transcript 解析

### 需要提取的字段

从 transcript JSONL 中解析：

```rust
// services/transcript.rs

#[derive(Debug)]
pub struct TranscriptMetrics {
    /// 最终结果摘要
    pub result: Option<String>,
    /// 输入 tokens
    pub input_tokens: u64,
    /// 输出 tokens
    pub output_tokens: u64,
    /// 对话轮数
    pub num_turns: u32,
    /// 是否正常完成（有 result 条目）
    pub completed: bool,
}

pub fn parse_transcript(path: &Path) -> Option<TranscriptMetrics> {
    // 解析 JSONL，提取：
    // - type: "result" 条目的 result 字段
    // - type: "assistant" 条目的 usage 信息
    // - 计算轮数
}
```

### Result 条目格式

```json
{
  "type": "result",
  "result": "Task completed successfully...",
  "num_turns": 23,
  "modelUsage": {
    "claude-sonnet": {
      "inputTokens": 45230,
      "outputTokens": 12450,
      "contextWindow": 200000
    }
  }
}
```

## status TUI 集成（可选）

未来可考虑在 `status --watch` TUI 中添加快捷键：

| 按键 | 功能 |
|------|------|
| `r` | 对选中的 Done 任务显示 review 命令 |

但当前优先级较低，用户可以直接运行 `wt review <task>`。

## 文件结构变更

### 新增文件

```
src/
├── commands/
│   └── review.rs       # wt review 命令实现
└── services/
    └── transcript.rs   # transcript 解析
```

### 修改文件

```
src/
├── cli.rs              # 添加 review 子命令
├── main.rs             # 路由 review 命令
├── models/
│   └── status.rs       # Instance 添加 session_id
└── commands/
    └── start.rs        # 生成 session_id
```

## CLI 定义

```rust
// cli.rs

#[derive(Subcommand)]
pub enum Commands {
    // ...existing commands...

    /// Review task results and optionally continue conversation
    Review {
        /// Task name to review
        name: String,

        /// Output as JSON for programmatic use
        #[arg(long)]
        json: bool,
    },
}
```

### JSON 输出格式

```bash
$ wt review auth --json
```

```json
{
  "task": "auth",
  "status": "done",
  "worktree_path": "../wt-auth",
  "session_id": "01jgxyz...",
  "metrics": {
    "duration_secs": 2723,
    "input_tokens": 45230,
    "output_tokens": 12450,
    "num_turns": 23,
    "context_percent": 28
  },
  "code_changes": {
    "insertions": 156,
    "deletions": 23
  },
  "summary": "Successfully implemented...",
  "commands": {
    "interactive": "cd ../wt-auth && claude -r 01jgxyz...",
    "non_interactive": "cd ../wt-auth && claude --output-format stream-json -r 01jgxyz... -p \"继续完成任务\""
  }
}
```

Agent 可直接使用 `.commands.non_interactive` 或提取字段自己构建命令。

## 错误处理

```rust
// error.rs

#[derive(Error, Debug)]
pub enum WtError {
    // ...existing errors...

    #[error("Cannot review task '{0}': task is still running. Use 'wt enter {0}' instead")]
    CannotReviewRunning(String),

    #[error("Cannot review task '{0}': worktree no longer exists")]
    WorktreeNotFound(String),

    #[error("Cannot review task '{0}': session transcript not found")]
    TranscriptNotFound(String),

    #[error("Cannot continue task '{0}': task did not complete normally (no result in transcript)")]
    TaskNotCompleted(String),
}
```

## 依赖变更

```toml
# Cargo.toml

[dependencies]
sha2 = "0.10"      # 用于计算 path hash
hex = "0.4"        # 用于 hex 编码
uuid = { version = "1.0", features = ["v4"] }  # 用于生成 session_id
dirs = "5.0"       # 用于获取 home 目录
```

## 测试计划

### 单元测试

- `transcript::project_hash()` - 验证 hash 计算与 Claude Code 一致
- `transcript::parse_transcript()` - 解析各种 transcript 格式
- `Instance` 序列化/反序列化包含 session_id

### 集成测试

- `wt review` 对 Running 任务报错
- `wt review` 对 Pending 任务报错
- `wt review` worktree 不存在时报错
- `wt review` transcript 不存在时报错
- `wt review` 正常完成任务显示结果

### E2E 测试

- 完整流程：start → done → review → continue

## 迁移说明

### 对现有用户的影响

1. **status.json 格式变更**：Instance 新增 session_id 字段
   - 旧格式仍可加载（session_id 默认为空）
   - 空 session_id 的任务无法使用 review 功能

2. **.wt/logs/ 目录废弃**
   - 不再写入新日志
   - 现有日志文件不会被删除
   - 用户可手动清理

3. **agent_command 无需变更**
   - `--output-format=stream-json` 不再影响日志行为
   - 用户可以选择任意输出格式

## 未来扩展

### @ 文件引用

未来可考虑在 `wt review -p` 中支持 `@path/to/file` 语法：

```bash
wt review auth -p "请查看 @src/error.rs 并修复错误"
```

但当前优先级较低，用户可以手动构造 prompt。
