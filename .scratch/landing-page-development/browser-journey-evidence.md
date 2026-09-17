# Ticket 01 — browser App journey

Implemented 2026-09-17 against App source a696b5e plus the working-tree changes for this ticket.

## Run locally

- `pnpm dev:landing` → http://127.0.0.1:5174/landing/
- `pnpm build:landing` → `ui/dist-landing/`
- `pnpm preview:landing` → http://127.0.0.1:4174/landing/
- Existing `pnpm dev`, `pnpm build`, and Tauri configuration retain their existing entry/output.

The page is a disposable integration proof, not the three-section marketing design. Its static wrapper is in `ui/landing/index.html`; `demo.html` mounts `ui/src/landing/demo/Journey.svelte`. The separate Vite configuration reuses the existing ui package and lockfile. No additional package installation or Rust process is needed.

## Chosen presentation boundary

Use one same-origin iframe for the complete interactive journey. Capture, Vocabulary, detail and Review all share one `LandingSession` inside that document. The host never copies App markup or receives mutable library data. `protocol.ts` defines a versioned ready/theme contract; both receivers check origin and source window, and incoming theme values are validated.

Scoped views in the host document were evaluated first against the current source:

- `styles.css` contains unscoped body minimum dimensions and selectors for main, nav, buttons and legacy views.
- `theme.css` puts App tokens and dark variants on `:root`.
- Sheets/dialogs portal to body, and the App mounts a document-level toaster and keyboard handlers.
- Capture previously targeted all html/#app nodes and mutated the body class.

A host-document solution would therefore require coordinated changes across legacy CSS, theme ownership, portal targets and keyboard scope, beyond a small view extraction. The iframe preserves those existing document assumptions. Browser checks below establish that the fallback works rather than treating it as an untested escape hatch. This keeps the Windows/macOS presentation boundary in ADR 0001 intact.

Small shared changes:

- `WindowsApp.embedded` defaults to false. Embedded navigation exposes Vocabulary and Review, starts on Vocabulary, and leaves root appearance ownership to the host message handler. Existing desktop navigation and appearance defaults remain intact.
- `WindowsFloatingCapture.embedded` defaults to false. Embedded mounting avoids body mutation; native document CSS now explicitly requires the native Capture body class. Capture still uses its real edit/save/undo/timer behavior.
- Existing `WindowsTitlebar`, `VocabularyTable`, `VocabularyDetail`, progress rings, Review markup and dialogs are reused without a second presentation implementation.

The frame stylesheet handles proof-specific sizing and narrow layout; the host imports none of the App styles. No full-repository reorganization or framework migration was needed.

## Browser-only session contract

`LandingSession` implements Backend; `createCaptureAdapter` implements WindowsCaptureBackend. Imports of their contracts are type-only. The shared presentation still imports native API modules for desktop defaults, but no native operation/listener is called with these injected adapters. The complete integration test asserts this explicitly.

Four fixed English Vocabulary Items have Simplified Chinese translations. Only serendipitous and resilient are initially due. The reading sample stages ephemeral; explicit Save creates an Encounter and notifies the mounted App. Repeated saves retain identity and add Encounters; Undo checks the latest Encounter and restores it without rolling back a later Review. Review submissions are idempotent by submission ID and reject items outside the curated scope. All values returned to the UI are copies.

Fixture clock: `2026-09-17T12:00:00.000Z` (UTC), injectable in session tests. Generated session IDs are deterministic. Review dates/metrics are illustrative fixture behavior, not a Rust scheduler running in the browser. The shared UI formats dates in the visitor's locale/time zone; the browser verification used Europe/Berlin. Fixed future-due estimates are calculated against the demo clock, not the visitor's calendar.

All session data lives in memory. Reload creates a new session. Translation capability on the Capture adapter means a local prepared translation; unsupported text fails explicitly and can be manually edited/saved without translation. No storage, credentials, database, OCR or translation provider is used. Native window controls remain disabled with an explanation in the surrounding page.

## Verification

- Type check: `pnpm check`.
- Shared journey and protocol tests: `pnpm --dir ui test src/landing/demo/Journey.test.ts src/landing/protocol.test.ts`.
- Desktop regressions: WindowsApp, WindowsFloatingCapture, WindowsNavigation, WindowsTitlebar and ResumeReview suites (77 tests).
- Independent landing and existing desktop frontend production builds.
- Browser verification in Codex's Chromium browser at desktop size and a 390 × 844 viewport: explicit Capture Save changes 4 items to 5; detail shows the original sentence, Chinese saved translation and source; two-word keyboard Review completes with 1 remembered / 1 forgot and 0 due.
- Both light and dark render correctly. At narrow width the host has no horizontal overflow; the Vocabulary table retains its own horizontal scrolling. Capture width was 333px with a 12px left inset inside the frame's available viewport. Detail stays inside the frame.
- Capture initial focus, Shift+Tab wrapping, Escape and focus return to the prepared word were checked. Detail Escape restores the selected Vocabulary Item button. Capture's two-second close preserves the session and returns focus.
- Theme changes are sent without remounting the frame, preserving saved Vocabulary and completed Review. The marketing wrapper keeps its independent typography/layout.

## Follow-up limits

- Ticket 02 replaces the proof wrapper with the marketing page and meaningful static content, demo loading/failure UI and final site entry conventions. Build output currently serves the proof at `/landing/`.
- Ticket 03 adds all reading samples and viewport-clamped anchored Capture. This proof uses a centered modal to validate the shared Capture boundary; it is not the final reading popup placement.
- Website tab preservation, Replay, atomic Reset and stale asynchronous action cancellation remain in their planned tickets. Current App navigation retains native pause/resume behavior.
- Ticket 06 adds the designed phone pan/expanded mode and the complete accessibility audit. Current narrow layout is usable but not final Paper fidelity. Fixed proof frame height and nested scrolling need refinement there.
- Ticket 07 handles cross-browser testing, production assets/links and release synchronization. The demo production chunk is about 547 kB before gzip and emits Vite's chunk-size warning; code splitting/performance budgets remain release work. The existing desktop build also emits its chunk-size warning.
- This is a local preview only; no deployment occurred.
