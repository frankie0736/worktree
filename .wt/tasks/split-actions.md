---
name: split-actions
depends:
- reuse-git-metrics
---

# 拆分 actions.rs

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

`src/commands/status/actions.rs` 有 456 行，包含 7 个 action handler，职责仍然较多。

## 任务

### 方案 A：提取公共逻辑（推荐）

不拆分文件，而是提取重复的响应构建逻辑：

```rust
// 添加 helper 函数
fn success_response(action: &str, task: TaskInfo) -> ActionResponse { ... }
fn error_response(action: &str, error: &str) -> ActionResponse { ... }
fn task_not_found_response(action: &str, name: &str) -> ActionResponse { ... }
```

### 方案 B：按 action 拆分（如果方案 A 效果不明显）

```
src/commands/status/actions/
├── mod.rs           # execute_action 入口
├── common.rs        # 公共响应构建
├── state.rs         # done, merged, archive
├── navigation.rs    # enter, tail
└── query.rs         # list
```

## 评估标准

- 如果提取 helper 后代码减少 50+ 行，方案 A 足够
- 否则考虑方案 B

## 验证

```bash
cargo test
cargo clippy
```

## 参考

- `src/commands/status/actions.rs` - 当前实现
- 观察每个 `handle_*_action` 函数的重复模式
