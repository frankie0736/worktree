---
name: reuse-git-metrics
---

# 复用 GitMetrics 结构体

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

`GitMetrics` 和 `TaskMetrics` 存在字段重复：

```rust
// src/services/git.rs:8-14
pub struct GitMetrics {
    pub additions: i32,
    pub deletions: i32,
    pub commits: i32,
    pub has_conflict: bool,
}

// src/commands/status/types.rs (TaskMetrics 中)
pub additions: Option<i32>,
pub deletions: Option<i32>,
pub commits: Option<i32>,
pub has_conflict: Option<bool>,
```

## 任务

### 1. 修改 TaskMetrics 复用 GitMetrics

```rust
// src/commands/status/types.rs
use crate::services::git::GitMetrics;

#[derive(Serialize)]
pub struct TaskMetrics {
    pub name: String,
    pub status: TaskStatus,
    // ... 其他字段 ...

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitMetrics>,

    // 删除重复字段: additions, deletions, commits, has_conflict
}
```

### 2. 为 GitMetrics 添加 Serialize

```rust
// src/services/git.rs
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct GitMetrics { ... }
```

### 3. 更新 display.rs 中构建 TaskMetrics 的代码

将分散的字段赋值改为直接使用 `git: Some(metrics)`。

## 验证

```bash
cargo test
cargo clippy
```

确认 JSON 输出格式不变（`#[serde(flatten)]` 会展平字段）。

## 参考

- `src/services/git.rs:8-35` - GitMetrics 定义
- `src/commands/status/types.rs:8-38` - TaskMetrics 定义
- `src/commands/status/display.rs` - 构建 TaskMetrics 的代码
