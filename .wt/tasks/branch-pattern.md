---
name: branch-pattern
---

# 添加 branch_pattern() 辅助函数

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

分支名模式 `wt/{task}-*` 硬编码在多处：

1. `src/services/git.rs:31` - `find_branches()` 中使用 `"wt/{}-*"`
2. `src/commands/reset.rs:111` - `find_branches(&format!("wt/{}-*", task_name))`

这违反了 SSOT（单一事实来源）原则。

## 任务

在 `src/constants.rs` 中添加 `branch_pattern()` 辅助函数：

```rust
/// 生成用于查找任务相关分支的 glob 模式
/// 例如：task_name = "auth" → "wt/auth-*"
pub fn branch_pattern(task_name: &str) -> String {
    format!("{}{}-*", BRANCH_PREFIX, task_name)
}
```

然后更新以下调用点：
- `src/services/git.rs` 中的 `find_branches()` 调用
- `src/commands/reset.rs` 中的 `find_branches()` 调用

## 验证

```bash
cargo test
cargo clippy
```

## 参考

- `src/constants.rs:32-35` - 现有的 `branch_name()` 函数
- `BRANCH_PREFIX` 常量值为 `"wt/"`
