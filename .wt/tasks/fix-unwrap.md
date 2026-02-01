---
name: fix-unwrap
---

# 修复 tail.rs 中不安全的 unwrap()

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

`src/commands/tail.rs:68` 使用了不安全的 `unwrap()`：

```rust
println!("{}", serde_json::to_string_pretty(&output).unwrap());
```

如果序列化失败会导致 panic。

## 任务

将 `unwrap()` 改为安全的错误处理：

```rust
println!("{}", serde_json::to_string_pretty(&output)?);
```

由于 `run()` 函数已经返回 `Result<()>`，可以直接使用 `?` 传播错误。

## 验证

```bash
cargo test
cargo clippy
```

检查 `src/commands/tail.rs` 没有其他 `unwrap()` 调用（测试代码除外）。

## 参考

- `.claude/rules/rust-style.md` - 错误处理规范
- `src/error.rs` - WtError 定义
