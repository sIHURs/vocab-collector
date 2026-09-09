# 06: Completely undo capture changes without overwriting later work

**What to build:** Undo fully reverses an uncontested save, including translation changes and Achieved-to-Learning restoration. If later work conflicts, it explains refusal and leaves data unchanged.

**Blocked by:** 05 — Recapture Achieved Vocabulary Items with the normal Save action.

**Status:** in-review

- [ ] Restore prior target-language defaults, learning state, review parameters, achievement/deletion timestamps, and withdraw the new Encounter and its successful-save contribution together.
- [ ] Preserve earlier translation snapshots and histories; respect Vocabulary log date and coverage semantics. Restore identity/references safely where this save performed language reconciliation, or refuse the entire operation if safe restoration is no longer possible.
- [ ] Detect subsequent save, translation edit, review, or lifecycle change, including deletion. Refuse the whole Undo rather than partially reverting or overwriting later state.
- [ ] Present success/refusal consistently through supported capture Undo controls; preserve valid UI state and do not close the flow as if a refused Undo succeeded.
- [ ] Verify successful rollback and every conflicting-change category with real storage transactions; cover translation creation/replacement, original new-item capture, auto reconciliation, Achieved restoration, and count invariants.
- [ ] Verify this slice end to end through upgrade/restart/save/Undo, alongside targeted UI checks. Confirm no obsolete target-language identity checks or stale single-translation write paths remain in the affected operation paths; record any platform validation limitations honestly.

## Scope and completion

Follow the confirmed Vocabulary Item identity and Achieved recapture specification (Q1–Q15) and the approved ticket breakdown. This ticket is one slice of the complete feature; do not expand it into cloud sync, linguistic lemmatization, or a capture redesign.

Run this ticket only when its blockers are complete. If this is the last remaining slice, run the full integrated acceptance matrix, including migration/restart/save/review/Undo and all capture presentations; report platform validation limitations explicitly.
