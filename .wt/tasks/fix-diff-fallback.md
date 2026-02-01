---
name: fix-diff-fallback
---

# 修复 get_diff_stats fallback 逻辑

> **开始前**：请先阅读 `REFACTOR_CONTEXT.md` 了解整体背景。

## 问题

`src/services/git.rs` 中的 `get_diff_stats` 函数 fallback 逻辑有 bug：

当 worktree 有未提交的改动但没有 commit 时：
1. `git diff --shortstat main...HEAD` 成功执行但返回空字符串
2. `parse_diff_stats` 正确返回 `None`
3. **但是这个 `None` 直接作为函数返回值了，没有 fallback 到 `diff HEAD`**

```rust
// 当前代码（有 bug）
if let Ok(stdout) = output {
    parse_diff_stats(&stdout)  // 返回 None，直接作为函数返回值！
} else {
    // fallback 只在命令失败时触发
}
```

## 解决方案

修改 `get_diff_stats` 函数，当 `parse_diff_stats` 返回 `None` 时也触发 fallback：

```rust
pub fn get_diff_stats(worktree_path: &str) -> Option<(i32, i32)> {
    let base = get_default_branch(worktree_path).unwrap_or_else(|| "main".to_string());

    // Try committed changes first (main...HEAD)
    let output = CommandRunner::new("git")
        .current_dir(worktree_path)
        .output(&["diff", "--shortstat", &format!("{}...HEAD", base)]);

    if let Ok(stdout) = output {
        if let Some(stats) = parse_diff_stats(&stdout) {
            return Some(stats);  // 有已提交的改动
        }
    }

    // Fallback: show uncommitted changes (diff HEAD)
    let output = CommandRunner::new("git")
        .current_dir(worktree_path)
        .output(&["diff", "--shortstat", "HEAD"]);
    output.ok().and_then(|s| parse_diff_stats(&s))
}
```

## 任务

1. 修改 `src/services/git.rs` 中的 `get_diff_stats` 函数
2. 关键改动：将 `if let Ok` 改为提前 return，让 fallback 逻辑总是执行

## 验证

```bash
cargo test
cargo clippy
```

手动测试：
```bash
# 在某个 worktree 中
cd .wt/worktrees/xxx
echo "test" >> README.md  # 添加未提交改动
cd ../../..
wt status --json | jq '.tasks[0] | {additions, deletions}'
# 应该显示 additions: 1，而不是 null
```

## 参考

- `src/services/git.rs:73-90` - get_diff_stats 函数
- `src/services/git.rs:117-135` - parse_diff_stats 函数
