# 测试指南

## 测试结构

```
src/
├── commands/
│   └── init.rs      # 包含 #[cfg(test)] mod tests
├── models/
│   ├── task.rs      # 包含 #[cfg(test)] mod tests
│   ├── store.rs     # 包含 #[cfg(test)] mod tests
│   └── config.rs    # 包含 #[cfg(test)] mod tests
├── services/
│   ├── command.rs   # 包含 #[cfg(test)] mod tests
│   └── workspace.rs # 包含 #[cfg(test)] mod tests
tests/
├── integration/          # 集成测试（解析、验证逻辑）
├── cli/                  # CLI E2E 测试（每个命令一个文件）
│   ├── init.rs
│   ├── create.rs
│   ├── list.rs
│   └── ...
├── cli.rs                # CLI 测试入口
└── integration.rs        # 集成测试入口
```

## 测试分类

| 类型 | 位置 | 特点 |
|------|------|------|
| 单元测试 | `src/**/*.rs` 内 `#[cfg(test)]` | 测试单个函数，快速 |
| 集成测试 | `tests/integration/` | 测试模块协作 |
| CLI/E2E | `tests/cli/` | 运行真实二进制，创建临时 git 仓库 |

## 运行测试

```bash
cargo test                    # 全部测试
cargo test --lib              # 仅单元测试
cargo test --test cli         # 仅 CLI 测试
cargo test --test cli init    # 仅 init 命令测试
cargo test test_name          # 运行特定测试
```

## 编写新测试

### 单元测试模板

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_功能_场景_预期() {
        // Arrange
        let input = ...;

        // Act
        let result = function(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

### CLI 测试模板

```rust
#[test]
fn test_command_scenario() {
    let dir = setup_test_repo();  // 创建临时 git 仓库

    let (ok, stdout, stderr) = run_wt(dir.path(), &["command", "args"]);

    assert!(ok);
    assert!(stdout.contains("expected output"));
}
```

## 测试覆盖要点

### 必须测试

- 所有公开 API 的正常路径
- 所有错误类型至少一个触发场景
- 边界条件（空输入、超长输入、特殊字符）

### 当前覆盖

- 任务名验证：空、空格、特殊字符、非法开头/结尾
- Markdown 解析：缺少 frontmatter、无效 YAML、Unicode
- 循环依赖：自引用、简单循环、长链循环、菱形图
- 状态转换：TaskStatus.can_transition_to 合法/非法转换
- 命令执行：CommandRunner 成功/失败场景
- 工作空间初始化：文件复制、脚本执行、prompt 文件写入
- CLI 命令：所有 10 个命令的正常和错误场景
