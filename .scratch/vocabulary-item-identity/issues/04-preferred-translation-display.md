# 04: Display the preferred saved translation in Vocabulary and Review

**What to build:** Vocabulary lists and revealed Review answers display the current target-language translation when saved, otherwise the most recently saved nonempty translation with its language. The detail sidebar continues to show all saved target languages.

**Blocked by:** 01 — Preserve saved translations and Encounter translation history in Vocabulary detail.

**Status:** in-review

- [ ] Apply the same selection rule to affected Vocabulary list states and revealed Review answers across presentations.
- [ ] Preserve existing empty-translation behavior when no translation is available. Do not expose the answer before reveal.
- [ ] Changing target-language settings changes selection only; it does not call translation providers, add Encounters, create items, or reset progress.
- [ ] Use deterministic ordering for legacy translations without claiming unavailable timestamps. Keep backend projections and presentation test doubles consistent.
- [ ] Verify preferred-language, fallback, empty, settings-change, and review-reveal behavior. Ticket 01's expanded model allows this slice to start without waiting for identity consolidation.

## Scope and completion

Follow the confirmed Vocabulary Item identity and Achieved recapture specification (Q1–Q15) and the approved ticket breakdown. This ticket is one slice of the complete feature; do not expand it into cloud sync, linguistic lemmatization, or a capture redesign.

Run this ticket only when its blockers are complete. If this is the last remaining slice, run the full integrated acceptance matrix, including migration/restart/save/review/Undo and all capture presentations; report platform validation limitations explicitly.
