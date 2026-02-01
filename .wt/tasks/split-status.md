---
name: split-status
depends:
- git-metrics
- auto-mark-done
---

# 拆分 commands/status.rs

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

`src/commands/status.rs` 有 800+ 行，包含过多职责：
1. TUI 模式处理
2. JSON 输出处理
3. 状态显示逻辑
4. Action API 执行（done、merged、archive、enter、tail）
5. 多个数据结构定义

这违反了 SRP（单一职责原则）。

## 任务

### 1. 创建新的模块结构

```
src/commands/status/
├── mod.rs          # 主入口，run() 函数
├── types.rs        # 数据结构 (TaskMetrics, StatusOutput, ActionResponse 等)
├── display.rs      # display_status() 等显示逻辑
└── actions.rs      # Action API (execute_action, handle_*_action)
```

### 2. 迁移内容

**types.rs**:
- `TaskMetrics`
- `StatusOutput`
- `TaskSummary`
- `ActionResponse`
- `ActionCommand`
- 相关的序列化实现

**display.rs**:
- `display_status()`
- 辅助函数

**actions.rs**:
- `execute_action()`
- `handle_done_action()`
- `handle_merged_action()`
- `handle_archive_action()`
- `handle_enter_action()`
- `handle_tail_action()`
- `handle_list_action()`

**mod.rs**:
- `run()` 入口函数
- 模块声明和导出

### 3. 更新导入

确保 `src/commands/mod.rs` 正确导出 `status` 模块。

## 验证

```bash
cargo test
cargo clippy
```

## 注意

- 保持公开 API 不变（`status::run()` 签名不变）
- 利用前置任务的改动（`git-metrics`、`auto-mark-done`）简化代码
- 如果某些结构体只在 status 内部使用，保持 `pub(crate)` 可见性

## 参考

- `src/commands/status.rs` - 当前实现
- `src/commands/` 下其他命令的模块结构
- 前置任务的改动
