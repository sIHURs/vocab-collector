# 03: Complete reading-to-Vocabulary capture exploration

**What to build:** Visitors switch among three reading contexts, collect or edit prepared words, and inspect the resulting Vocabulary Items and Encounter history.

**Blocked by:** 02 — Deliver the three-section landing page with a working demo.

**Status:** ready-for-agent

- [ ] Initial dataset has four Vocabulary Items; article switching preserves saved data and never triggers initial autoplay.
- [ ] Prepared-word interaction anchors the actual Capture UI beside the word, preferring above then below with the approved clearance; explicit Save is required.
- [ ] Capture supports correction, manual translation or explicit save without translation for unknown edited text; stale translations are not silently reused.
- [ ] Saving updates the library without forced scrolling: first new save gives five items, repeat save stays at five and adds an Encounter; Undo restores the applicable prior state.
- [ ] Detail/history retains translation and source per Encounter; in-scope search/sort operate on actual session data.
- [ ] Saved dismissal follows the approved two-second behavior; obsolete timers cannot dismiss replacement drafts. Verify loading/error handling and keyboard focus.
- [ ] Session/integration tests cover new save, repeat, Undo, edit and notifications; no new captures enter the curated Review scope.

## Scope and verification

Reuse actual Windows App UI and contracts with browser-local sample data. Preserve desktop defaults and Windows/macOS presentation boundaries. No native commands, production data, provider credentials or translation requests. English interface and Simplified Chinese translations; approved Paper appearance and current native UI remain authoritative. Include meaningful behavioral checks and relevant shared-App regression tests. Implementation does not authorize production deployment.

