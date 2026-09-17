# 04: Complete the deliberate Review experience

**What to build:** A visitor switches to Review, answers the curated two-word batch, sees accurate progress/results, and can leave and resume without losing their place.

**Blocked by:** 02 — Deliver the three-section landing page with a working demo.

**Status:** ready-for-agent

- [ ] Match current App entry, fixed-height content region, batch/word rings, Show answer, Forgot/Remembered, feedback, Next and completion.
- [ ] Both response choices produce correct results and dates under a controlled fixture clock; duplicate submissions do not double-count.
- [ ] Website tab switching preserves answer/rating state; native close follows native pause/resume behavior.
- [ ] Completion reflects actual choices, one completed batch and no remaining due sample words, ending with End review.
- [ ] Next supports the approved flip and reduced motion; focus and screen-reader feedback follow state transitions.
- [ ] Test both rating branches, tab return, pause/resume and stale callbacks. Expose a deliberate session reset boundary for ticket 05.

## Scope and verification

Reuse actual Windows App UI and contracts with browser-local sample data. Preserve desktop defaults and Windows/macOS presentation boundaries. No native commands, production data, provider credentials or translation requests. English interface and Simplified Chinese translations; approved Paper appearance and current native UI remain authoritative. Include meaningful behavioral checks and relevant shared-App regression tests. Implementation does not authorize production deployment.

