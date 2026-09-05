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

## 2026-09-05 - Ticket 04: Windows 11 physical verification

### Implemented and observed

- Confirmed the host reports Windows 11 Pro build 26100 on an HP Pavilion
  Gaming Laptop 15-dk1xxx.
- Started the real Tauri development executable successfully and stopped it
  cleanly after startup inspection.
- Confirmed the app-owned SQLite database migrated to schema version 4,
  contains all three lifecycle columns, and passes `PRAGMA integrity_check`.
- Added an automated populated-version-3 migration test proving a legacy
  Mastered item receives `masteredAt` without becoming Achieved.

### Key design decisions

- Windows physical host identity and process startup are evidence, but they do
  not prove controls that this session could not operate or observe.
- The Codex computer-use surface exposed no native app windows, so native
  WebView claims remain deliberately unchecked.

### Main files

- `crates/storage/tests/sqlite_repository.rs`
- `.scratch/achieved-vocabulary-lifecycle/issues/04-windows-11-lifecycle-verification.md`

### Tests and results

- `cargo test -p vocab-storage legacy_mastered_items_receive_migration_time`:
  PASS.
- `pnpm --dir ui test`: PASS, 78 tests.
- `pnpm --dir ui check`: PASS.
- `pnpm --dir ui build`: PASS.
- `cargo check --workspace`: PASS.
- `cargo test --workspace`: FAIL only while linking the existing macOS native
  FFI adapter test on Windows (`LNK2019` for `vocab_mac_*` symbols). Focused
  shared Rust and Windows desktop suites pass.
- `pnpm tauri dev`: PASS for compilation, process launch, schema migration, and
  startup only.

### Not yet verified

- Manual Achieve, search/select, batch Unachieve/delete, confirmation focus,
  countdown styling, and restart readback through the real WebView.
- Native notification delivery and a real local-date daily wake-up.
- Time-driven automatic purge against disposable physical test data.
- Item deletion and Lifetime Insight stability in the user's production data
  were intentionally not exercised destructively.

### Next ticket starting point

- Resume Ticket 04 with a native-window-capable UI session. Use disposable
  Mastered/Achieved data, perform the unchecked scenarios, then mark the ticket
  complete only after recording direct evidence.

## 2026-09-05 - Post-implementation review remediation

### Implemented

- Bounded lifecycle batches to 500 unique IDs and deduplicated repeated IDs
  before storage, preventing duplicate archive totals.
- Exposed compact archived Vocabulary, Encounter, and Review totals through
  database diagnostics instead of leaving the aggregate table write-only.
- Added normalized lemma to the Achieved DTO/search contract.
- Added the missing Settings explanation that retention changes affect future
  Achieve actions only.
- Preserved startup lifecycle results long enough for the Windows adapter to
  issue a summary notification.

### Review outcome

- Standards review: no documented repository-standard violations; three
  judgement-call smells remain (retention primitive, broad Windows root
  component, repeated SQL projection).
- Spec review: batch bounds, aggregate observability, normalized search,
  Settings copy, and startup summary handling were corrected.
- Still partial: the capture failure is typed in Rust and instructs the user to
  Unachieve, but the capture dialog does not yet contain a direct recovery
  button. The separate Global Insight dashboard query described by its own plan
  is not present on this branch, so archived totals are exposed at diagnostics
  rather than merged into a dashboard DTO.

### Verification

- Shared Rust suites: PASS, including 17 application and 9 storage tests.
- `cargo test -p vocab-desktop`: PASS, 40 tests.
- `cargo check --workspace`: PASS.
- `pnpm --dir ui test`: PASS, 78 tests.
- `pnpm --dir ui check`: PASS.

## 2026-09-05 - Follow-up: recapture recovery and Lifetime Insight

### Implemented

- Added an inline `Achieved` tag to both Windows capture presentations. A
  confirmed action returns the item to Learning and saves the new Encounter.
- Added an atomic shared Rust use case for recapturing an Achieved Vocabulary
  Item, including stale-ID protection.
- Added the shared Global Insight query and rendered archive-aware Lifetime
  totals on the Review page.
- Preserved Remembered and Forgotten totals during permanent purge through
  SQLite schema version 5.
- Added the Achieved date to every Achieved-list row alongside its existing
  deletion countdown and deadline.

### Key design decisions

- Recapture is not the same transition as list-based Unachieve: Unachieve
  returns to Mastered, while a newly encountered Achieved item returns to
  Learning.
- The restore and Encounter write share one SQLite transaction; Windows UI
  only presents the conflict and records consent.
- The Review dashboard labels its values as all-time totals. It does not mix
  archived aggregate-only data into detailed time-window charts.

### Main files

- `crates/application/src/lib.rs`
- `crates/application/src/platform_capture.rs`
- `crates/domain/src/views.rs`
- `crates/storage/src/lib.rs`
- `apps/desktop/src-tauri/src/commands/{capture,library}.rs`
- `ui/src/windows/{WindowsApp,WindowsFloatingCapture}.svelte`

### Tests and results

- Application seam tests for consent and atomic return to Learning: PASS.
- Application seam test for stable Lifetime totals across purge: PASS.
- `cargo test -p vocab-application`: PASS, 40 tests.
- `cargo test -p vocab-storage`: PASS, 9 tests.
- `cargo test -p vocab-domain`: PASS, 11 tests.
- `cargo test -p vocab-desktop --tests`: PASS, 40 tests.
- `pnpm --dir ui test`: PASS, 82 tests.
- `pnpm --dir ui build`: PASS.
- `cargo check --workspace`: PASS.
- `cargo test --workspace`: blocked only by the existing macOS native FFI
  linker symbols on Windows.

### Not yet verified

- The new buttons, focus behavior, locale-formatted dates, and dashboard layout
  have not yet been exercised manually in the physical Windows 11 WebView.
- Remembered/Forgotten breakdowns from databases that had already purged
  reviews before schema version 5 remain explicitly marked incomplete; the
  aggregate total Review count is still preserved.

### Next starting point

- Run the final UI type check, then exercise a disposable Achieved item through
  both Manual Capture and Selection Capture in the physical Windows app.

## 2026-09-05 - Human-test remediation: auto-detected source language

### Implemented

- Reproduced an Achieved lookup miss when the stored Vocabulary Item used
  source language `auto` and a later browser Selection Capture resolved it to
  `en`.
- Made `auto` compatible with one unambiguous resolved source-language match
  while preserving separation between two explicitly different languages.
- Applied the same compatibility validation inside the atomic restore-and-save
  transaction.
- Removed the Windows Manual Capture adapter's hard-coded `en` to `de` request
  and now builds requests from persisted language settings.

### Evidence and verification

- The inspected desktop database stored `validate` as `auto` to `zh-hans`; this
  is the exact language-key pattern covered by the regression test.
- `achieved_capture_matches_an_auto_detected_source_when_recapture_resolves_the_language`
  failed before the fix and passes afterward.
- `cargo test -p vocab-application`: PASS, 41 tests.
- `cargo test -p vocab-storage`: PASS, 9 tests.
- `cargo test -p vocab-desktop --tests`: PASS, 40 tests.
- Focused UI/backend tests: PASS, 54 tests.
- `pnpm --dir ui check`: PASS.
- `cargo check --workspace`: PASS.

### Still requires human confirmation

- The inspected database currently reports `validate` as Learning with no
  Achieved timestamps. The tester must first confirm that the running app and
  Achieved list refer to the same database, then Achieve the item and repeat
  Selection Capture using the rebuilt process.
