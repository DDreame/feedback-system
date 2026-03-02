# Feedback System - 技术架构文档

## 1. 系统概述

一个面向独立开发者的用户反馈系统。用户通过嵌入式 SDK 在 App/Web 中打开聊天窗口与开发者实时沟通；开发者通过管理面板收集反馈、管理 Issue、规划排期。系统内置 AI Agent 自动分流和解答常见问题。

## 2. 系统架构总览

```
┌─────────────────────────────────────────────────────────────────────┐
│                          用户侧 (User Side)                        │
│                                                                     │
│   ┌─────────────┐    ┌─────────────┐    ┌──────────────────────┐   │
│   │  Web SDK     │    │ Flutter SDK │    │  测试页面 (Test Page) │   │
│   │  (JS/TS)     │    │  (Dart)     │    │  (Web)               │   │
│   └──────┬───────┘    └──────┬──────┘    └──────────┬───────────┘   │
│          │                   │                      │               │
└──────────┼───────────────────┼──────────────────────┼───────────────┘
           │                   │                      │
           │        WebSocket / HTTP REST             │
           │                   │                      │
┌──────────▼───────────────────▼──────────────────────▼───────────────┐
│                        后端服务 (Rust)                               │
│                                                                     │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │
│   │ API 网关  │  │ 聊天服务  │  │ Issue    │  │  AI Agent 服务   │   │
│   │ Gateway   │  │ Chat     │  │ Service  │  │  (LLM 集成)      │   │
│   └──────────┘  └──────────┘  └──────────┘  └──────────────────┘   │
│                                                                     │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │
│   │ 认证服务  │  │ 项目管理  │  │ 日志收集  │  │  通知服务        │   │
│   │ Auth     │  │ Project  │  │ LogIngest│  │  Notification    │   │
│   └──────────┘  └──────────┘  └──────────┘  └──────────────────┘   │
│                                                                     │
└─────────────────────────┬───────────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────────────┐
│                     开发者侧 (Developer Side)                       │
│                                                                     │
│   ┌──────────────────────────────────────────────────────────────┐  │
│   │              管理面板 (Admin Dashboard - Web)                 │  │
│   │                                                              │  │
│   │   会话管理 │ Issue 管理 │ 排期规划 │ 项目设置 │ 数据统计      │  │
│   └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## 3. 技术选型

| 组件 | 技术 | 理由 |
|------|------|------|
| 后端 | Rust (Axum) | 高性能、内存安全、适合长连接 |
| 数据库 | PostgreSQL | 关系型数据、成熟稳定 |
| 缓存 | Redis | 会话状态、在线状态、消息队列 |
| 实时通信 | WebSocket | 聊天的实时消息推送 |
| Web SDK | TypeScript | 浏览器原生支持、类型安全 |
| Flutter SDK | Dart | 跨平台移动端 |
| 管理面板 | React + TypeScript | 组件生态丰富、适合复杂后台 |
| 测试页面 | React + TypeScript | 与管理面板共享组件 |
| AI Agent | LLM API (OpenAI/Claude 等) | 智能问答、问题分类 |
| 对象存储 | S3 兼容 (MinIO/云服务) | 聊天附件、日志文件 |
| 部署 | Docker + Docker Compose | 简化部署和运维 |

## 4. 核心模块设计

### 4.1 后端服务 (Rust)

后端采用单体服务 + 模块化的设计。对于独立开发者产品的规模，单体架构比微服务更合适——部署简单、维护成本低。

```
backend/
├── src/
│   ├── main.rs                # 入口、服务启动
│   ├── config.rs              # 配置管理
│   ├── lib.rs
│   │
│   ├── api/                   # HTTP/WebSocket 路由
│   │   ├── mod.rs
│   │   ├── auth.rs            # 认证相关 API
│   │   ├── chat.rs            # 聊天 API + WebSocket
│   │   ├── issue.rs           # Issue CRUD API
│   │   ├── project.rs         # 项目管理 API
│   │   ├── log_ingest.rs      # 日志上报 API
│   │   ├── schedule.rs        # 排期管理 API
│   │   └── admin.rs           # 管理面板专用 API
│   │
│   ├── service/               # 业务逻辑层
│   │   ├── mod.rs
│   │   ├── chat.rs
│   │   ├── issue.rs
│   │   ├── ai_agent.rs        # AI Agent 逻辑
│   │   ├── log_collector.rs
│   │   ├── notification.rs
│   │   └── schedule.rs
│   │
│   ├── model/                 # 数据模型
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── message.rs
│   │   ├── conversation.rs
│   │   ├── issue.rs
│   │   ├── project.rs
│   │   ├── log_entry.rs
│   │   └── schedule.rs
│   │
│   ├── db/                    # 数据库交互
│   │   ├── mod.rs
│   │   └── migrations/
│   │
│   ├── ws/                    # WebSocket 管理
│   │   ├── mod.rs
│   │   ├── handler.rs
│   │   └── session.rs
│   │
│   └── error.rs               # 统一错误处理
│
├── Cargo.toml
└── .env
```

**核心依赖：**
- `axum` - Web 框架
- `tokio` - 异步运行时
- `sqlx` - 数据库 (PostgreSQL)
- `redis` - 缓存和发布/订阅
- `serde` / `serde_json` - 序列化
- `jsonwebtoken` - JWT 认证
- `tower-http` - 中间件 (CORS、日志等)
- `uuid` - ID 生成
- `reqwest` - 调用外部 AI API

### 4.2 认证体系

系统有两类用户，认证方式不同：

```
终端用户 (App/Web User)
├── 匿名模式：SDK 初始化时生成 device_id，无需注册即可聊天
└── 已登录模式：开发者可通过 SDK 传入用户标识进行关联

开发者 (Developer)
├── 邮箱 + 密码注册/登录
├── JWT Token 认证
└── API Key 用于 SDK 接入
```

### 4.3 聊天系统

```
消息流转：

用户 SDK                    后端                      开发者管理面板
  │                          │                            │
  │── 建立 WebSocket ──────▶│                            │
  │                          │                            │
  │── 发送消息 ─────────────▶│── 持久化到 DB             │
  │                          │── AI Agent 预处理          │
  │                          │   ├── 能回答 → 直接回复    │
  │                          │   └── 不能回答 → 转人工 ──▶│ (通知开发者)
  │                          │                            │
  │◀── 接收 AI/人工回复 ────│◀── 开发者回复 ────────────│
  │                          │                            │
```

**消息类型：**
- `text` - 普通文本
- `image` - 图片 (截图)
- `log` - 客户端日志片段
- `system` - 系统消息 (Issue 状态变更通知等)

### 4.4 Issue 管理

```
会话 (Conversation)          Issue
  │                            │
  │  开发者手动创建  ─────────▶│
  │  或合并到已有 Issue ──────▶│
  │                            │
  │  关联日志上报              │── 标题
  │  关联截图附件              │── 描述 (从聊天摘要生成)
  │                            │── 状态: open / in_progress / resolved / closed
  │                            │── 优先级: low / medium / high / critical
  │                            │── 标签
  │                            │── 关联会话列表
  │                            │── 排期 (Milestone)
```

**从聊天到 Issue 的流程：**
1. 开发者在管理面板查看会话
2. 选择消息片段，一键创建 Issue（AI 可辅助生成标题和描述）
3. 或将当前会话合并到已有的 Issue
4. 关联的日志/附件自动归档到 Issue

### 4.5 排期管理

```
Project (项目)
  └── Milestone (里程碑/版本)
        └── Issue (问题)
              ├── 状态
              ├── 优先级
              └── 预计完成时间
```

开发者可以：
- 创建里程碑 (如 v1.2.0, Sprint 3)
- 将 Issue 拖拽/分配到某个里程碑
- 查看里程碑进度（已完成/总数）

### 4.6 日志收集

SDK 可采集客户端日志，与聊天会话关联：

```
SDK 端:
  - 自动采集: 最近 N 条 console 日志、崩溃信息
  - 手动上报: 开发者在 SDK 配置中自定义上报内容
  - 设备信息: OS、浏览器/App 版本、屏幕分辨率等

后端:
  - 接收日志 → 关联到当前会话
  - 结构化存储 (时间戳、级别、内容)
  - 开发者在管理面板查看关联日志
```

### 4.7 AI Agent

```
用户消息 ─▶ AI Agent 管线:

  1. 意图识别
     ├── 已知问题 → 查询 FAQ/知识库 → 直接回复
     ├── 已有排期 → "该问题已在 v1.3 修复计划中" → 回复
     ├── 简单咨询 → LLM 生成回复
     └── 复杂/未知 → 标记转人工

  2. 上下文增强
     ├── 注入项目的 FAQ 知识库
     ├── 注入已有 Issue 摘要
     └── 注入该用户历史会话摘要

  3. 回复策略
     ├── 高置信度 → 自动回复
     ├── 中置信度 → 回复 + 标记待开发者确认
     └── 低置信度 → 仅转交人工，不自动回复
```

## 5. 数据库设计

### 核心表结构

```sql
-- 开发者
developers
  id              UUID PRIMARY KEY
  email           VARCHAR UNIQUE NOT NULL
  password_hash   VARCHAR NOT NULL
  name            VARCHAR NOT NULL
  created_at      TIMESTAMPTZ

-- 项目
projects
  id              UUID PRIMARY KEY
  developer_id    UUID REFERENCES developers
  name            VARCHAR NOT NULL
  api_key         VARCHAR UNIQUE NOT NULL    -- SDK 接入密钥
  settings        JSONB                      -- 项目配置 (AI 开关、主题等)
  created_at      TIMESTAMPTZ

-- 终端用户
end_users
  id              UUID PRIMARY KEY
  project_id      UUID REFERENCES projects
  device_id       VARCHAR                    -- 匿名用户设备标识
  external_id     VARCHAR                    -- 开发者系统中的用户 ID
  metadata        JSONB                      -- 设备信息、自定义属性
  created_at      TIMESTAMPTZ
  UNIQUE(project_id, device_id)

-- 会话
conversations
  id              UUID PRIMARY KEY
  project_id      UUID REFERENCES projects
  end_user_id     UUID REFERENCES end_users
  status          VARCHAR DEFAULT 'open'     -- open / waiting / resolved / closed
  assigned_to     UUID REFERENCES developers -- 分配给哪个开发者 (多人团队扩展用)
  ai_enabled      BOOLEAN DEFAULT true
  created_at      TIMESTAMPTZ
  updated_at      TIMESTAMPTZ

-- 消息
messages
  id              UUID PRIMARY KEY
  conversation_id UUID REFERENCES conversations
  sender_type     VARCHAR NOT NULL           -- 'user' / 'developer' / 'ai'
  content         TEXT NOT NULL
  message_type    VARCHAR DEFAULT 'text'     -- text / image / log / system
  metadata        JSONB                      -- 附件 URL、额外信息
  created_at      TIMESTAMPTZ

-- Issue
issues
  id              UUID PRIMARY KEY
  project_id      UUID REFERENCES projects
  title           VARCHAR NOT NULL
  description     TEXT
  status          VARCHAR DEFAULT 'open'     -- open / in_progress / resolved / closed
  priority        VARCHAR DEFAULT 'medium'   -- low / medium / high / critical
  tags            TEXT[]
  milestone_id    UUID REFERENCES milestones
  created_at      TIMESTAMPTZ
  updated_at      TIMESTAMPTZ

-- Issue 与会话的关联 (多对多)
issue_conversations
  issue_id        UUID REFERENCES issues
  conversation_id UUID REFERENCES conversations
  PRIMARY KEY (issue_id, conversation_id)

-- 里程碑
milestones
  id              UUID PRIMARY KEY
  project_id      UUID REFERENCES projects
  title           VARCHAR NOT NULL
  description     TEXT
  due_date        DATE
  status          VARCHAR DEFAULT 'active'   -- active / completed / archived
  created_at      TIMESTAMPTZ

-- 客户端日志
client_logs
  id              UUID PRIMARY KEY
  conversation_id UUID REFERENCES conversations
  end_user_id     UUID REFERENCES end_users
  level           VARCHAR                    -- debug / info / warn / error
  content         TEXT
  device_info     JSONB
  created_at      TIMESTAMPTZ

-- AI 知识库 (FAQ)
knowledge_base
  id              UUID PRIMARY KEY
  project_id      UUID REFERENCES projects
  question        TEXT NOT NULL
  answer          TEXT NOT NULL
  embedding       VECTOR(1536)               -- 用于语义搜索 (pgvector)
  created_at      TIMESTAMPTZ
```

## 6. API 设计

### 6.1 SDK 端 API (终端用户使用)

```
POST   /api/v1/sdk/init                  # SDK 初始化，返回会话 token
WS     /api/v1/sdk/chat                  # WebSocket 聊天连接
POST   /api/v1/sdk/messages              # 发送消息 (HTTP 降级)
GET    /api/v1/sdk/messages              # 获取历史消息
POST   /api/v1/sdk/logs                  # 上报客户端日志
POST   /api/v1/sdk/attachments           # 上传附件 (截图等)
```

### 6.2 管理端 API (开发者使用)

```
# 认证
POST   /api/v1/auth/register             # 注册
POST   /api/v1/auth/login                # 登录
POST   /api/v1/auth/refresh              # 刷新 token

# 项目管理
GET    /api/v1/projects                  # 项目列表
POST   /api/v1/projects                  # 创建项目
PUT    /api/v1/projects/:id              # 更新项目
DELETE /api/v1/projects/:id              # 删除项目
POST   /api/v1/projects/:id/api-key      # 重新生成 API Key

# 会话管理
GET    /api/v1/projects/:id/conversations           # 会话列表
GET    /api/v1/conversations/:id                     # 会话详情
GET    /api/v1/conversations/:id/messages            # 消息列表
POST   /api/v1/conversations/:id/messages            # 发送回复
PUT    /api/v1/conversations/:id/status              # 更新会话状态
WS     /api/v1/admin/chat                            # 管理端 WebSocket

# Issue 管理
GET    /api/v1/projects/:id/issues                   # Issue 列表
POST   /api/v1/projects/:id/issues                   # 创建 Issue
PUT    /api/v1/issues/:id                            # 更新 Issue
DELETE /api/v1/issues/:id                            # 删除 Issue
POST   /api/v1/issues/:id/conversations              # 关联会话到 Issue
POST   /api/v1/issues/:id/merge                      # 合并 Issue

# 排期管理
GET    /api/v1/projects/:id/milestones               # 里程碑列表
POST   /api/v1/projects/:id/milestones               # 创建里程碑
PUT    /api/v1/milestones/:id                        # 更新里程碑
PUT    /api/v1/issues/:id/milestone                  # 分配 Issue 到里程碑

# 日志查看
GET    /api/v1/conversations/:id/logs                # 查看关联日志

# AI / 知识库
GET    /api/v1/projects/:id/knowledge                # 知识库列表
POST   /api/v1/projects/:id/knowledge                # 添加知识条目
PUT    /api/v1/knowledge/:id                         # 更新知识条目
DELETE /api/v1/knowledge/:id                         # 删除知识条目
```

## 7. SDK 设计

### 7.1 Web SDK (TypeScript)

```typescript
// 用户侧接入示例
import { FeedbackWidget } from '@feedback-system/web-sdk';

const widget = new FeedbackWidget({
  apiKey: 'proj_xxxxxxxxxxxx',
  // 可选: 关联已登录用户
  user: {
    id: 'user_123',
    name: '张三',
    email: 'zhang@example.com',
  },
  // 可选: 自定义日志采集
  logCollector: {
    enabled: true,
    maxEntries: 100,
    levels: ['warn', 'error'],
  },
  // 可选: 主题
  theme: {
    primaryColor: '#6366F1',
    position: 'bottom-right',
  },
});

widget.mount();
```

**SDK 核心功能：**
- 渲染悬浮按钮和聊天窗口 (Shadow DOM 隔离样式)
- WebSocket 连接管理（自动重连）
- 消息发送/接收/渲染
- 日志自动采集和手动上报
- 附件上传 (拖拽/粘贴截图)
- 本地消息缓存 (IndexedDB)

### 7.2 Flutter SDK (Dart)

```dart
// Flutter 接入示例
import 'package:feedback_system/feedback_system.dart';

FeedbackSystem.init(
  apiKey: 'proj_xxxxxxxxxxxx',
  user: FeedbackUser(
    id: 'user_123',
    name: '张三',
  ),
);

// 显示聊天窗口
FeedbackSystem.show();
```

**SDK 核心功能：**
- 提供 `FeedbackWidget` 可嵌入 Widget 树
- 提供 `FeedbackSystem.show()` 全局弹窗模式
- WebSocket 实时通信
- 日志采集 (对接 `dart:developer`)
- 图片/截图上传
- 离线消息队列

## 8. 管理面板设计

### 页面结构

```
管理面板 (Admin Dashboard)
│
├── 登录 / 注册
│
├── 项目选择 / 总览
│   └── 各项目的未读消息数、Issue 统计
│
├── 会话管理 (Inbox)
│   ├── 会话列表 (按状态筛选: 待处理 / 等待中 / 已解决)
│   ├── 会话详情 + 实时聊天
│   ├── 用户信息侧栏 (设备信息、历史会话)
│   └── 操作: 创建 Issue / 关联 Issue / 标记状态
│
├── Issue 管理
│   ├── 看板视图 (Kanban: Open → In Progress → Resolved)
│   ├── 列表视图 (支持筛选、排序、搜索)
│   ├── Issue 详情 (描述、关联会话、日志、评论)
│   └── 批量操作 (分配里程碑、修改状态等)
│
├── 排期管理
│   ├── 里程碑列表 (进度条展示)
│   ├── 里程碑详情 (包含的 Issue 列表)
│   └── 甘特图 / 时间线视图 (可选)
│
├── AI / 知识库
│   ├── FAQ 管理 (增删改查)
│   ├── AI 回复历史和效果统计
│   └── AI 行为配置 (自动回复阈值、工作时间等)
│
├── 数据统计
│   ├── 反馈趋势 (每日/周会话量)
│   ├── 响应时间统计
│   ├── Issue 分布 (按优先级、标签)
│   └── AI 解决率
│
└── 设置
    ├── 项目配置 (SDK 参数、API Key 管理)
    ├── 通知设置 (邮件、Webhook)
    ├── 团队成员管理 (多人扩展)
    └── 个人设置
```

## 9. 测试页面

独立的 Web 页面用于快速验证系统功能：

```
测试页面 (Test Page)
│
├── SDK 模拟器
│   ├── 模拟 Web SDK 聊天窗口
│   ├── 可切换不同用户身份
│   └── 手动触发日志上报
│
├── API 测试工具
│   ├── 各 API 端点的交互式测试
│   ├── 请求/响应查看
│   └── WebSocket 连接测试
│
├── AI Agent 测试
│   ├── 直接与 AI Agent 对话测试
│   ├── 查看意图识别结果
│   └── 测试知识库匹配
│
└── 系统状态
    ├── 后端健康检查
    ├── 数据库连接状态
    ├── Redis 连接状态
    └── WebSocket 连接数统计
```

## 10. 项目目录结构

```
feedback-system/
│
├── docs/                           # 文档
│   ├── architecture.md             # 本文档
│   ├── api-reference.md            # API 详细文档
│   └── deployment.md               # 部署文档
│
├── backend/                        # Rust 后端
│   ├── Cargo.toml
│   ├── src/
│   └── migrations/
│
├── sdks/
│   ├── web/                        # Web SDK (TypeScript)
│   │   ├── package.json
│   │   ├── src/
│   │   └── tsconfig.json
│   │
│   └── flutter/                    # Flutter SDK
│       ├── pubspec.yaml
│       └── lib/
│
├── admin-panel/                    # 管理面板 (React)
│   ├── package.json
│   ├── src/
│   └── tsconfig.json
│
├── test-page/                      # 测试页面 (React)
│   ├── package.json
│   └── src/
│
├── docker-compose.yml              # 本地开发环境
├── .gitignore
└── README.md
```

## 11. 开发阶段规划

### Phase 1: 核心基础 (MVP)
- [ ] 后端项目搭建、数据库设计和迁移
- [ ] 开发者注册/登录、项目创建、API Key 生成
- [ ] 基础聊天功能 (WebSocket 连接、消息收发、持久化)
- [ ] Web SDK 最小可用版本 (聊天窗口、消息发送/接收)
- [ ] 管理面板最小可用版本 (会话列表、实时聊天回复)
- [ ] 测试页面基础版

### Phase 2: Issue 与日志
- [ ] Issue CRUD 和状态管理
- [ ] 从会话创建/关联 Issue
- [ ] 客户端日志采集和查看
- [ ] 管理面板 Issue 看板
- [ ] 附件上传 (截图)

### Phase 3: 排期与AI
- [ ] 里程碑/排期管理
- [ ] Issue 分配到里程碑
- [ ] AI Agent 基础集成 (LLM 接入、意图识别)
- [ ] FAQ 知识库管理
- [ ] AI 自动回复

### Phase 4: 完善与扩展
- [ ] Flutter SDK
- [ ] 通知系统 (邮件/Webhook)
- [ ] 数据统计面板
- [ ] AI Agent 优化 (语义搜索、效果统计)
- [ ] 多成员协作支持
- [ ] 性能优化和安全加固

## 12. 关键设计决策

| 决策 | 选择 | 原因 |
|------|------|------|
| 架构模式 | 模块化单体 | 独立开发者场景，部署和维护简单 |
| 实时通信 | WebSocket | 聊天场景需要低延迟双向通信 |
| 消息持久化 | 先写 DB 再推送 | 保证消息不丢失 |
| SDK 样式隔离 | Shadow DOM (Web) | 不影响宿主页面样式 |
| AI 回复策略 | 置信度分级 | 避免 AI 给出错误回复影响用户体验 |
| ID 生成 | UUIDv7 | 有序、可排序、分布式安全 |
| 匿名用户 | device_id | 降低用户使用门槛，无需注册 |
