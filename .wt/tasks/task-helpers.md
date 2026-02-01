---
name: task-helpers
---

# 在 TaskStore 添加辅助方法

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

多个 command 文件重复相同的任务检查模式：

```rust
let _task = store
    .get(&name)
    .ok_or_else(|| WtError::TaskNotFound(name.clone()))?;
```

出现在：
- `src/commands/start.rs:79-82`
- `src/commands/done.rs:9-11`
- `src/commands/reset.rs:17-19`
- `src/commands/archive.rs:12-14`
- `src/commands/merged.rs:9-11`
- `src/commands/tail.rs:21-23`

同样，状态转换验证也有重复：
- `src/commands/done.rs:13-19`
- `src/commands/archive.rs:16-22`

## 任务

### 1. 在 `src/models/store.rs` 的 `TaskStore` 中添加：

```rust
/// 确保任务存在，否则返回 TaskNotFound 错误
pub fn ensure_exists(&self, name: &str) -> Result<&Task> {
    self.get(name)
        .ok_or_else(|| WtError::TaskNotFound(name.to_string()))
}

/// 验证状态转换是否合法
pub fn validate_transition(&self, name: &str, target: TaskStatus) -> Result<()> {
    let current = self.get_status(name);
    if !current.can_transition_to(&target) {
        return Err(WtError::InvalidStateTransition {
            from: current.display_name().to_string(),
            to: target.display_name().to_string(),
        });
    }
    Ok(())
}
```

### 2. 更新所有调用点

将重复代码替换为新方法调用。

## 验证

```bash
cargo test
cargo clippy
```

## 参考

- `src/models/store.rs` - TaskStore 定义
- `src/models/task.rs` - Task, TaskStatus 定义
- `src/error.rs` - WtError::TaskNotFound, WtError::InvalidStateTransition
