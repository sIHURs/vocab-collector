# 05: Make Replay and Reset safe across the whole session

**What to build:** Visitors can replay a sample or start over from any Capture/Review state without accidental saves, lost theme preference or late actions.

**Blocked by:** 03 — Complete reading-to-Vocabulary capture exploration; 04 — Complete the deliberate Review experience.

**Status:** ready-for-agent

- [ ] Replay restages only the current sample; never changes Vocabulary Items, Encounters or Review results and waits for unresolved drafts to be handled.
- [ ] Keep playback state separate from session data; visitor input takes over cleanly, with no initial autoplay or implicit save.
- [ ] Confirmed Reset restores the original article, four items, Encounters and due Review scope atomically, preserving theme and website tab.
- [ ] Cancel Reset retains data and leaves playback paused; confirmation focus is trapped and restored correctly.
- [ ] Reset from open Capture, revealed answer, rated feedback and completion cancels timers/subscriptions and prevents stale events from restoring old state.
- [ ] Background/offscreen interruption pauses presentation and resumes coherently; reload starts fresh data while retaining only theme.
- [ ] Integration tests cover cross-section resets, cancellation, replay invariants and interrupted work.

## Scope and verification

Reuse actual Windows App UI and contracts with browser-local sample data. Preserve desktop defaults and Windows/macOS presentation boundaries. No native commands, production data, provider credentials or translation requests. English interface and Simplified Chinese translations; approved Paper appearance and current native UI remain authoritative. Include meaningful behavioral checks and relevant shared-App regression tests. Implementation does not authorize production deployment.

