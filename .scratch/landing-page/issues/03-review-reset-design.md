# 03: Design the short Review journey and connected demo reset

**What to build:** A complete Paper journey through deliberate Review selection, answer reveal, remembered/forgotten choice, completion, and reset.

**Blocked by:** 02: Design the linked Vocabulary library and detail experience.

**Status:** ready-for-review

- [x] Show Review entry, unrevealed vocabulary, revealed answer, remembered/forgotten actions, progress, and completion using existing app Review visuals as references.
- [x] Use previously due example vocabulary; annotate that newly captured words do not enter this curated demonstration automatically.
- [x] Deliver readable desktop/mobile and dark/light states.
- [x] Specify view switching and returning to Vocabulary while preserving the visitor's place.
- [x] Show Reset demo restoring initial sample, library, Review, and capture states while preserving theme choice. Distinguish Replay, which restages only the hero without changing visitor data.
- [x] Annotate reset during unfinished capture, Review, or playback and appropriate focus destinations. Specify UX, not implementation architecture.

**Scope:** UX/UI design in Paper only. Deliver artboards and interaction annotations; no app code, backend, automated tests, builds, hosting, or deployment. Use existing app visuals as references. If Paper is unavailable, report the blocker rather than substituting tools. Mark unselected content/assets as placeholders. Prototype connections are optional where supported; do not represent annotated behavior as functional software.

## Delivery — 2026-09-13

[Paper: vocab-landing-page / Page 1](https://app.paper.design/file/01M2AMNRXMP0TZ753GPD28ZSJV/1-0)

- 22–31: paired dark/light desktop states: entry, recall, answer, rating feedback, completion.
- 32–37: paired dark/light phone player states: recall, reading the revealed answer, panned rating controls.
- 38: second-word answer/Forgot feedback with 2-of-2 progress, pause/resume, dark/light reset confirmations, reset result, navigation/focus/recording annotations.

Retains Style A, Fraunces marketing headings and Inter App controls. App visuals and behavior reference current `ui/src/windows/WindowsApp.svelte`, `ui/src/theme.css` and the actual button/progress components; reuses Ticket 2's Windows shell. Rating feedback remains visible until Next. Today stays active in the native sidebar while Review is open.

The curated session uses previously due serendipitous and resilient. New captures do not join it. The illustrated branch is one remembered / one forgot; future recording must cover both decisions and corresponding completion counts. Dates and estimated future due counts are illustrative fixtures, to be replaced by real seeded App results.

Website tabs preserve the current demo frame, revealed answer, rating, library selection and pan. Native App close follows its pause/resume behavior. Reset clears demo additions, draft, Undo and ratings, restores four original Vocabulary Items and first reading sample, preserves theme and active website tab, and stops playback. Replay does not mutate visitor data. Confirmation, cancel, interruption and focus destinations are annotated on 38.

Mobile retains native 1× type in a horizontally draggable crop, with Read / Show controls and Expand player affordances. Sentence and action regions do not fit simultaneously: this is an explicit design tradeoff for review. Entry/completion use centered crops as annotated; no separate phone App layout is invented. Actual hotspots, panning, expansion and playback are future work, not functioning Paper interactions.

Visual QA: reviewed desktop dark/light, mobile reading/actions and handoff artboards; corrected light-theme caption contrast and cloned frame sizing. Artboards fit their contents; only labeled player viewports intentionally crop App content. No App code or recordings were produced.


## App UI synchronization — 2026-09-17

Updated the existing Paper deliverables against App source at a696b5e. See ../handoff.md, "Verified UI revision", for the authoritative changes and current behavior. This revision supersedes conflicting older timing, progress and completion descriptions above. Status remains ready-for-review; application code is unchanged.
