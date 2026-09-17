# Landing page development plan

Status: planned — ready for review, implementation not started
Updated: 2026-09-17

## Confirmed direction

The user selected a real, browser-operable App UI with sample data. Capture, Vocabulary and Review must actually work together. This replaces a screenshot/video-plus-hotspots approach as the primary demo. Recording, if added later, means replayable demo actions; it is not required for the initial usable site.

Use [handoff.md](handoff.md) for current appearance and behavior, Paper Page 1 boards 44–48 for composition, and the current Windows App for product UI. Historical proposals in spec.md are background only. Existing issues 01–05 cover Paper design, not website implementation.

The current task produces a plan only. It does not start code changes or publishing.

## Implementation recommendation

Keep the site in this repository with a separate web entry and output, using the existing Svelte/TypeScript/Vite toolchain. Reuse actual Windows presentation components, theme tokens and frontend data contracts. Do not fork copies of App markup into a marketing-only imitation. Start with a small separate site entry under `ui/` so existing imports and dependency versions can be reused; settle exact paths and build configuration in the feasibility task. Do not migrate frameworks or reorganize the entire repository for this page.

Separate responsibilities:

- Marketing shell: semantic page content, navigation, Fraunces headings, downloads, responsive framing, theme and demo loading/fallback.
- Product presentation: shared Capture, Vocabulary/detail and Review UI, styled by App tokens. Any extraction must preserve desktop behavior and defaults.
- Demo session: fixed sample data, Capture Candidates and Encounters, two due Vocabulary Items, draft/review state and session reset.
- Demo adapters: implement existing Backend and WindowsCaptureBackend contracts without native commands, translation network calls or the user's database.
- Demo controls: website tabs, sample selection, anchored popup placement, Replay and Reset. They must not replace the App's underlying learning semantics.

ADR 0001 keeps Windows/macOS page presentations independent. This proposal reuses Windows presentation in a web demonstration; it does not merge Windows and macOS page components or change the shared domain.

## Evidence and integration risks

Source inspected at a696b5e; recheck HEAD before implementation:

| Existing code | Implication for the plan |
| --- | --- |
| `ui/src/windows/WindowsApp.svelte` accepts `api: Backend` | A demo adapter is feasible, but route, Review state and lifecycle remain inside the component. Add a small intentional integration interface or extract shared views; do not manipulate private DOM to navigate. |
| `WindowsFloatingCapture.svelte` accepts `captureBackend` | Reuse is feasible, but document/body mutations, window lifecycle and timers require an embedded presentation boundary. |
| `ui/src/test/capture-preview.html` mounts Capture with an adapter | Useful evidence and fixture reference, not a production website entry. |
| `ui/src/lib/backend.ts` has DemoBackend and native imports | Current seeds are German and new captures are due immediately. Add dedicated landing fixtures and curated due scope; sharing an instance alone is insufficient for UI refresh. Audit native import side effects rather than assuming browser safety. |
| Backend exposes optional `listenLibraryChanged` | Implement session notifications and unsubscribe/reset behavior for the landing adapter. |
| `ui/src/styles.css` has global selectors and minimum body dimensions | Isolate native styles from marketing layout and prevent two mounted presentations from overwriting root appearance/body sizing. |
| Native titlebar already guards window operations | Preserve truthful disabled native controls in a browser; do not pretend browser buttons control a desktop window. |
| Detail dialogs, toasts and keyboard listeners | Verify portal ownership, focus restoration, z-index, clipping and shortcut scope in an embedded demo. |

Preferred approach: shared presentational views with scoped styles and injected adapters in one document. First prove that extraction is small and preserves desktop behavior. If global styles/portals make this disproportionately invasive, evaluate a same-origin isolated frame with a typed message contract. Do not quietly choose an iframe without testing focus, popup placement, theme, sizing and session synchronization. Document the final boundary after the feasibility task.

## State contract

- Initial fixtures: four Vocabulary Items; serendipitous and resilient are the two previously due Review items. These are curated website fixtures, not production first-use Starter Vocabulary Items.
- English interface and Simplified Chinese translations. Preset translations require no external provider or API key. If Edit capture changes a prepared word to unknown text, do not reuse an unrelated translation silently; support manual translation/explicit save without translation according to the existing App flow.
- Clicking a prepared word creates a Capture Candidate and starts the short presentation. Explicit Save creates an Encounter and updates Vocabulary without scrolling away. Repeated capture updates the same Vocabulary Item. Undo removes only the applicable latest save; the adapter follows the App contract.
- Saved hides after two seconds under the currently approved native behavior. Cancel, Undo, reset or replacement must cancel obsolete timers and callbacks.
- Review operates on the curated due scope, not newly saved words. Show answer, rating, feedback, Next, two progress rings and completion reflect real session choices. App close/pause and website tab preservation remain distinct.
- Keep playback state separate from mutable sample data. Replay restages the selected sample without saving, erasing data or changing Review results. No initial autoplay.
- Reset atomically restores sample, draft, library, encounters and Review, clears timers and playback, preserves theme and active website tab, and restores focus. Cancel reset retains data.
- Reload starts a fresh demo; only theme preference persists. No production database, filesystem, OCR, global desktop shortcut or translation service is invoked.
- Use an injectable clock and deterministic IDs for tests. Recording fixtures must specify time zone, dates and counts; published example dates should be deliberately chosen rather than inherited from the developer's machine.
- Limit navigation to the designed demo journey. Do not expose Settings/Insights routes merely because the desktop shell contains them. Keep native visual chrome and provide an accessible, honest explanation for unavailable browser-demo actions.

## Development sequence and acceptance gates

These are proposed development work packages; create separate implementation issues after plan review. Do not reopen the five completed Paper tasks as coding tasks.

### D01 — Prove shared App UI in the browser

Build a disposable local integration view using current shared UI, not the full marketing page. Demonstrate a prepared Capture Candidate, explicit Save, live Vocabulary refresh, detail, and a complete two-word Review. Verify no native commands/network translation and no marketing-style leakage. Validate dark/light, dialog portals, keyboard focus, and a 390px viewport.

Deliver the chosen presentation boundary, files/components to extract, browser-safe adapter contract and a minimal working route. If extraction is too invasive, compare the isolated-frame alternative before continuing. Preserve desktop defaults and run relevant existing App checks/tests after shared changes.

Dependency: none. Gate: this vertical journey works before investing in full page polish.

### D02 — Establish website entry and page shell

Create a separate build entry/output and local preview command; preserve the desktop build. Implement semantic three-section content, navigation, theme, fonts, benefits/footer and responsive wrappers from Paper. Product demo can initially lazy-load behind an accessible placeholder.

Acceptance: marketing content and download explanation remain readable if the demo fails or JavaScript is unavailable; no page-wide horizontal overflow at phone width; clear keyboard navigation; no native global CSS leakage. Include metadata and static/prerendered meaningful content in the build plan. Confirm the exact rendering method before declaring SEO complete.

Dependency: D01 architecture decision.

### D03 — Implement the connected Capture and Vocabulary session

Implement fixed fixtures, adapter notifications, all three reading samples, prepared-word interactions, viewport-clamped Capture, Save/Undo, library sorting/search within scope, and detail/history. Sources and translations stay associated with each Encounter. Include loading/error and unknown-edited-word behavior.

Acceptance: initial count 4; first save 5; repeat save remains 5 and increments Encounters; Undo restores the prior state. No involuntary navigation or scroll. Dark/light and narrow Capture remain App-consistent.

Dependency: D01 and D02.

### D04 — Complete Review, Replay, Reset and accessibility

Implement real Review responses and completion for the two due items, preserve state across website tabs, retain native pause semantics, and connect reset/replay to the shared session. Add reduced-motion handling and the approved flip transition. Validate pause when offscreen/backgrounded and recover without late actions.

Acceptance: new captures never enter this curated Review; rating cannot submit twice; completion totals match either choice; reset works from open Capture, revealed answer, rated feedback and completion. Keyboard-only operation, focus restoration and announcements are coherent.

Dependency: D03.

### D05 — Verify fidelity and prepare release

Compare current shared UI to App and marketing layout to Paper at desktop and phone sizes, both themes. Test cross-browser layout, touch pan/expanded demo, 150–200% zoom, focus visibility, reduced motion and long text. Measure initial page/demo loading before selecting performance budgets; keep demo code out of the critical marketing path where practical.

Complete metadata, social preview, real installer destination, contact/privacy/license destinations and approved reading assets. Provide a preview build and a version manifest before production publishing. Confirm hosting/domain when the target is known; no deployment is implied by this plan.

Dependency: D02–D04; publication also needs final assets and destinations.

## Testing and delivery evidence

- Session tests: new/repeat save, Undo, notifications, curated due filtering, idempotent ratings, reset and cancellation of stale async work.
- Integration tests: real shared UI across Capture → Vocabulary → detail → Review, not tests that merely duplicate markup.
- Visual/browser checks: desktop/phone dark/light, portal placement, Chinese text, native card height, anchors and intentional pan crops.
- Regression checks: existing relevant Windows App/Capture tests and type/build checks when shared code changes; separate website and desktop builds.
- Public-site checks: no native invocation or provider requests, meaningful fallback content, real links, route refresh and asset loading on the chosen host.
- Deliver source revision, fixture revision, UI screenshots, measured loading results and unresolved limitations with each preview.

## Keep the website synchronized with the App

Shared UI means component changes appear after rebuilding the site; it is not automatic production publishing. Store `appCommit`, released App version, `fixtureVersion`, `demoFlowVersion` and build date with each site release. Build the public demo from an approved released App revision, not whichever development HEAD happens to be newest.

Classify updates: shared style/component changes need rebuild plus visual checks; contract/flow changes need adapter/script changes plus journey tests; release URLs/copy need content checks. Trigger appropriate checks when shared UI/contracts change, and publish only after reviewing a preview. Do not claim that recorded pixel coordinates or videos self-update.

If a guided action recorder is later useful, record semantic actions against the demo adapter (sample/word selection, reveal, etc.) with a versioned format. Do not record screen coordinates as the source of truth. Playback cannot silently save on the visitor's behalf and must yield cleanly to input.

## Remaining decisions

- D01: exact shared-view extraction versus isolated-frame boundary, based on the working proof.
- Publication: domain/host, Windows release URL, final product name, owner/contact/legal destinations and approved reading samples. Original placeholder prose can unblock development.
- Accessibility review: whether the native two-second Undo interval needs a website-specific accessible alternative. Keep any deviation explicit; current Paper timing is the baseline.
- Performance budgets and initial rendering mechanism, selected after the first production-build measurement.

## Reference

[craftzdog/interactive-demo-tutorial](https://github.com/craftzdog/interactive-demo-tutorial), inspected 2026-09-17, replays CodeMirror transactions into a live editor and supports visitor takeover. Borrow the separation between actions, playback and live UI. Its React/Waku/CodeMirror-specific implementation is not a drop-in recorder for this Svelte App. Our confirmed no-initial-autoplay and explicit-save behavior takes precedence.
