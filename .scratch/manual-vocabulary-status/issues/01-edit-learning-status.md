# 01: Edit an individual Vocabulary Item's Learning Status in Windows

**What to build:** Let the user open a Vocabulary Item's detail in Windows, choose Learning, Mastered, or Paused, and immediately see the saved status and correct list placement. Implement the behavior through the shared domain/application and persistence boundary, desktop command, backend adapter, Windows presentation, and relevant automated tests. Other operating systems retain their existing presentation.

**Blocked by:** None (can start immediately).

**Status:** implemented

- [x] The detail dropdown is the only new editing entry point. It saves one item immediately on selection; there is no bulk editing or custom status definition.
- [x] The selected status persists across reload/restart and applies to the entire Vocabulary Item across target-language translations. Existing records need no user intervention.
- [x] Learning to Paused preserves review scheduling. Paused to Learning preserves scheduling and becomes review-eligible when due.
- [x] Learning or Paused to Mastered preserves scheduling, excludes the item from review, and starts a mastery period at the time of the change.
- [x] Mastered to Learning resets scheduling to the initial state, due immediately, and clears the mastery period. Mastered to Paused also resets scheduling to the initial state due at the change, but remains excluded until resumed.
- [x] All transitions preserve historical Reviews and Encounters. Reviews continue to change scheduling only, never Learning Status.
- [x] Selecting the current status is a no-op, including timestamps and scheduling. Manual mastery participates in existing opt-in automatic Achieve after 30 uninterrupted days; leaving and re-entering Mastered starts a new period.
- [x] Achieved items cannot be edited through this operation, including direct backend requests. Existing Unachieve and explicit recapture behavior remain intact; missing/deleted items are rejected without mutation.
- [x] Successful changes refresh status badges, detail, Active/Mastered lists, and current due summaries. Active includes Learning and Paused. Keep the current view, search, and open detail; safely clamp pagination if the current page becomes empty.
- [x] A success notification follows the existing bottom-right visual style and identifies the resulting status/destination. Exact Undo is added by ticket 02.
- [x] Disable selection while saving. A failed write leaves the original state intact and exposes a retryable error. Distinguish a successful save followed by a refresh failure so the UI does not report a false rollback.
- [x] Automated tests cover the transition matrix, persistence, mastery timer boundaries, unchanged historical data, backend rejection, and Windows selection/list/error behavior. Include a reload demonstration and ensure existing non-Windows presentation still builds.

## Verification

Application flow: 36 passed. Windows Learning Status: 3 passed. Svelte check: 0 errors/warnings. Desktop cargo check passed. Final cross-feature review remains pending.

