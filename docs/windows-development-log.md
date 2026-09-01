# Windows Development Log

This log records implementation evidence for `docs/windows-platform-tickets.md`. Evidence uses the status vocabulary defined in `docs/windows-platform-plan-v2.md`: **Verified automated**, **Verified Windows physical**, **Not run**, **Blocked**, and **Unsupported**.

## 2026-09-01 - W-10 correct and save a Native Capture manually

### Implementation

- Added shared capture-session transitions for correcting a resolved Native Capture, attaching an optional user-entered translation, explicitly choosing no translation, saving once, and undoing once.
- Routed correction and request-bound Undo through `PlatformCaptureWorkflow` and target-independent desktop commands; no Windows API or adapter type entered the shared application boundary.
- Extended the independent Windows floating capture presentation with editable selected text/context, optional translation, explicit **Save without translation**, saved feedback, Undo, busy/error states, and request-aware backend calls.
- Kept automatic translation unavailable on Windows and states that limitation directly in the capture card. W-10 does not enable the translation capability or call a hidden provider.

### Key decisions

- The original portable `CaptureCandidate` remains owned by the shared coordinator. Correction replaces only its user-editable text/context while preserving origin, source metadata, URL, and bounds.
- A manual translation becomes a portable `TranslationResult` using the persisted source/target language settings. An empty translation moves the session to the existing explicit save-without-translation gate.
- Correction, save, Undo, and dismissal all validate the same request UUID. Save and Undo each have a single-success state; persistence failures leave the action retryable.
- Windows-specific code is limited to its Svelte presentation/backend bridge. `platform/windows` and `platform-api` required no W-10 changes.
- TDD was limited to the shared coordinator/application workflow, desktop command surface, and Windows capture UI seams because they contain the state transitions and regression risk.

### Main files changed

- `crates/capture/src/coordinator.rs` and `crates/capture/tests/coordinator.rs`
- `crates/application/src/platform_capture.rs` and `crates/application/tests/platform_fakes.rs`
- `apps/desktop/src-tauri/src/bootstrap.rs`
- `apps/desktop/src-tauri/src/commands/capture.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src-tauri/tests/command_contract.rs`
- `ui/src/windows/captureBackend.ts`
- `ui/src/windows/WindowsFloatingCapture.svelte`
- `ui/src/windows/WindowsFloatingCapture.test.ts`
- `docs/windows-platform-tickets.md`
- `docs/windows-development-log.md`

### Tests and results

| Command | Result | Evidence |
|---|---|---|
| `cargo fmt --all --check` | PASS | Verified automated; exit code 0 |
| `cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings` | PASS | Verified automated; no warnings |
| `cargo test -p vocab-capture -p vocab-application` | PASS | Verified automated; correction, manual translation, explicit untranslated save, save-once, request safety, persistence retry, and Undo tests passed |
| `cargo test -p vocab-desktop --test command_contract` | PASS | Verified automated; 26 tests passed and the additive correction/Undo commands are present on the platform-neutral command surface |
| `cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux` | PASS | Verified automated; 112 tests passed and one physical Notepad test remained ignored |
| `pnpm check` | PASS after sandbox-external rerun | Verified automated; 0 errors and 0 warnings. The managed-sandbox attempt was blocked by the known `esbuild spawn EPERM` restriction |
| `pnpm test -- ui/src/windows/WindowsFloatingCapture.test.ts` | PASS after sandbox-external rerun | Verified automated; the repository test script ran all 7 frontend files and all 46 tests passed |

### Not yet verified

- **Not run:** physical floating-window interaction for edit, manual translation, save, explicit untranslated save, Undo, dismissal, and source-focus restoration. The current session does not establish that this host is a Windows 11 x64 physical machine, so no Windows runtime behavior is marked verified.
- **Not run:** keyboard, Narrator, high-contrast, text-scaling, and visual-fit checks for the expanded capture form; comprehensive evidence remains owned by W-13.
- **Not run:** save/Undo interaction against a live WebView2 window and the on-device SQLite database.
- Automatic Windows translation remains **Unsupported** with the user-visible manual-translation and save-without-translation alternatives.
- OCR capture, OCR confirmation, candidate ranking, and Windows Graphics Capture remain outside W-10 and unchanged.

### Next ticket starting point

W-11 starts with a request-safe shared correction/save/Undo workflow and a Windows capture card capable of accepting one resolved candidate. It should add only explicit bounded, memory-only Windows Graphics Capture/OCR, remove the Windows use of Cocoa coordinate conversion, require confirmation before persistence, and keep cancellation persistence-free. It must not implement W-12 ambiguous-candidate ranking or accessible candidate-list selection early.

## 2026-08-31 - W-01 Windows automated baseline

### Implementation

- Published the approved tracer-bullet breakdown in `docs/windows-platform-tickets.md`.
- Established the Windows-target automated baseline without changing product code or enabling native capabilities.
- Confirmed the existing GitHub Actions Windows lane uses `windows-latest`, Rust 1.98.0, Node.js 22, and pnpm 11.19.0. It currently runs formatting, Clippy, workspace tests/build, the Windows adapter tests, and frontend check/test/build; it does not build an NSIS artifact.
- Confirmed the Windows adapter remains the static unsupported skeleton: all capability flags are false and all providers return capability-specific `PlatformError::Unsupported` results.

### Key decisions

- W-01 is an evidence/baseline ticket. It adds no core behavior, so no new TDD cycle or production test was warranted.
- Existing public seams are the baseline: Rust workspace tests, Windows adapter capability contracts, frontend typecheck/tests/build, and the desktop command contract.
- The current session runs on a Windows host, but the host has not been confirmed as a Windows 11 x64 physical machine. No runtime or manual result is classified as **Verified Windows physical**.
- Frontend commands that require the bundled `esbuild` helper were rerun outside the managed sandbox after their first attempt failed with `spawn EPERM`. The successful reruns used the exact ticket commands; the initial failures are environment restrictions rather than product failures.

### Main files changed

- `CONTEXT.md`
- `docs/adr/0001-independent-windows-presentation.md`
- `docs/windows-platform-plan-v2.md`
- `docs/windows-platform-tickets.md`
- `docs/windows-development-log.md`

The glossary, ADR, and v2 plan were approved planning inputs produced immediately before W-01 and committed with the ticket baseline. No product source, manifest, CI workflow, platform capability, or existing content in `docs/windows-development.md` was modified for W-01.

### Toolchain

| Tool | Observed version | Required/CI version | Status |
|---|---|---|---|
| `rustc` | 1.98.0 | 1.98.0 or newer | Verified automated |
| `cargo` | 1.98.0 | Rust 1.98 toolchain | Verified automated |
| Node.js | 24.19.0 | 22 in CI; 22 or newer in project docs | Verified automated; differs from CI major version |
| pnpm | 11.19.0 | 11.19.0 | Verified automated |

Baseline commit before W-01: `2844499583840bdc2d6c21f9ee36a7a8f36b53ea`.

### Tests and results

| Command | Result | Evidence |
|---|---|---|
| `cargo fmt --all --check` | PASS | Verified automated; exit code 0 |
| `cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings` | PASS | Verified automated; exit code 0 |
| `cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux` | PASS | Verified automated; 79 tests passed, independently recounted with the same package selection and `-- --list`, including domain, storage, application, capture, platform contracts, desktop command contracts, and Windows skeleton contracts |
| `cargo build --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux` | PASS | Verified automated; exit code 0 |
| `pnpm check` | PASS after sandbox-external rerun | Verified automated; `svelte-check` reported 0 errors and 0 warnings. Initial managed-sandbox attempt was Blocked by `esbuild` `spawn EPERM` |
| `pnpm test` | PASS after sandbox-external rerun | Verified automated; 4 files and 26 tests passed. Initial managed-sandbox attempt was Blocked by `esbuild` `spawn EPERM` |
| `pnpm build` | PASS after sandbox-external rerun | Verified automated; Vite transformed 119 modules and produced the production bundle. Initial managed-sandbox attempt was Blocked by `esbuild` `spawn EPERM` |
| `cargo test -p vocab-platform-windows` | PASS | Verified automated; 2 capability tests passed and confirmed that no native capability is enabled |

### Not yet verified

- **Not run:** `pnpm tauri dev`. The current session does not establish that the host is a Windows 11 x64 physical machine.
- **Not run:** main-window startup and WebView2 runtime behavior.
- **Not run:** guest SQLite initialization/restart persistence through the Tauri application.
- **Not run:** Today, Vocabulary, Review, Settings, and Manual Capture physical smoke tests.
- **Not run:** global shortcut registration and conflict behavior in a live Windows session.
- **Not run:** COM, UI Automation, OCR, permission/error recovery, capture-window activation/focus, mixed-DPI displays, tray, notifications, autostart, NSIS, upgrade, and uninstall.

The deferred runtime checks remain owned by their corresponding Hybrid or Physical tickets. No native Windows capability is claimed by W-01.

### Next ticket starting point

W-02 starts from a green automated baseline. It should centralize Windows presentation selection in the UI/desktop composition boundary, introduce a Windows-owned main and capture presentation without changing macOS page behavior, preserve the existing frontend backend contract, and omit the static Progress view. Before editing, rerun the narrow frontend and desktop command-contract baselines and review ADR 0001.

## 2026-08-31 - W-02 independent Windows presentation

### Implementation

- Added a desktop composition command that reports the compile target's presentation family as `windows` or `shared` without exposing a Windows API or native type.
- Centralized main/capture presentation selection in the frontend entry point.
- Added independent Windows main and capture Svelte roots. The Windows main root provides Today, Vocabulary, Review, and Settings navigation and deliberately omits the static Progress view.
- Preserved the existing shared/browser and macOS presentation roots as the fallback presentation family.
- Kept the Windows roots intentionally free of backend data and native capture behavior; those remain owned by W-03 and later tickets.

### Key decisions

- Presentation family is a desktop composition concern, not a platform capability and not an application-layer concept. The command lives in the Tauri desktop crate and never enters `platform-api` or shared application crates.
- `ui/src/main.ts` is the only point that combines presentation family with main/capture window kind. Windows feature components do not inspect an OS string.
- Browser/demo mode has no Tauri internals and therefore selects the existing shared presentation.
- A failed presentation-family invocation falls back to the existing shared presentation instead of leaving the WebView blank.
- W-02 establishes only the Windows shell. It does not copy the existing monolithic product page into Windows, which would prematurely implement W-03 through W-05 and import unfinished macOS behavior.
- The Windows capture root scopes a `windows-capture-document` body class to its mounted lifetime so the shared main-window `840x600` minimum does not overflow the configured `380x280` capture WebView. The class is removed on unmount to avoid leaking capture-window constraints into another presentation.

### Main files changed

- `apps/desktop/src-tauri/src/commands/mod.rs`
- `apps/desktop/src-tauri/src/commands/presentation.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src-tauri/tests/command_contract.rs`
- `ui/src/lib/presentation.ts`
- `ui/src/lib/presentation.test.ts`
- `ui/src/main.ts`
- `ui/src/windows/WindowsApp.svelte`
- `ui/src/windows/WindowsApp.test.ts`
- `ui/src/windows/WindowsFloatingCapture.svelte`
- `ui/src/windows/WindowsFloatingCapture.test.ts`
- `docs/windows-platform-tickets.md`
- `docs/windows-development-log.md`

No file in `crates/domain`, `crates/application`, `crates/storage`, `crates/capture`, `crates/platform-api`, or `platform/windows` changed.

### Tests and results

TDD seams:

- Frontend composition mapping: red because `ui/src/lib/presentation.ts` did not exist; green after the centralized mapping was added.
- Desktop presentation-family command: red because the command module did not exist; green after the target-aware composition command was registered.
- Windows main presentation behavior: red because the Windows root did not exist; green after the independent navigation shell was added.
- Windows capture document sizing: red because the capture root inherited the main-window minimum dimensions; green after capture-only document sizing and lifecycle cleanup were added.

| Command | Result | Evidence |
|---|---|---|
| `pnpm check` | PASS | Verified automated; `svelte-check` reported 0 errors and 0 warnings |
| `pnpm test` | PASS | Verified automated; 7 files and 29 tests passed |
| `pnpm build` | PASS | Verified automated; Vite transformed 124 modules and produced the production bundle |
| `cargo test -p vocab-desktop --test command_contract` | PASS | Verified automated; 11 tests passed |
| `cargo fmt --all --check` | PASS | Verified automated; exit code 0 during the ticket verification run |

### Not yet verified

- **Not run:** `pnpm tauri dev`; W-02's required commands do not include a runtime launch.
- **Not run:** visual inspection of the Windows main and capture roots in WebView2.
- **Not run:** macOS runtime presentation selection. Its compile-target mapping and browser fallback are covered automatically, but no macOS physical-machine run occurred.
- **Not run:** Manual Capture, SQLite persistence, Today/Vocabulary data, and Undo in the Windows presentation; owned by W-03.
- **Not run:** Review behavior, Settings behavior, tray, native capture, UIA, floating-window behavior, OCR, packaging, upgrade, and uninstall; owned by later tickets.
- Windows adapter capability flags remain unchanged and false.

### Next ticket starting point

W-03 starts with independent Windows roots selected through a green composition contract. It should connect the Windows presentation to the existing `Backend` for Manual Capture, Today, Vocabulary, detail, repeat Encounter, Undo, and their loading/empty/error states. It must not add Review behavior beyond its entry point, Settings side effects, tray integration, or any Windows native API.

## 2026-08-31 - W-03 Manual Capture through Today and Vocabulary

### Implementation

- Connected the independent Windows main presentation to the existing frontend `Backend` contract.
- Added Manual Capture with required word/context, optional user translation, disabled/saving states, and recoverable errors.
- Added Today summary and recent captures, Vocabulary search/list, Vocabulary detail with Encounter contexts, repeat Encounter feedback, and Undo.
- Refreshes Today, Vocabulary, and an open detail view after save or Undo so visible counts and contexts share one backend source of truth.
- Added explicit loading, empty collection, no search result, and recoverable backend failure states.
- Added status and last-seen metadata to reusable Windows vocabulary rows, plus review duration/empty-queue context on Today and source metadata in detail.
- Left Review and Settings as entry placeholders; their behavior remains owned by W-04 and W-05.

### Key decisions

- W-03 reuses the stable `Backend` and existing Tauri library commands. No new desktop command, application use case, storage API, or Windows-native type was required.
- The Windows root accepts a `Backend` dependency with `createBackend()` as its production default. Tests exercise the same public UI behavior with the existing `DemoBackend`, without mocking component internals.
- Repeat normalization, one Vocabulary Item with multiple Encounters, transactional persistence, and soft-delete Undo remain in shared application/storage. Windows UI only requests operations and refreshes frontend-ready views.
- The saved result remains independently visible while the user opens Vocabulary detail, allowing Undo to refresh both the list and the open detail.
- Manual Capture save failures remain inside the active dialog with the user's input intact; retry submits the same form rather than running an unrelated page refresh.
- Manual Capture sets only `captureOrigin: manual`; it does not fabricate a source application when none was supplied.

### Main files changed

- `ui/src/windows/WindowsApp.svelte`
- `ui/src/windows/WindowsApp.test.ts`
- `ui/src/windows/WindowsWordRow.svelte`
- `docs/windows-platform-tickets.md`
- `docs/windows-development-log.md`

No file in `crates/domain`, `crates/application`, `crates/storage`, `crates/capture`, `crates/platform-api`, `platform/windows`, or `apps/desktop` changed.

### Tests and results

TDD seam:

- Windows user workflow through the public `Backend` contract: red while the Windows root exposed only empty page sections; green after Manual Capture, repeat Encounter, Today/Vocabulary refresh, detail, Undo, and recoverable load failure behavior were added.
- Manual Capture retry: red while a failed save was reported behind the modal; green after the dialog received its own visible error state while preserving form input for retry.
- Existing application/storage public tests were retained as the contract evidence for normalization, repeat Encounter ownership, SQLite persistence, and Undo; no shared implementation changed.

| Command | Result | Evidence |
|---|---|---|
| `pnpm check` | PASS | Verified automated; `svelte-check` reported 0 errors and 0 warnings |
| `pnpm test` | PASS | Verified automated; 7 files and 32 tests passed |
| `cargo test -p vocab-application -p vocab-storage` | PASS | Verified automated; application and storage suites passed, including repeat Encounter, Undo, and cross-process SQLite persistence |
| `cargo test -p vocab-desktop --test command_contract` | PASS | Verified automated; 11 tests passed |
| `pnpm tauri dev` | PASS for startup only | Verified Windows physical; on Windows build 26100.7171 x64, Vite started at `127.0.0.1:5173`, the Rust dev build completed, and `vocab-desktop.exe` launched. The process was then intentionally stopped with Ctrl+C; this does not verify the manual workflow or restart retention |

### Not yet verified

- **Not run:** the interactive portion of the `pnpm tauri dev` restart smoke test against the real Windows WebView2 and SQLite application database; the executable launch was observed, but the native window could not be operated through the available tool channel.
- **Not run:** Manual Capture, repeat Encounter, detail, Undo, and restart retention through a packaged or development Tauri WebView on the Windows physical session.
- **Not run:** visual inspection at Windows scaling levels and with keyboard/accessibility tools; comprehensive coverage remains owned by W-13.
- Review workflow, Settings persistence/effects, tray, native selection, OCR, translation, and native floating-window behavior remain outside W-03 and unchanged.
- Windows adapter capability flags remain unchanged and false.

### Next ticket starting point

W-04 starts from a Windows presentation whose Today data refreshes through the shared backend after library mutations. It should implement only the shared due-queue Review workflow, forgotten/remembered outcomes, close/completion states, and Today refresh. It must not calculate review dates in the UI or introduce Settings/native Windows behavior.

## 2026-09-01 - W-04 Windows Review workflow

### Implementation

- Added a Start Review action to Windows Today using `TodayView.reviewQueue` and the backend-provided estimate.
- Added Windows Review empty, ready, in-progress, paused, recoverable failure, and completed states.
- Added Forgotten and Remembered actions through the existing `Backend.submitReview` contract.
- Closing a partially completed queue refreshes Today from the backend and preserves a paused session entry point for the remaining due queue.
- Completing the queue refreshes Today and exposes a completion state before returning to Today.

### Key decisions

- The Windows UI treats `TodayView.reviewQueue` as the authoritative due queue and submits only the existing `forgot` and `remembered` ratings.
- Review dates, stability, due ordering, queue limits, and persistence remain entirely in `vocab-domain` and `vocab-application`; the UI performs no date arithmetic.
- A close refreshes the backend queue and resumes from its first remaining item instead of retaining a stale frontend snapshot.
- A failed rating keeps the current Review card visible, preserves its position, and allows the same action to be retried.
- A failed close/completion refresh cannot expose Resume or claim that Today was refreshed. The already-saved rating is never resubmitted; a dedicated refresh retry reloads the authoritative queue first.
- Close is disabled while a rating submission is in flight so queue refresh and rating persistence cannot race.
- The global error retry delegates to Review recovery while the queue is gated, so a successful refresh from Today also re-enables Start Review without a second hidden recovery step.
- W-04 does not implement Settings, notifications, tray behavior, native capture, or Windows platform capabilities.

### Main files changed

- `ui/src/windows/WindowsApp.svelte`
- `ui/src/windows/WindowsApp.test.ts`
- `docs/windows-platform-tickets.md`
- `docs/windows-development-log.md`

No file in `crates/domain`, `crates/application`, `crates/storage`, `crates/capture`, `crates/platform-api`, `platform/windows`, or `apps/desktop` changed.

### Tests and results

TDD seam:

- Windows Review through the public `Backend`: red while Today had no Start Review action and Review was a placeholder; green after start, Forgotten/Remembered submission, close/refresh/resume, completion, and Today refresh were implemented.
- Empty queue and recoverable rating failure are covered through visible UI states. Rating tests assert the portable rating values but never derive or inspect review dates.
- Close-refresh failure, global retry recovery, and rating/close serialization have regression coverage to prevent duplicate scheduling, blocked starts, or skipped due items.

| Command | Result | Evidence |
|---|---|---|
| `pnpm check` | PASS | Verified automated; `svelte-check` reported 0 errors and 0 warnings |
| `pnpm test` | PASS | Verified automated; 7 files and 37 tests passed |
| `cargo test -p vocab-domain -p vocab-application` | PASS | Verified automated; shared queue ordering, rating scheduling, persistence, and application flow tests passed |
| `cargo test -p vocab-desktop --test command_contract` | PASS | Verified automated; desktop command-contract tests passed |

### Not yet verified

- **Not run:** interactive Review flow in the Windows WebView2 application. W-04 is a Current ticket and its required behavior is covered automatically; no runtime claim is made.
- **Not run:** keyboard, Narrator, high-contrast, and text-scaling review checks; comprehensive physical accessibility evidence remains owned by W-13.
- Review notifications and scheduled launch behavior remain owned by W-06.
- Settings behavior, native selection, OCR, translation, native floating-window behavior, and Windows capability flags remain unchanged.

### Next ticket starting point

W-05 starts with a Windows presentation whose Today, Vocabulary, Manual Capture, and Review workflows all use the stable frontend backend contract. It should implement persisted Settings plus theme and reduced-motion effects without adding OS shortcut registration, tray, notifications, autostart side effects, or any Windows-native API.

## 2026-09-01 - W-05 Windows Settings and visual preferences

### Implementation

- Added a Windows Settings page for source and target languages, daily Review limit, capture-shortcut preference, theme, and reduced motion.
- Saves through the existing portable `Backend.updateSettings` contract and applies the persisted Settings immediately after reading them back; Today then refreshes separately, with a refresh failure reported as stale page data rather than a Settings failure.
- Applied dark, light, and system-aware color tokens across the Windows shell, vocabulary rows, Review, Manual Capture, saved feedback, and Vocabulary detail while retaining the existing Vocab Collector accent and compact Windows 11 styling.
- Applied the saved reduced-motion preference and the operating system `prefers-reduced-motion` preference to the complete Windows presentation, including fixed overlays.
- Kept the presentation hidden over a transparent document background until the initial Settings load attempt finishes, preventing an asynchronously loaded preference from first painting the wrong theme.
- Added visible, retryable Settings persistence errors while retaining the edited draft and the last successfully applied visual preferences.

### Key decisions

- W-05 uses only the existing portable Settings and backend contracts. No Windows type or API was added to `crates/application`, `crates/domain`, or the frontend data model.
- Theme and reduced motion are driven by the last Settings value successfully read from persistence, not by the editable draft. A failed save or read-back cannot produce a success message or apply the requested visual change.
- Capture shortcut is persisted as a preference through `update_settings`; W-05 deliberately does not call `replace_shortcut`, because that command performs operating-system registration and conflict rollback owned by W-06.
- Review-time and launch-at-login controls remain out of the W-05 Windows UI. Exposing them with a generic save-success message before notification scheduling and autostart exist would falsely imply that their Windows side effects are active. W-06 owns those controls and their per-setting failure states.
- Existing shared/macOS presentation files were not changed. The Windows-owned CSS adopts the same established accent and component proportions without importing unfinished shared presentation behavior.

### Main files changed

- `ui/src/windows/WindowsApp.svelte`
- `ui/src/windows/WindowsApp.test.ts`
- `ui/src/windows/WindowsWordRow.svelte`
- `docs/windows-platform-tickets.md`
- `docs/windows-development-log.md`

No file in `crates/domain`, `crates/application`, `crates/storage`, `crates/capture`, `crates/platform-api`, `platform/windows`, or `apps/desktop` changed.

### Tests and results

TDD seam:

- Windows Settings through the public `Backend` contract was red while Settings remained a placeholder, then green after all W-05-owned values could be edited, persisted, read back, and reflected in the presentation.
- Save-failure behavior was red before Settings existed, then green after a failed update retained the draft, withheld the success state, and kept the previous theme and motion preference applied.
- Existing application/storage tests remain the contract evidence for Settings visibility in Today and SQLite persistence; W-05 did not duplicate those rules in frontend code.

| Command | Result | Evidence |
|---|---|---|
| `pnpm check` | PASS | Verified automated; `svelte-check` reported 0 errors and 0 warnings |
| `pnpm test` | PASS | Verified automated; 7 files and 40 tests passed |
| `cargo test -p vocab-storage -p vocab-application` | PASS | Verified automated; application and storage suites passed, including `settings_update_is_visible_to_today_view` and `settings_round_trip_without_an_account` |
| `cargo test -p vocab-desktop --test command_contract` | PASS | Verified automated; 11 desktop contract tests passed |

### Not yet verified

- **Not run:** interactive Settings save/restart verification in the Windows 11 WebView2 application against the real guest SQLite database.
- **Not run:** visual inspection of light, dark, and live Windows system-theme changes in WebView2; automated tests verify the applied presentation attributes and persistence sequencing, not rendered pixels.
- **Not run:** Windows system reduced-motion and keyboard/accessibility inspection in WebView2; the CSS media and saved-preference paths are present, while physical validation remains in W-13.
- **Not run:** global shortcut replacement, conflict rollback, tray lifecycle, launch-at-login, and review-time notifications. These are W-06 behaviors and no W-05 success state claims they are active.
- No UIA, OCR, translation, floating-window, or Windows adapter capability behavior changed in W-05.

### Next ticket starting point

W-06 starts with portable Settings persisted and visual preferences applied by the Windows presentation. It should add tray Open/Exit and close-to-tray lifecycle, connect shortcut changes through the existing register-before-unregister rollback command, and expose launch-at-login plus review-time notifications only when their Windows side effects and per-setting error recovery are real. It must preserve the portable Settings boundary and must not begin UIA, clipboard fallback, OCR, or translation work.

## 2026-09-01 - W-06 Tray lifecycle and Windows system settings

### Implementation

- Added a Windows tray icon with Open and Exit commands. Closing the main window now hides it without stopping the process; Open restores and focuses it; Exit attempts shortcut cleanup and always terminates.
- Kept the global capture shortcut registered while the main window is hidden. Settings replacement registers and validates the candidate before persistence and removal of the previous shortcut.
- Added launch-at-login through the Tauri autostart plugin and Review notifications through the Tauri notification plugin. These plugins are installed only in Windows desktop composition.
- Added Windows Settings controls for launch at login and local Review time, with independent shortcut, autostart, and notification error messages.
- Added startup recovery for unavailable persisted shortcuts and invalid persisted Review times. The safe defaults are persisted when possible and session-only recovery is stated when persistence fails.
- Added a process-owned Review scheduler. It checks current local time every 15 seconds, sends at most one successful notification per local calendar date when the due queue is nonempty, and retries failed delivery or due-queue reads no more frequently than every five minutes.
- Preserved unresolved system errors when a user saves an unrelated preference. Incomplete shortcut rollback is surfaced explicitly rather than reported as a successful restore.

### Key decisions

- `UserSettings` remains portable. Tauri commands translate persisted values into Windows side effects; no Win32, plugin, or Tauri type enters `vocab-domain` or `vocab-application`.
- The shared application remains the source of truth for Settings and the Today due count. The scheduler owns only timing and delivery acknowledgement.
- A notification date is marked delivered only after Windows accepts the notification, or after the application confirms there are no due words. Delivery and due-queue read failures remain retryable and visible; editing Review time does not clear them or reset the current date's successful-delivery state.
- Review time uses Windows local wall-clock time read on each poll. Local-date delivery prevents a repeated wall-clock hour from producing two successful notifications; physical DST and time-zone-change behavior remains unverified.
- Windows autostart state is applied and read back. A mismatch or plugin error is visible and does not falsely change the persisted setting.
- Tray Open/Exit and close interception stay in `apps/desktop/src-tauri`; portable frontend/backend contracts contain only serializable settings and error values.

### Main files changed

- `apps/desktop/src-tauri/src/lifecycle.rs`
- `apps/desktop/src-tauri/src/system_settings.rs`
- `apps/desktop/src-tauri/src/commands/settings.rs`
- `apps/desktop/src-tauri/src/commands/mod.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src-tauri/Cargo.toml`
- `apps/desktop/src-tauri/tests/command_contract.rs`
- `ui/src/lib/backend.ts`
- `ui/src/lib/types.ts`
- `ui/src/windows/WindowsApp.svelte`
- `ui/src/windows/WindowsApp.test.ts`
- `Cargo.lock`
- `docs/windows-platform-tickets.md`
- `docs/windows-development-log.md`

No Windows native adapter, UIA, clipboard, OCR, translation, or platform capability flag changed.

### Tests and results

TDD seam:

- Desktop settings transaction contract: candidate-first shortcut replacement, persistence ordering, complete rollback, rollback-failure reporting, and preservation of errors for effects not attempted.
- Startup recovery contract: shortcut conflicts and invalid Review times fall back without preventing application startup.
- Review scheduling contract: successful delivery is once per local date, while a failed delivery can retry after a bounded interval.
- Desktop lifecycle contract: restore shows before focus; Exit runs even when explicit cleanup reports an error.
- Windows Settings UI contract: partial Windows side-effect failures restore only the affected draft values while successfully persisted portable preferences remain applied.

| Command | Result | Evidence |
|---|---|---|
| `pnpm check` | PASS | Verified automated outside the restricted sandbox after its child-process policy blocked esbuild; 0 errors and 0 warnings |
| `pnpm test` | PASS | Verified automated; 7 files and 41 tests passed |
| `cargo test -p vocab-capture --test shortcut` | PASS | Verified automated; 4 tests passed |
| `cargo test -p vocab-desktop --test command_contract` | PASS | Verified automated; 24 tests passed, including schedule reconfiguration and bounded retry across a backward local time-zone change |
| `cargo fmt --all --check` | PASS | Verified automated after formatting |
| `cargo clippy -p vocab-desktop --all-targets -- -D warnings` | PASS | Verified automated; no warnings |
| `pnpm tauri dev` | PASS for latest-code startup | Verified on a physical HP Windows 11 Pro x64 device, build 26100; Vite started, the Rust desktop binary compiled, `vocab-desktop.exe` launched, and WebView2 processes were observed |
| Close main window | PASS | Verified physical; WM_CLOSE hid both Vocab Collector windows while the desktop process remained alive and responsive |
| Shortcut while hidden | PASS | Verified physical; `Alt+Shift+V` made the Capture window visible while the main window remained hidden |

The development process was intentionally stopped with Ctrl+C after the smoke test. Its `STATUS_CONTROL_C_EXIT` and WebView class-unregistration message are not evidence for the product Exit command.

### Not yet verified

- **Not run:** tray Open and tray Exit through the visible Windows notification-area UI; automated lifecycle contracts cover ordering and cleanup behavior, but no physical interaction evidence was captured.
- **Not run:** login/restart and launch-at-login registration across a real Windows sign-in.
- **Not run:** Explorer restart with tray icon recreation or continued access to the running application.
- **Not run:** notification delivery with notifications enabled, disabled, or restored after failure.
- **Not run:** changing Windows time zone, daylight-saving transitions, sleep/wake across Review time, and notification behavior after restart.
- **Not run:** physical shortcut conflict replacement and rollback against another installed application.
- **Not run:** macOS runtime regression. Windows-only autostart/notification plugin installation and scheduler ownership are compile-gated, while the existing global-shortcut composition remains shared.
- The W-06 physical-evidence acceptance criterion remains incomplete until the lifecycle matrix above is executed and recorded.

### Next ticket starting point

W-07 starts from a Windows desktop process that remains available in the tray and can invoke the existing capture pipeline from a global shortcut. It should implement only Notepad selection capture through the Windows UI Automation adapter, normalize native results into portable `CaptureCandidate` values, and update capability evidence only after a physical Notepad Unicode capture. It must not begin browser/editor compatibility expansion, clipboard fallback, OCR, translation, or floating-window positioning work.

## 2026-09-01 - W-07 Focused Notepad selection through UI Automation

### Implementation

- Replaced the Windows selection skeleton with a focused-element UI Automation `TextPattern` provider.
- Copied selected Unicode text, an enclosing paragraph or line, source process name, foreground-window title, and selected-range rectangles into Rust-owned values before returning a portable `CaptureCandidate`.
- Normalized UIA physical rectangle corners through the foreground window's per-monitor physical-to-logical conversion before they cross the adapter boundary. Bounds are omitted if conversion is unavailable rather than mixing coordinate spaces.
- Merged multiple selection rectangles into one portable bounding rectangle and retained `CaptureOrigin::Accessibility` for the native UIA path.
- Added a physical-only ignored contract for exact Notepad Unicode selection. The capability remains disabled because this physical check has not passed.

### Key decisions

- W-07 uses only the focused UIA element and `TextPattern`. It does not add `TextPattern2`, ancestor/descendant traversal, application-specific compatibility logic, clipboard fallback, OCR, translation, or window placement; those remain later tickets.
- The full blocking UIA operation runs on a Tokio blocking worker. COM initialization, UIA interface creation, value copying, interface release, and `CoUninitialize` all occur on that same worker thread.
- `RPC_E_CHANGED_MODE` means the worker thread already owns another valid COM apartment; the adapter uses that apartment without uninitializing ownership it did not acquire.
- Native failures map to existing typed portable errors or stable content-free operation messages. HRESULT values, captured text, document context, titles, and URLs are not logged or returned in diagnostics.
- Enclosing UIA context is bounded to paragraph with line fallback. When selected text occurs more than once in that context, the adapter keeps the whole enclosing context instead of guessing the wrong sentence.
- `selection_capture` and `selection_bounds` remain `false`. Per the plan, code compilation and fixture tests are insufficient to claim the capability without a successful physical Notepad shortcut run.

### Main files changed

- `platform/windows/Cargo.toml`
- `platform/windows/src/lib.rs`
- `platform/windows/src/selection.rs`
- `platform/windows/tests/capabilities.rs`
- `platform/windows/tests/fixtures/notepad-unicode.txt`
- `Cargo.lock`
- `docs/windows-platform-tickets.md`
- `docs/windows-development-log.md`

No shared application/domain/storage/capture contract, desktop command, frontend component, clipboard behavior, OCR provider, translation provider, or floating-window behavior changed.

### Tests and results

TDD seam:

- Windows adapter normalization: red before implementation, then green for exact Unicode, source metadata, enclosing context, multi-rectangle union, empty-selection errors, repeated-selection ambiguity, and physical-to-logical rectangle conversion.
- Existing platform-contract tests remain the portable `SelectionProvider` evidence for exact surface form and typed failures.
- The ignored Notepad test exercises the production provider and requires an exact `VOCAB_UIA_EXPECTED` value, nonempty source metadata, and available logical bounds.

| Command | Result | Evidence |
|---|---|---|
| `cargo fmt --all --check` | PASS | Verified automated |
| `cargo test -p vocab-platform-windows` | PASS | Verified automated; adapter tests passed and the physical-only Notepad test remained ignored in the normal suite |
| `cargo test -p vocab-platform-contract-tests` | PASS | Verified automated; 7 tests passed |
| `cargo test -p vocab-desktop --test command_contract` | PASS | Verified automated; 24 tests passed |
| `cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings` | PASS | Verified automated; no warnings |
| `pnpm tauri dev` | PASS for startup only | Verified Windows physical on HP Windows 11 Pro 24H2 x64, build 26100.7171; the UIA-enabled adapter compiled into `vocab-desktop.exe` and the process launched |
| `cargo test -p vocab-platform-windows physical_notepad_unicode_selection_matches_the_portable_contract -- --ignored --nocapture` | BLOCKED | Attempted on the same physical machine; the Codex execution session could not reliably transfer focus to the packaged Notepad fixture tab, so UIA observed another focused element and returned `UnsupportedElement` |

The Tauri smoke process was intentionally stopped with Ctrl+C. Its `STATUS_CONTROL_C_EXIT` and WebView class-unregistration message do not describe product behavior.

### Not yet verified

- **Blocked:** exact Unicode selection from a manually focused Notepad document through the production provider. Next action: run the ignored test from an interactive Developer PowerShell, switch to Notepad during its two-second delay, and leave the exact fixture selected.
- **Not run:** global shortcut over a selected Notepad fixture followed by visible `capture-ready` UI confirmation. This is required before enabling `selection_capture`.
- **Not run:** Notepad selection bounds at non-100% scaling and on a secondary monitor. The conversion seam is automated, while physical mixed-DPI placement remains W-09.
- **Not run:** empty selection and unsupported Notepad control through the physical application.
- **Not run:** browser, editor, Terminal, Office, and PDF UIA compatibility; these belong to W-08.
- `selection_capture` and `selection_bounds` remain false; the Windows UI must not present them as verified capabilities.

### Next ticket starting point

W-08 starts from a focused-element `TextPattern` implementation with content-free errors and portable normalization, but without a verified capability claim. Before expanding compatibility, complete the physical Notepad shortcut check and enable selection capability only if it passes. W-08 may then add bounded `TextPattern2`/`TextPattern` discovery, ancestor/descendant traversal, safe diagnostics, UTF-16 and multi-rectangle fixtures, and the required application matrix. It must not add clipboard fallback, OCR, translation, or floating-window behavior.

## 2026-09-01 - W-08 Bounded UIA compatibility discovery and safe diagnostics

### Implementation

- Expanded the Windows selection provider from focused-element-only discovery to a deterministic sequence: focused fast path, at most four nearby ancestors, then a breadth-first focused-subtree search limited to depth four and 64 total inspected elements.
- Added `TextPattern2` discovery before the existing `TextPattern` fallback. Both paths still return only the portable `CaptureCandidate` and existing typed `PlatformError` values.
- Continued past controls that expose no text pattern or only an empty/collapsed selection, returning `EmptySelection` when a text-capable element was found and `UnsupportedElement` otherwise.
- Added default content-safe diagnostics containing outcome, pattern kind, inspected-node/cap state, selected UTF-16 length, rectangle count, and source-metadata presence. Captured text, context, titles, URLs, HRESULT details, and process content are not printed.
- Added fixture coverage for traversal order and limits, UTF-16 preservation, multiple rectangles, empty selection, unsupported discovery, and missing metadata. Extended the shared platform privacy helper to check arbitrary diagnostic strings.

### Key decisions

- UIA traversal and COM interfaces remain entirely under `platform/windows`; shared application behavior continues to depend only on `platform-api`.
- Ancestors are checked before descendants because browser/editor document patterns are commonly owned above the focused leaf. Descendant search remains bounded and application-name-independent.
- TextPattern2 is capability-detected, with TextPattern as the fallback; there are no browser, Office, Terminal, editor, or PDF-reader name-specific branches.
- Capability flags remain false. Automated fixtures and compilation do not replace the required physical Notepad and compatibility-matrix evidence.
- W-08 does not add clipboard fallback, OCR, translation, capture-window positioning, or later-ticket UI behavior.

### Main files changed

- `platform/windows/src/selection.rs`
- `crates/platform-contract-tests/src/privacy.rs`
- `docs/windows-platform-tickets.md`
- `docs/windows-development-log.md`
- `docs/windows-development.md`

No Windows-specific type or conditional was added to `crates/application`, `crates/domain`, `crates/storage`, or `crates/capture`.

### Tests and results

TDD seams:

- Windows selection discovery policy through deterministic fixture trees: red before the bounded policy existed, then green for focused-first ordering, stop-on-selection, maximum depth, and maximum node count.
- Content-safe diagnostic rendering: red before the diagnostic type existed, then green for metadata/length output without private selected text or context.
- Existing public platform and application fake-provider seams verify exact Unicode and typed empty/unsupported failures without coupling tests to COM call order.

| Command | Result | Evidence |
|---|---|---|
| `cargo test -p vocab-platform-windows` | PASS | Verified automated; 9 unit tests and 2 capability tests passed; 1 physical-only Notepad test remained ignored |
| `cargo test -p vocab-platform-contract-tests` | PASS | Verified automated; 8 tests passed |
| `cargo test -p vocab-application --test platform_fakes` | PASS | Verified automated; 11 tests passed |
| `cargo clippy -p vocab-platform-windows -p vocab-platform-contract-tests -p vocab-application --all-targets -- -D warnings` | PASS | Verified automated; no warnings |
| `cargo fmt --all --check` | PASS | Verified automated |

### Not yet verified

- **Not run:** UIA text, bounds, context, and source metadata in Windows Terminal, VS Code, Edge, Chrome, Firefox, Word, and PDF readers. No interactive physical compatibility run was performed for W-08.
- **Not run:** TextPattern2 versus TextPattern behavior on real controls, traversal timing on large real UIA trees, and safe diagnostic output from a live capture session.
- **Blocked:** the W-07 physical Notepad Unicode selection remains without successful evidence; the earlier focus-transfer blocker has not been re-tested in this ticket.
- The required physical UIA compatibility-matrix columns remain `Not run`, and `selection_capture` plus `selection_bounds` remain false.

### Next ticket starting point

W-08 remains open at its physical evidence gate. Run the required application matrix for UIA text, bounds, context, and source metadata, recording the device/build and results without enabling capability flags unless the required evidence passes. After that gate, W-09 starts from portable selection bounds and should implement only non-activating capture-window presentation plus mixed-DPI/negative-origin placement.

## 2026-09-01 - W-09 Non-activating mixed-DPI capture window

### Implementation

- Added a Windows-only capture-window adapter that applies `WS_EX_NOACTIVATE` and `WS_EX_TOOLWINDOW`, removes `WS_EX_APPWINDOW`, keeps the window topmost without activation, and presents it with `SW_SHOWNOACTIVATE`.
- Captured the source foreground-window handle immediately before passive presentation. Explicit editing may activate the capture window; Done, Cancel, and the existing request-safe hide path attempt to restore the recorded source focus.
- Kept the desktop sequencing explicit: position in portable logical coordinates, show without activation, then emit `capture-ready`.
- Expanded portable placement fixtures across negative-origin work areas at 100%, 125%, 150%, and 200% scale, asserting that the complete 380×280 card remains inside the chosen work area.
- Replaced the Windows floating-window placeholder with a minimal capture result presentation. It does not add W-10 correction, manual translation, or save workflow behavior.

### Key decisions

- Win32 HWNDs, extended styles, foreground-window tracking, and focus APIs remain in `platform/windows`; shared application, domain, storage, and capture types remain Windows-independent.
- The desktop composition layer passes only the target-gated native handle and owns Tauri positioning/event publication. Shared `crates/capture` continues to own placement policy in logical top-left virtual-desktop coordinates.
- Window styles are installed at Windows desktop startup so the dormant implementation can be exercised by the required physical checks. `non_activating_window` remains `false`: automated style and sequencing tests plus process startup are not physical focus/taskbar evidence.
- Editing activation is an explicit user action. W-09 introduces only the focus transition; editable text, save correction, manual translation, and request-safe persistence remain W-10.
- Native failures return content-free `PlatformError::Operation` messages. No captured text, source metadata, or window content is logged.

### Main files changed

- `crates/capture/tests/placement.rs`
- `platform/windows/src/window.rs`
- `platform/windows/src/lib.rs`
- `platform/windows/tests/capabilities.rs`
- `apps/desktop/src-tauri/src/commands/capture.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src-tauri/tests/command_contract.rs`
- `ui/src/windows/WindowsFloatingCapture.svelte`
- `ui/src/windows/WindowsFloatingCapture.test.ts`
- `ui/src/windows/captureBackend.ts`
- `docs/architecture.md`
- `docs/windows-development.md`
- `docs/windows-development-log.md`

### Tests and results

TDD seams agreed before implementation:

- `crates/capture::place_floating_window`: red/green fixture for complete-card containment across negative origins and 100/125/150/200% scale factors.
- Desktop capture presentation boundary: red/green contract for position → show-without-activation → event ordering.
- `WindowsFloatingCapture`: red/green user-interaction tests proving passive receipt does not request focus, explicit Edit does, and Done/Cancel use focus-restoring commands.

| Command | Result | Evidence |
|---|---|---|
| `cargo test -p vocab-capture --test placement` | PASS | **Verified automated:** 5 tests passed |
| `cargo test -p vocab-platform-windows` | PASS | **Verified automated:** 12 tests passed and 1 physical-only Notepad test remained ignored |
| `cargo test -p vocab-desktop --test command_contract` | PASS | **Verified automated:** 26 tests passed |
| `pnpm test` | PASS | **Verified automated:** 44 tests passed across 7 files |
| `cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings` | PASS | **Verified automated:** no warnings |
| `cargo fmt --all --check` | PASS | **Verified automated:** passed |
| `pnpm check` | PASS | **Verified automated:** 0 errors and 0 warnings |
| `pnpm tauri dev` | PASS for startup only | **Verified automated:** process compiled and launched, then was intentionally stopped with Ctrl+C; this is not physical focus or DPI evidence |

### Not yet verified

- **Not run:** passive presentation preserving foreground focus in Notepad, browsers, editors, Terminal, Office, and PDF readers. The current execution context was not independently established as an eligible Windows 11 x64 physical test setup; owner: W-09 physical gate.
- **Not run:** taskbar and Alt+Tab absence, Win+D, lock/unlock, source-window closure, and focus restoration after explicit Done/Cancel; owner: W-09 physical gate.
- **Not run:** placement on real negative-origin displays at 100%, 125%, 150%, and 200%, including mixed scales and taskbars on different edges; owner: W-09 physical gate.
- **Not run:** display disconnect/reconnect while the capture window is visible; owner: W-09 physical gate.
- `non_activating_window`, `selection_capture`, and `selection_bounds` remain false pending their required physical evidence.

### Next ticket starting point

W-09 remains open at its physical evidence gate. First run the focus/taskbar and mixed-monitor matrix and enable `non_activating_window` only if it passes. W-10 then starts from the explicit Edit/focus seam and the existing shared request coordinator; it should add correction, optional manual translation, save-without-translation, save-once, Undo, and dismissal without moving those rules into the Windows adapter.
