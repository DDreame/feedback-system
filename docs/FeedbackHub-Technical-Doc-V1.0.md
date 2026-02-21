# FeedbackHub 技术文档

## 1 系统概述

### 1.1 项目背景

在软件开发和产品运营过程中，开发者与用户之间的有效沟通至关重要。传统的问题反馈渠道（如应用商店评论、邮件反馈）存在反馈周期长、沟通效率低、无法实时交流等问题。开发者需要一个能够嵌入到自己软件产品中的用户反馈系统，以便及时收集用户反馈、与用户进行双向沟通、并主动向用户推送重要信息。

FeedbackHub 是一个面向独立开发者和小型团队的开源用户反馈与沟通系统。该系统提供多语言SDK，支持将反馈功能嵌入到各类软件产品中，同时配备功能完善的开发者移动端App和Web端管理后台，使开发者能够随时随地接收用户反馈并与用户进行实时交流。

### 1.2 系统目标

FeedbackHub 系统的核心目标是为软件开发者和终端用户之间建立一座高效、便捷的沟通桥梁。具体目标包括：

第一，降低用户反馈门槛。通过在应用程序中嵌入轻量级的SDK，用户可以随时随地提交反馈，无需切换到其他渠道。用户可以提交文本、图片、附件甚至语音消息，反馈内容可以包含截图和设备信息，帮助开发者快速定位问题。

第二，实现实时双向沟通。系统支持用户和开发者之间的实时消息传递，开发者可以通过移动端App或Web端即时回复用户消息。这种即时沟通模式大幅提升了问题解决效率，也增强了用户对产品的信任感。

第三，支持主动消息推送。开发者不仅能够被动接收用户反馈，还可以主动向用户推送通知、版本更新信息、活动公告等消息。这种双向沟通能力使开发者能够更好地维护用户关系，提升产品活跃度。

第四，提供多平台支持。系统提供覆盖主流开发平台的SDK，包括Web端（JavaScript/TypeScript）、iOS端（Swift）、Android端（Kotlin）。开发者可以根据自己的产品需求选择合适的SDK进行集成。

### 1.3 系统定位

FeedbackHub 定位为轻量级、开源自托管的用户反馈解决方案，主要服务于以下用户群体：

独立开发者是本系统的首要目标用户群体。独立开发者通常没有专门的客服团队，需要一个高效的工具来收集用户反馈并及时响应。FeedbackHub 的移动端App使开发者即使不在电脑前也能处理用户反馈，非常适合移动办公场景。

小型创业团队是另一重要用户群体。小型团队虽然有产品经理或运营人员负责用户沟通，但通常缺乏专业的工单系统。FeedbackHub 可以作为轻量级的客户沟通工具，帮助团队规范化用户反馈处理流程。

开源项目维护者也是潜在用户群体。开源项目通常依赖社区反馈来改进产品，FeedbackHub 可以帮助开源项目维护者更好地收集和管理来自用户的反馈意见。

### 1.4 核心功能列表

FeedbackHub 系统提供以下核心功能：

用户反馈功能包括：嵌入SDK后用户可以通过浮动按钮或快捷入口提交反馈；支持文本、图片、附件（文件）和语音等多种消息类型；自动收集设备信息、操作系统版本、应用版本等上下文数据；支持反馈优先级分类（普通、紧急、问题报告、建议等）；支持反馈状态跟踪（待处理、处理中、已解决、已关闭等）。

开发者管理功能包括：开发者注册和认证系统；多项目管理（支持一个账号管理多个产品）；API密钥管理（为每个产品生成独立的SDK接入密钥）；团队协作（支持添加团队成员，设置不同权限角色）；数据统计（反馈数量、响应时间、用户满意度等指标）。

消息沟通功能包括：实时消息推送（基于WebSocket实现即时通讯）；消息已读未读状态跟踪；开发者主动回复用户消息；支持富媒体消息（图片、文件、语音）；消息历史记录存储和查询。

消息推送功能包括：向指定用户或用户群组推送通知；支持定时推送和立即推送；推送模板管理；推送效果统计（送达率、点击率）。

### 1.5 关键约束与假设

在设计 FeedbackHub 系统时，我们做出以下关键约束和假设：

关于系统规模的假设：本系统设计时假设单个产品的日活跃用户数在数百到数千级别，反馈消息量在每日数百条以内。对于更大规模的应用场景，系统架构需要进行相应的扩展优化。

关于自托管的假设：系统设计为可以完全自托管部署，开发者需要自行准备服务器和数据库。系统不依赖任何第三方云服务，所有功能都可以在本地环境运行。

关于安全性的假设：系统需要支持HTTPS加密传输；需要实现完善的身份认证和授权机制；需要确保用户数据的存储安全。

关于扩展性的假设：系统架构设计需要支持后续添加更多SDK语言支持；需要支持插件机制以扩展功能；需要支持与第三方工具（如Slack、Discord、邮件系统等）的集成。

---

## 2 系统架构

### 2.1 整体架构设计

FeedbackHub 系统采用前后端分离的微服务架构设计，整体架构分为四个主要层次：客户端层、接入层、服务层和数据层。这种分层架构设计使得各组件职责清晰，便于独立开发、部署和扩展。

客户端层包括三个主要部分：用户端SDK（嵌入到开发者的应用程序中）、开发者Web管理后台、开发者移动端App（iOS和Android）。这些客户端通过HTTPS和WebSocket协议与服务层进行通信。

接入层负责处理所有外部请求，主要包括API网关和消息网关两个组件。API网关处理RESTful API请求，负责请求路由、负载均衡、限流熔断等功能。消息网关处理WebSocket连接，负责实时消息的推送和接收。

服务层包含多个微服务组件：用户服务处理用户注册、认证和基本信息管理；反馈服务处理反馈的创建、查询、状态更新等操作；消息服务处理消息的发送、接收和存储；推送服务处理向用户推送通知消息；通知服务处理系统内的各种通知（如新反馈提醒、消息提醒等）；统计分析服务处理数据统计和报表生成。

数据层包括关系型数据库PostgreSQL（存储核心业务数据）、Redis（缓存和消息队列）、对象存储（存储图片、文件等二进制数据）。数据层还包含消息队列（如RabbitMQ或Kafka），用于异步处理和系统解耦。

### 2.2 系统架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              客户端层                                        │
├─────────────────┬─────────────────────┬───────────────────────────────────┤
│   用户端SDK      │   开发者Web后台      │         开发者移动端App            │
│  JS/Web SDK     │    React + TS       │       iOS (Swift)                  │
│  Swift SDK      │    管理界面          │       Android (Kotlin)             │
│  Kotlin SDK     │                     │       Web (PWA)                     │
└────────┬────────┴──────────┬──────────┴───────────────┬───────────────────┘
         │                    │                            │
         │              HTTPS / HTTP                       │
         │                    │                            │
┌────────▼────────────────────▼────────────────────────────▼───────────────────┐
│                              接入层                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                         API Gateway (Nginx/Gateway)                         │
│                    ┌─────────────────┬─────────────────┐                     │
│                    │   REST API      │  WebSocket      │                     │
│                    │   Gateway       │  Gateway        │                     │
│                    └─────────────────┴─────────────────┘                     │
└────────────────────────────────┬────────────────────────────────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌────────▼────────┐    ┌─────────▼─────────┐    ┌───────▼───────┐
│   用户服务      │    │    反馈服务        │    │   消息服务     │
│  (User Service)│    │ (Feedback Service)│    │(Message Service)│
└────────┬────────┘    └─────────┬─────────┘    └───────┬───────┘
         │                       │                       │
┌────────▼────────┐    ┌─────────▼─────────┐    ┌───────▼───────┐
│   推送服务       │    │    通知服务        │    │ 统计分析服务   │
│(Push Service)   │    │ (Notification Svc) │    │ (Analytics)    │
└─────────────────┘    └───────────────────┘    └───────────────┘
         │                       │                       │
─────────┼───────────────────────┼───────────────────────┼────────────────────
         │                       │                       │
┌────────▼──────────────────────▼───────────────────────▼───────────────────┐
│                              数据层                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│  PostgreSQL          │        Redis              │     对象存储            │
│  (主数据库)          │  (缓存/会话/消息队列)     │   (MinIO/S3)           │
│  用户/反馈/消息       │  Token/缓存/实时消息     │  图片/文件/语音         │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 技术栈选型

FeedbackHub 系统的技术栈选型遵循以下原则：优先选择成熟稳定、社区支持完善的开源技术；优先选择跨平台、易于部署的技术；优先选择对开发者友好、学习曲线平缓的技术。

后端技术栈方面，服务端开发语言选择 Go 或 Node.js。Go 语言以其优秀的并发处理能力、高性能和简洁的部署方式著称，非常适合处理实时消息和高并发场景。Node.js 则以其事件驱动模型和丰富的生态系统见长，开发效率高，TypeScript的支持也非常完善。我们推荐使用 Node.js + TypeScript 作为主要开发语言，便于前后端技术栈统一。

数据库选择 PostgreSQL 作为主数据库。PostgreSQL 是功能最强大的开源关系型数据库，支持复杂查询、事务处理、JSON类型等特性，完全能够满足业务需求。对于缓存和会话存储，选择 Redis。对于对象存储，支持 MinIO（自托管场景）或 S3 兼容存储。

消息队列选择 RabbitMQ 或 Kafka。RabbitMQ 部署简单，功能完善，适合中小规模系统。Kafka 则以其高吞吐量和持久化特性，适合大规模消息处理场景。

前端技术栈方面，开发者Web管理后台选择 React + TypeScript + Tailwind CSS。React 是最流行的前端框架，生态丰富。TypeScript 提供完善的类型支持。Tailwind CSS 提供高效的样式开发体验。

移动端技术栈方面，iOS端选择 Swift + SwiftUI。Swift 是 Apple 官方推荐的语言，SwiftUI 是现代化的声明式UI框架。Android端选择 Kotlin + Jetpack Compose。Kotlin 是 Android 官方推荐语言，Jetpack Compose 是现代化的 Android UI 工具包。跨平台方案也可以考虑 React Native 或 Flutter。

### 2.4 服务间通信

服务间通信采用同步和异步两种方式结合的模式。

同步通信使用 HTTP/gRPC 用于需要实时返回结果的场景，如用户认证、获取反馈详情等操作。我们推荐使用 gRPC 进行服务间通信，因为它具有高性能、强类型、自动生成客户端代码等优点。对于简单的场景，也可以使用 RESTful API。

异步通信使用消息队列用于需要解耦或耗时处理的场景，如发送推送通知、生成统计报表等操作。通过消息队列，系统可以在高并发场景下保持稳定运行，消息不会丢失。

服务发现使用 Consul 或 etcd 实现微服务注册与发现。每个服务启动时向注册中心注册自己的地址，其他服务通过注册中心发现目标服务。健康检查机制确保只有健康的服务实例接收请求。

### 2.5 部署架构

 FeedbackHub 支持多种部署模式以适应不同场景需求。

对于小型部署（个人开发者），可以采用单服务器部署模式。所有服务（API、数据库、Redis、存储）部署在一台服务器上，使用 Docker Compose 进行容器编排。这种部署模式简单易行，成本低廉。

对于中型部署（小型团队），可以采用多服务分离部署模式。将数据库、Redis、对象存储部署在独立服务器上，API服务可以部署多实例实现负载均衡。使用 Docker Swarm 或轻量级 Kubernetes（如 K3s）进行容器编排。

对于生产环境部署，建议采用 Kubernetes 部署模式。API 服务部署为 Deployment，通过 HPA 实现自动伸缩。使用 ConfigMap 和 Secret 管理配置。使用 Ingress 或 Gateway 管理外部访问。数据库使用主从复制确保高可用。

---

## 3 功能模块设计

### 3.1 用户反馈模块

用户反馈模块是 FeedbackHub 系统的核心功能之一，负责收集和管理用户反馈信息。

反馈提交功能支持多种反馈类型，包括问题报告（用户报告产品中存在的bug或问题）、功能建议（用户对产品功能的改进建议）、使用咨询（用户对产品使用方法的咨询）、投诉建议（用户对产品或服务的不满和建议）以及其他类型的反馈。每种类型可以设置不同的处理流程和优先级。

反馈提交接口设计如下：SDK 提供简洁的 API 供应用调用。用户调用 `submitFeedback(options)` 方法提交反馈，options 参数包含反馈类型、标题、内容、附件列表等。SDK 收集设备信息（设备型号、操作系统版本、屏幕分辨率等）、应用信息（应用版本、渠道、Build号等）以及用户可选信息（用户ID、联系方式等），将这些信息与反馈内容一起提交到服务端。

反馈处理工作流包括以下状态：新建（用户提交反馈后自动创建）、已读（开发者查看了反馈详情）、处理中（开发者开始处理反馈）、等待用户回复（需要用户提供更多信息）、已解决（问题已解决或建议已采纳）、已关闭（反馈处理完成，用户确认或超时自动关闭）。变更可以触发相应的通知。

状态反馈分配与团队协作功能支持将反馈分配给团队成员处理。每个反馈可以指定负责人，负责人会收到通知提醒。团队成员可以看到分配给自己的反馈列表，支持评论和内部备注功能。

### 3.2 消息沟通模块

消息沟通模块实现用户与开发者之间的实时双向沟通。

实时消息功能基于 WebSocket 协议实现。当用户提交反馈后，可以进入持续对话模式，用户和开发者可以发送消息进行实时沟通。消息类型支持纯文本、富文本（带格式）、图片、文件（任意类型附件）、语音（录音消息）。消息长度限制为单条消息最大 10KB 文本，附件最大 100MB。

消息同步机制确保多端消息一致性。用户端SDK和开发者App都保持与服务器的 WebSocket 连接。消息发送后首先存储到服务器，然后推送到接收方的在线设备。对于离线设备，消息会存储在服务器端，下次连接时同步。消息序列号机制确保消息不丢失、不重复。

消息已读状态帮助用户了解对方是否已读消息。每条消息都有已读状态标记，包括未读、已读。开发者查看反馈详情时，自动将相关消息标记为已读。用户发送新消息时，开发者端收到新消息提醒。

消息历史功能允许用户和开发者查看完整的对话历史。支持按时间范围、关键词搜索消息。提供消息导出功能，可以导出为 PDF 或 Markdown 格式。

### 3.3 消息推送模块

消息推送模块允许开发者主动向用户推送消息。

推送类型包括通知推送（向用户发送提醒通知，如新功能上线、活动开始等）、消息推送（直接推送消息到用户的反馈对话中）、系统消息（发送重要的系统通知，如账户相关通知）。

推送目标选择支持按用户ID推送（向特定用户发送消息）、按标签推送（按用户标签分组推送）、按设备推送（向特定设备发送消息）、全员推送（向所有用户发送广播消息）。

推送效果统计记录每条推送的发送时间、送达数量、点击数量、转化效果等数据。这些统计数据帮助开发者评估推送效果，优化推送策略。

推送频率控制防止过度打扰用户。系统支持设置全局推送频率限制（如每天最多接收N条推送）、用户偏好设置（用户可以关闭某些类型的推送）、免打扰时段（用户可以设置免打扰时间段）。

### 3.4 用户管理模块

用户管理模块处理与用户相关的功能。

匿名用户与注册用户区分处理。默认情况下，SDK 接入的用户为匿名用户，系统通过设备指纹识别用户。匿名用户可以提交反馈、参与对话，但无法享受部分高级功能。用户提供邮箱或手机号后可以升级为注册用户，获得更好的个性化服务。

用户画像功能收集和管理用户信息。基础信息包括用户ID（匿名ID或注册ID）、注册时间、最后活跃时间。设备信息包括设备类型、操作系统、屏幕分辨率、应用版本等。行为信息包括提交反馈数量、参与对话活跃度、反馈采纳率等。开发者可以自定义用户标签，用于用户分群和精准推送。

用户偏好设置允许用户管理自己的通知偏好。推送开关可以开启或关闭各类推送。免打扰时段可以设置不接收推送的时间段。消息预览设置可以控制锁屏通知是否显示消息内容。

### 3.5 开发者管理模块

开发者管理模块为开发者提供账户管理和产品管理功能。

账户管理功能包括开发者注册（支持邮箱注册和第三方登录如GitHub、Google）、账户验证（邮箱验证确保账户安全）、密码管理（密码重置、双因素认证）、账户设置（个人资料、头像、通知设置）。

项目管理功能包括创建产品（每个产品对应一个需要收集反馈的应用程序）、产品设置（产品名称、图标、描述、分类）、SDK配置（为每个产品生成唯一的 App ID 和 App Secret）、删除产品（软删除，保留历史数据）。

团队协作功能支持邀请团队成员加入产品项目。成员角色包括所有者（拥有所有权限，可以管理其他成员）、管理员（拥有大部分管理权限）、开发者（可以处理反馈和回复用户）、查看者（只读权限，可以查看反馈列表）。每个成员可以加入多个产品项目，可以在不同项目中担任不同角色。

API密钥管理为每个产品生成 API 密钥。开发者可以使用 API 密钥调用服务端 API 实现自动化操作，如通过脚本自动标记反馈状态、批量导出反馈数据等。API 密钥支持设置权限范围和有效期。

### 3.6 统计分析模块

统计分析模块为开发者提供数据分析和报表功能。

核心指标包括反馈数量（按日、周、月统计新增反馈数量）、反馈类型分布（各类反馈的比例）、平均响应时间（从用户提交反馈到开发者首次回复的平均时间）、问题解决率（已解决问题的反馈占比）、用户满意度（用户对处理结果的评价分数）。

可视化报表功能提供多维度数据展示。趋势图展示关键指标随时间的变化趋势。饼图或柱状图展示分类数据的占比和对比。热力图可以展示用户活跃时间段分布。

数据导出功能允许开发者导出原始数据。导出格式支持 CSV、Excel、JSON。导出内容可以选择全部数据或指定时间范围。导出操作支持后台异步执行，完成后通过邮件或站内通知提醒下载。

---

## 4 API 接口设计

### 4.1 API 设计原则

FeedbackHub 的 API 设计遵循以下原则：

RESTful 风格：API 使用 RESTful 设计风格，资源命名使用名词（如 /feedbacks、/messages），HTTP 方法表示操作类型（GET 获取、POST 创建、PUT 更新、DELETE 删除）。

版本控制：API 使用 URL 路径进行版本控制，如 /api/v1/feedbacks。版本号在 URL 中明确体现，便于后续版本迭代和兼容处理。

统一响应格式：所有 API 响应使用统一的 JSON 格式，包含 code（状态码）、message（状态信息）、data（业务数据）三个字段。分页查询返回 total（总数）、page（当前页码）、pageSize（每页数量）、list（数据列表）。

错误处理：使用标准 HTTP 状态码表示请求结果。4xx 客户端错误返回详细的错误信息。5xx 服务器错误返回通用（错误信息详细日志记录错误便于排查）。

认证授权：需要认证的 API 使用 Bearer Token 进行身份验证。API 密钥用于 SDK 与服务端的通信认证。

### 4.2 认证接口

开发者认证接口用于开发者的账户登录和验证。

开发者登录接口路径为 POST /api/v1/auth/login，请求参数包括 email（邮箱地址）、password（密码）。返回结果包括 accessToken（访问令牌，有效期建议 24 小时）、refreshToken（刷新令牌，有效期建议 7 天）、expiresIn（令牌过期时间戳）、user（用户基本信息）。

开发者注册接口路径为 POST /api/v1/auth/register，请求参数包括 email（邮箱地址）、password（密码）、confirmPassword（确认密码）、name（显示名称，可选）。返回结果包括 accessToken、refreshToken、user（用户基本信息）。

刷新令牌接口路径为 POST /api/v1/auth/refresh，请求参数包括 refreshToken。返回结果包括新的 accessToken 和 refreshToken。

退出登录接口路径为 POST /api/v1/auth/logout，请求头需要包含 Authorization: Bearer {token}。返回结果为操作成功状态。

### 4.3 反馈管理接口

反馈管理接口用于创建、查询、更新反馈信息。

创建反馈接口路径为 POST /api/v1/feedbacks，用于 SDK 端提交用户反馈。请求参数包括 appId（应用ID，从SDK配置获取）、type（反馈类型：problem/suggestion/consult/complaint）、title（标题，最多 200 字符）、content（内容，最多 5000 字符）、attachments（附件列表，文件上传后返回的URL）、metadata（可选的附加信息，JSON对象）。请求头需要包含 X-App-Secret（应用密钥）。返回结果包括 feedbackId（反馈ID）、createdAt（创建时间）。

查询反馈列表接口路径为 GET /api/v1/feedbacks，用于开发者获取反馈列表。查询参数包括 page（页码，默认 1）、pageSize（每页数量，默认 20）、type（按类型筛选）、status（按状态筛选）、keyword（关键词搜索标题和内容）、startDate（开始日期）、endDate（结束日期）、assignedTo（分配给的用户ID）。返回结果包括分页的反馈列表，每条反馈包含详细信息。

查询反馈详情接口路径为 GET /api/v1/feedbacks/:id，用于获取单个反馈的完整信息。返回结果包括反馈的所有字段，以及关联的对话消息列表、附件列表、操作日志等。

更新反馈状态接口路径为 PUT /api/v1/feedbacks/:id/status，用于更新反馈状态。请求参数包括 status（目标状态）、comment（处理备注，可选）。请求头需要包含认证信息。返回结果为更新后的反馈信息。

分配反馈接口路径为 PUT /api/v1/feedbacks/:id/assign，用于分配反馈给团队成员。请求参数包括 userId（被分配的团队成员ID）。返回结果为更新后的反馈信息。

### 4.4 消息接口

消息接口用于消息的发送和查询。

发送消息接口路径为 POST /api/v1/messages，用于开发者回复用户消息。请求参数包括 feedbackId（反馈ID）、content（消息内容）、type（消息类型：text/image/file/audio）、attachmentUrl（附件URL，type为非text时需要）。返回结果包括 messageId、createdAt。

查询消息列表接口路径为 GET /api/v1/feedbacks/:feedbackId/messages，用于获取某个反馈的对话消息。查询参数包括 page、pageSize、beforeId（用于游标分页）。返回结果包括分页的消息列表。

标记已读接口路径为 PUT /api/v1/messages/read，用于标记消息为已读。请求参数包括 messageIds（需要标记的消息ID数组）或 feedbackId（标记该反馈下所有消息为已读）。

WebSocket 连接接口路径为 WS /api/v1/ws，用于建立实时消息连接。连接时需要在 URL 参数或握手头中携带认证信息。服务端推送消息事件包括 newMessage（新消息）、messageRead（消息已读）、typing（对方正在输入）。

### 4.5 推送接口

推送接口用于向用户发送主动推送消息。

创建推送任务接口路径为 POST /api/v1/push/tasks。请求参数包括 title（推送标题）、content（推送内容）、targetType（推送目标类型：user/tag/device/all）、targetIds（目标ID列表，targetType为user/tag/device时需要）、scheduleTime（定时发送时间，null为立即发送）、templateId（推送模板ID）。返回结果包括 taskId、status。

查询推送任务接口路径为 GET /api/v1/push/tasks/:id。返回结果包括任务详情、发送统计（已发送、送达、点击数）。

查询推送历史接口路径为 GET /api/v1/push/history。查询参数包括 page、pageSize、startDate、endDate。返回结果包括推送任务列表及统计信息。

### 4.6 用户管理接口

用户管理接口用于管理开发者账户和设置。

获取用户信息接口路径为 GET /api/v1/user/profile。返回结果包括用户的基本信息、账户设置等。

更新用户信息接口路径为 PUT /api/v1/user/profile。请求参数包括 name、avatar（头像URL）、notificationSettings（通知设置）等。

修改密码接口路径为 PUT /api/v1/user/password。请求参数包括 oldPassword、newPassword、confirmPassword。

### 4.7 产品管理接口

产品管理接口用于管理开发者创建的产品项目。

创建产品接口路径为 POST /api/v1/apps。请求参数包括 name（产品名称）、description（产品描述）、icon（产品图标URL）、category（产品分类）。返回结果包括 appId、appSecret（SDK接入密钥）。

产品列表接口路径为 GET /api/v1/apps。返回结果包括用户有权限访问的所有产品列表。

产品详情接口路径为 GET /api/v1/apps/:id。返回结果包括产品的详细信息、统计概览、团队成员列表等。

更新产品接口路径为 PUT /api/v1/apps/:id。请求参数包括 name、description、icon 等可更新字段。

删除产品接口路径为 DELETE /api/v1/apps/:id。使用软删除，产品数据不会立即物理删除。

---

## 5 数据库设计

### 5.1 数据库概览

FeedbackHub 系统使用 PostgreSQL 作为主数据库，存储所有核心业务数据。数据库设计遵循第三范式，适当考虑性能进行反范式化设计。核心表包括用户表、产品表、反馈表、消息表、推送任务表等。

以下是数据库的核心表结构设计：

### 5.2 开发者用户表

开发者用户表（developers）存储开发者的账户信息。

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | UUID | PK | 用户唯一标识 |
| email | VARCHAR(255) | UNIQUE, NOT NULL | 邮箱地址 |
| password_hash | VARCHAR(255) | NOT NULL | 加密后的密码 |
| name | VARCHAR(100) | NOT NULL | 显示名称 |
| avatar_url | VARCHAR(500) | NULL | 头像URL |
| status | ENUM | NOT NULL | 账户状态：active/inactive/banned |
| email_verified | BOOLEAN | DEFAULT false | 邮箱是否验证 |
| created_at | TIMESTAMP | NOT NULL | 创建时间 |
| updated_at | TIMESTAMP | NOT NULL | 更新时间 |
| last_login_at | TIMESTAMP | NULL | 最后登录时间 |

### 5.3 产品表

产品表（applications）存储开发者创建的产品项目信息。

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | UUID | PK | 产品唯一标识 |
| owner_id | UUID | FK -> developers.id | 所有者ID |
| name | VARCHAR(100) | NOT NULL | 产品名称 |
| description | TEXT | NULL | 产品描述 |
| icon_url | VARCHAR(500) | NULL | 产品图标 |
| category | VARCHAR(50) | NULL | 产品分类 |
| app_key | VARCHAR(64) | UNIQUE | SDK接入密钥 |
| app_secret | VARCHAR(128) | NOT NULL | SDK接入密钥（加密存储） |
| status | ENUM | NOT NULL | 产品状态：active/inactive/deleted |
| settings | JSONB | DEFAULT {} | 产品设置 |
| created_at | TIMESTAMP | NOT NULL | 创建时间 |
| updated_at | TIMESTAMP | NOT NULL | 更新时间 |

### 5.4 团队成员表

团队成员表（app_members）存储产品团队的成员关系。

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | UUID | PK | 成员关系ID |
| app_id | UUID | FK -> applications.id | 产品ID |
| user_id | UUID | FK -> developers.id | 开发者ID |
| role | ENUM | NOT NULL | 角色：owner/admin/dev/viewer |
| created_at | TIMESTAMP | NOT NULL | 添加时间 |

### 5.5 用户表（终端用户）

用户表（end_users）存储通过SDK接入的终端用户信息。

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | UUID | PK | 用户唯一标识 |
| app_id | UUID | FK -> applications.id | 所属产品ID |
| device_fingerprint | VARCHAR(128) | NOT NULL | 设备指纹 |
| user_identifier | VARCHAR(255) | NULL | 用户标识（邮箱/手机号） |
| user_type | ENUM | NOT NULL | 用户类型：anonymous/registered |
| platform | VARCHAR(20) | NOT NULL | 平台：ios/android/web |
| device_info | JSONB | NULL | 设备详细信息 |
| metadata | JSONB | NULL | 自定义元数据 |
| tags | VARCHAR[] | DEFAULT {} | 用户标签 |
| notification_enabled | BOOLEAN | DEFAULT true | 是否接收推送 |
| created_at | TIMESTAMP | NOT NULL | 首次使用时间 |
| updated_at | TIMESTAMP | NOT NULL | 最后活跃时间 |

### 5.6 反馈表

反馈表（feedbacks）存储用户提交的反馈信息。

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | UUID | PK | 反馈唯一标识 |
| app_id | UUID | FK -> applications.id | 所属产品ID |
| user_id | UUID | FK -> end_users.id | 提交用户ID |
| type | ENUM | NOT NULL | 反馈类型：problem/suggestion/consult/complaint |
| title | VARCHAR(200) | NOT NULL | 反馈标题 |
| content | TEXT | NOT NULL | 反馈内容 |
| status | ENUM | NOT NULL | 状态：new/read/processing/pending/reolved/closed |
| priority | ENUM | DEFAULT normal | 优先级：low/normal/high/urgent |
| assigned_to | UUID | NULL | 分配给的用户ID |
| rating | INTEGER | NULL | 用户满意度评分（1-5） |
| created_at | TIMESTAMP | NOT NULL | 提交时间 |
| updated_at | TIMESTAMP | NOT NULL | 更新时间 |
| resolved_at | TIMESTAMP | NULL | 解决时间 |

### 5.7 反馈附件表

反馈附件表（feedback_attachments）存储反馈中的附件信息。

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | UUID | PK | 附件唯一标识 |
| feedback_id | UUID | FK -> feedbacks.id | 所属反馈ID |
| file_name | VARCHAR(255) | NOT NULL | 原始文件名 |
| file_type | VARCHAR(50) | NOT NULL | 文件MIME类型 |
| file_size | BIGINT | NOT NULL | 文件大小（字节） |
| file_url | VARCHAR(500) | NOT NULL | 文件存储URL |
| thumbnail_url | VARCHAR(500) | NULL | 缩略图URL（图片/视频） |
| storage_key | VARCHAR(255) | NOT NULL | 对象存储键名 |
| created_at | TIMESTAMP | NOT NULL | 上传时间 |

### 5.8 消息表

消息表（messages）存储用户与开发者之间的对话消息。

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | UUID | PK | 消息唯一标识 |
| feedback_id | UUID | FK -> feedbacks.id | 所属反馈ID |
| sender_type | ENUM | NOT NULL | 发送者类型：user/developer |
| sender_id | UUID | NOT NULL | 发送者ID |
| content | TEXT | NOT NULL | 消息内容 |
| message_type | ENUM | NOT NULL | 消息类型：text/image/file/audio |
| attachment_url | VARCHAR(500) | NULL | 附件URL |
| is_system | BOOLEAN | DEFAULT false | 是否系统消息 |
| read_at | TIMESTAMP | NULL | 阅读时间 |
| created_at | TIMESTAMP | NOT NULL | 发送时间 |

### 5.9 推送任务表

推送任务表（push_tasks）存储开发者创建的推送任务信息。

| 字段名 | 类型 | 约束 | 说明 |
|--------|------|------|------|
| id | UUID | PK | 任务唯一标识 |
| app_id | UUID | FK -> applications.id | 所属产品ID |
| creator_id | UUID | FK -> developers.id | 创建者ID |
| title | VARCHAR(200) | NOT NULL | 推送标题 |
| content | TEXT | NOT NULL | 推送内容 |
| target_type | ENUM | NOT NULL | 目标类型：user/tag/device/all |
| target_filter | JSONB | NULL | 目标筛选条件 |
| schedule_time | TIMESTAMP | NULL | 定时发送时间 |
| status | ENUM | NOT NULL | 任务状态：draft/scheduled/sending/completed/failed |
| total_count | INTEGER | DEFAULT 0 | 目标用户数 |
| sent_count | INTEGER | DEFAULT 0 | 已发送数 |
| delivered_count | INTEGER | DEFAULT 0 | 送达数 |
| clicked_count | INTEGER | DEFAULT 0 | 点击数 |
| created_at | TIMESTAMP | NOT NULL | 创建时间 |
| completed_at | TIMESTAMP | NULL | 完成时间 |

### 5.10 索引设计

为了保证查询性能，以下字段需要建立索引：

| 表名 | 索引字段 | 索引类型 | 说明 |
|------|----------|----------|------|
| developers | email | UNIQUE | 登录查询 |
| applications | app_key | UNIQUE | SDK认证 |
| end_users | app_id, device_fingerprint | UNIQUE | 用户识别 |
| feedbacks | app_id, status | INDEX | 反馈列表查询 |
| feedbacks | user_id, created_at | INDEX | 用户反馈历史 |
| messages | feedback_id, created_at | INDEX | 对话历史查询 |
| push_tasks | app_id, status, created_at | INDEX | 推送历史查询 |

---

## 6 SDK 架构设计

### 6.1 SDK 概述

FeedbackHub 提供多语言SDK，使开发者能够轻松将用户反馈功能集成到自己的应用程序中。目前支持三种SDK：JavaScript SDK（用于Web应用）、Swift SDK（用于iOS应用）、Kotlin SDK（用于Android应用）。

所有SDK遵循统一的设计理念：轻量级（SDK体积小，对应用性能影响微乎其微）、易集成（提供简洁的API，开发者只需几行代码即可完成集成）、可定制（支持丰富的配置选项，满足不同场景需求）、安全可靠（所有通信使用HTTPS加密，保护用户数据安全）。

### 6.2 JavaScript SDK

JavaScript SDK 用于 Web 应用，可以嵌入到任何基于 HTML/CSS/JavaScript 的网页中。

SDK 提供多种安装方式。npm 安装方式执行命令 npm install @feedbackhub/web-sdk，然后在代码中引入使用。CDN 引入方式在 HTML 中添加 script 标签引入 SDK 脚本。NPM 包导出 CommonJS、ES Module、UMD 三种模块格式，兼容各种构建工具。

初始化配置示例代码如下：

```javascript
import FeedbackHub from '@feedbackhub/web-sdk';

const feedback = new FeedbackHub({
  appId: 'YOUR_APP_ID',
  appSecret: 'YOUR_APP_SECRET',
  // 可选配置
  position: 'bottom-right', // 悬浮按钮位置
  themeColor: '#2563EB', // 主题颜色
  language: 'zh-CN', // 语言设置
  enableScreenshot: true, // 是否支持截图
  enableVoiceMessage: true, // 是否支持语音消息
  customFields: { // 自定义用户字段
    version: '1.0.0'
  }
});

// 初始化完成
feedback.init();
```

提交反馈示例代码如下：

```javascript
// 简单文本反馈
feedback.submit({
  type: 'problem',
  title: '页面加载缓慢',
  content: '首页加载时间超过10秒，影响使用体验'
}).then(result => {
  console.log('反馈提交成功', result.feedbackId);
}).catch(error => {
  console.error('提交失败', error.message);
});

// 带附件的反馈
feedback.submit({
  type: 'problem',
  title: '图片显示异常',
  content: '用户头像显示为黑屏',
  attachments: [
    await feedback.uploadFile(fileObject)
  ]
});

// 设置用户信息（可选）
feedback.setUserInfo({
  id: 'user_123',
  name: '张三',
  email: 'zhangsan@example.com'
});

// 监听消息
feedback.on('message', (message) => {
  console.log('收到新消息', message);
});
```

### 6.3 Swift SDK

Swift SDK 用于 iOS 应用，支持 Swift 5.0 以上版本，推荐使用 SwiftUI 或 UIKit 开发的项目集成。

SDK 通过 Swift Package Manager 安装，在 Xcode 中选择 File -> Swift Packages -> Add Package Dependency，输入仓库地址即可。也可以通过 CocoaPods 安装，在 Podfile 中添加 pod 'FeedbackHub'。

初始化配置在 AppDelegate 或 SceneDelegate 中进行：

```swift
import FeedbackHub

func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
    
    FeedbackHub.shared.configure(
        appId: "YOUR_APP_ID",
        appSecret: "YOUR_APP_SECRET",
        options: FHConfigOptions(
            position: .bottomRight,
            themeColor: UIColor(red: 0.145, green: 0.388, blue: 0.922, alpha: 1.0),
            enableScreenshot: true,
            enableVoiceMessage: true,
            language: .simplifiedChinese
        )
    )
    
    return true
}
```

提交反馈的示例代码如下：

```swift
import FeedbackHub

// 简单文本反馈
FeedbackHub.shared.submitFeedback(
    type: .problem,
    title: "应用崩溃",
    content: "在打开设置页面时应用闪退",
    metadata: [
        "scene": "settings",
        "crash_log": "..."
    ]
) { result in
    switch result {
    case .success(let feedbackId):
        print("反馈提交成功: \(feedbackId)")
    case .failure(let error):
        print("提交失败: \(error.localizedDescription)")
    }
}

// 带图片附件的反馈
FeedbackHub.shared.submitFeedback(
    type: .problem,
    title: "界面显示问题",
    content: "登录按钮文字显示不全",
    attachments: [imageData]
)

// 上传文件
FeedbackHub.shared.uploadFile(data: imageData, fileName: "screenshot.png") { result in
    // 获取文件URL后提交反馈
}

// 监听实时消息
FeedbackHub.shared.onMessageReceived = { message in
    print("收到新消息: \(message.content)")
}
```

### 6.4 Kotlin SDK

Kotlin SDK 用于 Android 应用，支持 Kotlin 1.6 以上版本，适用于使用 Jetpack Compose 或传统 XML 布局的项目。

SDK 通过 Gradle 安装，在 app 模块的 build.gradle 中添加依赖：

```groovy
dependencies {
    implementation 'com.feedbackhub:android-sdk:1.0.0'
}
```

在 AndroidManifest.xml 中添加必要权限：

```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.RECORD_AUDIO" />
```

初始化配置在 Application 类中进行：

```kotlin
import com.feedbackhub.sdk.FeedbackHub
import com.feedbackhub.sdk.FeedbackConfig

class MyApplication : Application() {
    
    override fun onCreate() {
        super.onCreate()
        
        FeedbackHub.init(
            context = this,
            appId = "YOUR_APP_ID",
            appSecret = "YOUR_APP_SECRET",
            config = FeedbackConfig.Builder()
                .setPosition(Position.BOTTOM_RIGHT)
                .setThemeColor(0xFF2563EB.toInt())
                .setEnableScreenshot(true)
                .setEnableVoiceMessage(true)
                .setLanguage(Language.SIMPLIFIED_CHINESE)
                .build()
        )
    }
}
```

提交反馈的示例代码如下：

```kotlin
import com.feedbackhub.sdk.FeedbackHub
import com.feedbackhub.sdk.FeedbackType
import com.feedbackhub.sdk.model.FeedbackResult

// 简单文本反馈
FeedbackHub.submitFeedback(
    type = FeedbackType.PROBLEM,
    title = "应用无响应",
    content = "点击登录按钮后应用卡住不动",
    metadata = mapOf(
        "screen" to "LoginActivity",
        "device_model" to Build.MODEL
    ),
    callback = object : FeedbackResult.Callback {
        override fun onSuccess(feedbackId: String) {
            Log.d("FeedbackHub", "反馈提交成功: $feedbackId")
        }
        
        override fun onError(error: Exception) {
            Log.e("FeedbackHub", "提交失败: ${error.message}")
        }
    }
)

// 使用协程
suspend fun submitFeedback() {
    try {
        val feedbackId = FeedbackHub.submitFeedback(
            type = FeedbackType.SUGGESTION,
            title = "希望增加深色模式",
            content = "晚上使用白色界面太刺眼，希望增加深色模式"
        )
        Log.d("FeedbackHub", "反馈提交成功: $feedbackId}")
    } catch (e: Exception) {
        Log.e("FeedbackHub", "提交失败: ${e.message}")
    }
}

// 监听实时消息
FeedbackHub.setOnMessageListener { message ->
    Log.d("FeedbackHub", "收到新消息: ${message.content}")
}
```

### 6.5 SDK 通用架构

所有SDK采用统一的架构设计，确保功能一致性和代码可维护性。

SDK 核心组件包括：配置管理模块（ConfigManager）负责管理 SDK 的初始化配置，包括应用ID、密钥、主题样式等；网络模块（NetworkClient）负责与服务端进行 HTTP/WebSocket 通信，包括请求封装、响应解析、错误处理等；本地存储模块（Storage）负责缓存用户信息、对话记录等数据，支持离线查看；附件处理模块（AttachmentHandler）负责处理图片、文件、语音等附件的上传和下载；实时通信模块（RealtimeClient）负责维护 WebSocket 连接，处理实时消息推送。

SDK 与服务端通信采用 HTTPS 协议，所有请求都经过签名验证以确保安全性。附件上传使用独立的文件服务，采用分片上传支持大文件。网络请求支持重试机制，在网络不稳定时保证可靠性。

---

## 7 开发者 App 设计

### 7.1 开发者 App 概述

FeedbackHub 开发者 App 是专门为开发者设计的移动端应用，使开发者能够随时随地接收用户反馈并与用户进行实时交流。App 支持 iOS 和 Android 双平台，同时提供 Web 版 PWA 应用，满足不同场景的使用需求。

开发者 App 的核心价值在于：即时通知（新反馈或新消息时立即推送通知）、快速响应（在移动端直接回复用户消息）、随时管理（在手机上查看反馈列表、处理反馈、查看统计）。

### 7.2 功能模块

开发者 App 包含以下主要功能模块：

首页概览模块显示今日反馈概览，包括新增反馈数量、待处理数量、已解决数量；今日消息概览，包括新消息数、未读数；本周趋势图表，展示反馈数量变化趋势；快捷操作按钮，如查看最新反馈、回复待处理消息等。

反馈管理模块是 App 的核心功能。反馈列表展示所有反馈，支持按状态、类型、时间筛选；反馈详情展示反馈内容、附件、对话历史；反馈操作包括标记已读、分配给自己、改变状态、添加备注；批量操作支持批量标记状态、批量分配等。

消息交流模块提供完整的实时通讯体验。消息列表展示所有对话会话，按最新消息排序；实时聊天界面支持发送文本、图片、文件、语音消息；消息状态显示已发送、已送达、已读状态；输入状态显示对方正在输入的提示。

推送管理模块允许开发者向用户发送推送。推送创建界面支持选择推送目标、编辑推送内容、设置发送时间；推送历史展示已发送的推送任务及统计数据；推送模板保存常用推送内容，快速创建新推送。

统计分析模块提供数据分析和可视化展示。核心指标卡片展示关键数据；趋势图表展示日周月变化趋势；分布图表展示反馈类型、设备分布等；数据导出支持导出为 CSV 或 PDF 格式。

消息推送模块用于接收系统通知。新消息提醒在收到新反馈或消息时推送通知；处理提醒在反馈被分配或状态变更时推送；系统公告接收平台发布的更新公告和通知。

### 7.3 界面设计

开发者 App 的界面设计遵循以下原则：简洁高效（减少操作步骤，快速完成任务）、信息密度适中（在有限屏幕空间内展示关键信息）、一致性与识别性（统一的设计语言，清晰的视觉层级）。

首页采用卡片式布局，将不同类型的信息分块展示。顶部是今日概览卡片，简洁展示关键数据；中部是快捷操作区和趋势图表；底部是最新反馈和消息列表。

反馈列表采用列表-详情架构。列表页使用下拉刷新和上拉加载更多；每条反馈卡片展示标题、状态标签、时间、操作按钮；支持左滑快捷操作（标记、分配）；详情页使用分段视图展示反馈内容、对话历史、附件、操作日志。

消息界面采用即时通讯应用的标准设计。会话列表使用头像、用户名、最新消息、时间、未读数布局；聊天页面使用气泡消息样式区分发送方和接收方；底部输入区域支持文本输入、附件按钮、语音按钮。

### 7.4 技术架构

iOS 端采用 SwiftUI 作为主要 UI 框架，响应式声明式 UI 开发效率高。架构模式采用 MVVM + Combine，数据层使用 Repository 模式。网络层使用 Alamofire + Moya。实时通信使用 Starscream 处理 WebSocket。图片加载使用 Kingfisher。存储使用 UserDefaults + SQLite。

Android 端采用 Jetpack Compose 作为主要 UI 框架。架构模式采用 MVVM + Kotlin Coroutines + Flow。依赖注入使用 Hilt。网络层使用 Retrofit + OkHttp。实时通信使用 OkHttp WebSocket。图片加载使用 Coil。存储使用 Room + DataStore。

Web 端采用 React + TypeScript 开发。UI 组件库可以使用 Material UI 或 Ant Design。状态管理使用 Redux Toolkit 或 Zustand。PWA 支持使用 Workbox 实现离线能力和推送通知。实时通信使用 Socket.io-client。

---

## 8 安全方案设计

### 8.1 传输安全

所有客户端与服务端之间的通信必须使用 HTTPS 协议，确保数据在传输过程中加密。TLS 版本要求 1.2 及以上，禁止使用存在安全漏洞的旧版本。证书使用权威 CA 签发的 SSL 证书，自签名证书仅用于开发环境。

WebSocket 连接同样使用 WSS 协议（WebSocket Secure），确保实时消息传输安全。SDK 与服务端的通信也必须使用 HTTPS，禁用 HTTP 明文传输。

### 8.2 身份认证

开发者账户认证采用 JWT（JSON Web Token）机制。用户登录成功后获取 Access Token 和 Refresh Token。Access Token 有效期较短（24 小时），用于请求需要认证的 API。Refresh Token 有效期较长（7 天），用于刷新 Access Token。

SDK 认证使用 App Key + App Secret 机制。每个产品项目拥有唯一的 App Key 和 App Secret。SDK 初始化时需要配置这些凭证，请求时在 HTTP 头中携带进行认证。服务端对请求进行签名验证，防止凭证泄露后被滥用。

设备指纹认证用于识别匿名用户。SDK 收集设备的多维度信息生成设备指纹，包括设备型号、操作系统版本、屏幕分辨率、已安装字体列表等。设备指纹用于识别同一设备的多次访问，但不会收集个人隐私信息。

### 8.3 数据安全

敏感数据加密存储。用户密码使用 bcrypt 或 Argon2 算法加密存储，确保即使数据库泄露也无法还原明文密码。App Secret 使用 AES-256 对称加密存储，密钥通过环境变量或密钥管理服务获取。

用户数据隔离确保不同产品之间的数据完全隔离。每个产品只能访问自己产品下的反馈和用户数据。团队成员只能访问自己有权限的产品数据。跨产品数据访问需要超级管理员授权。

数据备份与恢复是数据安全的重要组成部分。数据库每日自动备份，备份文件加密存储。备份保留策略为最近 30 天的每日备份、最近 12 个月的每周备份。提供数据恢复流程文档和演练。

### 8.4 访问控制

基于角色的访问控制（RBAC）确保用户只能访问被授权的资源。开发者角色包括所有者、管理员、开发者、查看者四种。每种角色有对应的权限集合。权限控制精确到接口级别，未授权请求会被拒绝。

API 访问频率限制防止恶意请求和资源滥用。登录接口限制为每分钟 5 次。普通 API 限制为每分钟 60 次。推送接口根据产品套餐限制每日推送次数。超出限制返回 429 状态码并提示重试。

审计日志记录关键操作便于安全追溯。记录内容包括用户登录登出、敏感数据查询、重要数据修改、权限变更等。日志保留至少 180 天，支持查询和导出。

### 8.5 隐私保护

用户隐私保护是系统设计的重要考量。收集用户设备信息时遵守最小必要原则，仅收集功能必需的信息。用户反馈可以匿名提交，无需提供个人身份信息。提供用户数据删除功能，用户可以请求删除自己的数据。

GDPR 合规功能支持数据主体权利。用户可以查看自己被收集的数据。可以导出自己的数据为标准格式。可以请求删除个人数据。可以撤回数据处理同意。

---

## 9 部署方案

### 9.1 部署模式

FeedbackHub 支持多种部署模式以适应不同场景需求。

单体部署模式适合个人开发者或小规模试用。所有服务组件部署在同一台服务器上，使用 Docker Compose 进行容器编排。硬件要求最低为 2 核 CPU、4GB 内存、40GB SSD 硬盘。这种模式部署简单，维护成本低，但扩展性有限。

微服务部署模式适合团队使用。将 API 服务、数据库、缓存、存储分离部署。API 服务可以水平扩展以应对更高并发。使用 Kubernetes 或 Docker Swarm 进行容器编排。这种模式具有更好的扩展性和可靠性，但部署维护复杂度较高。

### 9.2 Docker Compose 部署

以下是单体部署模式使用的 docker-compose.yml 配置示例：

```yaml
version: '3.8'

services:
  # PostgreSQL 数据库
  postgres:
    image: postgres:15-alpine
    container_name: feedbackhub-db
    environment:
      POSTGRES_USER: feedbackhub
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: feedbackhub
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U feedbackhub"]
      interval: 10s
      timeout: 5s
      retries: 5

  # Redis 缓存和消息队列
  redis:
    image: redis:7-alpine
    container_name: feedbackhub-redis
    command: redis-server --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"

  # MinIO 对象存储
  minio:
    image: minio/minio:latest
    container_name: feedbackhub-minio
    environment:
      MINIO_ROOT_USER: ${MINIO_USER}
      MINIO_ROOT_PASSWORD: ${MINIO_PASSWORD}
    volumes:
      - minio_data:/data
    ports:
      - "9000:9000"
      - "9001:9001"
    command: server /data --console-address ":9001"

  # API 服务
  api:
    image: feedbackhub/api:latest
    container_name: feedbackhub-api
    environment:
      DATABASE_URL: postgresql://feedbackhub:${DB_PASSWORD}@postgres:5432/feedbackhub
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379
      MINIO_ENDPOINT: minio:9000
      MINIO_ACCESS_KEY: ${MINIO_USER}
      MINIO_SECRET_KEY: ${MINIO_PASSWORD}
      JWT_SECRET: ${JWT_SECRET}
      API_SECRET: ${API_SECRET}
    ports:
      - "3000:3000"
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started

  # Web 前端
  web:
    image: feedbackhub/web:latest
    container_name: feedbackhub-web
    ports:
      - "80:80"
      - "443:443"

volumes:
  postgres_data:
  redis_data:
  minio_data:
```

### 9.3 环境配置

部署前需要配置以下环境变量：

数据库配置变量包括 DB_PASSWORD（PostgreSQL 密码）、REDIS_PASSWORD（Redis 密码）。存储配置变量包括 MINIO_USER（MinIO 访问密钥）、MINIO_PASSWORD（MinIO 秘密密钥）。安全配置变量包括 JWT_SECRET（JWT 签名密钥，建议使用 32 位以上随机字符串）、API_SECRET（API 内部通信密钥）。

建议将敏感信息存储在 .env 文件中，不要提交到代码仓库。生产环境建议使用 Docker Secret 或 Kubernetes Secret 管理敏感信息。

### 9.4 运维监控

系统监控是保证服务稳定运行的重要手段。

健康检查配置：API 服务提供 /health 端点，返回服务健康状态。使用 Docker HEALTHCHECK 或 Kubernetes Probe 配置健康检查。定时检查及时发现服务异常。

日志管理采用结构化日志格式，便于查询和分析。日志输出到标准输出，由容器运行时收集。使用 ELK（Elasticsearch + Logstash + Kibana）或 Loki + Grafana 进行日志收集和展示。设置日志保留策略，避免磁盘空间耗尽。

性能监控包括系统指标监控（CPU、内存、磁盘、网络）和应用指标监控（请求延迟、错误率、并发数）。使用 Prometheus + Grafana 搭建监控平台。设置告警规则，异常时及时通知。

---

## 10 后续规划

### 10.1 功能迭代路线图

 FeedbackHub 系统将持续迭代，以下是主要的功能规划：

近期版本（v1.1-v1.2）计划增加多语言支持，扩大SDK支持范围。增加反馈标签和分类功能，优化反馈管理效率。增加满意度评价功能，收集用户对处理结果的评价。完善统计分析功能，提供更丰富的数据报表。

中期版本（v1.3-v1.5）计划支持与第三方工具集成，包括 Slack、Discord、飞书、钉钉等。增加工单系统功能，支持更复杂的问题处理流程。增加自动化规则功能，支持自动分配、自动回复等。增加小程序支持，扩展SDK覆盖范围。

长期版本（v2.0+）计划提供 AI 辅助功能，使用 AI 自动分类反馈、生成回复建议。支持大规模企业部署，提供集群模式和高可用方案。开放平台能力，支持第三方插件扩展功能。

### 10.2 开源生态

 FeedbackHub 作为开源项目，欢迎社区贡献。

代码贡献方面，欢迎提交 Pull Request 修复问题或添加新功能。请遵循项目的代码规范和提交流程。所有的贡献者都会在项目文档中致谢。

文档完善方面，欢迎完善用户文档、API 文档、部署文档等。可以通过提交 Issue 指出文档问题，或直接提交文档改进。

社区支持方面，项目维护者会定期回复 GitHub Issue。会在社区讨论组中解答用户问题。会持续发布版本更新和安全补丁。

---

*文档版本：v1.0.0*
*最后更新：2026年2月21日*
