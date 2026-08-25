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

## Run the prototype

Install Rust, Node.js 22+, and pnpm 11+, then:

```bash
pnpm install
pnpm dev
```

Open <http://localhost:5173>. The browser prototype starts with representative vocabulary data and supports capture, Undo, review, vocabulary search, context detail, progress, and settings.

For the native macOS application with durable SQLite storage:

```bash
pnpm tauri dev
```

The first native capture is entered through **Quick capture**. System-wide Accessibility selection and Apple Translation are intentionally isolated behind platform interfaces and are the next native milestone.

Run all verification:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check
pnpm test
pnpm build
```
