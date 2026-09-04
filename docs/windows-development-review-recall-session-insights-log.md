# Windows Review Recall and Session Insights Development Log

## Ticket 01 — Require active recall and return a Review result

### Implemented

- Added a platform-neutral Review result returned after the Review Log and
  Review State are persisted.
- Added Windows Recall, Revealed, and Result states. Translation and rating
  actions remain absent until the user selects `Show answer`.
- Added explicit `Next` progression so users can read the scheduling result
  before advancing.
- Updated the browser/demo backend and typed desktop boundary to use the same
  Review result contract.

### Key design decisions

- Reveal remains Windows presentation state and creates no durable event.
- Rust returns scheduling facts and timestamps; Windows owns user-facing copy.
- The existing `Forgot` and `Remembered` scheduling rules remain unchanged.
- Shared domain, application, storage, and command types contain no Windows API
  concepts.

### Main files changed

- Shared Review view models and application service.
- Desktop library command contract.
- TypeScript backend and portable frontend types.
- Windows main presentation and its behavior tests.
- Application flow tests.

### Tests run

- `cargo test -p vocab-application --test application_flow`: 9 passed.
- `cargo check -p vocab-desktop --tests`: passed.
- `cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux --exclude vocab-desktop`: passed; Windows physical UIA test and live provider tests remained ignored by design.
- `pnpm --dir ui check`: 0 errors and 0 warnings.
- `pnpm test -- --run`: 72 passed.
- Unfiltered `cargo test --workspace`: not a valid Windows gate because the macOS adapter test binary cannot link its Swift symbols on Windows.
- Workspace test including `vocab-desktop`: could not replace a running `vocab-desktop.exe`; the desktop command contract was instead verified with `cargo check -p vocab-desktop --tests`.

### Not yet verified

- No physical Windows runtime pass was performed for focus movement, screen
  reader announcement, or the native desktop command invocation.
- macOS presentation behavior is intentionally not implemented or runtime
  verified in this branch.

### Next ticket starting point

Ticket 02 can enrich the Review result with Encounter count and a deterministic
repeated-forgetting signal, then render one actionable Review Insight in the
existing Windows Result state.

## Ticket 02 — Explain Encounter history and repeated forgetting

### Implemented

- Enriched the portable Review result with the current non-deleted Encounter
  count and a repeated-forgetting eligibility flag.
- Added a deterministic three-consecutive-`Forgot` threshold using persisted
  Review Logs.
- Added Windows Result feedback for Encounter history and one actionable
  `Review contexts` path when the threshold is reached.

### Key design decisions

- The shared core returns counts and eligibility only; Windows owns all prose.
- Repeated forgetting means the latest three persisted Reviews are all
  `Forgot`; an earlier lapse separated by `Remembered` does not qualify.
- Insight reads do not create new learning events or alter the existing Review
  schedule.

### Main files changed

- Shared Review result model and application review submission flow.
- Frontend Review result contract and demo backend.
- Windows Review Result presentation and behavior tests.
- Application flow tests for repeated Encounter and Review history.

### Tests run

- `cargo test -p vocab-application --test application_flow`: 10 passed.
- `cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux --exclude vocab-desktop`: passed.
- `cargo check -p vocab-desktop --tests`: passed.
- `pnpm test -- --run`: 73 passed.
- `pnpm --dir ui check`: 0 errors and 0 warnings.

### Not yet verified

- The `Review contexts` drawer and screen-reader announcement have not been
  exercised manually in the physical Windows runtime.
- macOS presentation remains intentionally unchanged and unverified.

### Next ticket starting point

Ticket 03 can change Today from one ambiguous capped count to explicit complete
due and planned Review counts while leaving queue ordering and scheduling
unchanged.

## Ticket 03 — Separate the complete due backlog from today's plan

### Implemented

- Replaced the ambiguous capped due count with portable `total_due_count` and
  `planned_review_count` fields.
- Counted the full ordered due backlog before applying the configured daily
  limit, while keeping the returned Review Queue capped.
- Updated Windows Today and idle Review to show the planned session size and
  the complete due backlog separately.
- Migrated the generic UI, desktop notification path, demo backend, and tests
  to the explicit count meanings.

### Key design decisions

- Queue construction and due-date ordering remain shared application behavior;
  Windows only chooses the labels and layout.
- Estimated time is based on the planned queue, not the complete backlog.
- The desktop notification count remains the actionable planned count, avoiding
  any Windows-only scheduling rule.

### Main files changed

- Shared Today view model and application service.
- Desktop notification mapping.
- TypeScript contracts, demo backend, and generic application shell.
- Windows Today/Review presentation and behavior tests.
- Shared application and debug-session tests.

### Tests run

- `cargo test -p vocab-application --test application_flow`: 11 passed.
- `cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux --exclude vocab-desktop`: passed; live provider tests and the opt-in physical Windows UIA test remained ignored by design.
- `cargo check -p vocab-desktop --tests`: passed.
- `pnpm --dir ui test -- --run src/windows/WindowsApp.test.ts`: 22 passed.
- `pnpm test -- --run`: 74 passed.
- `pnpm --dir ui check`: 0 errors and 0 warnings.

### Not yet verified

- The count copy and layout have not been exercised manually in the physical
  Windows desktop runtime or with a Windows screen reader.
- macOS presentation remains intentionally unchanged and unverified.

### Next ticket starting point

Ticket 04 can make Review lifecycle transitions resilient: closing after a
successful submission must resume at the next unreviewed item, reveal-only
state must resume safely, and retrying one logical submission must not produce
duplicate durable Review events.

## Ticket 04 — Make the Review lifecycle resumable and single-submit

### Implemented

- Added a caller-owned Review submission ID to the portable application and
  desktop command boundary.
- Made sequential retries of one logical submission idempotent before applying
  scheduling changes or appending another Review Log.
- Preserved the revealed card and a rating-specific retry action after errors,
  while disabling conflicting or repeated rating actions in flight.
- Defined reveal-only pause/resume to restart the same card in Recall state.
- Guarded stale responses with request versions and restored keyboard focus to
  the next meaningful Review action after each transition.

### Key design decisions

- Idempotency belongs to shared application behavior; Windows generates and
  reuses an opaque UUID but owns no persistence rule.
- A submission ID cannot be reused with another word or rating.
- Closing is unavailable while a submission is in flight; stale response guards
  provide a second boundary against navigation races.
- Reveal remains ephemeral and never creates a Review Log.

### Main files changed

- Shared application Review submission flow and application tests.
- Desktop Review command contract.
- TypeScript backend contract and idempotent demo backend.
- Windows Review state machine, focus behavior, and lifecycle tests.

### Tests run

- TDD red/green: `retrying_one_logical_review_submission_is_idempotent` failed before the new application seam and then passed.
- `cargo test -p vocab-application --test application_flow`: 12 passed.
- `cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux --exclude vocab-desktop`: passed; live provider tests and the opt-in physical Windows UIA test remained ignored by design.
- `cargo check -p vocab-desktop --tests`: passed.
- `pnpm test -- --run`: 76 passed.
- `pnpm --dir ui check`: 0 errors and 0 warnings.

### Not yet verified

- Native window close timing, keyboard focus, and screen-reader announcements
  have not been exercised in the physical Windows desktop runtime.
- Concurrent submissions from separate processes are outside this ticket's
  sequential lost-response retry contract and remain unverified.
- macOS presentation remains intentionally unchanged and unverified.

### Next ticket starting point

Ticket 05 can accumulate each successful Review result once, summarize the
session in shared application logic, calculate the next local calendar day's
expected workload, and render the resulting Insight in the Windows Review page.

## Ticket 05 — Complete Review with a Session Insight

### Implemented

- Added a portable Review Session Insight with reviewed, remembered, forgotten,
  repeated-forgetting attention IDs, and next-day due counts.
- Added shared application aggregation over successfully persisted Review
  results and the persisted vocabulary schedule.
- Accumulated each successful Windows session result once across pause/resume
  and ambiguous-response retries.
- Replaced the generic completion message with totals, a clearly labelled
  next-day estimate, and neutral attention guidance.
- Refreshed persisted Today state both when completing and when returning to
  Today, with a focused completion heading for keyboard and assistive use.

### Key design decisions

- Windows supplies the end of the next local calendar day as a UTC instant;
  shared application logic evaluates the portable due schedule against it.
- The estimate includes every Learning Vocabulary Item due by that boundary,
  including any backlog remaining beyond today's configured plan.
- Shared Rust returns IDs and counts; Windows maps attention IDs to session card
  labels and owns all explanatory prose.
- Only resolved Review results enter the session array, and word IDs are
  de-duplicated before aggregation.

### Main files changed

- Portable Review Session Insight view model.
- Shared application aggregation and tests.
- Desktop command registration and command contract.
- TypeScript backend contract, demo implementation, and frontend types.
- Windows completion flow, focus behavior, and presentation tests.

### Tests run

- TDD red/green: `session_insight_counts_successful_results_and_next_day_workload` failed before the shared application method and then passed.
- `cargo fmt --all -- --check`: passed after formatting.
- `cargo clippy --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux --exclude vocab-desktop -- -D warnings`: passed.
- `cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux --exclude vocab-desktop`: passed; live provider tests and the opt-in physical Windows UIA test remained ignored by design.
- `cargo check -p vocab-platform-macos -p vocab-platform-linux`: passed on the Windows host as compile-only adapter checks.
- `cargo check -p vocab-desktop --tests`: passed.
- `pnpm test -- --run`: 76 passed.
- `pnpm --dir ui check`: 0 errors and 0 warnings.
- `pnpm --dir ui build`: passed.

### Not yet verified

- The Windows completion layout, native focus behavior, screen-reader output,
  and local-midnight boundary have not been exercised manually in the physical
  Windows desktop runtime.
- macOS UI behavior and runtime remain intentionally out of scope and are not
  marked as verified; only its Rust adapter compiled on this host.
- Live Azure and DeepL provider tests remain opt-in and were not run.

### Next ticket starting point

All five tracer-bullet tickets are implemented. The next step is a final
standards/spec review of the complete branch, followed by any review fixes and
a physical Windows runtime acceptance pass by the developer.
