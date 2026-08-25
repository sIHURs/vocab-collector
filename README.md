# Vocab Collector

A lightweight, local-first vocabulary coach for macOS, built with Tauri 2, Rust, Svelte 5, SQLite, and an optional Supabase synchronization boundary.

## Workspace

- `apps/desktop` — Tauri shell and typed commands
- `ui` — Svelte interface and design system
- `crates/domain` — entities, normalization, review rules, and use cases
- `crates/storage` — SQLite migrations and repositories
- `crates/platform` — platform provider contracts
- `crates/sync` — synchronization model and orchestration
- `platform/macos` — Swift bridge for Accessibility, Translation, ScreenCaptureKit, and Vision
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

Select text in Safari, Preview, Books, or another accessible application and press **⌥ ⇧ V**. The first attempt prompts for Accessibility permission. The floating window translates, saves, and dismisses automatically. If an application does not expose its selection, choose the explicit OCR fallback.

See `docs/native-capture-development.md` for native architecture, permissions, troubleshooting, and packaging.

Run all verification:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check
pnpm test
pnpm build
```
