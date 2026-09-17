# 02: Design the linked Vocabulary library and detail experience

**What to build:** A second-section Paper journey where visitors find the word saved in the hero and inspect its sentence and source.

**Blocked by:** 01: Design the capture hero from first impression to saved feedback.

**Status:** ready-for-review

- [x] Design Your words, ready when you are. and one product frame with Vocabulary/Review switches, defaulting to Vocabulary.
- [x] Reference existing app library/detail visuals; limit demo navigation without inventing a separate product design language.
- [x] Show the seeded library, a newly saved Vocabulary Item, selected detail, and Encounter sentence/source with Simplified Chinese translation.
- [x] Annotate that a hero save appears prominently without auto-scrolling or switching sections. Repeated capture adds an Encounter to the same Vocabulary Item.
- [x] Deliver desktop/mobile and dark/light states, including a readable mobile detail view and return action.
- [x] Place a quiet Reset demo control with clear visual hierarchy.

## Delivery — 2026-09-13

Paper: https://app.paper.design/file/01M2AMNRXMP0TZ753GPD28ZSJV/1-0

- Artboards 11–16: desktop initial, saved and detail states in dark/light.
- Artboards 17–20: mobile pan viewport of the actual desktop table and native detail fields, with a website-level Back to app preview action, in dark/light.
- Artboard 21: seed data, hero linkage, repeated Encounter example, Undo, focus and Reset contract.
- Style A with Fraunces Medium marketing headings. App UI follows the supplied actual-app screenshots and current Windows implementation. Vocabulary/Review and Reset are website controls outside the App window; the real titlebar, sidebar and product controls remain visible.
- Original sample sentences and source metadata remain placeholders. These are annotated Paper states, not functional interactions. Review and final reset flow remain Ticket 03.
- Reviewed screenshots for all new artboards; checked text wrapping, contrast, aligned table columns, footer visibility and mobile detail readability. No app code changed.

## Review revision — Actual App fidelity

- Replaced the simplified product mockup in all desktop and mobile variants. Restored the native titlebar search/Add word/window controls, 184px sidebar, Active/Mastered/Achieved tabs, local search, item count, neutral bordered table, pill status badges and pagination.
- Restored the full-height 440px detail sheet with blurred backdrop, Learning status select, actual Capture history source/timestamp structure and close action. The list starts at its normal top position; the supplied screenshot was scrolled below the page heading.
- Removed marketing highlights and hints from inside the App; sample data stays separate from visual UI. Mobile uses a readable pan viewport of the same desktop App instead of an invented card-list UI. Mobile detail adapts only viewing width and keeps native fields.
- Artboard 21 now includes a recording plan: fixed 1040 × 750 App window, deterministic sample data, theme-specific captures, actual control hotspots and synchronized pan/zoom coordinates. Production recordings must come from the real App build; no recording or software interactions were created here.

**Scope:** UX/UI design in Paper only. Deliver artboards and interaction annotations; no app code, backend, automated tests, builds, hosting, or deployment. Use existing app visuals as references. If Paper is unavailable, report the blocker rather than substituting tools. Mark unselected content/assets as placeholders. Prototype connections are optional where supported; do not represent annotated behavior as functional software.


## App UI synchronization — 2026-09-17

Updated the existing Paper deliverables against App source at a696b5e. See ../handoff.md, "Verified UI revision", for the authoritative changes and current behavior. This revision supersedes conflicting older timing, progress and completion descriptions above. Status remains ready-for-review; application code is unchanged.
