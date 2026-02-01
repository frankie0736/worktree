---
name: tmux-helpers
---

# 添加 kill_window_if_exists()

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

多处重复检查并关闭 tmux 窗口的代码：

1. `src/commands/done.rs:22-27`
2. `src/tui/app.rs:255-260`
3. `src/commands/status.rs:183-184`

相同模式：
```rust
if tmux::window_exists(&instance.tmux_session, &instance.tmux_window) {
    tmux::kill_window(&instance.tmux_session, &instance.tmux_window)?;
}
```

## 任务

### 1. 在 `src/services/tmux.rs` 添加：

```rust
/// 如果窗口存在则关闭，返回是否执行了关闭操作
pub fn kill_window_if_exists(session: &str, window: &str) -> Result<bool> {
    if window_exists(session, window) {
        kill_window(session, window)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
```

### 2. 更新调用点

将重复代码替换为：
```rust
tmux::kill_window_if_exists(&instance.tmux_session, &instance.tmux_window)?;
```

## 验证

```bash
cargo test
cargo clippy
```

## 参考

- `src/services/tmux.rs` - tmux 操作函数
- `window_exists()` 和 `kill_window()` 函数签名
