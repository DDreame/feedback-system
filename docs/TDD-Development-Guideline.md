# FeedbackHub 开发规范

**版本：** v1.0  
**日期：** 2026年2月21日

---

## 核心开发原则

### 1. TDD 测试驱动开发

**强制要求：** 所有功能开发必须遵循红-绿-重构循环。

```
1. 编写失败的测试 (Red)    → 测试未实现的功能
2. 编写最小代码使其通过 (Green)  → 只写刚好能让测试通过代码
3. 重构代码 (Refactor)      → 改善代码质量，保持测试通过
```

**工作流程：**
```
[写测试] → [运行测试，失败] → [写实现] → [运行测试，通过] → [重构] → [下一轮]
```

### 2. 测试覆盖率要求

| 模块 | 最低覆盖率 |
|------|-----------|
| 核心业务逻辑 | 90% |
| API 接口 | 80% |
| 数据存储层 | 85% |
| 工具函数 | 100% |

### 3. 测试通过标准

- **所有测试必须通过** 才能合并代码
- **不允许** 跳过测试或注释测试
- **不允许** 提交失败的测试
- CI/CD 流水线必须 100% 通过才能部署

### 4. 测试真实性要求

**禁止行为：**
- ❌ 禁止使用 `#[ignore]` 跳过测试
- ❌ 禁止使用 `unimplemented!()` 或 `todo!()` 虚设返回
- ❌ 禁止使用 `panic!()` 模拟失败
- ❌ 禁止硬编码断言 `assert!(true)`
- ❌ 禁止使用条件编译 `#[cfg(test)]` 绕过正常逻辑

**必须要求：**
- ✅ 测试必须真实调用被测代码
- ✅ 断言必须基于实际运行结果
- ✅ 边界条件必须真实验证
- ✅ 错误处理必须真实触发和验证

```rust
// ❌ 错误示例 - 虚拟行为
#[test]
fn test_feedback_create() {
    // 直接返回成功，没有验证任何逻辑
    assert!(true);
}

#[test]
#[ignore] // 禁止跳过
fn test_complex_scenario() {
    todo!();
}

// ✅ 正确示例 - 真实验证
#[test]
fn test_feedback_create_with_valid_input() {
    // 真实创建对象
    let feedback = Feedback::new(
        "app_id".to_string(),
        "user_id".to_string(),
        FeedbackType::Problem,
        "标题".to_string(),
        "内容".to_string(),
    );
    
    // 真实验证状态
    assert_eq!(feedback.status, FeedbackStatus::New);
    
    // 验证生成的ID
    assert!(!feedback.id.is_empty());
}

#[test]
fn test_feedback_create_with_empty_title_should_fail() {
    // 真实调用，可能返回 Result
    let result = Feedback::new(
        "app_id".to_string(),
        "user_id".to_string(),
        FeedbackType::Problem,
        "".to_string(),  // 空标题
        "内容".to_string(),
    );
    
    // 验证错误情况
    assert!(result.is_err());
}
```

### 5. 任务拆分原则

**适用场景：** 单个任务工作量过大，难以在一个迭代内完成。

**拆分规则：**
- 按功能边界拆分（创建 → 查询 → 更新 → 删除）
- 按测试粒度拆分（单元 → 集成 → E2E）
- 每个子任务必须可独立测试和交付

**拆分示例：**

| 原任务 | 拆分为 | 优先级 |
|--------|--------|--------|
| 实现反馈模块 | 1. Feedback 数据模型 + 单元测试 | 高 |
| | 2. Feedback CRUD 接口 + 集成测试 | 高 |
| | 3. Feedback 状态流转 + E2E | 中 |

**完成标准：** 每个子任务独立测试通过后视为完成。

### 6. 测试优先原则

**执行顺序：**

```
1. 编写测试 (Test First)
       ↓
2. 运行测试 (验证失败)
       ↓
3. 编写实现代码
       ↓
4. 运行测试 (验证通过)
       ↓
5. 重构代码
       ↓
6. 运行测试 (验证重构后仍通过)
```

**时间分配：**
- 测试编写时间应占任务总时间的 30%-50%
- 禁止在测试未通过的情况下提交实现代码
- 修复 bug 必须先写复现测试

### 7. 环境问题自主解决

**原则：** 环境问题不应成为测试延迟的理由。

**必须自行解决的问题：**

| 问题类型 | 解决方案 |
|----------|----------|
| 依赖缺失 | 安装/配置开发环境 |
| 工具链问题 | 更新/重新安装工具 |
| 端口占用 | 查找并释放端口 |
| 权限问题 | 申请权限或使用替代方案 |
| 网络问题 | 配置代理或离线模式 |
| 构建失败 | 清理缓存、重新构建 |

**禁止行为：**
- ❌ 禁止以"环境问题"为由跳过测试
- ❌ 禁止提交带有已知环境问题的代码
- ❌ 禁止绕过环境检查

**排查流程：**

```bash
# Step 1: 诊断问题
cargo build 2>&1 | head -50

# Step 2: 清理重建
cargo clean
cargo update
cargo build

# Step 3: 检查依赖
cargo tree
cargo outdated

# Step 4: 如需帮助，准备以下信息
# - 操作系统版本
# - 工具链版本 (rustc --version)
# - 完整错误信息
# - 已尝试的解决方案
```

---

## 测试环境配置

### Rust 后端

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test feedback_service

# 显示测试输出
cargo test -- --nocapture

# 集成测试
cargo test --test integration
```

### Flutter App

```bash
# 运行所有测试
flutter test

# 运行特定测试文件
flutter test test/feedback_test.dart

# 显示详细输出
flutter test --reporter expanded

# 代码覆盖率
flutter test --coverage
```

### JavaScript SDK

```bash
# 运行测试
npm test

# 监视模式
npm test -- --watch

# 覆盖率
npm test -- --coverage
```

---

## 测试分类

### 单元测试 (Unit Tests)

- 测试单个函数、方法、类的行为
- 不依赖外部服务（数据库、网络）
- 执行速度快（毫秒级）

```rust
// Rust 示例
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feedback_creation() {
        let feedback = Feedback::new(
            "app_id".to_string(),
            "user_id".to_string(),
            FeedbackType::Problem,
            "标题".to_string(),
            "内容".to_string(),
        );
        
        assert_eq!(feedback.status, FeedbackStatus::New);
        assert_eq!(feedback.created_at.timestamp(), now());
    }
}
```

### 集成测试 (Integration Tests)

- 测试多个组件协作
- 可能依赖数据库、缓存
- 执行速度较慢（秒级）

```rust
// Rust 集成测试示例
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_and_retrieve_feedback() {
        let db = setup_test_database().await;
        
        // 创建反馈
        let feedback = feedback_service::create(&db, CreateFeedbackRequest {
            app_id: "app_123".to_string(),
            user_id: "user_456".to_string(),
            title: "测试反馈".to_string(),
            content: "测试内容".to_string(),
            feedback_type: "problem".to_string(),
        }).await.unwrap();
        
        // 查询反馈
        let retrieved = feedback_service::get_by_id(&db, &feedback.id).await.unwrap();
        
        assert_eq!(retrieved.title, "测试反馈");
    }
}
```

### 端到端测试 (E2E Tests)

- 模拟真实用户操作
- 完整系统流程验证
- 执行速度最慢（分钟级）

```typescript
// E2E 测试示例
import { test, expect } from '@playwright/test';

test('用户提交反馈流程', async ({ page }) => {
  // 打开嵌入SDK的应用
  await page.goto('http://localhost:3000');
  
  // 点击反馈按钮
  await page.click('[data-testid=feedback-button]');
  
  // 填写反馈表单
  await page.fill('[data-testid=feedback-title]', '测试标题');
  await page.fill('[data-testid=feedback-content]', '测试内容');
  await page.selectOption('[data-testid=feedback-type]', 'problem');
  
  // 提交
  await page.click('[data-testid=submit-button]');
  
  // 验证成功提示
  await expect(page.locator('.success-message')).toBeVisible();
});
```

---

## 测试命名规范

### 命名模式

```
test_功能_场景_预期结果
test_模块_方法_边界条件
```

### 示例

```rust
// 正确
#[test]
fn test_feedback_create_with_valid_input() {}

#[test]
fn test_feedback_create_with_empty_title_should_fail() {}

#[test]
fn test_message_send_to_offline_user_should_queue() {}

// 错误
#[test]
fn test_create() {}           // 太模糊
#[test]
fn test_feedback() {}         // 不明确
#[test]
fn test_case1() {}            // 无意义
```

---

## 测试断言规范

### 必须包含有意义的断言消息

```rust
// 好的断言
assert_eq!(
    feedback.status,
    FeedbackStatus::New,
    "新创建的反馈状态应为 New，实际为: {:?}",
    feedback.status
);

// 避免
assert_eq!(feedback.status, FeedbackStatus::New);
```

### 常用断言方法

| 方法 | 用途 |
|------|------|
| `assert_eq!(a, b)` | 断言相等 |
| `assert_ne!(a, b)` | 断言不等 |
| `assert!(condition)` | 断言为真 |
| `assert!(!condition)` | 断言为假 |
| `assert_matches!(pattern, value)` | 断言匹配模式 |
| `panic!("错误信息")` | 显式失败 |

---

## Mock 和 Stub

### 使用 Mock 库

```rust
// Rust 使用 mockall
use mockall::automock;

#[derive(Automock)]
trait FeedbackRepository {
    fn create(&self, feedback: Feedback) -> Result<Feedback, Error>;
    fn find_by_id(&self, id: &str) -> Result<Option<Feedback>, Error>;
}

// 测试中使用
mod tests {
    use super::*;
    
    #[test]
    fn test_feedback_service_create() {
        // Arrange
        let mut mock_repo = MockFeedbackRepository::new();
        mock_repo
            .expect_create()
            .returning(|_| Ok(Feedback::default()));
            
        let service = FeedbackService::new(mock_repo);
        
        // Act & Assert
        let result = service.create(CreateRequest::default()).unwrap();
        assert!(result.id.is_some());
    }
}
```

### 避免的 Mock 过度

```
✓ Mock 外部依赖（数据库、HTTP）
✓ Mock 不稳定组件（时间、随机数）
✗ Mock 业务逻辑
✗ Mock 内部实现细节
```

---

## 持续集成要求

### GitHub Actions 配置

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
      
      - name: Run tests
        run: cargo test --all-features --all-targets
      
      - name: Run Clippy
        run: cargo clippy -- -D warnings
      
      - name: Check formatting
        run: cargo fmt -- --check
```

### 通过标准

```
✅ cargo test           → 100% 通过
✅ cargo clippy         → 0 warnings
✅ cargo fmt --check   → 格式正确
✅ 覆盖率检查          → 达标
```

---

## 测试环境问题排查

### 常见问题及解决方案

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| 测试超时 | 异步操作未等待 | 使用 `tokio::test` + `await` |
| 数据库连接失败 | 未初始化测试数据库 | 使用测试容器或内存数据库 |
| 环境变量未设置 | 配置缺失 | 使用 `.env.test` 文件 |
| 依赖冲突 | Cargo.lock 过期 | 运行 `cargo update` |
| 权限不足 | 文件/目录权限问题 | 检查并修复权限 |

### 自助排查命令

```bash
# Rust
cargo clean && cargo test        # 清理后重试
cargo update                     # 更新依赖
RUST_BACKTRACE=1 cargo test     # 详细错误信息

# Flutter
flutter clean && flutter test   # 清理缓存
flutter pub get                 # 获取依赖

# Node.js
rm -rf node_modules && npm install  # 重新安装
```

---

## 任务完成标准

```
✓ 测试编写完成（红）
✓ 测试通过（绿）
✓ 代码重构完成
✓ 提交信息包含 [TEST] 前缀
✓ 代码审查通过
✓ CI/CD 流水线全部通过
```

**注意：** 测试不通过 = 任务未完成

---

## 快速参考

```bash
# Rust 测试命令
cargo test              # 运行所有测试
cargo test --lib       # 只运行库测试
cargo test --test '*'  # 只运行集成测试
cargo test -- --nocapture  # 显示 println! 输出

# Flutter 测试命令
flutter test           # 运行所有测试
flutter test path/to/test.dart  # 运行单个文件
flutter test --coverage  # 生成覆盖率报告

# JavaScript 测试命令
npm test               # 运行所有测试
npm test -- --coverage # 覆盖率
```

---

*本规范为 FeedbackHub 项目强制执行标准*
