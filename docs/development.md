# Development

Requirements:

- Rust 1.98+
- Node.js 22+
- pnpm 11+
- macOS is required for the native Tauri shell; the UI can run in any supported browser.

## Setup

```bash
pnpm install
cargo fetch
```

## Development modes

- `pnpm dev` runs the Svelte interface at `http://localhost:5173` with an isolated in-memory backend and seeded review data. Use this for quick UI work.
- `pnpm tauri dev` runs the real desktop boundary. Captures and reviews are stored in `guest.db` under the macOS application-data directory.
- `pnpm build` creates the optimized browser assets consumed by Tauri.

The browser backend and Rust application service implement the same frontend-facing contract. New UI interactions must be added to that contract and the Tauri command layer together.

## Development-only Azure Translator

Windows debug builds may use a developer-owned Azure Translator resource. Copy
the empty `.env.example` to an untracked workspace-root `.env.local`, then set
`VOCAB_AZURE_TRANSLATOR_KEY`. The endpoint defaults to Azure's global endpoint;
set `VOCAB_AZURE_TRANSLATOR_REGION` only when the resource requires it and adjust
`VOCAB_AZURE_TRANSLATOR_TIMEOUT_MS` only when needed. Existing process environment
variables take precedence over `.env.local`.

Missing configuration keeps translation unavailable. Partial or invalid
configuration fails safely during startup without exposing configured values.
Release builds do not load `.env.local`. Never commit that file or put credentials
in frontend state, logs, screenshots, test output, or documentation.

All default tests use local mock HTTP servers and do not contact Azure. On the
target Windows physical machine, a developer may explicitly run the ignored
fixed-text smoke test after setting credentials:

```powershell
cargo test -p vocab-translation-azure --test live_translation -- --ignored
```

That provider smoke test alone does not physically verify the capture window or
OCR workflow. Accounts, proxying, quota ownership, release credentials,
installers, and production readiness remain out of scope for this development path.

## Core debugging

Run the lightweight core REPL without Tauri or a platform adapter:

```powershell
cargo run -p vocab-application --example core_debug
```

It uses session-local in-memory SQLite unless `--db <path>` is supplied. See
the [core debug REPL guide](core-debug-repl.md) for copyable capture,
deduplication, review, Undo, settings, database-summary, outbox, and
error-isolation cases. The REPL validates shared core behavior only; it does
not establish native adapter support.

## Quality gates

On macOS, exclude the Linux and Windows adapter skeletons from native Rust
checks. Their compilation and automated tests run on physical target hosts or
the corresponding target-specific CI jobs. Runtime, permission, and native
integration verification runs only on the physical target machines.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --exclude vocab-platform-linux --exclude vocab-platform-windows -- -D warnings
cargo test --workspace --exclude vocab-platform-linux --exclude vocab-platform-windows
pnpm check
pnpm test
pnpm build
```

## Supabase

The migration in `supabase/migrations` creates the account-scoped replication schema, indexes, and row-level security policies. The prototype does not require a Supabase project. Authentication and the delta-sync transport can be connected later without changing local capture or review.
