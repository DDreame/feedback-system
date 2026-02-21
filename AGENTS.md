# PROJECT KNOWLEDGE BASE

**Generated:** 2026-02-21
**Project:** FeedbackHub - User Feedback & Communication System

## OVERVIEW

FeedbackHub is an open-source user feedback and communication system for independent developers and small teams. The system provides multi-language SDKs (JavaScript, Swift, Kotlin) for embedding feedback functionality into applications, plus a developer mobile app (Flutter) for receiving and responding to user feedback in real-time.

## STRUCTURE

```
feedback/
├── docs/                     # System design documents
│   └── FeedbackHub-Technical-Doc.md    # Technical specification
├── plans/                   # Development task planning
│   └── feedbackhub-tasks.md            # Task breakdown
└── design/                  # UI/UX design files (empty)
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| System architecture | docs/FeedbackHub-Technical-Doc.md | Complete technical spec |
| API design | docs/ (Section 4) | RESTful API definitions |
| Database schema | docs/ (Section 5) | SQLite/PostgreSQL design |
| SDK design | docs/ (Section 6) | JS/Swift/Kotlin SDKs |
| Development tasks | plans/feedbackhub-tasks.md | 109 tasks, ~76 days |
| Deploy config | docs/ (Section 7) | All-in-One binary + distributed |
| **TDD 开发规范** | docs/TDD-Development-Guideline.md | **强制测试驱动开发** |

## CODE MAP

**Project Status:** Greenfield - No code yet. Only documentation.

| Component | Status | Location |
|-----------|--------|----------|
| Backend (Rust) | Planned | TBD |
| Flutter App | Planned | TBD |
| JS SDK | Planned | TBD |
| Swift SDK | Planned | TBD |
| Kotlin SDK | Planned | TBD |

## KEY DECISIONS

| Area | Decision | Rationale |
|------|----------|-----------|
| Backend | Rust + Axum | High performance, low memory, async |
| App | Flutter | Cross-platform (iOS/Android/Web) |
| Storage | SQLite (single) / PostgreSQL (cluster) | Embedded for单机, RDBMS for scaling |
| Cache | sled (single) / Redis (cluster) | Embedded KV for单机 |
| Storage (files) | Local (single) / MinIO (cluster) | Object storage for attachments |
| Deployment | All-in-One binary | Single command run, no Docker dependency |

## CONVENTIONS

- **TDD 强制**: 所有功能必须先写测试，测试通过后才能提交
- **部署模式**: Single binary (All-in-One) for单机, distributed for scaling
- **存储抽象**: Support both embedded (SQLite/sled) and distributed (PostgreSQL/Redis) backends
- **SDK parity**: All SDKs (JS/Swift/Kotlin) implement identical functionality

## ANTI-PATTERNS

- **禁止跳过测试**: 测试不通过 = 任务未完成
- **禁止注释测试**: 不允许通过注释测试来"通过"CI
- No external service dependencies for单机 mode
- No Docker Compose required for basic deployment

## COMMANDS

```bash
# Not applicable yet - greenfield project
# Will be added when code is implemented
```

## NOTES

- This is a documentation-first project - system design completed before code
- docs/ contains complete technical specification ready for implementation
- plans/ contains detailed task breakdown with priorities
- Next step: Initialize Rust backend project
