# Paper handoff — Tickets 03–05

Achieve button revision: all eight Achieve buttons in the Mastered lists and Mastered detail state examples use red text and a red outline, retaining their existing geometry and neutral background. Use the theme's destructive semantic color for text/border: Light `#B64949`, Dark `#E89696`. This visual change does not alter the Achieve confirmation or lifecycle behavior.

Scope: Paper design only, continuing the approved Tickets 01–02. Application implementation, automated accessibility checks and physical Tauri verification remain open.

Design complete, 2026-09-08, in dependency order:

- [03 · Vocabulary & Detail](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/4-0): 18 artboards.
- [04 · Insights & Learning Summary](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/5-0): 8 artboards, including paired Minimum scrolled/day-tooltip proofs.
- [05 · Native Capture & Accessibility](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/6-0): 40 artboards.

Light/Dark pairs cover product states; separate 150% text and Forced Colors examples are labeled accessibility proofs. Snapshot and previously approved Ticket 01–02 pages are preserved.

## Ticket 03 behavior inventory

Authority: WindowsApp.svelte, WindowsWordRow.svelte, WindowsApp.test.ts, lib/types.ts, CONTEXT.md and ADR 0004.

- Active excludes Mastered and includes Learning/Paused. Mastered is separate from Achieved. Switching views clears search and returns to page 1. Active/Mastered search trims and case-folds display form and translation; Achieved also searches lemma.
- Active/Mastered use 10 items per page, First/Previous, editable page number, Next/Last. Page entry clamps/floors to valid bounds on blur/submit. Achieved is a scrolling filtered list without pagination.
- Mastered row and detail expose Achieve after confirmation stating the deletion date. Success switches to Achieved and refreshes. Achieve failure remains a page error. Confirmation remains the existing native confirmation boundary; Paper only documents its content.
- Achieved rows expose achieved date, deletion deadline, remaining days and normal/warning/urgent state. Select all affects filtered items; selection may retain other items. Unachieve returns selected items to Mastered and exposes Undo/dismiss. Undo re-achieves the same items. Permanent delete requires confirmation and has no Undo.
- Row selection awaits getWord before opening detail, initially focuses close, and returns focus to the trigger on dismissal. A failed detail request remains a page error; no invented retry command. Existing Try again refreshes the page. The existing row can be selected again.
- Sheet hierarchy: word, optional translation fallback, status/Encounter count, Mastered-only Achieve, Encounter contexts and source attribution. Keep encounter order and complete source text. No new edit/delete/source-link actions. Escape/focus trap are implementation accessibility requirements.
- Empty Active/Mastered, no matching results, no Achieved, initial loading, refresh errors, lifecycle errors and Undo errors remain distinct. Samples are illustrative data.

## Ticket 04 behavior inventory and data limits

Authority: WindowsApp.svelte, App.svelte, lib/types.ts, backend.ts and WindowsApp.test.ts. GlobalInsight provides all-time Vocabulary encountered, Encounters saved, Reviews completed, Remembered, Forgot and Currently achieved. It retains anonymous lifetime totals after item deletion. The lifetimeRatingBreakdownComplete flag requires the existing coverage explanation when false. TodayView provides total due and planned review counts; neither is an all-time value.

The user subsequently requested a GitHub-style Vocabulary log from the new Snapshot reference, replacing the Weekly Rhythm placeholder and placed in the second content row, below Captured/Reviewed/Due and above All-time detail. The user explicitly chose to count every successful vocabulary save, including repeated saves of the same word, and requested blue rather than green intensity colors. Reviews, failed saves and translation previews are not vocabulary saves. This is a newly requested daily-count metric, not a reinterpretation of the existing distinct lifetime Vocabulary count.

The old Shared Progress bars are hardcoded and the current interface has no daily series. Paper's populated heatmaps are illustrative samples: 242 saves over September 9, 2025–September 8, 2026, with 7 on September 8. Application implementation must supply real per-day successful-save counts; until then use the explicit unavailable state. Never infer a daily distribution from lifetime totals. The daily contract was accepted on 2026-09-09 in `docs/adr/0005-local-date-vocabulary-log.md`: fixed local save dates, successful Undo deducted once from the original date, indefinite anonymous daily retention unaffected by lifecycle deletion, and explicit unknown/partial legacy coverage. See `docs/plans/2026-09-09-paper-ui-implementation.md` for implementation sequencing. These decisions are planned, not implemented.

Vocabulary log uses 53 Sunday-first calendar columns with month labels, 10px cells, 3px gaps, 2px cell corners, and a five-level blue legend: 0, 1–2, 3–5, 6–9, 10+ saves. Theme tokens are `--color-{light,dark}-activity-{0..4}`. Light values: `#EBEDF0`, `#D6E9FF`, `#9CCBFF`, `#4A96E0`, `#1763AB`. Dark values: `#262626`, `#173458`, `#205C91`, `#338BD0`, `#79C0FF`. Dates outside the rolling range are blank; missing history is unavailable, not zero. Known zero-save days use the neutral level. Preserve geometry with Skeleton while loading.

Compose the heatmap with shadcn-svelte Tooltip, ScrollArea and Skeleton; the calendar grid itself is a small custom data visualization, not the date-picking Calendar component. Tooltip on hover/focus gives the full date and exact save count. Provide an accessible grid with one tab stop, arrow-key day/week movement, an accessible date/count label for each day, and Escape dismissal. Do not rely only on color. Minimum preserves 12px labels and 10px cells, keeps weekday labels fixed and scrolls horizontally to older dates; latest dates are initially visible. The page scrolls vertically to keep All-time detail and Review Insights reachable. Paired scrolled proofs show the full lower sections and the date tooltip.

ReviewSessionInsight exists only for completed sessions; do not invent persistent recommendations or scores on Insights. Available session counts and attention wording may be shown when that session object exists; otherwise use the explicit no-session state. GlobalInsight is optional, so unavailable is distinct from numeric zero. Existing page refresh/Try again is the recovery path; no period selector was added.

## Ticket 05 behavior inventory and platform differences

Authority: FloatingCapture.svelte, WindowsFloatingCapture.svelte, WindowsOcrOverlay.svelte, their tests, captureBackend.ts, lib/types.ts and ADR 0003.

- Shared Selection Capture automatically translates and persists after translation success. Typed translation_unavailable/translation_failed offers Retry translation or Save without translation. Generic diagnostics do not imply those actions. OCR uses a read-only suggestion and Use this text confirmation before the same translate/persist flow.
- Windows Selection Capture automatically previews translation but requires explicit Save capture. Edit capture focuses the window; independent Selected text/Context/optional Translation fields use Apply changes. Changed vocabulary marks a retained translation stale and offers Translate again. Blank translation uses the existing Save capture action with withoutTranslation=true. OCR joins multiple recognized words into an editable Vocabulary/optional Context draft; Confirm precedes translation, and Save capture remains separate. A failed Confirm can be retried, and blank Vocabulary disables it.
- This is an actual platform difference. Ticket 05's blanket automatic-save wording conflicts with Windows implementation/tests and ADR 0003. Conversely Shared automatic save differs from ADR 0003's explicit final save decision. Preserve both implemented flows in this presentation work; reconciliation requires a separate behavior decision.
- Windows handles Achieved conflict with explicit Return to Learning/Cancel, atomic restore-and-save, and retained candidate on failure. Shared does not implement this UI; do not add it there.
- Shared permission_required exposes Allow Accessibility or Allow Screen Recording according to the typed command path. Empty selection offers Start Region OCR only when screenshot OCR is supported. Other failures retain their existing close/shortcut recovery. Windows empty_selection/unsupported_element can expose Start OCR, Manual Capture and Cancel; OCR failure offers Try Again. Windows permission errors have no invented system-settings button.
- Shared saved result contains word/translation/context and Undo. Windows saved result contains word/translation/Encounter result and Undo. Saved timeout is 4000ms. Shared hover pauses it; Windows hover and focus pause it. Request IDs suppress stale responses, old saves and dismissals; do not show a new stale-request error. Translation-stale is a distinct visible Windows state.
- Undo and save errors follow existing branches: Shared general failure replaces result presentation; Windows retains saved result and Undo with error. Window header close remains when no equivalent Cancel is present. Remove duplicate header X only in Windows OCR, OCR recovery and Achieved conflict states with Cancel/Cancel OCR. Windows Escape remains; do not claim Shared has an Escape handler absent from its code.
- OCR Overlay captures one display-local rectangle, normalizes reverse drags, ignores width/height under 4px, and converts with outerPosition/scaleFactor before submission. Escape cancels. The instruction is visible before dragging and hidden during selection. Selection rectangle is theme-independent with a double light/dark boundary; Forced Colors uses system Highlight/CanvasText. Full-screen OCR is not a rounded application window.

## Shared visual contract

Use the existing semantic palettes, Inter 24/14/12px hierarchy, 32px controls and 14px outer window corners. Minimum keeps the same type sizes. Capture uses the Snapshot plus control at the upper-right on root Today/Vocabulary/Insights only. Detail and nested flows omit it. Cancel and duplicate dismissal X must not coexist. Preserve Snapshot references and previously approved pages.

## Verification and implementation handoff

- Vocabulary uses Table, Tabs, Input, Badge, Pagination, Checkbox, Sheet, Alert, Skeleton and Sonner compositions. Native capture uses compact fields and buttons with a scrollable body and reachable footer. Paper mockups are editable drawing nodes; implementation must use the shadcn-svelte component APIs and existing package runner.
- Official component references consulted: [Table](https://www.shadcn-svelte.com/docs/components/table), [Sheet](https://www.shadcn-svelte.com/docs/components/sheet), [Tabs](https://www.shadcn-svelte.com/docs/components/tabs) and [Pagination](https://www.shadcn-svelte.com/docs/components/pagination), alongside the component set already reviewed for Tickets 01–02. No production components were installed for this design-only task.
- Computed-style comparisons confirmed Vocabulary and Insights regular/Minimum page headings remain 24px/29px, navigation and input text 14px/20px, and search captions 12px/17px. Minimum layout does not change the type scale.
- Screenshot review covered Light/Dark table alignment, long multilingual words/contexts, metadata wrapping, fixed pagination and bulk actions, Sheet scrolling, native Capture actions and corner clipping. Scroll indicators were moved outside text lanes; Light checked-state icons were corrected to primary foreground.
- The 150% native proof is explicitly separate from Minimum and demonstrates enlarged text with reachable Undo and scrolling. Forced Colors proofs illustrate one contrast palette; runtime must bind system colors rather than hardcode the example black/white/yellow values. These screenshots are not evidence of an actual Windows Forced Colors or OS-scaling run.
- During review, eight Shared capture artboards were found on the Vocabulary page and moved, preserving their layers, to Native Capture & Accessibility. Subsequent artboard creation and duplication explicitly reparent to the intended page root, independent of the page being browsed.
- Tests were read to preserve behavior, but application source was not changed and runtime tests/builds were not run. UI regression suites, keyboard focus/traps/return, screen readers, reduced motion, all supported OS text-scaling/window configurations, native activation/drag/dismissal, OCR coordinates and physical Tauri verification remain implementation acceptance work. No duplicate production styles were removed.
