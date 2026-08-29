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
