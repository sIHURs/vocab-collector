# 06: Complete phone, expanded-demo and accessible exploration

**What to build:** A phone or keyboard user can complete Capture → Vocabulary → Review → Reset with readable App UI and reliable navigation.

**Blocked by:** 05 — Make Replay and Reset safe across the whole session.

**Status:** ready-for-agent

- [ ] Implement approved native-size pan viewports and an accessible expanded-demo experience; constrain scrolling to the intended region.
- [ ] Capture remains word-adjacent and detail remains readable at narrow widths; Review reading/actions are reachable without shrinking native text.
- [ ] Expanded presentation preserves the same session and restores focus/position when closed; no duplicate independent demo state.
- [ ] Verify touch, keyboard-only navigation, focus visibility, portal placement, 150–200% zoom, long text, Chinese rendering and reduced motion in both themes.
- [ ] Evaluate the two-second Undo timing for accessibility and record any proposed website deviation explicitly rather than silently changing the App contract.
- [ ] Compare against Paper and current App, fix defects, and retain screenshot plus end-to-end evidence for desktop and phone.

## Scope and verification

Reuse actual Windows App UI and contracts with browser-local sample data. Preserve desktop defaults and Windows/macOS presentation boundaries. No native commands, production data, provider credentials or translation requests. English interface and Simplified Chinese translations; approved Paper appearance and current native UI remain authoritative. Include meaningful behavioral checks and relevant shared-App regression tests. Implementation does not authorize production deployment.

