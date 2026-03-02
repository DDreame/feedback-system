# Feedback System - TDD 开发计划

> 本计划严格遵循 TDD（测试驱动开发）原则。每个任务按 Red-Green-Refactor 循环执行，每完成一个任务执行一次 commit。

## 总览

| 阶段 | 名称 | 预估任务数 | 说明 |
|------|------|-----------|------|
| Phase 1 | 核心基础 (MVP) | ~45 | 后端基础、认证、聊天、Web SDK、管理面板 |
| Phase 2 | Issue 与日志 | ~25 | Issue CRUD、日志采集、附件上传 |
| Phase 3 | 排期与 AI | ~20 | 里程碑管理、AI Agent 集成 |
| Phase 4 | 完善与扩展 | ~20 | Flutter SDK、通知、统计、优化 |

---

## Phase 1：核心基础 (MVP)

### 1.1 项目初始化与基础设施

#### Task 1.1.1：初始化 Rust 后端项目
- **测试**：验证项目可以编译并运行 `cargo test`
- **实现**：`cargo init backend`，配置 `Cargo.toml` 基础依赖（axum, tokio, serde, sqlx）
- **Commit**：`chore(backend): initialize rust project with core dependencies`

#### Task 1.1.2：配置管理模块
- **RED**：测试从环境变量/文件加载配置（数据库 URL、服务端口、JWT 密钥等）
- **GREEN**：实现 `config.rs`，使用 `dotenvy` 加载 `.env`
- **REFACTOR**：提取配置结构体，添加默认值
- **Commit**：`feat(backend): add configuration management module`

#### Task 1.1.3：统一错误处理
- **RED**：测试自定义错误类型能正确转换为 HTTP 响应（400/401/404/500）
- **GREEN**：实现 `error.rs`，使用 `thiserror` 定义错误枚举，实现 `IntoResponse`
- **REFACTOR**：确保错误消息对 API 消费者友好
- **Commit**：`feat(backend): add unified error handling with proper HTTP status codes`

#### Task 1.1.4：数据库连接与连接池
- **RED**：测试能建立数据库连接并执行简单查询
- **GREEN**：配置 sqlx 连接池，添加到应用状态
- **REFACTOR**：提取数据库初始化逻辑
- **Commit**：`feat(backend): add PostgreSQL connection pool setup`

#### Task 1.1.5：HTTP 服务器基础框架
- **RED**：测试 `GET /health` 返回 200 和状态 JSON
- **GREEN**：搭建 Axum 路由，实现健康检查端点
- **REFACTOR**：添加 CORS、日志中间件
- **Commit**：`feat(backend): add HTTP server with health check endpoint`

#### Task 1.1.6：数据库迁移基础设施
- **RED**：测试迁移可以正确执行和回滚
- **GREEN**：配置 sqlx-cli 迁移，创建初始迁移文件
- **REFACTOR**：编写迁移运行脚本
- **Commit**：`chore(backend): setup database migration infrastructure`

#### Task 1.1.7：Docker 开发环境
- **测试**：验证 `docker-compose up` 可以启动 PostgreSQL 和 Redis
- **实现**：编写 `docker-compose.yml`，包含 postgres、redis 服务
- **Commit**：`chore: add docker-compose for local development environment`

---

### 1.2 认证系统

#### Task 1.2.1：开发者数据模型 - 数据库迁移
- **RED**：测试 `developers` 表存在且字段正确
- **GREEN**：创建迁移文件 `create_developers_table`
- **REFACTOR**：添加索引（email 唯一索引）
- **Commit**：`feat(db): add developers table migration`

#### Task 1.2.2：开发者数据模型 - Rust 结构体
- **RED**：测试 `Developer` 结构体可以从数据库行反序列化
- **GREEN**：定义 `model/user.rs` 中的 `Developer` 结构体
- **REFACTOR**：添加 `CreateDeveloper`、`DeveloperResponse` DTO
- **Commit**：`feat(backend): add Developer model and DTOs`

#### Task 1.2.3：密码哈希工具
- **RED**：测试密码可以被哈希，且哈希后可验证
- **GREEN**：使用 `argon2` 实现密码哈希和验证函数
- **REFACTOR**：错误处理、提取为独立工具模块
- **Commit**：`feat(backend): add password hashing utility with argon2`

#### Task 1.2.4：开发者注册 - Service 层
- **RED**：测试注册成功返回开发者信息；测试重复邮箱返回错误
- **GREEN**：实现 `service/auth.rs` 中的 `register` 函数
- **REFACTOR**：输入验证（邮箱格式、密码强度）
- **Commit**：`feat(backend): implement developer registration service`

#### Task 1.2.5：开发者注册 - API 端点
- **RED**：测试 `POST /api/v1/auth/register` 请求/响应格式
- **GREEN**：实现 `api/auth.rs` 中的注册路由处理
- **REFACTOR**：请求体验证，统一响应格式
- **Commit**：`feat(backend): add developer registration API endpoint`

#### Task 1.2.6：JWT Token 生成与验证
- **RED**：测试 token 生成包含正确 claims；测试过期 token 验证失败
- **GREEN**：使用 `jsonwebtoken` 实现 token 生成和验证
- **REFACTOR**：支持 access token 和 refresh token
- **Commit**：`feat(backend): implement JWT token generation and validation`

#### Task 1.2.7：开发者登录 - Service 层
- **RED**：测试正确凭证返回 token；测试错误密码返回错误
- **GREEN**：实现 `service/auth.rs` 中的 `login` 函数
- **REFACTOR**：统一认证错误信息（避免泄露用户是否存在）
- **Commit**：`feat(backend): implement developer login service`

#### Task 1.2.8：开发者登录 - API 端点
- **RED**：测试 `POST /api/v1/auth/login` 请求/响应格式
- **GREEN**：实现登录路由处理
- **REFACTOR**：添加 rate limiting 考虑
- **Commit**：`feat(backend): add developer login API endpoint`

#### Task 1.2.9：认证中间件
- **RED**：测试带有效 token 的请求通过；测试无/无效 token 返回 401
- **GREEN**：实现 Axum 认证中间件/提取器
- **REFACTOR**：支持从 Header 和 Query 中提取 token
- **Commit**：`feat(backend): add JWT authentication middleware`

#### Task 1.2.10：Token 刷新
- **RED**：测试有效 refresh token 返回新 access token
- **GREEN**：实现 `POST /api/v1/auth/refresh` 端点
- **REFACTOR**：refresh token 轮换策略
- **Commit**：`feat(backend): add token refresh endpoint`

---

### 1.3 项目管理

#### Task 1.3.1：项目数据模型 - 数据库迁移
- **RED**：测试 `projects` 表存在且外键关系正确
- **GREEN**：创建迁移文件 `create_projects_table`
- **REFACTOR**：添加索引（api_key 唯一索引）
- **Commit**：`feat(db): add projects table migration`

#### Task 1.3.2：项目数据模型 - Rust 结构体
- **RED**：测试 `Project` 结构体序列化/反序列化
- **GREEN**：定义 `model/project.rs`
- **REFACTOR**：添加 DTO（`CreateProject`, `ProjectResponse`）
- **Commit**：`feat(backend): add Project model and DTOs`

#### Task 1.3.3：API Key 生成
- **RED**：测试生成的 API Key 格式正确（`proj_` 前缀 + 随机串）、唯一
- **GREEN**：实现 API Key 生成函数
- **REFACTOR**：密码学安全随机数
- **Commit**：`feat(backend): implement API key generation`

#### Task 1.3.4：项目 CRUD - Service 层
- **RED**：测试创建/读取/更新/删除项目；测试权限（只能操作自己的项目）
- **GREEN**：实现 `service/project.rs`
- **REFACTOR**：提取分页查询逻辑
- **Commit**：`feat(backend): implement project CRUD service`

#### Task 1.3.5：项目 CRUD - API 端点
- **RED**：测试所有项目 API 端点的请求/响应
- **GREEN**：实现 `api/project.rs` 路由
- **REFACTOR**：统一分页响应格式
- **Commit**：`feat(backend): add project CRUD API endpoints`

#### Task 1.3.6：API Key 重新生成
- **RED**：测试重新生成后旧 Key 失效，新 Key 可用
- **GREEN**：实现 `POST /api/v1/projects/:id/api-key`
- **REFACTOR**：添加确认机制
- **Commit**：`feat(backend): add API key regeneration endpoint`

#### Task 1.3.7：SDK 认证中间件（API Key 验证）
- **RED**：测试有效 API Key 通过；测试无效/过期 Key 返回 401
- **GREEN**：实现 API Key 认证提取器
- **REFACTOR**：添加 Redis 缓存加速验证
- **Commit**：`feat(backend): add API key authentication for SDK endpoints`

---

### 1.4 终端用户与会话

#### Task 1.4.1：终端用户和会话 - 数据库迁移
- **RED**：测试 `end_users` 和 `conversations` 表存在
- **GREEN**：创建迁移文件
- **REFACTOR**：添加复合唯一约束和索引
- **Commit**：`feat(db): add end_users and conversations table migrations`

#### Task 1.4.2：终端用户和会话 - 数据模型
- **RED**：测试模型序列化/反序列化
- **GREEN**：定义 `model/user.rs`（EndUser）和 `model/conversation.rs`
- **REFACTOR**：添加 DTO
- **Commit**：`feat(backend): add EndUser and Conversation models`

#### Task 1.4.3：SDK 初始化端点
- **RED**：测试 `POST /api/v1/sdk/init` 创建或恢复终端用户和会话
- **GREEN**：实现 SDK 初始化逻辑（device_id 匹配或创建新用户）
- **REFACTOR**：处理匿名/已登录两种模式
- **Commit**：`feat(backend): implement SDK initialization endpoint`

---

### 1.5 聊天系统

#### Task 1.5.1：消息数据模型 - 数据库迁移
- **RED**：测试 `messages` 表存在且字段正确
- **GREEN**：创建迁移文件 `create_messages_table`
- **REFACTOR**：添加索引（conversation_id + created_at）
- **Commit**：`feat(db): add messages table migration`

#### Task 1.5.2：消息数据模型 - Rust 结构体
- **RED**：测试 `Message` 结构体，测试 `sender_type` 和 `message_type` 枚举
- **GREEN**：定义 `model/message.rs`
- **REFACTOR**：添加消息 DTO
- **Commit**：`feat(backend): add Message model with type enums`

#### Task 1.5.3：消息持久化 - Service 层
- **RED**：测试发送消息被保存到数据库；测试获取会话消息列表（分页）
- **GREEN**：实现 `service/chat.rs` 中的消息存储和查询
- **REFACTOR**：游标分页
- **Commit**：`feat(backend): implement message persistence service`

#### Task 1.5.4：消息 HTTP API
- **RED**：测试 `POST /api/v1/sdk/messages` 发送消息；测试 `GET` 获取历史消息
- **GREEN**：实现 HTTP 消息端点
- **REFACTOR**：统一消息响应格式
- **Commit**：`feat(backend): add message HTTP API endpoints`

#### Task 1.5.5：WebSocket 连接管理
- **RED**：测试 WebSocket 连接建立和关闭；测试连接鉴权
- **GREEN**：实现 `ws/handler.rs` WebSocket 升级和会话管理
- **REFACTOR**：提取连接状态管理到 `ws/session.rs`
- **Commit**：`feat(backend): implement WebSocket connection management`

#### Task 1.5.6：WebSocket 消息收发
- **RED**：测试通过 WebSocket 发送消息被持久化；测试消息广播到同一会话的其他连接
- **GREEN**：实现 WebSocket 消息处理
- **REFACTOR**：消息格式统一（JSON envelope）
- **Commit**：`feat(backend): implement WebSocket message send and receive`

#### Task 1.5.7：Redis 在线状态与消息分发
- **RED**：测试用户上线/下线状态更新；测试跨实例消息分发（pub/sub）
- **GREEN**：实现 Redis 在线状态管理和消息发布/订阅
- **REFACTOR**：心跳检测、连接清理
- **Commit**：`feat(backend): add Redis-based online status and message distribution`

#### Task 1.5.8：管理端会话 API
- **RED**：测试开发者获取会话列表（筛选/排序）；测试获取会话详情和消息
- **GREEN**：实现 `api/admin.rs` 中的会话管理 API
- **REFACTOR**：添加未读消息计数
- **Commit**：`feat(backend): add admin conversation management API`

#### Task 1.5.9：管理端 WebSocket
- **RED**：测试开发者通过 WebSocket 回复用户；测试消息实时推送到用户端
- **GREEN**：实现管理端 WebSocket 端点
- **REFACTOR**：多会话同时监听
- **Commit**：`feat(backend): implement admin WebSocket for real-time replies`

---

### 1.6 Web SDK

#### Task 1.6.1：初始化 Web SDK 项目
- **测试**：验证项目可以编译和运行测试
- **实现**：初始化 TypeScript 项目，配置 Vitest、tsup 打包
- **Commit**：`chore(web-sdk): initialize TypeScript project with Vitest`

#### Task 1.6.2：SDK 核心配置
- **RED**：测试 `FeedbackWidget` 构造函数接受配置并验证必填项
- **GREEN**：实现配置类型定义和验证逻辑
- **REFACTOR**：默认值处理
- **Commit**：`feat(web-sdk): implement SDK configuration and validation`

#### Task 1.6.3：HTTP 客户端
- **RED**：测试 API 请求包含正确的 headers（API Key）；测试错误处理
- **GREEN**：封装 fetch 客户端
- **REFACTOR**：重试逻辑、超时处理
- **Commit**：`feat(web-sdk): implement HTTP client with auth headers`

#### Task 1.6.4：SDK 初始化流程
- **RED**：测试调用 `/sdk/init` 获取会话 token
- **GREEN**：实现初始化流程
- **REFACTOR**：缓存 token 到 localStorage
- **Commit**：`feat(web-sdk): implement SDK initialization flow`

#### Task 1.6.5：WebSocket 连接管理
- **RED**：测试 WebSocket 连接、断开、自动重连
- **GREEN**：实现 WebSocket 管理器
- **REFACTOR**：指数退避重连策略
- **Commit**：`feat(web-sdk): implement WebSocket connection with auto-reconnect`

#### Task 1.6.6：消息收发逻辑
- **RED**：测试发送消息通过 WebSocket；测试接收消息触发回调
- **GREEN**：实现消息发送/接收
- **REFACTOR**：消息队列（离线时缓存）
- **Commit**：`feat(web-sdk): implement message send and receive logic`

#### Task 1.6.7：聊天窗口 UI - Shadow DOM
- **RED**：测试 Widget 挂载到指定容器；测试 Shadow DOM 隔离
- **GREEN**：实现聊天窗口 DOM 结构和样式（Shadow DOM）
- **REFACTOR**：主题定制支持
- **Commit**：`feat(web-sdk): implement chat window UI with Shadow DOM isolation`

#### Task 1.6.8：悬浮按钮与窗口切换
- **RED**：测试点击悬浮按钮打开/关闭聊天窗口
- **GREEN**：实现悬浮按钮和窗口切换逻辑
- **REFACTOR**：动画效果、位置配置
- **Commit**：`feat(web-sdk): add floating button and window toggle`

#### Task 1.6.9：消息列表渲染
- **RED**：测试消息正确渲染（文本、时间戳、发送者标识）
- **GREEN**：实现消息列表 UI
- **REFACTOR**：虚拟滚动（大量消息）、自动滚动到底部
- **Commit**：`feat(web-sdk): implement message list rendering`

#### Task 1.6.10：消息输入与发送
- **RED**：测试输入框提交消息；测试空消息不发送
- **GREEN**：实现消息输入框和发送按钮
- **REFACTOR**：支持 Enter 发送、Shift+Enter 换行
- **Commit**：`feat(web-sdk): implement message input and send UI`

---

### 1.7 管理面板 (Admin Panel)

#### Task 1.7.1：初始化管理面板项目
- **测试**：验证项目可以编译和运行测试
- **实现**：使用 Vite + React + TypeScript 初始化，配置 Vitest + React Testing Library
- **Commit**：`chore(admin-panel): initialize React project with Vitest`

#### Task 1.7.2：路由与布局
- **RED**：测试路由正确渲染对应页面组件
- **GREEN**：配置 React Router，实现基础布局（侧边栏 + 内容区）
- **REFACTOR**：懒加载路由
- **Commit**：`feat(admin-panel): add routing and base layout`

#### Task 1.7.3：认证状态管理
- **RED**：测试登录/登出状态切换；测试 token 存储和读取
- **GREEN**：实现认证状态管理（Zustand/Context）
- **REFACTOR**：自动 token 刷新
- **Commit**：`feat(admin-panel): implement authentication state management`

#### Task 1.7.4：登录/注册页面
- **RED**：测试表单验证（邮箱格式、密码长度）；测试成功登录后跳转
- **GREEN**：实现登录和注册页面 UI 及逻辑
- **REFACTOR**：错误提示、加载状态
- **Commit**：`feat(admin-panel): add login and registration pages`

#### Task 1.7.5：API 客户端封装
- **RED**：测试请求自动携带 auth token；测试 401 响应触发重新登录
- **GREEN**：封装 API 客户端（axios/fetch）
- **REFACTOR**：请求/响应拦截器
- **Commit**：`feat(admin-panel): implement API client with auth interceptor`

#### Task 1.7.6：项目列表页面
- **RED**：测试渲染项目列表；测试创建项目表单
- **GREEN**：实现项目列表和创建项目 UI
- **REFACTOR**：空状态、加载骨架屏
- **Commit**：`feat(admin-panel): add project list and creation page`

#### Task 1.7.7：会话列表页面
- **RED**：测试渲染会话列表；测试状态筛选
- **GREEN**：实现 Inbox 会话列表 UI
- **REFACTOR**：实时未读数更新
- **Commit**：`feat(admin-panel): add conversation inbox page`

#### Task 1.7.8：实时聊天界面
- **RED**：测试消息列表渲染；测试发送回复；测试 WebSocket 实时更新
- **GREEN**：实现聊天详情页面，集成 WebSocket
- **REFACTOR**：用户信息侧栏
- **Commit**：`feat(admin-panel): implement real-time chat interface`

---

### 1.8 测试页面

#### Task 1.8.1：初始化测试页面项目
- **测试**：验证项目可以编译
- **实现**：使用 Vite + React 初始化
- **Commit**：`chore(test-page): initialize test page project`

#### Task 1.8.2：SDK 模拟器
- **RED**：测试嵌入 Web SDK 并可交互
- **GREEN**：实现 SDK 模拟器页面（加载 Web SDK、切换用户身份）
- **REFACTOR**：多用户同时模拟
- **Commit**：`feat(test-page): implement SDK simulator`

#### Task 1.8.3：系统状态页面
- **RED**：测试健康检查显示正确状态
- **GREEN**：实现系统状态展示（后端、数据库、Redis、WebSocket）
- **REFACTOR**：自动刷新
- **Commit**：`feat(test-page): add system health status page`

---

## Phase 2：Issue 与日志

### 2.1 Issue 管理

#### Task 2.1.1：Issue 数据模型 - 数据库迁移
- **RED**：测试 `issues` 和 `issue_conversations` 表存在
- **GREEN**：创建迁移文件
- **REFACTOR**：添加索引
- **Commit**：`feat(db): add issues and issue_conversations table migrations`

#### Task 2.1.2：Issue 数据模型 - Rust 结构体
- **RED**：测试模型序列化，测试状态/优先级枚举
- **GREEN**：定义 `model/issue.rs`
- **REFACTOR**：添加 DTO
- **Commit**：`feat(backend): add Issue model with status and priority enums`

#### Task 2.1.3：Issue CRUD - Service 层
- **RED**：测试创建/读取/更新/删除 Issue；测试按状态/优先级/标签筛选
- **GREEN**：实现 `service/issue.rs`
- **REFACTOR**：分页、排序
- **Commit**：`feat(backend): implement Issue CRUD service`

#### Task 2.1.4：Issue CRUD - API 端点
- **RED**：测试所有 Issue API 端点
- **GREEN**：实现 `api/issue.rs` 路由
- **REFACTOR**：批量操作支持
- **Commit**：`feat(backend): add Issue CRUD API endpoints`

#### Task 2.1.5：会话关联 Issue
- **RED**：测试从会话创建 Issue；测试将会话关联到已有 Issue
- **GREEN**：实现会话-Issue 关联逻辑
- **REFACTOR**：自动从聊天记录生成 Issue 描述（占位，AI 增强在 Phase 3）
- **Commit**：`feat(backend): implement conversation-to-issue association`

#### Task 2.1.6：管理面板 - Issue 列表页面
- **RED**：测试列表视图渲染；测试筛选和排序
- **GREEN**：实现 Issue 列表页面 UI
- **REFACTOR**：列表/看板视图切换
- **Commit**：`feat(admin-panel): add Issue list page with filters`

#### Task 2.1.7：管理面板 - Issue 看板视图
- **RED**：测试看板列渲染（Open/In Progress/Resolved）；测试拖拽更新状态
- **GREEN**：实现 Kanban 看板 UI
- **REFACTOR**：拖拽动画、乐观更新
- **Commit**：`feat(admin-panel): implement Issue Kanban board view`

#### Task 2.1.8：管理面板 - Issue 详情页面
- **RED**：测试详情渲染；测试编辑功能；测试关联会话列表
- **GREEN**：实现 Issue 详情页面
- **REFACTOR**：内联编辑、标签管理
- **Commit**：`feat(admin-panel): add Issue detail page`

#### Task 2.1.9：会话中创建/关联 Issue 操作
- **RED**：测试从聊天界面一键创建 Issue；测试选择已有 Issue 关联
- **GREEN**：在聊天界面添加 Issue 操作按钮和弹窗
- **REFACTOR**：消息片段选择、AI 辅助标题生成（Phase 3 增强）
- **Commit**：`feat(admin-panel): add create/link Issue from chat interface`

---

### 2.2 日志收集

#### Task 2.2.1：客户端日志 - 数据库迁移
- **RED**：测试 `client_logs` 表存在
- **GREEN**：创建迁移文件
- **REFACTOR**：添加索引（conversation_id + created_at）
- **Commit**：`feat(db): add client_logs table migration`

#### Task 2.2.2：日志上报 - Service 与 API
- **RED**：测试 `POST /api/v1/sdk/logs` 接收并存储日志
- **GREEN**：实现日志上报端点和存储逻辑
- **REFACTOR**：批量上报支持、速率限制
- **Commit**：`feat(backend): implement client log ingestion API`

#### Task 2.2.3：日志查看 - API 与管理面板
- **RED**：测试 `GET /api/v1/conversations/:id/logs` 返回关联日志
- **GREEN**：实现日志查询 API 和管理面板日志查看 UI
- **REFACTOR**：日志级别筛选、搜索
- **Commit**：`feat(backend): add log viewing API and admin UI`

#### Task 2.2.4：Web SDK 日志采集
- **RED**：测试自动拦截 `console.warn` 和 `console.error`
- **GREEN**：实现日志采集器（拦截 console 方法、采集错误事件）
- **REFACTOR**：可配置采集级别和最大条数
- **Commit**：`feat(web-sdk): implement automatic log collection`

#### Task 2.2.5：Web SDK 日志上报
- **RED**：测试日志批量上报到后端
- **GREEN**：实现日志上报逻辑（定时批量发送或聊天时附带）
- **REFACTOR**：离线缓存、去重
- **Commit**：`feat(web-sdk): implement log upload to backend`

---

### 2.3 附件上传

#### Task 2.3.1：对象存储集成
- **RED**：测试文件上传到 S3 兼容存储并返回 URL
- **GREEN**：集成 MinIO/S3 SDK，实现上传/下载
- **REFACTOR**：文件大小限制、类型白名单
- **Commit**：`feat(backend): integrate S3-compatible object storage`

#### Task 2.3.2：附件上传 API
- **RED**：测试 `POST /api/v1/sdk/attachments` 接收文件
- **GREEN**：实现附件上传端点
- **REFACTOR**：图片压缩/缩略图生成
- **Commit**：`feat(backend): add attachment upload API endpoint`

#### Task 2.3.3：Web SDK 附件上传
- **RED**：测试拖拽图片上传；测试粘贴截图上传
- **GREEN**：实现附件上传 UI（拖拽区域、粘贴监听）
- **REFACTOR**：上传进度显示、预览
- **Commit**：`feat(web-sdk): implement image upload with drag-and-paste support`

---

## Phase 3：排期与 AI

### 3.1 排期管理

#### Task 3.1.1：里程碑 - 数据库迁移与模型
- **RED**：测试 `milestones` 表存在；测试模型序列化
- **GREEN**：创建迁移和 Rust 模型
- **Commit**：`feat(backend): add milestones table and model`

#### Task 3.1.2：里程碑 CRUD - Service 与 API
- **RED**：测试里程碑 CRUD；测试 Issue 分配到里程碑
- **GREEN**：实现 Service 和 API 端点
- **Commit**：`feat(backend): implement milestone CRUD and Issue assignment API`

#### Task 3.1.3：管理面板 - 排期管理页面
- **RED**：测试里程碑列表渲染（进度条）；测试创建/编辑里程碑
- **GREEN**：实现排期管理 UI
- **REFACTOR**：Issue 拖拽分配到里程碑
- **Commit**：`feat(admin-panel): add milestone management page`

#### Task 3.1.4：管理面板 - 里程碑详情
- **RED**：测试里程碑下的 Issue 列表；测试进度统计
- **GREEN**：实现里程碑详情页面
- **REFACTOR**：完成度百分比、剩余时间
- **Commit**：`feat(admin-panel): add milestone detail page with progress`

---

### 3.2 AI Agent

#### Task 3.2.1：知识库 - 数据库迁移与模型
- **RED**：测试 `knowledge_base` 表存在（含 vector 扩展）
- **GREEN**：创建迁移文件（包含 pgvector 扩展启用），定义模型
- **Commit**：`feat(backend): add knowledge_base table with pgvector support`

#### Task 3.2.2：知识库 CRUD
- **RED**：测试 FAQ 条目增删改查
- **GREEN**：实现知识库 Service 和 API
- **Commit**：`feat(backend): implement knowledge base CRUD API`

#### Task 3.2.3：LLM 客户端封装
- **RED**：测试调用 LLM API（使用 mock）；测试错误处理和超时
- **GREEN**：使用 `reqwest` 封装 LLM API 调用（支持 OpenAI/Claude 接口格式）
- **REFACTOR**：重试策略、流式响应支持
- **Commit**：`feat(backend): implement LLM API client wrapper`

#### Task 3.2.4：意图识别
- **RED**：测试消息意图分类（已知问题/简单咨询/复杂问题）
- **GREEN**：实现意图识别逻辑（关键词 + LLM 分类）
- **REFACTOR**：置信度评分
- **Commit**：`feat(backend): implement message intent classification`

#### Task 3.2.5：知识库语义搜索
- **RED**：测试文本嵌入生成；测试语义相似度搜索
- **GREEN**：实现 embedding 生成和 pgvector 相似度查询
- **REFACTOR**：搜索结果排序和阈值
- **Commit**：`feat(backend): implement semantic search for knowledge base`

#### Task 3.2.6：AI 自动回复管线
- **RED**：测试 AI 对已知问题自动回复；测试低置信度转人工
- **GREEN**：实现完整的 AI Agent 管线（意图识别 → 知识库查询 → LLM 回复 → 置信度判断）
- **REFACTOR**：上下文增强（用户历史、Issue 摘要）
- **Commit**：`feat(backend): implement AI auto-reply pipeline`

#### Task 3.2.7：管理面板 - 知识库管理
- **RED**：测试 FAQ 列表渲染；测试增删改操作
- **GREEN**：实现知识库管理 UI
- **Commit**：`feat(admin-panel): add knowledge base management page`

#### Task 3.2.8：管理面板 - AI 配置与统计
- **RED**：测试 AI 开关配置；测试 AI 回复历史展示
- **GREEN**：实现 AI 行为配置页面和效果统计
- **Commit**：`feat(admin-panel): add AI configuration and statistics page`

---

## Phase 4：完善与扩展

### 4.1 Flutter SDK

#### Task 4.1.1：初始化 Flutter SDK 项目
- **Commit**：`chore(flutter-sdk): initialize Flutter SDK package`

#### Task 4.1.2：核心通信层
- **RED**：测试 HTTP 客户端和 WebSocket 管理
- **GREEN**：实现核心通信逻辑
- **Commit**：`feat(flutter-sdk): implement core communication layer`

#### Task 4.1.3：聊天 UI Widget
- **RED**：Widget 测试渲染和交互
- **GREEN**：实现 `FeedbackWidget` 和弹窗模式
- **Commit**：`feat(flutter-sdk): implement chat UI widgets`

#### Task 4.1.4：日志采集与上报
- **RED**：测试日志拦截和批量上报
- **GREEN**：实现日志采集
- **Commit**：`feat(flutter-sdk): implement log collection and upload`

---

### 4.2 通知系统

#### Task 4.2.1：通知服务
- **RED**：测试新消息触发通知；测试通知渠道配置
- **GREEN**：实现通知服务（邮件 + Webhook）
- **Commit**：`feat(backend): implement notification service with email and webhook`

#### Task 4.2.2：通知配置管理
- **RED**：测试管理面板通知设置保存/读取
- **GREEN**：实现通知配置 UI
- **Commit**：`feat(admin-panel): add notification settings page`

---

### 4.3 数据统计

#### Task 4.3.1：统计数据聚合
- **RED**：测试统计查询（日会话量、响应时间、Issue 分布、AI 解决率）
- **GREEN**：实现统计聚合查询 Service 和 API
- **Commit**：`feat(backend): implement analytics aggregation API`

#### Task 4.3.2：统计面板 UI
- **RED**：测试图表渲染
- **GREEN**：使用 Chart.js/Recharts 实现数据统计面板
- **Commit**：`feat(admin-panel): add analytics dashboard with charts`

---

### 4.4 安全与性能

#### Task 4.4.1：安全加固
- **RED**：测试 SQL 注入防护；测试 XSS 防护；测试 CSRF 防护
- **GREEN**：添加安全中间件和输入消毒
- **Commit**：`feat(backend): add security hardening middleware`

#### Task 4.4.2：性能优化
- **RED**：测试 API 响应时间在阈值内；测试并发连接处理
- **GREEN**：添加缓存层、查询优化、连接池调优
- **Commit**：`feat(backend): optimize performance with caching and query tuning`

#### Task 4.4.3：多成员协作
- **RED**：测试团队成员邀请/移除；测试权限控制
- **GREEN**：实现团队管理 Service 和 API
- **Commit**：`feat(backend): implement team member management`

---

## 附录：TDD 执行流程图

```
开始任务
  │
  ▼
编写失败测试 (RED)
  │
  ▼
运行测试 → 确认失败 ✗
  │
  ▼
编写最小实现代码 (GREEN)
  │
  ▼
运行测试 → 确认通过 ✓
  │
  ▼
重构代码 (REFACTOR)
  │
  ▼
运行测试 → 确认仍通过 ✓
  │
  ▼
Git Commit（描述本次变更）
  │
  ▼
下一个任务
```

## 附录：开发优先级与依赖关系

```
Task 1.1 (基础设施) ──┐
                       ├──▶ Task 1.2 (认证) ──▶ Task 1.3 (项目管理)
                       │                              │
                       │                              ▼
                       │                      Task 1.4 (用户/会话)
                       │                              │
                       │                              ▼
                       │                      Task 1.5 (聊天系统) ──┐
                       │                                            │
                       ├──▶ Task 1.6 (Web SDK) ◀────────────────────┤
                       │                                            │
                       └──▶ Task 1.7 (管理面板) ◀───────────────────┘
                                    │
                                    ▼
                          Task 1.8 (测试页面)
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
              Phase 2          Phase 3          Phase 4
           (Issue/日志)      (排期/AI)        (扩展/优化)
```
