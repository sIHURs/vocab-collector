# 05: Recapture Achieved Vocabulary Items with the normal Save action

**What to build:** Capturing an Achieved Vocabulary Item shows a short notice in the existing window before Save. Saving returns that same item to Learning with a fresh review schedule and new Encounter, using unchanged normal controls.

**Blocked by:** 03 — Resolve unknown source language consistently during capture.

**Status:** in-review

- [ ] Cover Manual Capture, Selection Capture, and Region OCR Capture; preserve OCR Confirmation, editing, translation, cancellation, and normal Save labels. Recheck after candidate/language edits and ignore stale asynchronous lookup responses.
- [ ] Opening/cancelling the window changes nothing. The ordinary Save action conveys consent; no separate Return to Learning confirmation.
- [ ] Revalidate the previewed identity at Save. If still Achieved, atomically clear mastery/achievement/deletion markers, initialize review parameters due now, store translations and Encounter, and update successful-save counts.
- [ ] If that same item is now Learning or Mastered, perform ordinary repeated capture without resetting progress. If it was deleted or attribution became ambiguous, retain the draft and require rechecking without silent replacement or retargeting.
- [ ] Preserve lifetime review/Encounter history; refresh affected list and review views after successful saves.
- [ ] Keep an operation snapshot/version sufficient for Ticket 06. Until complete rollback is supported for a state-changing save, fail safely rather than perform a destructive partial Undo.
- [ ] Verify original cross-target robust scenario, all capture entry points, candidate edits, cancellation, failures, stale preview/state transitions, and atomic persistence/count updates. Use storage/application tests and UI interaction tests, not UI mocks alone.

## Scope and completion

Follow the confirmed Vocabulary Item identity and Achieved recapture specification (Q1–Q15) and the approved ticket breakdown. This ticket is one slice of the complete feature; do not expand it into cloud sync, linguistic lemmatization, or a capture redesign.

Run this ticket only when its blockers are complete. If this is the last remaining slice, run the full integrated acceptance matrix, including migration/restart/save/review/Undo and all capture presentations; report platform validation limitations explicitly.
