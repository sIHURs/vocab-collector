# Windows Development Log: Achieved Vocabulary Lifecycle

## 2026-09-05 - Ticket 01: Manual Achieve

### Implemented

- Added portable Mastered/Achieved lifecycle timestamps without extending
  `WordStatus`.
- Added legacy-safe retention Settings and SQLite schema version 4 migration.
- Added the single-item Achieve application and desktop command path.
- Added `Vocabulary > Active | Mastered | Achieved`, single-item actions,
  confirmation with the deletion date, and the basic Achieved view.

### Key design decisions

- Learning status and retirement lifecycle remain separate.
- Eligibility and deadlines are enforced in shared Rust; Windows renders the
  result and collects confirmation.
- Existing Mastered rows are backfilled at migration time to avoid immediate
  future automation.

### Main files

- `crates/domain/src/models.rs`, `crates/domain/src/views.rs`
- `crates/storage/src/lib.rs`
- `crates/application/src/lib.rs`
- `apps/desktop/src-tauri/src/commands/library.rs`
- `ui/src/lib/types.ts`, `ui/src/lib/backend.ts`
- `ui/src/windows/WindowsApp.svelte`, `WindowsWordRow.svelte`

### Tests and results

- Domain red-green tests for eligibility and retention: PASS.
- Application single-item Achieve test: PASS.
- `cargo test -p vocab-domain -p vocab-storage -p vocab-application`: PASS.
- `cargo test -p vocab-desktop`: PASS. The ticket listed the non-existent
  package name `vocab-desktop-lib`; the actual package is `vocab-desktop`.
- `pnpm --dir ui test`: PASS, 77 tests.
- `pnpm --dir ui check`: PASS, no diagnostics.
- `cargo check --workspace`: PASS.

### Not yet verified

- No Windows 11 physical WebView interaction or restart persistence is claimed.

### Next ticket starting point

- Ticket 02 starts from persisted Achieved items and adds complete filtered
  selection, atomic batch Unachieve, and confirmed permanent purge.
