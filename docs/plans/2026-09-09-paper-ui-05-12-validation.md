# Paper UI Tickets 05–12 implementation and validation

Date: 2026-09-09. Environment: Windows. Implementation baseline: `99e5b1467e1dd023cc413d1dd811399265cecd2e` on `codex/windows-platform-v2`.

Tickets 05–11 are implemented. Ticket 12 is **verification-partial**, not closed: the automated checks and the observations below passed, but the full native and accessibility matrix remains outstanding. The approved local Paper handoff documents supplied the design reference; the Paper connector was unavailable for a fresh comparison of all five pages.

## Ticket progress

| Ticket | Implemented result | Remaining acceptance |
| --- | --- | --- |
| 05 | Today plan, bounded recent list, loading/empty/recovery states, independent optional Insight failure | Full design/native matrix |
| 06 | Focused Review presentation, safe pause/Today return, completion and existing retry semantics | Complete native keyboard/theme matrix |
| 07 | Real lifetime and due metrics, explicit missing/history/session states | Screen reader verification |
| 08 | Persistent daily counts, atomic original-day Undo, migration coverage and accessible rolling calendar | Native capture checks on each supported platform |
| 09 | Windows capture layout, editable OCR confirmation, fixed reachable actions, combined hover/focus timeout guard | Native selection, activation, closing, scaling and Forced Colors |
| 10 | Shared capture presentation with existing auto-save and typed recovery | macOS/Linux native checks |
| 11 | OCR instruction and dual boundary styling, Forced Colors roles, reverse-drag regression | Multi-monitor and mixed-scale native OCR |
| 12 | Unused replaced styles removed, tests/build and two review tracks completed | Full physical matrix listed below |

## Code changing boundary

Today and Insights were extracted into presentation components; Windows and Shared business pages remain separate. Review retains its submission IDs, rating and retry semantics, request invalidation and scheduling. Capture retains platform-specific save, permission, translation and OCR workflows. OCR coordinate conversion and native bridges were not changed. Shared styles still used by existing pages were retained.

Core changes serve only ADR 0005: domain query types and repository contract, application delegation, desktop query registration, SQLite schema 6 migration and transactional count updates. Each successful save records the system local save date, independently of supplied capture timestamps. Repeated words count as separate saves. Undo decrements the original date once in the same transaction as encounter removal; failures roll back both. Anonymous daily rows survive Achieve, Unachieve and purge, while purged encounter/date associations are removed. Migration history is unknown before the coverage boundary, partial on the migration day and complete thereafter; unknown dates are not represented as zero. A clock seam covers date, restart and timezone cases in tests. The UI reads backend totals instead of adjusting them locally.

Shared Capture still auto-saves after translation and uses a read-only OCR suggestion. This is the existing conflict with ADR 0003's editable confirmation followed by separate saving; ticket 10 explicitly requires preserving that behavior. No scheduler, normalization, provider, permission implementation or Achieve eligibility changes were introduced.

## Automated verification

| Command | Result |
| --- | --- |
| `pnpm --dir ui test` | 10 files, 103 tests passed |
| `pnpm --dir ui check` | 0 errors, 0 warnings |
| `pnpm --dir ui build` | Passed |
| `cargo test -p vocab-domain -p vocab-storage -p vocab-application -p vocab-desktop --lib --tests` | 106 tests passed |

Regression coverage includes optional Insight failure without blocking Today, Review return/retry behavior, calendar keyboard navigation and date rollover, save/Undo rollback, duplicate saves and Undo, migration/restart, local date versus capture timestamp, timezone/DST changes, recapture and purge, capture timeout overlap, OCR cancellation and reverse dragging. Demo purge/Undo now preserves anonymous daily totals just like the storage contract.

Initial infrastructure failures were resolved: sandboxed esbuild child processes required approved execution outside the sandbox, and a running validation executable had to be stopped before Rust could replace it. One full-suite calendar test exceeded its default timeout; its timeout was increased to 15 seconds and the final full suite passed. These were not left as ignored failures. Final logs are local under `.scratch/paper-ui-implementation/12-*` and are not committed.

## Browser observations

Checked the actual rendered app in the in-app browser: 1040×720 dark Today/Insights; 840×600 light with independent 150% text and long English, Chinese and German content on Today/Review; and 840×600 dark with 150% text on Insights. Review actions remained reachable by scrolling/focus. The root had no horizontal overflow. The calendar retained 10px cells, horizontally scrolled to the latest date, and used one tab stop. Arrow navigation updated the active day; Escape dismissed its tooltip while preserving day focus. Five legend colors were inspected in the rendered styles.

The development-only capture fixture was checked at 380×280 with 150% text: light selection/edit states and dark editable OCR confirmation. Long content scrolled while footer actions remained reachable. These browser observations do not establish native window activation, dragging or permission behavior. Shared capture behavior was covered by tests, not a corresponding platform screenshot.

## Windows native observations

Ran a separate validation application identifier, `app.vocabcollector.paper-ui-validation`, to keep its SQLite data separate from the user's normal application. In the native main window, Manual Capture opened with visible input focus; Shift+Tab wrapped to Cancel; Escape closed it and returned focus to the header trigger; Enter reopened it. Saved a synthetic vocabulary item and multilingual context through the real desktop command and SQLite path. Today then showed the real item and due count.

Opening the recent item displayed its Encounter in the detail Sheet. Tab stayed on the Sheet's available close control; Escape returned focus to the recent item. Native Insights showed Captured 1, Reviewed 0, Due 1, and the current calendar day as `1 capture recorded; partial history`. Native Undo was not exercised; automated UI and Rust tests cover it. The validation process was stopped after inspection. An initial UI Automation set-value failure was bypassed with normal click/type input; it was a tool limitation, not an application failure.

## Unverified physical acceptance

- Fresh comparison with all five current Paper pages.
- macOS/Linux Shared capture permissions, save-to-log flow and window behavior.
- Real screen reader announcements, operating-system Reduced Motion and Windows Forced Colors.
- OS display scaling, mixed-scale/multiple monitors and native OCR selection/cancellation coordinates.
- Windows native selection capture, capture-window drag/activation/exit and full timeout interaction.
- Saved theme propagation between separate WebViews and the complete Light/Dark/System matrix.

CSS support, accessible markup, browser fixtures and jsdom tests are not substitutes for these checks. The corresponding ticket checkboxes remain open.

## Standards review

The review identified a custom Today recovery container where the shared Alert component was required. It was replaced with Alert.Root, Alert.Title and Alert.Description, and the redundant styles were removed. Re-review confirmed the finding resolved. No outstanding standards findings.

## Spec review

Two findings were fixed with regression tests: Windows capture hover and focus needed a combined timeout guard; Demo purge had to remove saved-date associations so an expired Undo could not decrement retained anonymous counts. Shared asynchronous save completion also respects an already-hovered window. Re-review confirmed both findings resolved and reported no additional actionable spec issues.
