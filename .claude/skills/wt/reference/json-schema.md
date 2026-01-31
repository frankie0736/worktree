# wt create JSON 格式

## 完整格式

```json
{
  "name": "task-name",
  "depends": ["dep1", "dep2"],
  "description": "任务描述"
}
```

## 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | 任务名，用于文件名和引用 |
| `depends` | string[] | 否 | 依赖的任务名列表 |
| `description` | string | 是 | 任务描述，支持 Markdown |

## name 规则

- 只能包含：小写字母、数字、连字符
- 不能包含：空格、斜杠、特殊字符
- 建议：简短、有意义

```
✓ auth
✓ user-api
✓ setup-database
✗ User Auth（大写、空格）
✗ api/users（斜杠）
```

## depends 规则

- 必须是已存在的任务名
- 空数组 `[]` 表示无依赖
- 创建时自动验证

```json
// 无依赖
{"name": "auth", "depends": [], "description": "..."}

// 单个依赖
{"name": "api", "depends": ["auth"], "description": "..."}

// 多个依赖
{"name": "dashboard", "depends": ["auth", "database"], "description": "..."}
```

## description 最佳实践

```json
{
  "name": "user-auth",
  "depends": [],
  "description": "实现用户认证功能\n\n## 要求\n- 使用 NextAuth.js\n- 支持 GitHub OAuth\n- 支持邮箱密码登录\n\n## 验收标准\n- [ ] 登录页面\n- [ ] 注册页面\n- [ ] Session 管理\n- [ ] 保护路由中间件"
}
```

## 生成的文件

`wt create` 会生成 `.wt/tasks/{name}.md`：

```markdown
---
name: user-auth
depends: []
---

实现用户认证功能

## 要求
- 使用 NextAuth.js
- 支持 GitHub OAuth
- 支持邮箱密码登录

## 验收标准
- [ ] 登录页面
- [ ] 注册页面
- [ ] Session 管理
- [ ] 保护路由中间件
```

> 注：任务状态存储在 `.wt/status.json` 中，不在 markdown frontmatter 里。

## 错误处理

| 错误 | 原因 | 解决 |
|------|------|------|
| `Invalid JSON` | JSON 格式错误 | 检查引号、逗号 |
| `name cannot be empty` | name 为空 | 提供有效 name |
| `Task 'x' already exists` | 任务已存在 | 换个名字 |
| `Dependency 'x' not found` | 依赖不存在 | 先创建依赖任务 |
