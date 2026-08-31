# Windows Development Log

This log records implementation evidence for `docs/windows-platform-tickets.md`. Evidence uses the status vocabulary defined in `docs/windows-platform-plan-v2.md`: **Verified automated**, **Verified Windows physical**, **Not run**, **Blocked**, and **Unsupported**.

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
