# Feedback System - 开发规范文档

## 1. 开发原则

### 1.1 TDD (测试驱动开发)

本项目严格遵循 TDD 开发流程，所有功能代码必须先有失败的测试，再编写实现代码。

**Red-Green-Refactor 循环：**

1. **RED（红）**：编写一个会失败的测试，明确定义预期行为
2. **GREEN（绿）**：编写最少量的代码让测试通过
3. **REFACTOR（重构）**：在测试保护下优化代码质量，确保测试仍然通过

**规则：**
- 禁止在没有对应测试的情况下编写业务代码
- 每个测试只测一个行为
- 测试必须独立，不依赖执行顺序
- 使用有意义的测试名称，描述被测行为和预期结果

### 1.2 提交规范

每完成一个最小功能单元（通常是一个 TDD 循环）就执行一次 commit。

**Commit Message 格式：**

```
<type>(<scope>): <subject>

[optional body]
```

**Type：**
| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `test` | 添加或修改测试 |
| `refactor` | 重构（不改变外部行为） |
| `docs` | 文档变更 |
| `chore` | 构建/工具/依赖变更 |
| `ci` | CI/CD 配置变更 |

**Scope（可选）：**
`backend`, `web-sdk`, `flutter-sdk`, `admin-panel`, `test-page`, `docs`, `db`

**示例：**
```
test(backend): add failing test for developer registration
feat(backend): implement developer registration endpoint
refactor(backend): extract password hashing into utility
```

### 1.3 分支策略

```
main                    # 稳定分支，通过所有测试
├── develop             # 开发集成分支
│   ├── feat/phase1-*   # Phase 1 功能分支
│   ├── feat/phase2-*   # Phase 2 功能分支
│   └── fix/*           # Bug 修复分支
```

## 2. 技术规范

### 2.1 后端 (Rust)

**测试框架：**
- 单元测试：Rust 内置 `#[cfg(test)]` + `#[test]`
- 集成测试：`tests/` 目录
- HTTP 测试：`axum::test` / `tower::ServiceExt`
- 数据库测试：使用事务回滚或独立测试数据库

**代码风格：**
- 使用 `rustfmt` 格式化
- 使用 `clippy` 检查
- 错误处理统一使用 `thiserror` + 自定义错误类型
- 异步函数使用 `async/await`

**测试命令：**
```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --lib service::auth

# 运行集成测试
cargo test --test integration_tests
```

### 2.2 前端 (TypeScript / React)

**测试框架：**
- 单元测试：Vitest
- 组件测试：React Testing Library
- E2E 测试：Playwright（后期引入）

**代码风格：**
- ESLint + Prettier
- 严格 TypeScript（`strict: true`）
- 函数组件 + Hooks
- 状态管理：Zustand 或 React Context

**测试命令：**
```bash
# 运行所有测试
pnpm test

# 运行带覆盖率
pnpm test -- --coverage

# 监听模式
pnpm test -- --watch
```

### 2.3 Web SDK (TypeScript)

**测试框架：**
- 单元测试：Vitest
- DOM 测试：jsdom / happy-dom
- WebSocket 测试：Mock WebSocket

**打包工具：** Vite (library mode) 或 tsup

### 2.4 Flutter SDK (Dart)

**测试框架：**
- 单元测试：`flutter_test`
- Widget 测试：`flutter_test`

**测试命令：**
```bash
flutter test
```

## 3. 目录结构规范

测试文件放置原则：

```
# Rust - 单元测试在源文件内，集成测试在 tests/ 目录
backend/
├── src/
│   └── service/
│       └── auth.rs          # 包含 #[cfg(test)] mod tests
└── tests/
    └── api/
        └── auth_test.rs     # 集成测试

# TypeScript - 测试文件与源文件同目录
admin-panel/
└── src/
    └── components/
        ├── ChatWindow.tsx
        └── ChatWindow.test.tsx

# Web SDK - 同上
sdks/web/
└── src/
    ├── core/
    │   ├── connection.ts
    │   └── connection.test.ts
```

## 4. 数据库规范

- 使用 sqlx 迁移管理数据库变更
- 每个迁移文件有 up/down
- 迁移文件命名：`YYYYMMDDHHMMSS_description.sql`
- 测试使用独立数据库或事务回滚

## 5. CI/CD 要求

每次 push 触发：
1. 代码格式检查（`rustfmt`, `eslint`, `prettier`）
2. 静态分析（`clippy`）
3. 全量测试（所有组件）
4. 测试覆盖率报告

## 6. 环境配置

```bash
# 开发环境依赖
- Rust >= 1.75 (stable)
- Node.js >= 20 LTS
- pnpm >= 9
- PostgreSQL >= 15
- Redis >= 7
- Docker + Docker Compose
- Flutter >= 3.16 (Phase 4)
```

## 7. 开发流程检查清单

每个功能开发前确认：
- [ ] 明确需求和验收标准
- [ ] 设计测试用例
- [ ] 编写失败测试（RED）
- [ ] 实现最小代码（GREEN）
- [ ] 重构优化（REFACTOR）
- [ ] 确保所有测试通过
- [ ] Commit（附带有意义的消息）
- [ ] 必要时更新文档
