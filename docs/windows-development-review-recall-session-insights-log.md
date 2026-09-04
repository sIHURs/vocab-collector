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
