# Paper design handoff — Tickets 01–02

Date: 2026-09-08. Scope: Paper UI/UX design, not application implementation.
File: https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459
Reference: existing snapshot page (preserved).

## Visual decision

The user's new Snapshot references supersede the previous Paper visual baseline: graphite surfaces, neutral primary buttons, 32px controls, 10px control radius and 14px dialog radius. Inter provides a consistent multilingual type hierarchy. A restrained blue-gray accent remains for focus and selection. Light and Dark share semantic roles; System follows the OS after a successful settings save.

## Behavior inventory before design

Authority inspected: WindowsApp.svelte, WindowsApp.test.ts, WindowsWordRow.svelte, App.svelte, ShortcutRecorder.svelte, types.ts, ADRs 0001/0003/0004 and the UI design specification.

| Area | Existing behavior / explicit design states |
| --- | --- |
| Shell | Today, Vocabulary, Insights, Settings in the specified order; Today remains active during focused Review. Per user revision, Capture uses the Snapshot plus icon at the upper-right beside the heading, only on Today/Vocabulary/Insights root pages. Settings and nested screens, including Review and Complete, omit it. Native title and drag regions stay separate from interactive content. |
| Settings language | Source: auto/en/de/fr/es/zh-Hans/zh-Hant. Target: same explicit languages, no auto. |
| Settings review | Review time; daily limit 1–50; recent captures 1–100; Achieved retention 10/20/30/60 days, prospective only; automatic Achieve after 30 uninterrupted days. |
| Settings capture | Two independent shortcuts; blank disables; Launch at login. Windows text entry remains text entry. Shared ShortcutRecorder separately represents idle, recording, modifier-only, invalid unmodified key, recorded and Escape cancellation. |
| Settings appearance | System/Light/Dark and Reduce motion. Explicit Save settings, disabled while saving. Read-back/persisted values apply only after save succeeds. Save failure retains previous applied theme. |
| Settings feedback | 1200ms success message; per-control notification/shortcut/autostart errors; general save failure. Successful persistence followed by Today refresh failure keeps new saved preferences. |
| Manual Capture | Word or phrase and Context required after trimming; Translation optional. Open clears form, focuses first field. Tab trap, Escape/Cancel and return focus. Per user revision, Cancel is the single visible dialog dismissal; duplicate X removed. No new source or translation action. |
| Capture save | Pending disables save; failure retains draft. Success closes dialog, refreshes and exposes saved encounter plus Undo and dismiss. Existing-item success reports updated Encounter count. Undo failure preserves saved notice and page error. |
| Achieved conflict | Inline Achieved notice, explicit Return to Learning action then atomic restore-and-capture. No automatic restore. |
| Today | Planned count, total due, estimate from existing data; recent rows open detail. Initial loading, zero captures, nothing due, refresh failure/retry and stable-content recovery. No new dashboard calculations. |
| Review | Valid non-stale queue → recall → revealed → pending → result → Next. Word and context only during recall. Translation unavailable and context unavailable have explicit fallback text. |
| Review result | Rating, next date, Encounter count; repeated forgetting gives guidance and Review contexts. Existing Windows result replaces the translation block. |
| Review failure | Preserve revealed word/context/translation; Retry Remembered or Retry Forgot uses the same logical submission ID and original rating. No Next or alternate rating. Close disabled during submission. |
| Review pause | Close clears reveal/result, invalidates stale requests and refreshes; fresh queue allows Resume in recall. Failed refresh blocks Resume and offers Retry Review refresh. Spec moves presentation back to Today without changing queue logic. |
| Completion | Last Next obtains session insight then refreshes; insight failure retains result + Next retry. Refresh failure shows Review saved + Retry Review refresh. Complete shows reviewed/remembered/forgotten, next-day estimate, attention words, Back to Today. |
| Details | Recent row and Review contexts use existing right Sheet, Escape, close, focus trap/return. Full Vocabulary work remains Ticket 03. |

## Conflicts / boundaries

- The later Ticket 05 inventory clarifies the Native Capture conflict: Shared saves after successful translation, while Windows requires an explicit final Save capture, matching ADR 0003. Shared still differs from ADR 0003/current domain docs. Follow `paper-ticket-03-05-handoff.md` for the implemented platform flows; do not infer behavior from Snapshot examples.
- Current Windows completion attention words are text, without a context button. Keep them as text; Review contexts remains on the repeated-forgetting result.
- Do not add shortcut-recording behavior to Windows just to match Shared.
- Static Paper frames describe focus/scroll/keyboard behavior; they cannot prove runtime focus traps, Forced Colors, reduced motion or screen-reader behavior. Existing implementation checkboxes remain open.

## Verification and deliverables

Design revision complete. Snapshot contains six reference artboards, including the user's added Capture component; references were preserved. All design work is on two separate pages:

- [Ticket 01](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/2-0): 16 artboards, eight matched Light/Dark pairs. Settings top, Settings scrolled to Capture/Appearance, Manual Capture Dialog, foundations/components, behavior states, Minimum Settings top, Minimum Settings scrolled and Minimum Manual Capture.
- [Ticket 02](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/3-0): 18 artboards, nine matched Light/Dark pairs. Today, Recall, Revealed, repeated-forgetting result, Complete, Today states, Review states, Minimum Today and Minimum Revealed Review.

Regular application frames are 1040×720. Minimum frames are 840×600 with the same typography: page titles 24px, labels/navigation 14px, captions 12px, Review word 36px and context 18px. Compact spacing and vertical scrolling accommodate content. Settings is one scrollable form, represented by top and lower scroll positions at both sizes. Minimum Today shows three recent rows in a scroll viewport, with further rows retained below. State sheets use content-driven height rather than a simulated application window. Explicit user/OS 150% text scaling is a separate runtime accessibility check, not a consequence of the Minimum window size.

Revision verification compared computed font sizes between regular and Minimum frames, and reviewed Light/Dark screenshots of Settings, Today, Review and Manual Capture. No lower-right Capture remains; Settings and nested Review screens have no Capture entry. All four Manual Capture variants retain Cancel and Escape without a duplicate dialog X. Review retains its close control because it has no Cancel; native window controls remain distinct application-level actions. Paper scroll positions specify the intended behavior but do not test runtime scrolling.

Screenshot review covered spacing, typography, contrast, row alignment, primary-control fit, multilingual content and unnecessary repetition. Initial renderer inheritance and delayed screenshot issues were corrected and rechecked. Theme-paired screenshots verified the final controls and actions, including removal of Capture during focused Review. Completion initially focuses its heading; Back to Today receives its visible ring only after keyboard navigation. Do not automatically focus the completion button.

Computed palette checks (WCAG relative luminance):

| Pair | Light | Dark |
| --- | --- | --- |
| Foreground/background | 17.93:1 | 17.18:1 |
| Muted text/card | 5.36:1 | 6.31:1 |
| Primary button text/fill | 14.50:1 | 14.23:1 |
| Accent text/fill | 6.10:1 | 7.99:1 |
| Error text/card | 4.85:1 | 7.16:1 |
| Input boundary/card (corrected) | 3.27:1 | 3.64:1 |

Input tokens were increased from the reference's faint boundaries to Light #898989 / Dark #777777. Decorative separators retain the quieter border token. Disabled controls are intentionally dimmed. These static color calculations do not certify the running app's accessibility.

## Implementation notes

- Application-window frames use 14px outer corners (`--radius-lg`) with clipped overflow across Light, Dark and Minimum variants, including modal backgrounds. Reference and component/state sheets retain their original canvas shape. Native window integration remains implementation work.

- The 106 Paper tokens use Tailwind-style namespaces: `--color-{light|dark}-*`, default `--color-*` aliases, `--font-sans`, `--text-*`, `--space-*`, `--radius-*`, `--container-*`, `--breakpoint-*`, weight, tracking and leading. Map these to the specification's semantic roles, not literal page colors.
- `sidebar-width`: 184px at regular and Minimum sizes; `content-max-width`: 760px; `capture-width`: 380px reserved for Ticket 05; minimum desktop: 840px.
- Shadow contract: `shadow-floating: 0 4px 12px #00000022` remains reserved for floating surfaces; Capture is now in the page heading. `shadow-dialog: 0 20px 64px #00000030` applies to dialogs. Paper's token API does not have a shadow type, so these values are recorded here.
- Component geometry: 32px controls at regular and Minimum sizes; Capture icon button 36×32px, based on the Snapshot component; 6/10/14px radius scale. Explicit label line-height and no wrapping for primary controls. Wrap/stack surrounding layout instead. Preserve the accessible name Manual capture on the plus-only control.
- Official shadcn-svelte documentation consulted: Button, Field, Dialog, Progress, Alert, Empty, Native Select, Switch, Sonner, Skeleton, Badge and Card at https://www.shadcn-svelte.com/docs/components. No components.json was present; no CLI initialization or production component installation was needed for this Paper-only request.
- These are editable Paper mockups and documented compositions, not imported Svelte components. Implementation should use the existing repository package runner and the shadcn-svelte skill's component composition rules.
- Preserve existing accessible names even where presentation labels differ. Native title controls/drag region are placeholders for the platform's existing window behavior, not a new custom-window implementation.
- Retain existing controls' save/Undo/Escape behavior and exact backend messages. Error copy and metrics in the artboards are sample data, not new error APIs or analytics.
- Completion omits the attention section when there are no attention words. Full Vocabulary/detail design remains Ticket 03; existing detail behavior is retained for Recent Captures and Review contexts.
- Forced Colors implementation: use system Canvas/CanvasText/ButtonText/Highlight roles, retain visible borders/focus and text status, and do not rely on box shadow alone. Honor reduced motion without changing operation timing. These need runtime verification.

No application source was changed and no application tests were run for this design-only work. Existing test names were inspected to map behavior. Runtime keyboard focus, accessibility semantics, OS integration and regression acceptance remain open in the implementation tickets.
