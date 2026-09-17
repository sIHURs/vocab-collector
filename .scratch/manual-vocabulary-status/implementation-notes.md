# Implementation and verification

Tickets 01, 02, and 03 were implemented in that order on the existing branch.

- Shared domain/application/storage behavior owns the status transition matrix and exact Undo. SQLite schema 9 adds one undo snapshot per Vocabulary Item; existing revision tracking prevents stale restoration after subsequent changes, reviews, or lifecycle operations.
- Windows detail exposes the existing Learning, Mastered, and Paused choices. It refreshes list placement without navigating, distinguishes write failures from post-save refresh failures, and uses the existing bottom-right Undo notification.
- Resume obtains a fresh queue with completed session items excluded before the daily limit. The active session queue is independent of Today refreshes, preserving the pairing of submitted cards and their results when editing through Review contexts.

## Verification

- `cargo test --workspace --exclude vocab-platform-macos`: 203 tests passed, including desktop, shared core, storage migrations, and platform contract tests available on this host.
- `cargo test --workspace` was attempted first. The macOS adapter contract target cannot link its native macOS symbols on Windows. This platform limitation remains; no macOS implementation was changed.
- `pnpm --dir ui test`: all 123 tests across 13 files passed after review fixes.
- `pnpm --dir ui check`: zero errors and warnings.
- `pnpm --dir ui build`: production build passed.
- Actual browser QA using the Windows presentation and demo backend verified the status dropdown, visible Mastered result, bottom-right notification, and successful Undo while detail remained open. Persistence and exact mastery/scheduling restoration were verified through Rust application tests, including database reopening.
- `git diff --check`: passed.

## Code review

Review baseline: `856547f117f6b67281ff01028ce3fdea4a52c8ef`.

### Standards

Independent review found no documented violations or substantial baseline smells. Final delta review found no remaining definite standards issue.

### Spec

Review found two edge cases, both fixed and covered by regression tests: completed items consuming queue-limit slots after becoming due again, and Today refresh replacing a card while its previous Review result remained displayed. Final review found no remaining definite specification issue.

Remaining findings: Standards 0; Spec 0.
