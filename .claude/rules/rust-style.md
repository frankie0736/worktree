# Rust 编码规范

## 错误处理

使用 `thiserror` 定义错误类型，统一在 `src/error.rs`：

```rust
#[derive(Error, Debug)]
pub enum WtError {
    #[error("Task '{0}' not found")]
    TaskNotFound(String),
    // ...
}

pub type Result<T> = std::result::Result<T, WtError>;
```

- 所有公开函数返回 `Result<T>`
- 错误消息面向用户，清晰说明问题和解决方法
- 使用 `?` 传播错误，避免 `.unwrap()`

## 模块组织

```
src/
├── models/     # 数据结构和业务逻辑
├── commands/   # CLI 子命令（薄层，调用 models/services）
└── services/   # 外部依赖封装（git, tmux）
```

- `commands/` 只做参数解析和输出格式化
- 业务逻辑放 `models/`
- 外部命令调用放 `services/`

## 命名约定

| 类型 | 约定 | 示例 |
|------|------|------|
| 结构体 | PascalCase | `TaskStore`, `WtConfig` |
| 函数/方法 | snake_case | `parse_markdown`, `validate_task_name` |
| 常量 | SCREAMING_SNAKE | `TASKS_DIR` |
| 模块 | snake_case | `task.rs`, `git.rs` |

## 依赖管理

- 尽量使用标准库
- 必要依赖：`clap`(CLI), `serde`(序列化), `thiserror`(错误)
- 测试依赖：`tempfile`

## 文档

- 公开 API 添加 `///` 文档注释
- 复杂逻辑添加 `//` 行内注释
- 不为显而易见的代码添加注释
