# 01: Design the capture hero from first impression to saved feedback

**What to build:** A complete Paper journey showing how visitors understand the product and try collecting unfamiliar vocabulary while reading.

**Blocked by:** None (can start immediately).

**Status:** ready-for-review

- [x] Design desktop/mobile hero layouts in dark/light themes, with dark as the first-visit presentation and a theme switch.
- [x] Use provisional Vocab Collector branding, serif marketing headings, sans-serif body text, app-aligned colors, generous whitespace, and a quiet reading photograph. No film-derived branding, palette, or imagery.
- [x] Compose Capture words. Keep reading., supporting copy, and Download for Windows, marking unresolved assets and release details as placeholders.
- [x] Show Reading sample with V's introduction as default, Everyday reading, and Tech reading. Use a labeled placeholder for the exact monologue excerpt until available; reserve three prepared vocabulary choices and Simplified Chinese translations.
- [x] Show a simplified browser with three context tabs and highlighted words embedded in the article. Clicking a word starts playback; there is no playback on page load or separate Capture launcher.
- [x] Anchor the capture window beside the selected word and match the current Windows app capture layout. Show pointer/touch participation, keyboard focus, confirmation, Saved with Undo, and return to reading.
- [x] Annotate visitor takeover, Replay, sample switching without data loss/autoplay restart, and reduced-motion/manual presentation as intended behavior, not implemented functionality.
- [x] Use readable mobile capture layouts rather than shrinking a desktop window. Record the visual choices established by this section for reuse.

**Scope:** UX/UI design in Paper only. Deliver artboards and interaction annotations; no app code, backend, automated tests, builds, hosting, or deployment. Use existing app visuals as references. If Paper is unavailable, report the blocker rather than substituting tools. Mark unselected content/assets as placeholders. Prototype connections are optional where supported; do not represent annotated behavior as functional software.
## Review delivery — 2026-09-12

Completed through Paper MCP only for this implementation turn. No desktop input or application code changes. Tickets 02–05 are not started.

Paper project: https://app.paper.design/file/01M2AMNRXMP0TZ753GPD28ZSJV/1-0

Eight named artboards:
- 01 · Hero / Desktop / Dark
- 02 · Hero / Desktop / Light
- 03 · Hero / Mobile / Dark
- 04 · Hero / Mobile / Light
- 05 · Capture journey / States & behavior
- 06 · Hero / Desktop / Capture open
- 07 · Hero / Mobile / Capture open / Dark
- 08 · Hero / Mobile / Capture open / Light

Reviewed Paper screenshots for spacing, typography, contrast, alignment, artboard fit, and repetition. Fixed dark text inheritance, light-theme icons, serif preservation, clipped artboard height, and storyboard card alignment. Mobile uses full-size controls and separate confirmation layout. Artboards are design states with annotations, not a working capture prototype.

Monologue excerpt and final download destination remain clearly marked placeholders as permitted by this ticket. Sample sentences are original placeholder copy, not film dialogue. Temporary V mark is not a final brand asset. Reading photo study: Sanika V, https://unsplash.com/photos/J75vNLEr4aE . Final photo selection remains subject to design review.

Local review screenshots are saved under the landing-page review folder for ticket 01. Awaiting user visual review before proceeding.

## Review revision — 2026-09-12

Applied the user's three requested corrections through Paper MCP only. This revision supersedes the original automatic-playback and separate Capture-control design above.

- Replaced the hero's word-list widget with a simplified browser, context tabs, and three inline highlighted words. Clicking a word starts its capture sequence; initial load waits for input.
- Matched the current Windows implementation (`ui/src/windows/WindowsFloatingCapture.svelte` and `ui/src/native-capture.css`), after comparing the native-capture designs in `vocab-collector-uiux`. The implementation is authoritative where the older Paper reference differs: Captured precedes the word, context uses a left border, Edit is outlined, and Save is filled.
- Desktop capture is 380 × 280px. Positioning follows `crates/capture/src/placement.rs`: center on the selection, prefer above, otherwise below, with a 12px gap and an 8px viewport margin. The staged browser is the demo's placement boundary.
- Mobile keeps the article visible behind a 340px-wide capture window. Text wraps at readable sizes; the popup is clamped inside the browser rather than replacing it.
- Reworked 05 into a two-by-two browser storyboard, retaining the native popup size. Saved stays in the same window with Undo; dismiss after four seconds unless hovered or focused.
- Added 09 · Browser / Context variants for Everyday reading and Tech reading, each with three inline words and Simplified Chinese translation mappings. The V excerpt remains a labeled placeholder using original sample copy.
- Reviewed all themes and affected states for spacing, typography, contrast, alignment, clipping, and consistency. Fixed mobile context wrapping and light Replay icon contrast.

Paper contains design states and interaction annotations, not executable playback. Tickets 02–05 and application code remain untouched. Status remains ready-for-review.
