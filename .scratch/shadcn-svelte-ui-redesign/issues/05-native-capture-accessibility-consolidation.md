# 05: Unify Native Capture and complete cross-platform accessibility

**What to build:** Apply the approved visual system to Shared Capture, Windows Capture, and the OCR Overlay, then verify the complete redesigned application across its supported window sizes, themes, accessibility modes, and platform-specific boundaries. Native Capture behavior remains exactly as implemented: Selection Capture automatically translates and saves, Region OCR Capture requires confirmation, and existing Undo, permission, dismissal, and recovery paths remain intact.

**Blocked by:** 01: Establish the visual system through Settings and Manual Capture; 02: Deliver Today and the focused Review flow; 03: Deliver Vocabulary management and detail; 04: Deliver the restrained Insights learning summary.

**Status:** ready-for-agent

- [ ] Before designing or implementing Native Capture, document the current state machines and platform differences for Selection Capture, Region OCR Capture, Translation, save, Undo, timeout dismissal, permission handling, and stale or failed requests; each transition retains an explicit presentation.
- [ ] Shared Capture and Windows Capture use the agreed hierarchy of Vocabulary Item, Translation, Context, status/source, and bottom actions within the actual 380×280 window constraint.
- [ ] Ready, Capture Candidate, translating, saved, existing Vocabulary Item, Undo, translation unavailable, translation failed, save without translation, permission required, empty selection, OCR Confirmation, saving, stale request, and recoverable-error states preserve their current triggers and available actions.
- [ ] The OCR Overlay keeps its current coordinate and cancellation behavior, provides a clearly visible instruction and selection rectangle, and remains visible in Light Theme, Dark Theme, and Windows Forced Colors.
- [ ] Motion is limited to brief opening and closing transitions, respects reduced-motion preferences, and does not alter activation, focus, dismissal timing, or interaction availability.
- [ ] Glass styling is not introduced; it is recorded only as a possible future Appearance setting and is not required by any current component or token.
- [ ] The complete application is visually and functionally verified at 1040×720, 840×600, 380×280, and 150% text scaling, including long English, German, and Chinese content.
- [ ] Keyboard focus, screen-reader names, status semantics, contrast, forced colors, platform window controls, title and drag regions, and focus return are verified across the redesigned surfaces.
- [ ] Superseded duplicated presentation styles are removed only after their replacements are verified, and existing UI tests, production builds, and relevant physical Tauri checks pass without changing application behavior.
