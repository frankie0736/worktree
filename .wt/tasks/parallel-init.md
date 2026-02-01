---
name: parallel-init
---

# 并行化 init_script 执行

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

当前 `wt start --all` 顺序执行每个任务的 init_script：

```rust
// src/commands/start.rs - execute_single()
if let Some(ref script) = config.init_script {
    println!("  Running init script...");
    initializer.run_init_script(script)?;  // 阻塞等待完成
}
// 然后才创建 tmux 窗口
tmux::create_window(..., &agent_cmd)?;
```

5 个任务，每个 `cargo check` 10 秒 = 等待 50 秒。

## 解决方案

将 init_script 移到 tmux 窗口内执行：

```rust
// 构建命令：init_script && agent_cmd
let full_cmd = if let Some(ref script) = config.init_script {
    format!("{} && {}", script, agent_cmd)
} else {
    agent_cmd
};

tmux::create_window(&config.tmux_session, &name, &worktree_path, &full_cmd)?;
```

## 任务

### 1. 修改 `src/commands/start.rs`

在 `execute_single()` 函数中：

1. 移除同步执行 init_script 的代码
2. 将 init_script 拼接到 tmux 命令中

```rust
// 删除这段
if let Some(ref script) = config.init_script {
    println!("  Running init script...");
    initializer.run_init_script(script)?;
}

// 修改 agent_cmd 构建逻辑
let full_cmd = match &config.init_script {
    Some(script) => format!("({}) && {}", script, agent_cmd),
    None => agent_cmd,
};

tmux::create_window(&config.tmux_session, &name, &worktree_path, &full_cmd)?;
```

### 2. 更新输出提示

```rust
// 修改前
println!("  Running init script...");

// 修改后（在创建窗口后提示）
if config.init_script.is_some() {
    println!("  Init script will run in tmux window");
}
```

### 3. 可选：保留同步模式

添加 `--sync-init` flag 供需要顺序执行的场景：

```rust
// cli.rs
Start {
    name: Option<String>,
    #[arg(long)]
    all: bool,
    #[arg(long)]
    sync_init: bool,  // 可选：保留原有行为
}
```

如果不需要此功能，可以跳过。

## 验证

```bash
cargo test
cargo clippy
```

手动测试：
```bash
# 创建两个测试任务
wt create --json '{"name": "test1", "depends": []}'
wt create --json '{"name": "test2", "depends": []}'

# 启动并观察是否快速返回
time wt start --all

# 在 tmux 中观察 init_script 是否在运行
wt status
```

## 参考

- `src/commands/start.rs:53-56` - 当前 init_script 执行位置
- `src/commands/start.rs:75` - tmux::create_window 调用
- `src/services/workspace.rs` - WorkspaceInitializer
