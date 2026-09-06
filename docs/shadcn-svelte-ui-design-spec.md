# Vocab Collector UI Design Specification v2

## 1. Purpose and authority

This is the agreed Windows handoff for the next desktop UI. Design direction is settled, work is split into five incremental tickets, and Paper design is complete for Tickets 01–02. No application UI implementation was performed during this design phase.

This is a presentation/component-system redesign, not a feature redesign.

> Non-negotiable: the redesigned UI must follow currently implemented app behavior. Components and styling may improve, but no action, state transition, validation, persistence rule, keyboard behavior, accessible name, Tauri boundary, or platform-window behavior may be added, removed, reordered, or silently reinterpreted.

When sources conflict, use this order:

1. running application behavior and automated tests;
2. this specification;
3. approved Paper screens;
4. shadcn-svelte and Notion as visual references.

Before each ticket, inventory the relevant current states, commands, transitions, loading/error recovery, focus behavior, and persistence, then map every behavior to an explicit UI state.

## 2. Current UI boundary

| Presentation | Current component |
| --- | --- |
| Shared main | `ui/src/App.svelte` |
| Windows main | `ui/src/windows/WindowsApp.svelte` |
| Shared capture | `ui/src/FloatingCapture.svelte` |
| Windows capture | `ui/src/windows/WindowsFloatingCapture.svelte` |
| Windows OCR overlay | `ui/src/windows/WindowsOcrOverlay.svelte` |

Where the shared main UI is older/simpler, the richer Windows implementation and its tests are authoritative. Share the visual system, but keep platform-specific presentation code until deliberately consolidated.

## 3. Agreed visual direction

- Notion-inspired hierarchy: calm, compact, content-first.
- shadcn-svelte-inspired primitives, borders, state treatments, and composition.
- Neutral black/white/gray surfaces with one low-saturation accent; not strict monochrome.
- Light-first; Dark derives from the same semantic tokens; also support System.
- Fixed narrow sidebar with Lucide-style line icons.
- Simple linear mark plus `Vocab Collector` wordmark.
- Restrained radii, borders, and elevation; avoid cards around every row.
- Inter in Paper; system sans-serif fallback in code, covering English, German, and Chinese.
- Minimal functional motion, mainly brief opening/closing transitions.
- Respect reduced motion. No decorative ambient animation.
- Glass is not in the current design; record it only as a possible future Appearance theme.

Avoid oversized dashboard metrics, gradients, high-saturation color, streaks/scores, pressure copy, color-only status, unfamiliar desktop interactions, and copying demo layouts that do not fit the product.

Button and badge labels require explicit line height, centered alignment, and no wrapping unless a responsive rule explicitly permits it.

## 4. Window and responsive requirements

| Surface | Size |
| --- | --- |
| Main | `1040 × 720` |
| Minimum main | `840 × 600` |
| Native capture | `380 × 280` |

Verify applicable screens at 150% text scaling. Preserve identity, status, navigation, and primary actions. Reduce padding/gaps before type size; allow scrolling; wrap/move/hide lower-priority helper text or metadata; never deform or truncate primary controls. Native capture needs purpose-built layouts, not scaled desktop screens.

## 5. Shell and navigation

Fixed sidebar order:

1. Today
2. Vocabulary
3. Insights
4. Settings

`Insights` is final and first-class, replacing presentation-level `Progress` naming without changing services. Include brand, active route, Local mode, visible focus, and platform-safe title/drag regions.

Normal main pages have a bottom-right `+ Capture` floating action. Review opens in the main content area with the sidebar retained and Today active; it is not a fifth navigation destination. Hide `+ Capture` during focused Review.

## 6. Tokens and themes

Required semantic colors:

```text
background foreground card card-foreground popover popover-foreground
primary primary-foreground secondary secondary-foreground muted muted-foreground
accent accent-foreground destructive destructive-foreground border input ring
success success-foreground warning warning-foreground
sidebar sidebar-foreground sidebar-accent sidebar-accent-foreground
```

Required structural tokens:

```text
radius-sm radius-md radius-lg shadow-floating shadow-dialog
space-1 through space-8 font-sans
font-size-caption font-size-body font-size-title font-size-word
sidebar-width content-max-width capture-width breakpoint-min-desktop
```

Approved Paper baseline: primary `#4F7085`, accent `#E8EFF3`, success `#4E7A61`, warning `#9A7138`, destructive `#B64949`, light border `#E3E3DF`, dark background `#191A1B`, dark surface `#222426`, dark border `#35383B`, dark primary `#87A8BB`. Small contrast corrections are allowed if semantic usage remains stable.

Do not hard-code theme values in pages. Verify contrast and Windows Forced Colors. The OCR rectangle must remain visible independently of the theme.

## 7. shadcn-svelte component policy

Paper uses shadcn-svelte as a visual/semantic reference; its components are hand-drawn mockups, not imported production code. In implementation use the maintained [component docs](https://www.shadcn-svelte.com/docs/components), add only needed primitives, and compose product-owned workflows.

| Need | Primitive |
| --- | --- |
| Actions | Button / Button Group |
| Fields | Input / Input Group / Field / Label / Textarea |
| Choices | Native Select first; Select only when justified; Switch |
| Grouping | Card / Separator |
| Lifecycle views/status | Tabs / Badge |
| Manual Capture/detail | Dialog / right Sheet |
| Feedback | Sonner for transient success; Alert for actionable/persistent feedback |
| Loading/empty | Skeleton / Spinner / Empty |
| Review progress | Progress plus textual count |
| Pagination/shortcuts | Pagination / Kbd |
| Help | Tooltip; optional Popover, never for essential-only instructions |
| Vocabulary | Table; defer Data Table |
| Insights | Chart only with real data and equivalent text |
| Shell | Sidebar or compact project-owned composition |

Initial bounded set:

```text
Button Card Input Input Group Textarea Field Label Native Select Switch Tabs
Badge Alert Dialog Sheet Sonner Progress Skeleton Spinner Empty Pagination
Kbd Separator
```

Defer Data Table, Chart, Command, Popover, and the full Sidebar until a demonstrated need justifies them.

Project components remain named concepts: `AppShell`, `VocabularyRow`, `ReviewCard`, `CaptureCard`, `ShortcutRecorder`, `EncounterTimeline`, `SummaryStat`, `SettingsSection`, and `OcrRegionOverlay`.

## 8. Screen behavior

### Today

Priority: title/context, Today's Plan, planned review count and `Start review (n)`, total due and estimated time, Recent Captures, then `+ Capture`. No streaks or scores. Recent rows open the existing detail experience.

States: data, stable-geometry loading, no captures with Manual Capture, nothing due with review unavailable, and refresh failure with only the existing retry action. Preserve last stable content when possible.

### Focused Review

Authoritative flow:

1. Start only with a valid queue and no refresh requirement.
2. Show bar and text progress (`Word 1 of 3`).
3. Recall shows word/context, not translation or rating.
4. `Show answer` reveals translation and Forgot/Remembered.
5. Submission disables close and rating actions.
6. Success shows rating, next due date, and encounter count.
7. Repeated forgetting adds guidance and `Review contexts`.
8. `Next` advances only after success.
9. Final card opens Review Complete.
10. Close pauses, clears reveal/result presentation, refreshes, and returns to Today.
11. Refresh failure blocks stale Resume and exposes the existing refresh retry.

On submission failure retain the same revealed card, do not advance, and retry the same rating with the same logical submission ID.

Complete shows reviewed/remembered/forgotten counts, estimated due by end of tomorrow, attention words, relevant context action, and `Back to Today`.

### Vocabulary

Use a compact Notion-database-style Table: Word, Translation, Status, Encounters, Last Seen. Preserve Active/Mastered/Achieved, canonical lifecycle terms, search, count, pagination, loading, no-results, empty, and recovery.

Selecting a row opens an accessible right Sheet ordered Word, Translation, Status, Encounter Timeline. Preserve Escape, focus trap, close, and focus return. At narrow widths keep identity/status/actions and reduce secondary metadata.

### Insights

First-class content is Captured, Reviewed, Due, Weekly Rhythm, and Review Insights. Use only existing metrics/calculations/periods. Charts need textual interpretation and cannot rely on color. Define real, loading, empty, partial, and recoverable states. Do not invent analytics, scores, streaks, or recommendations.

### Settings and Manual Capture

Settings is one scrollable column: Languages, Review, Capture, Appearance. Preserve every existing value, constraint, shortcut state, save timing, validation, and failure.

Manual Capture opens from `+ Capture` as an accessible Dialog with existing fields, validation, save, Undo, Escape, focus trap, and focus return.

Successful page-level saves use a brief toast. Validation/actionable errors remain beside the affected control.

### Native Capture and OCR

Selection Capture automatically translates and saves. Region OCR requires confirmation. Preserve Undo, permissions, dismissal/timeout, stale requests, failures, and recovery.

Within `380 × 280`, order content as Vocabulary Item, Translation, Context, source/status, actions. Cover ready, candidate, translating, translation unavailable/failed, edit, OCR confirmation/candidates, saving, saved/Undo, existing item, permission, empty selection, stale request, and recovery states where implemented.

OCR stays a project-owned full-screen overlay preserving coordinates and Escape cancellation; Dialog/Card positioning must not interfere.

## 9. Accessibility and feedback

- Preserve accessible names and semantic native controls.
- Visible keyboard focus everywhere.
- Dialog/Sheet trap focus, support Escape where currently implemented, and return focus.
- Progress always includes text; status always includes readable text/icon.
- Essential instructions never exist only in a tooltip.
- Verify screen-reader status semantics without changing operation timing.
- Verify themes, contrast, reduced motion, and Windows Forced Colors.
- Keep stable content during recoverable errors; never discard a Review card on failed save.

## 10. Paper handoff

Project: `vocab-collector-ui-design-v2`

File ID: `01M1VPGM8REJ3ETVGRWV6WMY23`

- [Ticket 01 page](https://app.paper.design/file/01M1VPGM8REJ3ETVGRWV6WMY23/2-0)
- [Ticket 02 page](https://app.paper.design/file/01M1VPGM8REJ3ETVGRWV6WMY23/3-0)

Ticket 01 completed artboards:

- Foundations / Light
- Foundations / Dark
- Settings / Default / 1040×720
- Settings / Minimum / 840×600
- Manual Capture / Dialog / 1040×720
- Component States / Ticket 01

It defines 59 semantic tokens. Button labels were checked for centered single-line typography. The ambiguous `Core controls` heading was renamed `Primitive Components / States`.

Ticket 02 completed artboards:

- Today / Default / 1040×720
- Today / States
- Review / Recall / 1040×720
- Review / Revealed + Result
- Review / Complete / 1040×720
- Today + Review / Minimum / 840×600

When continuing on Windows, reuse tokens and names, use realistic multilingual content, create native-size artboards, and screenshot each section to inspect hierarchy, spacing, clipping, contrast, repetition, and button-label alignment. Record conflicts instead of inventing behavior.

## 11. Incremental tickets

1. **Foundation, Settings, Manual Capture** — tokens, shell, themes, settings, dialog, feedback, responsive/accessibility baseline. Paper complete.
2. **Today and Focused Review** — review-first Today and full existing Review state flow. Blocked by 01. Paper complete.
3. **Vocabulary Management and Detail** — compact Table, lifecycle views, search/count/pagination, row states, right Sheet. Blocked by 01.
4. **Insights Learning Summary** — Captured, Reviewed, Due, Weekly Rhythm, Review Insights using current data. Blocked by 01.
5. **Native Capture and Cross-platform Accessibility** — shared/Windows Capture, OCR, themes, Forced Colors, native sizing, text scaling, accessibility, and verified consolidation. Blocked by 01–04.

Detailed ticket files are in `.scratch/shadcn-svelte-ui-redesign/issues/`.

## 12. Implementation and completion checks

For each ticket: inventory behavior/tests; map states; add the smallest primitive set; build product components; migrate one working slice; verify supported normal/loading/empty/disabled/success/error/recovery states; verify all relevant sizes and 150%; verify keyboard, focus return, screen readers, motion, contrast, and Forced Colors; run existing tests and add focused presentation/accessibility tests; remove duplicated styles only after replacement verification.

Preserve Svelte/Vite/Tauri boundaries, frontend events/backend commands, scheduling and metrics, save/Undo/retry/submission semantics, native sizing/focus/drag/dismissal timing, and OCR coordinates.

The redesign is complete only when all listed screens and current states are represented; multilingual content and scaling do not break controls; accessibility/platform checks pass; existing workflows and tests remain valid; reusable tokens/components replace appropriate page-local primitives; and shared/Windows presentations have one deliberate vocabulary-learning identity.
