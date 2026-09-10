# 02: Undo a Vocabulary Item Learning Status change from the bottom-right notification

**What to build:** Add one exact Undo to a successful status change using the existing bottom-right notification design. Carry the undoable change through shared application/storage behavior and the Windows interaction so restoration is safe, not merely a reverse status transition.

**Blocked by:** 01: Edit an individual Vocabulary Item's Learning Status in Windows.

**Status:** implemented

- [x] The status-change notification offers Undo with styling, placement, and interaction conventions consistent with existing bottom-right Undo notifications.
- [x] Undo atomically restores the prior Learning Status, complete review scheduling state, and mastery start time. It preserves historical Reviews, Encounters, and unrelated item data.
- [x] Undoing Mastered to Learning restores the original mastery timestamp, rather than starting a new 30-day period. Undoing a reset restores the prior scheduling values exactly.
- [x] A subsequent successful change of the same item or a Review submission for it invalidates the old Undo. Enforce validity at the mutation boundary, not solely by hiding a UI control; same-status no-ops and failed writes do not count as successful changes.
- [x] Undo never overwrites newer lifecycle changes, restores a permanently deleted item, or bypasses Achieved rules. Stale requests fail without mutation and surface a clear outcome.
- [x] Successful Undo refreshes detail, list placement, and due summaries while keeping navigation stable. Disable duplicate Undo submission while in flight; failure does not falsely display a restored state.
- [x] Automated tests cover exact restoration, changed-again and reviewed-again invalidation, stale lifecycle/deletion requests, and the bottom-right Windows interaction including failures.

## Verification

Exact Undo and invalidation: 3 application tests passed. SQLite migration/repository: 11 passed. Windows status/Undo: 5 passed. Final cross-feature review remains pending.

