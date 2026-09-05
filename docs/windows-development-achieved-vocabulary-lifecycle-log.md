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

## 2026-09-05 - Ticket 02: Batch management and permanent purge

### Implemented

- Added deadline-ordered Achieved search, filtered select-all, batch
  Unachieve, short-lived Undo, and confirmed permanent deletion.
- Added atomic SQLite batch validation and mutation.
- Added compact lifetime archive counters before deleting item-level Encounter
  and Review history.
- Prevented a repeated capture from silently adding an Encounter to a still
  Achieved Vocabulary Item.

### Key design decisions

- Every selected ID is validated before any batch write.
- Permanent deletion and lifetime aggregation share one transaction.
- The UI confirmation names the selected count and irrecoverable history loss.

### Main files

- `crates/storage/src/lib.rs`
- `crates/application/src/lib.rs`
- `apps/desktop/src-tauri/src/commands/library.rs`
- `ui/src/lib/backend.ts`
- `ui/src/windows/WindowsApp.svelte`

### Tests and results

- Application batch lifecycle test: PASS.
- Focused `WindowsApp.test.ts`: PASS, 25 tests.
- Shared Rust suite: PASS.
- Full UI suite before the focused addition: PASS, 77 tests.

### Not yet verified

- Native confirmation focus, forced-colors rendering, and physical SQLite file
  size are not verified on a Windows 11 physical machine.

### Next ticket starting point

- Ticket 03 can invoke the same transition and purge primitives from the
  opt-in automatic lifecycle sweep.

## 2026-09-05 - Ticket 03: Automatic Achieve and expiry

### Implemented

- Added the opt-in automatic Achieve setting and 10/20/30/60-day retention
  selector to the Windows Settings presentation.
- Added a portable lifecycle sweep that Achieves eligible Mastered items and
  purges expired Achieved items with bounded result counts.
- Invoked the same sweep at startup, before Vocabulary refreshes, and once per
  local calendar day while the Windows process remains alive.
- Added a Windows summary notification adapter without moving eligibility into
  Windows code.

### Key design decisions

- The exact 30-day decision uses explicit UTC instants in shared domain code.
- Daily wake-up detects a local date change, avoiding a drifting 24-hour timer.
- Existing deletion deadlines remain immutable after Settings changes.

### Main files

- `crates/domain/src/models.rs`, `crates/domain/src/views.rs`
- `crates/application/src/lib.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src-tauri/src/commands/library.rs`
- `ui/src/windows/WindowsApp.svelte`

### Tests and results

- Domain exact-boundary test: PASS.
- Application opt-in, expiry, opt-out, and immutable-deadline tests: PASS.
- Focused `WindowsApp.test.ts`: PASS, 25 tests.
- Full required verification is run again before Ticket 03 commit.

### Not yet verified

- Native notification delivery, local-date wake-up, and restart catch-up are
  not marked Windows 11 physical verified.

### Next ticket starting point

- Ticket 04 starts with the complete automated feature and records real
  Windows 11 WebView, notification, restart, and SQLite evidence.
