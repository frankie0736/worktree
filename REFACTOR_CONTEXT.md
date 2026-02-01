# 重构背景文档

> **重要**：每个 agent 开始工作前，请先阅读此文档了解整体背景。

## 项目概述

wt 是一个多 agent 并行开发任务管理 CLI 工具，使用 Rust 编写。

## 代码结构

```
src/
├── main.rs           # CLI 入口
├── lib.rs            # 库导出
├── cli.rs            # Clap 命令定义
├── constants.rs      # 路径常量
├── display.rs        # 显示格式化
├── error.rs          # WtError 错误类型
├── models/
│   ├── task.rs       # Task, TaskStatus, Instance
│   ├── status.rs     # StatusStore, TaskState
│   ├── store.rs      # TaskStore
│   └── config.rs     # WtConfig
├── commands/         # 各子命令实现
├── services/
│   ├── git.rs        # git 操作
│   ├── tmux.rs       # tmux 操作
│   ├── transcript.rs # Claude transcript 解析
│   └── ...
└── tui/              # TUI 界面
```

## 重构目标

本次重构主要解决以下问题：

### 1. DRY（不重复原则）

多处代码存在重复模式：
- 任务存在性检查
- transcript 路径查找
- tmux 窗口管理
- git metrics 获取
- auto-mark-done 逻辑

### 2. SSOT（单一事实来源）

分支名模式 `wt/{task}-*` 硬编码在多处。

### 3. 错误处理

个别 `unwrap()` 需要改为安全的错误处理。

### 4. SRP（单一职责）

`commands/status.rs` 文件过大（800+ 行），需要拆分。

## 重构原则

1. **小步前进**：每个任务只做一件事
2. **测试优先**：改完立即 `cargo test`
3. **保持兼容**：不改变公开 API 行为
4. **文档同步**：如有必要更新注释

## 验证方式

每个任务完成后执行：
```bash
cargo test
cargo clippy
```

## 编码规范

参考 `.claude/rules/rust-style.md`：
- 使用 `thiserror` 定义错误
- 函数返回 `Result<T>`
- 使用 `?` 传播错误，避免 `.unwrap()`
