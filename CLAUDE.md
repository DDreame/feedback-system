# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A user feedback system for indie developers. Users chat with developers via embedded SDKs (Web/Flutter); developers manage conversations, issues, and schedules through an admin panel. An AI agent auto-triages and answers common questions.

**Current status:** Early Phase 1 (MVP). Tasks 1.1.1–1.1.3 complete (project init, config module, error handling). See `progress.txt` for up-to-date status and `plans/tdd-development-plan.md` for the full task breakdown.

## Architecture

Modular monolith with these planned components:

| Component | Tech | Location |
|-----------|------|----------|
| Backend | Rust (Axum + Tokio + sqlx) | `backend/` |
| Web SDK | TypeScript | `sdks/web/` (not yet created) |
| Flutter SDK | Dart | `sdks/flutter/` (not yet created) |
| Admin Panel | React + TypeScript | `admin-panel/` (not yet created) |
| Test Page | React + TypeScript | `test-page/` (not yet created) |

Infrastructure: PostgreSQL, Redis, S3-compatible storage, Docker Compose.

The backend is a single Axum service with layered modules: `api/` (routes) → `service/` (business logic) → `model/` (data types) → `db/` (persistence). See `docs/architecture.md` for full details.

## Build & Test Commands

### Backend (Rust)

```bash
cd backend

# Build
cargo build

# Run all tests
cargo test

# Run specific module tests
cargo test --lib service::auth

# Run integration tests
cargo test --test integration_tests

# Lint
cargo clippy
cargo fmt --check
```

### Frontend (TypeScript/React) — when created

```bash
pnpm test              # run all tests
pnpm test -- --watch   # watch mode
pnpm test -- --coverage
```

## Development Methodology — TDD

This project strictly follows Test-Driven Development. **Never write implementation code without a failing test first.**

1. **RED**: Write a failing test that defines expected behavior
2. **GREEN**: Write the minimum code to pass the test
3. **REFACTOR**: Clean up under test protection

Each TDD cycle = one commit.

## Commit Convention

```
<type>(<scope>): <subject>
```

**Types:** `feat`, `fix`, `test`, `refactor`, `docs`, `chore`, `ci`
**Scopes:** `backend`, `web-sdk`, `flutter-sdk`, `admin-panel`, `test-page`, `docs`, `db`

Examples:
```
test(backend): add failing test for developer registration
feat(backend): implement developer registration endpoint
feat(db): add developers table migration
```

## Key Technical Decisions

- **IDs**: UUIDv7 (ordered, sortable)
- **Error handling**: `thiserror` → `AppError` enum → auto-converts to HTTP responses with `{ "error": { "code": N, "message": "..." } }`
- **Config**: `AppConfig::from_env()` with `dotenvy`. Required: `DATABASE_URL`, `JWT_SECRET`. Optional vars have defaults (see `backend/src/config.rs`)
- **Auth**: JWT for developers, API Key (`proj_` prefix) for SDK endpoints, anonymous `device_id` for end users
- **Passwords**: argon2 hashing
- **DB migrations**: sqlx-cli, files named `YYYYMMDDHHMMSS_description.sql`
- **Rust tests**: unit tests inline (`#[cfg(test)]`), integration tests in `backend/tests/`
- **Frontend tests**: colocated with source files (e.g., `Component.test.tsx`)

## Environment Requirements

- Rust >= 1.75 (stable), edition 2024
- Node.js >= 20 LTS, pnpm >= 9
- PostgreSQL >= 15, Redis >= 7
- Docker + Docker Compose
