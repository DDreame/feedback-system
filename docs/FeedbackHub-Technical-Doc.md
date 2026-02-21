# FeedbackHub 技术文档

## 1 系统概述

### 1.1 项目背景

FeedbackHub 是一个面向独立开发者和小型团队的开源用户反馈与沟通系统。该系统提供多语言SDK，支持将反馈功能嵌入到各类软件产品中，同时配备功能完善的开发者移动端App和Web端管理后台，使开发者能够随时随地接收用户反馈并与用户进行实时交流。

### 1.2 系统目标

- **降低用户反馈门槛**：通过嵌入SDK，用户可随时提交文本、图片、附件、语音等多媒体反馈
- **实现实时双向沟通**：基于WebSocket的即时消息传递，支持用户与开发者实时交流
- **支持主动消息推送**：开发者可向用户推送通知、版本更新、活动公告等
- **多平台SDK支持**：JavaScript (Web)、Swift (iOS)、Kotlin (Android)

### 1.3 核心功能列表

| 功能模块 | 说明 |
|---------|------|
| 用户反馈 | SDK嵌入、多种消息类型、设备信息自动收集、状态跟踪 |
| 消息沟通 | 实时WebSocket、富媒体消息、已读状态、历史记录 |
| 消息推送 | 通知推送、精准定向、模板管理、效果统计 |
| 开发者管理 | 多产品管理、团队协作、API密钥、统计分析 |

### 1.4 技术选型

| 层级 | 技术栈 |
|------|--------|
| 后端 | Rust (Axum/Warp 框架) |
| 开发者App | Flutter (iOS/Android/Web) |
| 数据库 | SQLite (单机) / PostgreSQL (分布式) |
| 缓存 | sled (嵌入式) / Redis (分布式) |
| 对象存储 | 本地文件系统 (单机) / MinIO (分布式) |

---

## 2 系统架构

### 2.1 整体架构设计

FeedbackHub 采用 **模块化单体架构**，支持灵活部署：

- **单机模式 (All-in-One)**：所有服务打包为单个二进制，内嵌数据库、缓存、存储，零外部依赖
- **分布式模式**：各服务独立部署，支持横向扩展，负载均衡

### 2.2 部署模式

#### 单机模式 (All-in-One)

```
┌─────────────────────────────────────────────────────────────┐
│                    FeedbackHub Binary                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │
│  │  HTTP   │  │WebSocket│  │  Admin  │  │  Dashboard  │  │
│  │ Server  │  │ Server  │  │  API    │  │   Web UI    │  │
│  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬──────┘  │
│       │            │            │               │          │
│  ┌────▼────────────▼────────────▼───────────────▼──────┐  │
│  │                    Core Engine                       │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐  │  │
│  │  │ Feedback │ │ Message │ │  Push   │ │   Auth  │  │  │
│  │  │ Service  │ │ Service │ │ Service │ │ Service │  │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘  │  │
│  └───────────────────────┬─────────────────────────────┘  │
│                          │                                  │
│  ┌───────────────────────▼─────────────────────────────┐  │
│  │              Embedded Storage Layer                  │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────────────────┐  │  │
│  │  │ SQLite  │  │  sled   │  │  File Storage      │  │  │
│  │  │ (Data)  │  │ (Cache) │  │  (Attachments)    │  │  │
│  │  └─────────┘  └─────────┘  └─────────────────────┘  │  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**单机模式特点**：
- 单个二进制文件，包含所有功能
- 内嵌 SQLite 数据库（业务数据）
- 内嵌 sled 缓存（内存缓存、会话）
- 内嵌文件存储（附件、图片）
- 无需安装 Docker 或任何外部依赖
- 一条命令启动：`./feedbackhub run`

#### 分布式模式 (Cluster)

```
                          ┌─────────────────┐
                          │   Load Balancer  │
                          │    (Nginx/HA)   │
                          └────────┬────────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        │                          │                          │
┌───────▼───────┐          ┌───────▼───────┐          ┌───────▼───────┐
│  API Node 1   │          │  API Node 2   │          │  API Node N   │
│  (Rust/Axum)  │          │  (Rust/Axum)  │          │  (Rust/Axum)  │
└───────┬───────┘          └───────┬───────┘          └───────┬───────┘
        │                          │                          │
        │                          │                          │
        └──────────────────────────┼──────────────────────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    │      Message Queue         │
                    │    (Redis/RabbitMQ)        │
                    └──────────────┬──────────────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        │                          │                          │
┌───────▼───────┐          ┌───────▼───────┐          ┌───────▼───────┐
│ Feedback Svc  │          │  Message Svc   │          │   Push Svc    │
│   (Worker)    │          │   (Worker)     │          │   (Worker)    │
└───────────────┘          └───────────────┘          └───────────────┘
        │                          │                          │
┌───────▼──────────────────────────▼──────────────────────────▼───────┐
│                         Data Layer                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────┐    │
│  │ PostgreSQL   │  │    Redis     │  │    MinIO / S3         │    │
│  │   (Master)   │  │  (Cluster)   │  │   (Object Storage)    │    │
│  └──────────────┘  └──────────────┘  └────────────────────────┘    │
└──────────────────────────────────────────────────────────────────────┘
```

**分布式模式特点**：
- API 节点：无状态，可水平扩展
- Worker 节点：处理异步任务（消息推送、统计）
- 共享存储：PostgreSQL + Redis + MinIO
- 负载均衡：Nginx/HAProxy 流量分发

---

## 3 核心模块设计

### 3.1 后端服务 (Rust)

#### 3.1.1 框架选择

推荐使用 **Axum** 框架，理由：
- 现代化异步框架， Tokio 生态系统
- 优秀的性能表现
- 简洁的 API 设计
- 良好的扩展性

#### 3.1.2 核心模块划分

```
src/
├── main.rs                 # 入口点
├── bin/
│   └── server.rs          # All-in-One 服务器
├── lib.rs                 # 库入口
├── config/                # 配置管理
│   ├── mod.rs
│   ├── app.rs            # 应用配置
│   ├── storage.rs        # 存储配置
│   └── cluster.rs        # 集群配置
├── api/                   # API 层
│   ├── mod.rs
│   ├── v1/               # API v1
│   │   ├── auth.rs       # 认证接口
│   │   ├── feedback.rs   # 反馈接口
│   │   ├── message.rs    # 消息接口
│   │   ├── push.rs       # 推送接口
│   │   └── app.rs        # 产品管理
│   └── middleware/       # 中间件
├── service/              # 业务逻辑层
│   ├── mod.rs
│   ├── auth_service.rs
│   ├── feedback_service.rs
│   ├── message_service.rs
│   └── push_service.rs
├── domain/               # 领域模型
│   ├── mod.rs
│   ├── user.rs
│   ├── feedback.rs
│   └── message.rs
├── storage/              # 存储层
│   ├── mod.rs
│   ├── sqlite/          # SQLite 嵌入存储
│   ├── sled/            # sled 缓存
│   ├── postgres/        # PostgreSQL 分布式
│   └── redis/           # Redis 集群
└── utils/               # 工具函数
```

#### 3.1.3 嵌入式存储 (单机模式)

**SQLite 集成**：

```rust
// 使用 rusqlite + r2d2 连接池
use rusqlite::{Connection, params};
use r2d2::Pool;

pub struct SqliteStorage {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteStorage {
    pub fn new(path: &Path) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)?;
        
        Ok(Self { pool })
    }
    
    // 初始化表结构
    pub fn init_schema(&self) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS developers (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS applications (
                id TEXT PRIMARY KEY,
                owner_id TEXT NOT NULL,
                name TEXT NOT NULL,
                app_key TEXT UNIQUE NOT NULL,
                app_secret TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS feedbacks (
                id TEXT PRIMARY KEY,
                app_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                type TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'new',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                feedback_id TEXT NOT NULL,
                sender_type TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                content TEXT NOT NULL,
                message_type TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            "#
        )?;
        Ok(())
    }
}
```

**sled 缓存集成**：

```rust
// 使用 sled 嵌入式 KV 存储
use sled::{Db, IVec};

pub struct SledCache {
    db: Db,
}

impl SledCache {
    pub fn new(path: &Path) -> Result<Self> {
        let db = Db::open(path)?;
        Ok(Self { db })
    }
    
    // 会话存储
    pub fn set_session(&self, key: &str, value: &[u8]) -> Result<()> {
        self.db.insert(format!("session:{}", key), IVec::from(value))?;
        Ok(())
    }
    
    pub fn get_session(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(format!("session:{}", key))?
            .map(|v| v.to_vec()))
    }
    
    // 缓存用户会话
    pub fn cache_user_token(&self, user_id: &str, token: &str, ttl: Duration) -> Result<()> {
        let key = format!("token:{}", user_id);
        self.db.insert(&key, token.as_bytes())?;
        // 设置过期
        self.db.expire(&key, ttl)?;
        Ok(())
    }
}
```

#### 3.1.4 API 定义

**认证接口**：

```rust
// POST /api/v1/auth/login
#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    user: UserInfo,
}

// POST /api/v1/feedbacks (SDK 提交反馈)
#[derive(Deserialize)]
struct SubmitFeedbackRequest {
    #[serde(rename = "appId")]
    app_id: String,
    #[serde(rename = "appSecret")]
    app_secret: String,
    #[serde(rename = "type")]
    feedback_type: String,
    title: String,
    content: String,
    attachments: Option<Vec<String>>,
    metadata: Option<JsonValue>,
}
```

### 3.2 开发者 App (Flutter)

#### 3.2.1 技术架构

```
lib/
├── main.dart
├── app/                     # 应用入口
│   ├── app.dart            # MaterialApp 配置
│   └── router.dart         # 路由配置
├── core/                   # 核心层
│   ├── config/             # 配置
│   ├── network/            # 网络请求 (dio)
│   ├── storage/            # 本地存储
│   └── utils/               # 工具函数
├── features/               # 功能模块
│   ├── auth/               # 登录注册
│   ├── home/               # 首页
│   ├── feedback/           # 反馈管理
│   ├── message/            # 消息交流
│   ├── push/               # 推送管理
│   ├── stats/              # 统计分析
│   └── settings/           # 设置
├── models/                 # 数据模型
├── repositories/           # 数据仓库
├── services/               # 服务层
└── widgets/                # 通用组件
```

#### 3.2.2 核心依赖

```yaml
# pubspec.yaml
dependencies:
  flutter:
    sdk: flutter
  
  # 状态管理
  flutter_bloc: ^8.1.3
  equatable: ^2.0.5
  
  # 网络
  dio: ^5.3.3
  web_socket_channel: ^2.4.0
  
  # 本地存储
  shared_preferences: ^2.2.2
  hive: ^2.2.3
  
  # UI
  flutter_svg: ^2.0.9
  cached_network_image: ^3.3.0
  shimmer: ^3.0.0
  
  # 工具
  intl: ^0.18.1
  uuid: ^4.2.1
  path_provider: ^2.1.1
  
  # 推送
  firebase_messaging: ^14.6.1
  flutter_local_notifications: ^16.2.0
```

#### 3.2.3 核心页面

| 页面 | 功能 |
|------|------|
| 登录页 | 邮箱登录、第三方登录、注册 |
| 首页 | 今日概览、快捷操作、最新反馈 |
| 反馈列表 | 筛选、搜索、批量操作 |
| 反馈详情 | 查看内容、附件、对话历史 |
| 消息列表 | 会话列表、实时更新 |
| 聊天页 | 发送文本/图片/文件/语音 |
| 推送管理 | 创建推送、查看历史 |
| 统计分析 | 图表展示、数据导出 |
| 设置 | 账户、通知、主题 |

---

## 4 数据库设计

### 4.1 SQLite 表结构 (单机模式)

```sql
-- 开发者账户
CREATE TABLE developers (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    avatar_url TEXT,
    status TEXT DEFAULT 'active',
    email_verified INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    last_login_at INTEGER
);

-- 产品应用
CREATE TABLE applications (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES developers(id),
    name TEXT NOT NULL,
    description TEXT,
    icon_url TEXT,
    category TEXT,
    app_key TEXT UNIQUE NOT NULL,
    app_secret TEXT NOT NULL,
    status TEXT DEFAULT 'active',
    settings TEXT DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 团队成员
CREATE TABLE app_members (
    id TEXT PRIMARY KEY,
    app_id TEXT NOT NULL REFERENCES applications(id),
    user_id TEXT NOT NULL REFERENCES developers(id),
    role TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE(app_id, user_id)
);

-- 终端用户
CREATE TABLE end_users (
    id TEXT PRIMARY KEY,
    app_id TEXT NOT NULL REFERENCES applications(id),
    device_fingerprint TEXT NOT NULL,
    user_identifier TEXT,
    user_type TEXT DEFAULT 'anonymous',
    platform TEXT NOT NULL,
    device_info TEXT,
    metadata TEXT,
    tags TEXT DEFAULT '[]',
    notification_enabled INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(app_id, device_fingerprint)
);

-- 用户反馈
CREATE TABLE feedbacks (
    id TEXT PRIMARY KEY,
    app_id TEXT NOT NULL REFERENCES applications(id),
    user_id TEXT NOT NULL REFERENCES end_users(id),
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    status TEXT DEFAULT 'new',
    priority TEXT DEFAULT 'normal',
    assigned_to TEXT REFERENCES developers(id),
    rating INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    resolved_at INTEGER
);

-- 反馈附件
CREATE TABLE feedback_attachments (
    id TEXT PRIMARY KEY,
    feedback_id TEXT NOT NULL REFERENCES feedbacks(id),
    file_name TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    file_url TEXT NOT NULL,
    thumbnail_url TEXT,
    storage_key TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

-- 消息
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    feedback_id TEXT NOT NULL REFERENCES feedbacks(id),
    sender_type TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    content TEXT NOT NULL,
    message_type TEXT DEFAULT 'text',
    attachment_url TEXT,
    is_system INTEGER DEFAULT 0,
    read_at INTEGER,
    created_at INTEGER NOT NULL
);

-- 推送任务
CREATE TABLE push_tasks (
    id TEXT PRIMARY KEY,
    app_id TEXT NOT NULL REFERENCES applications(id),
    creator_id TEXT NOT NULL REFERENCES developers(id),
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_filter TEXT,
    schedule_time INTEGER,
    status TEXT DEFAULT 'draft',
    total_count INTEGER DEFAULT 0,
    sent_count INTEGER DEFAULT 0,
    delivered_count INTEGER DEFAULT 0,
    clicked_count INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    completed_at INTEGER
);

-- 索引
CREATE INDEX idx_feedbacks_app_status ON feedbacks(app_id, status);
CREATE INDEX idx_messages_feedback ON messages(feedback_id, created_at);
CREATE INDEX idx_push_tasks_app ON push_tasks(app_id, created_at);
```

---

## 5 SDK 设计

### 5.1 JavaScript SDK

```typescript
// src/index.ts
export class FeedbackHub {
  private config: FeedbackConfig;
  private ws?: WebSocket;
  private messageListeners: ((msg: Message) => void)[] = [];

  constructor(config: FeedbackConfig) {
    this.config = config;
  }

  async init(): Promise<void> {
    // 初始化连接
    await this.connectWebSocket();
    // 显示悬浮按钮
    this.renderWidget();
  }

  async submit(options: FeedbackOptions): Promise<FeedbackResult> {
    const response = await fetch(`${this.config.apiUrl}/api/v1/feedbacks`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-App-Key': this.config.appKey,
        'X-App-Secret': this.config.appSecret,
      },
      body: JSON.stringify({
        type: options.type,
        title: options.title,
        content: options.content,
        attachments: options.attachments,
        metadata: options.metadata,
      }),
    });
    
    return response.json();
  }

  onMessage(callback: (msg: Message) => void): void {
    this.messageListeners.push(callback);
  }

  private handleMessage(data: Message): void {
    this.messageListeners.forEach(cb => cb(data));
  }
}
```

### 5.2 Swift SDK

```swift
// Sources/FeedbackHub/FeedbackHub.swift
public final class FeedbackHub {
    private let config: FeedbackConfig
    private var webSocket: URLSessionWebSocketTask?
    
    public static let shared = FeedbackHub()
    
    private init() {}
    
    public func configure(appId: String, appSecret: String, options: FeedbackOptions = .default) {
        self.config = FeedbackConfig(appId: appId, appSecret: appSecret, options: options)
    }
    
    public func submitFeedback(
        type: FeedbackType,
        title: String,
        content: String,
        attachments: [Data]? = nil,
        metadata: [String: Any]? = nil,
        completion: @escaping (Result<String, Error>) -> Void
    ) {
        // 实现反馈提交逻辑
    }
}
```

### 5.3 Kotlin SDK

```kotlin
// android/src/main/kotlin/com/feedbackhub/sdk/FeedbackHub.kt
class FeedbackHub private constructor(
    private val config: FeedbackConfig
) {
    private var webSocket: WebSocket? = null
    
    companion object {
        @Volatile
        private var instance: FeedbackHub? = null
        
        fun init(context: Context, appId: String, appSecret: String, block: ConfigBuilder.() -> Unit) {
            val builder = ConfigBuilder().apply(block)
            val config = FeedbackConfig(appId, appSecret, builder.build())
            instance = FeedbackHub(config)
        }
    }
    
    suspend fun submitFeedback(
        type: FeedbackType,
        title: String,
        content: String,
        attachments: List<ByteArray>? = null
    ): Result<String> = suspendCoroutine { continuation ->
        // 实现反馈提交逻辑
    }
}
```

---

## 6 安全方案

### 6.1 认证机制

| 场景 | 认证方式 |
|------|---------|
| 开发者登录 | JWT (Access + Refresh Token) |
| SDK 请求 | AppKey + AppSecret 签名 |
| WebSocket | JWT Token 握手 |

### 6.2 数据安全

- 密码使用 bcrypt 加密存储
- AppSecret 使用 AES-256 加密
- HTTPS 强制传输加密
- API 请求签名验证

### 6.3 访问控制

- RBAC 角色：owner / admin / developer / viewer
- 接口级别权限校验
- 请求频率限制 (Rate Limiting)

---

## 7 部署方案

### 7.1 单机部署 (All-in-One)

```bash
# 下载二进制
curl -L -o feedbackhub https://github.com/feedbackhub/feedbackhub/releases/latest/download/feedbackhub-linux-x64

# 初始化 (首次运行)
./feedbackhub init --data-dir /var/lib/feedbackhub

# 启动服务
./feedbackhub run \
  --data-dir /var/lib/feedbackhub \
  --http-port 8080 \
  --admin-port 9090 \
  --web-ui-dir ./web-ui

# 或使用配置文件
./feedbackhub run --config feedbackhub.yaml
```

**配置文件示例**：

```yaml
# feedbackhub.yaml
server:
  http_port: 8080
  admin_port: 9090
  web_ui_dir: ./web-ui

storage:
  data_dir: /var/lib/feedbackhub/data
  cache_dir: /var/lib/feedbackhub/cache
  attachment_dir: /var/lib/feedbackhub/attachments

security:
  jwt_secret: "your-32-char-secret-key-here"
  api_secret: "your-api-secret-key-here"
  
app:
  log_level: info
  max_upload_size: 100mb
```

### 7.2 分布式部署

```yaml
# cluster.yaml
cluster:
  mode: distributed
  node_id: api-1
  
api:
  host: 0.0.0.0
  port: 8080
  workers: 4
  
database:
  url: postgresql://user:pass@postgres:5432/feedbackhub
  
redis:
  url: redis://redis:6379
  
storage:
  type: minio
  endpoint: http://minio:9000
  access_key: minioadmin
  secret_key: minioadmin
  bucket: feedbackhub
  
message_queue:
  type: redis
  url: redis://redis:6379
```

### 7.3 扩展策略

| 组件 | 扩展方式 |
|------|---------|
| API 节点 | 无状态，水平扩展，Nginx 负载均衡 |
| Worker 节点 | 消息队列消费者，横向扩展 |
| 数据库 | PostgreSQL 主从复制 |
| 缓存 | Redis 集群模式 |
| 存储 | MinIO 分布式模式 |

---

## 8 技术选型总结

| 层级 | 技术选型 | 理由 |
|------|---------|------|
| 后端语言 | Rust | 高性能、低内存、安全性强 |
| Web 框架 | Axum | 现代化、异步、高性能 |
| 嵌入式数据库 | SQLite + rusqlite | 零依赖、单机模式 |
| 嵌入式缓存 | sled | 纯 Rust、嵌入式 |
| 分布式数据库 | PostgreSQL | 成熟稳定 |
| 分布式缓存 | Redis | 业界标准 |
| 对象存储 | MinIO | S3 兼容、开源 |
| 开发者 App | Flutter | 跨平台 (iOS/Android/Web) |
| SDK | TypeScript / Swift / Kotlin | 主流平台全覆盖 |

---

*文档版本：v1.1*
*最后更新：2026年2月21日*
