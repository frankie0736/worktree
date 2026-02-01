---
name: auto-mark-done
depends:
- task-helpers
- tmux-helpers
- transcript-helpers
---

# 提取 auto-mark-done 逻辑

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

完全相同的 auto-mark-done 逻辑在两处重复：

1. `src/tui/app.rs:75-104`
2. `src/commands/status.rs:169-198`

逻辑：检查 Running 状态的任务，如果 tmux 窗口已关闭，自动标记为 Done。

## 任务

### 1. 在 `src/models/store.rs` 的 `TaskStore` 添加：

```rust
use crate::services::tmux;

impl TaskStore {
    /// 检查任务是否需要自动标记为 Done
    /// 条件：状态为 Running，但 tmux 窗口已关闭
    /// 返回：是否执行了自动标记
    pub fn auto_mark_done_if_needed(&mut self, task_name: &str) -> Result<bool> {
        let status = self.get_status(task_name);
        if status != TaskStatus::Running {
            return Ok(false);
        }

        let state = match self.status_store.get(task_name) {
            Some(s) => s,
            None => return Ok(false),
        };

        let instance = match &state.instance {
            Some(inst) => inst,
            None => return Ok(false),
        };

        // 检查 tmux 窗口是否还存在
        if tmux::window_exists(&instance.tmux_session, &instance.tmux_window) {
            return Ok(false);
        }

        // 窗口已关闭，自动标记为 Done
        self.set_status(task_name, TaskStatus::Done)?;
        Ok(true)
    }
}
```

### 2. 更新调用点

- `src/tui/app.rs` - 调用 `store.auto_mark_done_if_needed()`
- `src/commands/status.rs` - 调用 `store.auto_mark_done_if_needed()`

## 验证

```bash
cargo test
cargo clippy
```

## 注意

- 需要在 `store.rs` 添加 `use crate::services::tmux;`
- 此方法会修改状态，调用者需要持有 `&mut TaskStore`

## 参考

- `src/models/store.rs` - TaskStore 定义
- `src/services/tmux.rs` - `window_exists()` 函数
- 前置任务 `task-helpers`、`tmux-helpers` 的改动
