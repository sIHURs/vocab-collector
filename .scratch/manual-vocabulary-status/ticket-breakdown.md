# Manual Vocabulary Item learning status: approved ticket breakdown

Status: implemented

The user approved this three-ticket breakdown. The individual issues are published under the feature's issues directory with complete status. No standalone prefactor is necessary based on the current exploration. Keep any enabling refactor narrow and within its slice.

## 01: Edit an individual Vocabulary Item's Learning Status in Windows

**What to build:** Let the user open a Vocabulary Item's detail in Windows, choose Learning, Mastered, or Paused, and immediately see the saved status and correct list placement. Implement the behavior through the shared domain/application and persistence boundary, desktop command, backend adapter, Windows presentation, and relevant automated tests. Other operating systems retain their existing presentation.

**Blocked by:** None (can start immediately).

**Status:** complete

- [ ] The detail dropdown is the only new editing entry point. It saves one item immediately on selection; there is no bulk editing or custom status definition.
- [ ] The selected status persists across reload/restart and applies to the entire Vocabulary Item across target-language translations. Existing records need no user intervention.
- [ ] Learning to Paused preserves review scheduling. Paused to Learning preserves scheduling and becomes review-eligible when due.
- [ ] Learning or Paused to Mastered preserves scheduling, excludes the item from review, and starts a mastery period at the time of the change.
- [ ] Mastered to Learning resets scheduling to the initial state, due immediately, and clears the mastery period. Mastered to Paused also resets scheduling to the initial state due at the change, but remains excluded until resumed.
- [ ] All transitions preserve historical Reviews and Encounters. Reviews continue to change scheduling only, never Learning Status.
- [ ] Selecting the current status is a no-op, including timestamps and scheduling. Manual mastery participates in existing opt-in automatic Achieve after 30 uninterrupted days; leaving and re-entering Mastered starts a new period.
- [ ] Achieved items cannot be edited through this operation, including direct backend requests. Existing Unachieve and explicit recapture behavior remain intact; missing/deleted items are rejected without mutation.
- [ ] Successful changes refresh status badges, detail, Active/Mastered lists, and current due summaries. Active includes Learning and Paused. Keep the current view, search, and open detail; safely clamp pagination if the current page becomes empty.
- [ ] A success notification follows the existing bottom-right visual style and identifies the resulting status/destination. Exact Undo is added by ticket 02.
- [ ] Disable selection while saving. A failed write leaves the original state intact and exposes a retryable error. Distinguish a successful save followed by a refresh failure so the UI does not report a false rollback.
- [ ] Automated tests cover the transition matrix, persistence, mastery timer boundaries, unchanged historical data, backend rejection, and Windows selection/list/error behavior. Include a reload demonstration and ensure existing non-Windows presentation still builds.

## 02: Undo a Vocabulary Item Learning Status change from the bottom-right notification

**What to build:** Add one exact Undo to a successful status change using the existing bottom-right notification design. Carry the undoable change through shared application/storage behavior and the Windows interaction so restoration is safe, not merely a reverse status transition.

**Blocked by:** 01: Edit an individual Vocabulary Item's Learning Status in Windows.

**Status:** complete

- [ ] The status-change notification offers Undo with styling, placement, and interaction conventions consistent with existing bottom-right Undo notifications.
- [ ] Undo atomically restores the prior Learning Status, complete review scheduling state, and mastery start time. It preserves historical Reviews, Encounters, and unrelated item data.
- [ ] Undoing Mastered to Learning restores the original mastery timestamp, rather than starting a new 30-day period. Undoing a reset restores the prior scheduling values exactly.
- [ ] A subsequent successful change of the same item or a Review submission for it invalidates the old Undo. Enforce validity at the mutation boundary, not solely by hiding a UI control; same-status no-ops and failed writes do not count as successful changes.
- [ ] Undo never overwrites newer lifecycle changes, restores a permanently deleted item, or bypasses Achieved rules. Stale requests fail without mutation and surface a clear outcome.
- [ ] Successful Undo refreshes detail, list placement, and due summaries while keeping navigation stable. Disable duplicate Undo submission while in flight; failure does not falsely display a restored state.
- [ ] Automated tests cover exact restoration, changed-again and reviewed-again invalidation, stale lifecycle/deletion requests, and the bottom-right Windows interaction including failures.

## 03: Resume Review using current Vocabulary Item learning statuses

**What to build:** When a user pauses Review, edits or undoes a Vocabulary Item's status, and resumes, rebuild the remaining queue from current persisted status and scheduling while retaining the results already completed in that session.

**Blocked by:** 01: Edit an individual Vocabulary Item's Learning Status in Windows.

**Status:** complete

- [ ] Resuming a paused Review fetches/recomputes the remaining queue using current status and due times. Paused and Mastered are absent; eligible Learning items can enter under existing queue ordering and daily-limit behavior.
- [ ] Completed session results remain available for the session summary and Review Insight. A refresh does not discard, duplicate, or resubmit completed Reviews.
- [ ] When the refreshed remaining queue is empty, show session completion with the retained results rather than a stale card or unusable Resume action.
- [ ] Clear stale card/reveal/submission UI state as needed when the remaining queue changes. A failed refresh exposes retry and prevents resuming the stale queue.
- [ ] Queue handling depends on current persisted state rather than which action changed it, so the same refresh path also supports status Undo once ticket 02 is available.
- [ ] Automated tests demonstrate pausing a session, editing a remaining item to Mastered or Paused, adding a due Learning item through a status transition, and resuming. Cover retained results, existing daily limit, empty completion, and refresh retry.

## Dependency rationale

01 creates the usable mutation and Windows interaction required by both later slices. 02 adds exact reversal; 03 refreshes from persisted state and does not require the Undo interaction, so neither blocks the other. Every ticket includes its own meaningful verification; no separate horizontal testing ticket is needed.
