# Landing page development ticket proposal

Status: approved-and-published
Date: 2026-09-17

Source: the confirmed landing page development plan and latest Paper handoff. These are development tickets, separate from the five delivered Paper design tickets. No implementation or production deployment has started. Approved by the user on 2026-09-17 and published as seven separate local issues with Status: ready-for-agent. See spec.md for the ticket index.

## Shared constraints

Reuse actual Windows App presentation and contracts with browser-local sample data. Keep desktop defaults and Windows/macOS boundaries intact. No native commands, production data, provider credentials or translation requests. English interface, Simplified Chinese translations, current Paper appearance and native UI are authoritative. Each functional ticket includes its own meaningful tests and relevant shared-App regression checks; accessibility is not deferred entirely to final QA. Missing final reading assets and release URLs may use clearly marked development placeholders.

## 01 — Prove the shared App journey in a browser

**What to build:** A minimal local browser route where a visitor saves one prepared word, sees its Vocabulary Item and Encounter, and completes the two-word Review using actual App UI.

**Blocked by:** None (can start immediately).

- [ ] First make the minimum presentation/adapter separation needed for embedding; preserve desktop defaults and avoid duplicated App markup.
- [ ] Choose and document scoped shared views or, if demonstrably necessary, same-origin frame isolation based on working evidence, including portals, theme and focus.
- [ ] One prepared Capture Candidate can be explicitly saved; Vocabulary refreshes without reloading; detail shows sentence, saved translation and source.
- [ ] Two previously due sample Vocabulary Items support reveal, rating and completion; the newly captured item is excluded.
- [ ] Verify browser execution without native commands/provider requests and with isolated session data; demonstrate dark/light and a usable narrow viewport.
- [ ] Run relevant existing App regressions after shared changes and record the chosen integration boundary and follow-up limitations. Full marketing styling and all sample branches are outside this slice.

## 02 — Deliver the three-section landing page with a working demo

**What to build:** A visitor can open the independently built website, understand the product, navigate its three sections and launch the working journey from 01.

**Blocked by:** 01 — Prove the shared App journey in a browser.

- [ ] Separate website entry, build output and local preview from desktop startup, using existing Svelte/TypeScript/Vite dependencies.
- [ ] Implement Paper marketing composition, fonts, navigation, benefits, footer and download area; integrate the 01 demo rather than leaving only a disconnected placeholder.
- [ ] Dark initial theme and remembered theme selection work without wiping session data or leaking native styles into marketing content.
- [ ] Meaningful static/prerendered marketing content and download explanation remain usable without JavaScript or if the demo fails; choose and document the rendering mechanism.
- [ ] No page-wide overflow at 390px; navigation is keyboard usable and demo loading/failure states are accessible.
- [ ] Website and desktop builds both pass; incomplete public destinations are clearly marked in development and cannot masquerade as working downloads.

## 03 — Complete reading-to-Vocabulary capture exploration

**What to build:** Visitors switch among three reading contexts, collect or edit prepared words, and inspect the resulting Vocabulary Items and Encounter history.

**Blocked by:** 02 — Deliver the three-section landing page with a working demo.

- [ ] Initial dataset has four Vocabulary Items; article switching preserves saved data and never triggers initial autoplay.
- [ ] Prepared-word interaction anchors the actual Capture UI beside the word, preferring above then below with the approved clearance; explicit Save is required.
- [ ] Capture supports correction, manual translation or explicit save without translation for unknown edited text; stale translations are not silently reused.
- [ ] Saving updates the library without forced scrolling: first new save gives five items, repeat save stays at five and adds an Encounter; Undo restores the applicable prior state.
- [ ] Detail/history retains translation and source per Encounter; in-scope search/sort operate on actual session data.
- [ ] Saved dismissal follows the approved two-second behavior; obsolete timers cannot dismiss replacement drafts. Verify loading/error handling and keyboard focus.
- [ ] Session/integration tests cover new save, repeat, Undo, edit and notifications; no new captures enter the curated Review scope.

## 04 — Complete the deliberate Review experience

**What to build:** A visitor switches to Review, answers the curated two-word batch, sees accurate progress/results, and can leave and resume without losing their place.

**Blocked by:** 02 — Deliver the three-section landing page with a working demo.

- [ ] Match current App entry, fixed-height content region, batch/word rings, Show answer, Forgot/Remembered, feedback, Next and completion.
- [ ] Both response choices produce correct results and dates under a controlled fixture clock; duplicate submissions do not double-count.
- [ ] Website tab switching preserves answer/rating state; native close follows native pause/resume behavior.
- [ ] Completion reflects actual choices, one completed batch and no remaining due sample words, ending with End review.
- [ ] Next supports the approved flip and reduced motion; focus and screen-reader feedback follow state transitions.
- [ ] Test both rating branches, tab return, pause/resume and stale callbacks. Expose a deliberate session reset boundary for ticket 05.

## 05 — Make Replay and Reset safe across the whole session

**What to build:** Visitors can replay a sample or start over from any Capture/Review state without accidental saves, lost theme preference or late actions.

**Blocked by:** 03 — Complete reading-to-Vocabulary capture exploration; 04 — Complete the deliberate Review experience.

- [ ] Replay restages only the current sample; never changes Vocabulary Items, Encounters or Review results and waits for unresolved drafts to be handled.
- [ ] Keep playback state separate from session data; visitor input takes over cleanly, with no initial autoplay or implicit save.
- [ ] Confirmed Reset restores the original article, four items, Encounters and due Review scope atomically, preserving theme and website tab.
- [ ] Cancel Reset retains data and leaves playback paused; confirmation focus is trapped and restored correctly.
- [ ] Reset from open Capture, revealed answer, rated feedback and completion cancels timers/subscriptions and prevents stale events from restoring old state.
- [ ] Background/offscreen interruption pauses presentation and resumes coherently; reload starts fresh data while retaining only theme.
- [ ] Integration tests cover cross-section resets, cancellation, replay invariants and interrupted work.

## 06 — Complete phone, expanded-demo and accessible exploration

**What to build:** A phone or keyboard user can complete Capture → Vocabulary → Review → Reset with readable App UI and reliable navigation.

**Blocked by:** 05 — Make Replay and Reset safe across the whole session.

- [ ] Implement approved native-size pan viewports and an accessible expanded-demo experience; constrain scrolling to the intended region.
- [ ] Capture remains word-adjacent and detail remains readable at narrow widths; Review reading/actions are reachable without shrinking native text.
- [ ] Expanded presentation preserves the same session and restores focus/position when closed; no duplicate independent demo state.
- [ ] Verify touch, keyboard-only navigation, focus visibility, portal placement, 150–200% zoom, long text, Chinese rendering and reduced motion in both themes.
- [ ] Evaluate the two-second Undo timing for accessibility and record any proposed website deviation explicitly rather than silently changing the App contract.
- [ ] Compare against Paper and current App, fix defects, and retain screenshot plus end-to-end evidence for desktop and phone.

## 07 — Deliver a release-ready preview and App synchronization workflow

**What to build:** The owner receives a reproducible preview with finalized publication content and a repeatable way to update the site when the released App changes.

**Blocked by:** 06 — Complete phone, expanded-demo and accessible exploration.

- [ ] Measure production page/demo loading, establish justified budgets, and optimize loading without breaking static content or interactions.
- [ ] Verify supported browsers, metadata/social preview, assets, direct navigation and demo failure fallback.
- [ ] Replace placeholders with approved reading samples, branding, installer URL and contact/privacy/license destinations; missing inputs block publication, not independent verification work.
- [ ] Record released App version/commit, fixture and demo-flow versions, and build date in a release manifest; build public previews from an approved App revision.
- [ ] Add appropriate checks for shared UI/contracts and document how to rebuild, review and release website changes without automatically publishing every App commit.
- [ ] Provide a preview artifact, full journey evidence, known limitations and hosting-specific readiness checks once a host is chosen. Production deployment requires a separate explicit release request.

## Dependency review

01 → 02 → {03, 04} → 05 → 06 → 07.

Tickets 03 and 04 share the contracts established in 01–02, but neither must wait for the other's additional behavior. Existing baseline accessibility is required throughout; 06 finishes the complete adaptive journey. A standalone action recorder is intentionally excluded from version one.

The user approved this granularity and dependency graph. No implementation work has started.

