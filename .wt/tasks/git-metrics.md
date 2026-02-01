---
name: git-metrics
depends:
- task-helpers
---

# 创建 GitMetrics 结构体

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

获取 git diff、commit count、conflict 状态的逻辑重复在：

1. `src/tui/app.rs:126-166`
2. `src/commands/status.rs:235-247`

相同模式：获取 additions、deletions、commits、has_conflict。

## 任务

### 0. 修复 `parse_diff_stats` bug

`src/services/git.rs` 中的 `parse_diff_stats` 函数有 bug：当 `main...HEAD` 返回空字符串（没有 commit）时，返回 `Some((0, 0))` 而不是 `None`，导致不会 fallback 到 `diff HEAD` 显示未提交的改动。

修复：
```rust
fn parse_diff_stats(output: &str) -> Option<(i32, i32)> {
    let output = output.trim();
    if output.is_empty() {
        return None;  // 改为 None，触发 fallback 显示未提交改动
    }
    // ... 其余不变
}
```

### 1. 在 `src/services/git.rs` 添加：

```rust
/// Git 工作区统计信息
#[derive(Debug, Clone, Default)]
pub struct GitMetrics {
    pub additions: i32,
    pub deletions: i32,
    pub commits: i32,
    pub has_conflict: bool,
}

/// 获取 worktree 的 git 统计信息
pub fn get_worktree_metrics(worktree_path: &str) -> Option<GitMetrics> {
    let path = Path::new(worktree_path);
    if !path.exists() {
        return None;
    }

    let (additions, deletions) = get_diff_stats(worktree_path).unwrap_or((0, 0));
    let commits = get_commit_count(worktree_path).unwrap_or(0);
    let has_conflict = has_merge_conflict(worktree_path);

    Some(GitMetrics {
        additions,
        deletions,
        commits,
        has_conflict,
    })
}
```

### 2. 更新调用点

- `src/tui/app.rs` - 使用 `get_worktree_metrics()`
- `src/commands/status.rs` - 使用 `get_worktree_metrics()`

### 3. 导出结构体

在 `src/services/mod.rs` 确保 `GitMetrics` 被导出。

## 验证

```bash
cargo test
cargo clippy
```

## 参考

- `src/services/git.rs` - 现有 `get_diff_stats()`, `get_commit_count()`, `has_merge_conflict()`
- `src/tui/app.rs:TaskMetrics` - 可参考现有的 TaskMetrics 用法
