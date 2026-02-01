---
name: transcript-helpers
---

# 添加 find_transcript_for_instance()

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

transcript 路径查找逻辑重复 3 处：

1. `src/tui/app.rs:106-117`
2. `src/commands/status.rs:207-215`
3. `src/commands/tail.rs:43-49`

相同逻辑：
```rust
let path_from_id = inst
    .session_id
    .as_ref()
    .and_then(|sid| transcript::transcript_path(&inst.worktree_path, sid))
    .filter(|p| p.exists());

path_from_id.or_else(|| transcript::find_latest_transcript(&inst.worktree_path))
```

## 任务

### 1. 在 `src/services/transcript.rs` 添加：

```rust
use crate::models::task::Instance;

/// 查找 Instance 对应的 transcript 文件
/// 优先使用 session_id 精确匹配，否则查找最新的 transcript
pub fn find_transcript_for_instance(instance: &Instance) -> Option<PathBuf> {
    instance
        .session_id
        .as_ref()
        .and_then(|sid| transcript_path(&instance.worktree_path, sid))
        .filter(|p| p.exists())
        .or_else(|| find_latest_transcript(&instance.worktree_path))
}
```

### 2. 更新调用点

- `src/tui/app.rs` - 使用新函数
- `src/commands/status.rs` - 使用新函数
- `src/commands/tail.rs` - 使用新函数

## 验证

```bash
cargo test
cargo clippy
```

## 注意

需要在 `transcript.rs` 顶部添加 `use crate::models::task::Instance;`

## 参考

- `src/services/transcript.rs` - transcript 相关函数
- `src/models/task.rs:Instance` - Instance 结构体定义
