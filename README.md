# Vocab Collector

A lightweight, local-first vocabulary coach built with Tauri 2, Rust, Svelte 5,
SQLite, and an optional Supabase synchronization boundary. The macOS adapter is
implemented; Linux and Windows currently have honest unsupported-capability
skeletons for Plan B development on their physical target machines.

## Workspace

- `apps/desktop` — Tauri shell and typed commands
- `ui` — Svelte interface and design system
- `crates/domain` — entities, normalization, and review rules
- `crates/application` — shared product use cases and platform capture workflow
- `crates/capture` — capture state machine, request identity, and placement
- `crates/storage` — SQLite migrations and repositories
- `crates/platform-api` — portable DTOs, errors, capabilities, and small provider traits
- `crates/platform-contract-tests` — reusable provider contract assertions and fakes
- `crates/sync` — synchronization model and orchestration
- `platform/macos/native` — Swift bridge for Accessibility, Translation, ScreenCaptureKit, Vision, permissions, and AppKit
- `platform/macos/rust` — safe Rust adapter around the Swift JSON/C ABI
- `platform/linux` and `platform/windows` — static Plan B adapter skeletons; native behavior is not implemented or verified
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

Run the macOS verification gate (the explicit exclusions keep the Plan B
Linux/Windows skeleton packages out of local native checks):

```bash
swift test --package-path platform/macos/native
cargo fmt --all --check
cargo clippy --workspace --all-targets --exclude vocab-platform-linux --exclude vocab-platform-windows -- -D warnings
cargo test --workspace --exclude vocab-platform-linux --exclude vocab-platform-windows
cargo build --workspace --exclude vocab-platform-linux --exclude vocab-platform-windows
pnpm check
pnpm test
pnpm build
pnpm tauri build --bundles app
```
