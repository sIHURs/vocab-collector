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
