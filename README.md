# Vocab Collector

A lightweight, local-first vocabulary coach for macOS, built with Tauri 2, Rust, Svelte 5, SQLite, and an optional Supabase synchronization boundary.

## Workspace

- `apps/desktop` — Tauri shell and typed commands
- `ui` — Svelte interface and design system
- `crates/domain` — entities, normalization, review rules, and use cases
- `crates/storage` — SQLite migrations and repositories
- `crates/platform` — platform provider contracts
- `crates/sync` — synchronization model and orchestration
- `platform/macos` — Swift bridge boundary for future native capture providers
- `supabase` — production database schema and row-level security
- `docs` — architecture and development documentation

Implementation follows the approved plan in the parent workspace. See `docs/architecture.md` for boundaries and `docs/development.md` for local commands.

