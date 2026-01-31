---
paths:
  - "src/commands/**/*.rs"
---

# CLI 命令实现规范

## 命令结构

每个命令一个文件，位于 `src/commands/`：

```rust
// src/commands/example.rs
use crate::error::{Result, WtError};
use crate::models::TaskStore;

pub fn execute(arg: String) -> Result<()> {
    // 1. 加载数据
    let store = TaskStore::load()?;

    // 2. 业务逻辑
    let task = store.get(&arg)
        .ok_or_else(|| WtError::TaskNotFound(arg.clone()))?;

    // 3. 输出结果
    println!("Task: {}", task.name());

    Ok(())
}
```

## 添加新命令步骤

1. **定义 CLI 参数** - `src/cli.rs`

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ...existing...

    /// 新命令的描述
    NewCommand {
        /// 参数描述
        #[arg(long)]
        arg: String,
    },
}
```

2. **实现命令** - `src/commands/new_command.rs`

3. **注册模块** - `src/commands/mod.rs`

```rust
pub mod new_command;
```

4. **路由命令** - `src/main.rs`

```rust
Commands::NewCommand { arg } => commands::new_command::execute(arg),
```

5. **添加测试** - `tests/cli/<command>.rs`

## 输出规范

| 场景 | 输出位置 | 格式 |
|------|----------|------|
| 成功信息 | stdout | `Task 'name' created.` |
| 提示信息 | stdout | `  File: path/to/file` |
| 警告 | stdout | `Warning: message` |
| 错误 | stderr | `Error: message` |

## 错误处理

- 返回 `Result<()>`，让 `main.rs` 统一处理错误输出
- 使用 `WtError` 枚举，不要直接 `eprintln!`
- 错误消息说明问题和解决方法

```rust
// Good
Err(WtError::TaskNotFound(name))

// Bad
eprintln!("Error: task not found");
std::process::exit(1);
```
