# 05: Complete Review with a Session Insight

**What to build:** Close a planned Review Queue with an accurate Session Insight assembled from successful Review results. Windows shows how many Vocabulary Items were reviewed, remembered, and forgotten, highlights eligible repeated-forgetting items, and reports the next local calendar day's expected due workload. The completed vertical flow must pass the repository's shared-core portability gate so macOS can later adopt it without redesigning the contracts.

**Blocked by:** 01: Require active recall and return a Review result; 02: Explain Encounter history and repeated forgetting; 03: Separate the complete due backlog from today's plan; 04: Make the Review lifecycle resumable and single-submit

**Status:** completed

- [x] Session totals include only successfully persisted Review results from the active planned queue.
- [x] Revealed-but-unrated, failed, cancelled, and duplicate submissions do not affect Session Insight counts.
- [x] Reviewed equals Remembered plus Forgot for empty, one-card, partially completed, resumed, and fully completed sessions.
- [x] The Session Insight lists only Vocabulary Items carrying the shared repeated-forgetting signal and uses neutral, actionable language.
- [x] The next-day expected due count is computed by shared application logic for the next local calendar day and is clearly presented as an estimate.
- [x] Completing the final result leads to Session Insight before returning to idle Review or Today.
- [x] Returning from completion refreshes the due backlog, today's plan, and next-day estimate from persisted state.
- [x] The Windows completion state has clear keyboard focus, screen-reader headings, error recovery, and reduced-motion behavior.
- [x] Shared Rust types and tests contain no Tauri, Win32, UI Automation, Swift, AppKit, or operating-system conditionals.
- [x] Formatting, linting, Rust workspace tests, desktop command-contract tests, Svelte checks, and Windows presentation tests pass without regressions to capture, Undo, Vocabulary, Settings, or the existing Review schedule.
- [x] Available non-Windows build and contract checks pass; macOS presentation work remains explicitly out of scope rather than being simulated or marked complete.
