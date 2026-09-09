# Vocab Collector UI Design Specification v2

## 1. Purpose and authority

This is the agreed Windows handoff for the next desktop UI. Design direction is settled, work is split into five incremental tickets, and Paper design is complete for Tickets 01–05. No application UI implementation was performed during this design phase; runtime acceptance remains open.

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

Minimum windows use the same typography and control sizes as normal windows. A narrower window must never trigger larger fonts. Reduce padding/gaps, rearrange content and allow vertical scrolling; use collapsible sections only if still needed. The current Settings design uses one scrollable form, and Recent Captures uses a bounded scrolling list. Separately verify user-requested/OS 150% text scaling as an accessibility test, not as the default Minimum design. Preserve identity, status, navigation and primary actions. Native capture needs purpose-built layouts, not scaled desktop screens.

The Paper application-window silhouette uses 14px corners (`radius-lg`) in Light, Dark and Minimum frames. Clip sidebar, title region and modal overlays to this shared outer shape. This user-approved visual requirement does not change native window actions; platform implementation remains separate.

## 5. Shell and navigation

Fixed sidebar order:

1. Today
2. Vocabulary
3. Insights
4. Settings

`Insights` is final and first-class, replacing presentation-level `Progress` naming without changing services. Include brand, active route, Local mode, visible focus, and platform-safe title/drag regions.

Top-level Today, Vocabulary and Insights pages have a compact Capture icon button at the upper right of the content area, beside the page title. Use the plus-only `capture` component in the Snapshot page and retain the accessible name `Manual capture`. Show it only on the first-level page reached from the sidebar. Settings never shows it. Review opens in the main content area with the sidebar retained and Today active; it is not a fifth navigation destination. Review, Review Complete and other nested content do not show this Capture entry. There is no bottom-right Capture floating action.

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

Current Paper proposal (2026-09-08), based on the user's Snapshot references: Light background `#FFFFFF`, surface `#F7F7F7`, primary `#262626`, accent `#E8EFF3`, border `#E3E3E3`, input boundary `#898989`; Dark background `#171717`, surface `#202020`, primary `#E5E5E5`, accent `#27343D`, border `#363636`, input boundary `#777777`. Focus uses `#4F7085` in Light and `#87A8BB` in Dark. Inter, 32px controls, 10px control radius and 14px dialog radius follow the new reference direction. Both themes are fully represented; System follows the OS after a successful settings save. This replaces the previous file's visual baseline and remains subject to user design review.

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

Priority: title/context, Today's Plan, planned review count and `Start review (n)`, total due and estimated time, then Recent Captures. Capture is a compact utility beside the title at the upper right. No streaks or scores. Recent rows open the existing detail experience.

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

First-class content is Captured, Reviewed, Due, Vocabulary log, All-time detail and Review Insights. Captured and Reviewed use lifetime Vocabulary encountered and Reviews completed; Due uses the current Today backlog. Include lifetime Encounter and rating totals, Currently achieved, and the existing incomplete-rating-history explanation. The user-requested Vocabulary log replaces Weekly Rhythm: a GitHub-style blue calendar heatmap in the second row, above All-time detail, counting every successful vocabulary save including repeated words. Show the past 12 months with full-date/exact-count tooltips, a five-level legend, and accessible keyboard navigation. Minimum keeps the same font and cell sizes with horizontal calendar scrolling and vertical page scrolling. The current interface has no daily-series contract: populated Paper cells are samples, and implementation must use real daily counts or the unavailable fallback. ReviewSessionInsight content appears only when the existing session object is available. Charts need textual interpretation and cannot rely on color. Define real, loading, empty, partial and recoverable states without inventing scores, streaks or recommendations. See the Ticket 03–05 handoff for count semantics and data-contract requirements.

### Settings and Manual Capture

Settings is one scrollable column: Languages, Review, Capture, Appearance. Preserve every existing value, constraint, shortcut state, save timing, validation, and failure.

Manual Capture opens from the top-level Capture icon as an accessible Dialog with existing fields, validation, save, Undo, Escape, focus trap, and focus return. A dialog with Cancel must not also display an X button performing the same dismissal. Cancel is Manual Capture's single visible dismissal action; Escape remains available. This user-authorized presentation revision removes duplicate controls without changing save/cancel semantics. Native application-window controls remain distinct from dialog dismissal.

Successful page-level saves use a brief toast. Validation/actionable errors remain beside the affected control.

### Native Capture and OCR

Shared Selection Capture automatically translates and saves. Windows automatically previews translation and requires explicit Save capture. Shared OCR uses Use this text before translation/save; Windows OCR confirms an editable draft, previews translation, then requires Save capture. Preserve these implemented platform differences, Undo, permissions, dismissal/timeout, stale requests, failures and recovery. Shared automatic saving conflicts with ADR 0003's separate final save decision; this presentation work records that conflict rather than changing behavior.

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

Current project: `vocab-collector-uiux`

File ID: `01M1P47W73ZMCYR34TWR3C1459`

- [Snapshot references — preserved](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/1-0)
- [Ticket 01 — Foundation, Settings & Capture](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/2-0)
- [Ticket 02 — Today & Focused Review](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/3-0)
- [Ticket 03 — Vocabulary & Detail](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/4-0)
- [Ticket 04 — Insights & Learning Summary](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/5-0)
- [Ticket 05 — Native Capture & Accessibility](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/6-0)

Ticket 01 has 16 artboards: paired Light/Dark Settings at 1040×720, scrolled Capture/Appearance Settings, Manual Capture Dialog, foundations/component states, Settings/Capture behavior states, and 840×600 Settings top/scrolled and Manual Capture layouts using unchanged typography.

Ticket 02 has 18 artboards: paired Light/Dark Today, Review Recall, Revealed, repeated-forgetting result, Complete, Today loading/empty/recovery, Review submission/results/recovery, and 840×600 Today and Revealed Review layouts using unchanged typography.

Ticket 03 has 18 artboards: paired Active, Mastered, Achieved, right detail Sheet, Minimum Active/Achieved/Sheet and loading/empty/recovery/lifecycle-confirmation sheets. Ticket 04 has 8 artboards: paired Insights overview, Minimum overview, Minimum scrolled/day-tooltip proofs and data/coverage/recovery sheets. The blue Vocabulary log shows illustrative daily successful-save counts; the unavailable fallback remains required until real daily data is available.

Ticket 05 has 40 artboards: 28 paired Windows/Shared Capture states at 380×280; separate Light/Dark 150% user-text proofs and a native Forced Colors example; Light/Dark OCR ready/selection and a Forced Colors selection example; paired Native Capture behavior and cross-surface accessibility handoffs. All Native Capture work is on its own page. Creation and duplication explicitly target the intended page root to prevent accidental placement while another page is being browsed.

The file defines 116 tokens (two semantic palettes, default aliases, structural/type tokens, and ten theme-specific Vocabulary log intensity tokens). Button labels, multilingual text, alignment, spacing, theme contrast and representative minimum-window layouts were visually inspected. Computed-style checks confirm normal and Minimum typography match. State sheets include loading, disabled, failure, retry, Undo, pause and completion semantics. Paper is a static design artifact: runtime keyboard, screen-reader, Forced Colors, user text-scaling and regression checks remain implementation work.

Behavior inventory, conflicts and implementation handoff are recorded in `.scratch/shadcn-svelte-ui-redesign/paper-ticket-01-02-handoff.md`.

Tickets 03–05 behavior inventories, platform differences, data limits and verification boundaries are recorded in `.scratch/shadcn-svelte-ui-redesign/paper-ticket-03-05-handoff.md`.

Previous draft, retained for history only: `vocab-collector-ui-design-v2`, file `01M1VPGM8REJ3ETVGRWV6WMY23`. It is not the current visual reference.

When continuing on Windows, reuse tokens and names, use realistic multilingual content, create native-size artboards, and screenshot each section to inspect hierarchy, spacing, clipping, contrast, repetition, and button-label alignment. Record conflicts instead of inventing behavior.

## 11. Incremental tickets

1. **Foundation, Settings, Manual Capture** — tokens, shell, themes, settings, dialog, feedback, responsive/accessibility baseline. Paper complete.
2. **Today and Focused Review** — review-first Today and full existing Review state flow. Blocked by 01. Paper complete.
3. **Vocabulary Management and Detail** — compact Table, lifecycle views, search/count/pagination, row states, right Sheet. Blocked by 01. Paper complete.
4. **Insights Learning Summary** — Captured, Reviewed, Due, Vocabulary log and Review Insights. Blocked by 01. Paper complete; the requested daily successful-save log requires a new daily data contract for implementation.
5. **Native Capture and Cross-platform Accessibility** — shared/Windows Capture, OCR, themes, Forced Colors, native sizing, text scaling, accessibility, and verified consolidation. Blocked by 01–04. Paper complete; runtime accessibility and production consolidation remain implementation work.

Detailed ticket files are in `.scratch/shadcn-svelte-ui-redesign/issues/`.

## 12. Implementation and completion checks

For each ticket: inventory behavior/tests; map states; add the smallest primitive set; build product components; migrate one working slice; verify supported normal/loading/empty/disabled/success/error/recovery states; verify all relevant sizes and 150%; verify keyboard, focus return, screen readers, motion, contrast, and Forced Colors; run existing tests and add focused presentation/accessibility tests; remove duplicated styles only after replacement verification.

Preserve Svelte/Vite/Tauri boundaries, frontend events/backend commands, scheduling and metrics, save/Undo/retry/submission semantics, native sizing/focus/drag/dismissal timing, and OCR coordinates.

The redesign is complete only when all listed screens and current states are represented; multilingual content and scaling do not break controls; accessibility/platform checks pass; existing workflows and tests remain valid; reusable tokens/components replace appropriate page-local primitives; and shared/Windows presentations have one deliberate vocabulary-learning identity.
