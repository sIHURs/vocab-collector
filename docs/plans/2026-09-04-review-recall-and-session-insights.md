# Review Recall and Session Insights Plan

## Goal

Make Review require active recall before self-rating, explain the scheduling
result after each answer, and close each session with a useful summary. The
shared Rust core owns review rules and portable insight facts. This branch
delivers the user-facing experience only in the Windows presentation; a later
macOS presentation must be able to consume the same core contracts without
changing them.

## Problem Statement

The current Review card shows the translation before the user chooses
`Forgot` or `Remembered`. This permits recognition instead of recall. Review
submission returns no result beyond success, so the UI cannot explain the next
due date or why it changed. Completing a queue produces only a generic success
message, and Today reports the daily capped queue as if it were the complete
backlog.

## Product Decisions

- Review remains a scheduled recall decision with exactly two ratings:
  `Forgot` and `Remembered`.
- A card first shows the Vocabulary Item and saved context, but not its
  translation.
- The user explicitly reveals the answer before rating. Rating controls are not
  available before reveal.
- Revealing an answer is presentation state, not a Review and not a durable
  learning event.
- After rating, the UI shows a short Review Insight before continuing. It may
  report the next review time, Encounter count, and a repeated-forgetting
  suggestion; it must not invent an explanation or change the rating.
- Completing the current planned queue shows a Session Insight with reviewed,
  remembered, and forgotten counts plus the next expected workload.
- Today distinguishes the complete due backlog from the number planned under
  the daily limit.
- The existing scheduling policy remains unchanged in this plan: `Forgot` is
  due after one day and `Remembered` increases stability according to the
  existing domain rule.

## Cross-Platform Architecture Gate

- `vocab-domain`, `vocab-application`, and `vocab-storage` remain free of Tauri,
  Win32, UI Automation, Swift, AppKit, and operating-system conditionals.
- Review result and insight DTOs use product concepts and timestamps, never
  Windows presentation labels or styling state.
- The desktop command boundary exposes the same contract on every target.
- Windows-specific work is limited to `ui/src/windows` and platform-neutral
  command wiring. No Windows dependency may enter the review domain.
- macOS UI implementation and physical macOS verification are deferred, but
  macOS compilation and shared contract tests must remain green.
- Linux may remain capability-limited, but shared review behavior must compile
  and pass without a native adapter.

## Shared Core Design

### Review submission result

Change review submission from a success-only operation to a portable result
that contains the facts required for post-answer feedback:

- Vocabulary Item identity and submitted rating;
- review timestamp and next due timestamp;
- resulting difficulty, stability, and lapse count;
- Encounter count;
- a core-derived repeated-forgetting signal;
- enough prior-state information to describe an interval increase without
  exposing storage or platform details.

The core returns facts, not final prose. Presentation layers own localization
and sentences such as “Next review in 3 days.”

### Queue counts

Build the complete ordered due set before applying the daily limit. Return
separate values for:

- `totalDueCount`: every currently due Learning item;
- `plannedReviewCount`: the capped queue presented in this session;
- `reviewQueue`: the same capped, ordered cards.

Do not silently redefine an existing field without updating all callers and
contract tests. Prefer an explicit contract migration with unambiguous names.

### Session summary

Add a pure domain/application summary operation over the successful review
results in the active session. It reports:

- total reviewed;
- remembered count;
- forgotten count;
- items that triggered the repeated-forgetting signal;
- the next calendar day's expected due count after the session.

The active session may remain presentation-owned and in memory. It must contain
only successful submissions; failed or merely revealed cards are excluded. A
pause keeps the completed results for the current window lifetime. Durable,
cross-restart session identity is outside this plan.

### Insight eligibility

The core exposes deterministic facts and flags. A repeated-forgetting insight
must be based on persisted review history or the resulting lapse state, with a
documented threshold covered by tests. It must not infer psychological state,
claim retention, or label the user as failing.

## Windows Experience

### Card states

1. **Recall**: show word, context, progress, and `Show answer`.
2. **Revealed**: show translation and enable `Forgot` / `Remembered`.
3. **Result**: show the rating confirmation, next due time, Encounter fact, and
   at most one actionable suggestion; provide `Next`.
4. **Complete**: show the Session Insight and navigation back to Review or
   Today.

Closing Review before rating pauses without recording an event. Closing after
a successful rating must not submit it again. Submission failure keeps the
revealed card and permits retry.

### Accessibility and interaction

- Focus moves predictably when the answer is revealed and after submission.
- Newly revealed content and submission errors use appropriate live regions.
- Rating buttons have explicit names and remain keyboard reachable.
- Reduced-motion behavior applies to any result transition.
- The translation must not be present in accessible content before reveal.
- Keyboard shortcuts may be added only when they do not interfere with text or
  assistive technology; visible buttons remain the primary interaction.

## Delivery Work Packages

1. **Lock shared behavior with tests**
   - Add domain tests for result facts, repeated-forgetting eligibility, session
     aggregation, and unchanged scheduling.
   - Add application tests for total versus planned due counts and next-day
     workload after a completed session.
2. **Extend storage and application contracts**
   - Read the minimal review history and Encounter count required by the result.
   - Return the portable result atomically after the Review Log and Review State
     are persisted.
   - Avoid a migration unless a new durable field is proven necessary; existing
     Review Logs and Review State should remain the source of truth.
3. **Update desktop command contracts**
   - Serialize the new review result and explicit queue counts.
   - Keep command names and DTO behavior identical across compiled targets.
4. **Implement the Windows state flow**
   - Add Recall, Revealed, Result, paused, retry, and completed states.
   - Build Session Insight only from successful results.
   - Update Today and Review copy to distinguish backlog from planned work.
5. **Verify portability and behavior**
   - Run Rust workspace formatting, linting, tests, and cross-target checks
     available in the repository.
   - Run Svelte type checks and all presentation tests.
   - Record Windows physical evidence separately from automated evidence.

## Test Matrix

- A translation is absent visually and from the accessibility tree before
  reveal.
- A rating cannot be submitted before reveal.
- Reveal alone creates no Review Log and changes no due date.
- `Forgot` and `Remembered` preserve their existing scheduling results.
- A successful submission returns the persisted next due date and one matching
  Review Log.
- Retrying after a response loss cannot create duplicate review records.
- Encounter count and repeated-forgetting eligibility are correct for new,
  repeated, and legacy items.
- Session counts exclude failed and unsubmitted cards.
- Pause/resume preserves the remaining queue without rating the active card.
- Total due count may exceed planned count; the queue length equals planned
  count.
- Empty, one-card, partially completed, and fully completed sessions have clear
  Windows states.
- Shared Rust tests run without Windows APIs; non-Windows targets continue to
  compile against the command contract.

## Completion Criteria

- A Windows user must reveal every answer before rating.
- Every successful rating produces accurate, core-derived post-answer facts.
- Review completion shows an accurate Session Insight.
- Today and Review never present the daily cap as the complete due backlog.
- No shared crate imports or branches on a native operating-system API.
- Existing capture, Undo, Vocabulary, Settings, and scheduling tests remain
  green.
- macOS presentation changes are not required for completion, but shared Rust
  and desktop contract changes do not break the macOS build boundary.

## Out of Scope

- Four-grade Review ratings or a new scheduling algorithm.
- Using reveal time, response time, or hints to change scheduling.
- Durable Review Session records or recovery after application restart.
- Global history charts and range filters; those belong to the separate Review
  History and Global Insights plan.
- Context optionality, duplicate-translation replacement, Vocabulary Item
  editing, pause/master controls, deletion, or merge.
- Delaying the first review of a newly captured item.
- macOS presentation implementation.

