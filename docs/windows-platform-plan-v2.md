# Windows Platform v2 Implementation Plan

> **For agentic workers:** Implement this plan milestone by milestone. Use `superpowers:subagent-driven-development` or `superpowers:executing-plans`, apply test-driven development within each milestone, and update the handoff evidence before moving to the next milestone.

**Goal:** Deliver a complete, local-first Vocab Collector desktop application for Windows 11 x64 with an independently maintained Windows presentation, UI Automation capture, explicit OCR fallback, persistent review workflows, system integration, and a verifiable NSIS release.

**Architecture:** Domain, application, storage, capture coordination, portable platform contracts, and frontend-facing DTOs remain shared. Windows owns its Svelte presentation and native adapter; Tauri remains the composition boundary between product workflows and Windows services. UI Automation, Win32, WinRT, COM, HRESULT, HWND, and native image types must remain below `platform-api` or inside narrowly scoped desktop window integration.

**Tech Stack:** Rust 1.98+ with the MSVC toolchain, Tauri 2, Svelte 5, TypeScript, SQLite, `windows-rs` after feature validation, UI Automation, Windows Graphics Capture, `Windows.Media.Ocr`, NSIS, Node.js 22, and pnpm 11.19.0.

**Inputs:** `docs/windows-development.md`, `docs/architecture.md`, `docs/cross-platform-porting-guide.md`, `docs/native-capture-development.md`, `CONTEXT.md`, `docs/adr/0001-independent-windows-presentation.md`, and the repository state inspected on 2026-08-31.

## Global Constraints

- Target Windows 11 x64 on a physical machine. ARM64 and Windows 10 are not claimed by this plan.
- Preserve the dependency direction `desktop/application -> platform-api <- platform/windows`; shared crates must not import Windows APIs or the Windows adapter.
- Keep Windows page-level UI and platform styling independent from the unfinished macOS presentation. Share stable DTOs, backend contracts, event contracts, and visual tokens only where their behavior is already proven.
- Match the existing Vocab Collector visual identity where practical: colors, density, typography hierarchy, component proportions, and capture-card tone. Windows interaction, focus, title bar, scaling, and accessibility behavior take precedence over pixel identity.
- Native Capture uses UI Automation first and explicit OCR second. Clipboard capture is not part of the initial release path and may be enabled only by the evidence gate in Milestone 8.
- OCR screenshots remain in memory, are released promptly, and are never included in production logs or diagnostics.
- The Windows preview reports translation as unavailable. It supports user-entered translations and explicit save-without-translation; an automatic translation engine is a separate project.
- Closing the main window keeps the process in the system tray. Explicit Exit ends the process and unregisters session resources.
- A capability remains `false` until its provider, automated contract tests, and relevant Windows physical-device evidence exist.
- Do not overwrite or rewrite the existing uncommitted content in `docs/windows-development.md`. Append or make narrow evidence updates only after reviewing its current diff.
- Do not infer success from hosted CI for focus, permissions, UIA compatibility, OCR output, mixed DPI, WebView2 installation, or installer lifecycle behavior.

---

## 1. Current State

### 1.1 Shared core already completed

The following code is implemented and reusable without Windows-specific branches:

- `crates/domain` owns Vocabulary Items, Encounters, capture origin, settings, review state, deterministic review scheduling, and frontend-ready views.
- `crates/storage` owns SQLite initialization, schema migration, foreign-key enforcement, settings persistence, capture deduplication, Encounter history, soft-delete Undo, and the outbox.
- `crates/application` exposes manual capture, Today, Vocabulary, word detail, Review, Settings, and the portable native-capture workflow.
- `crates/capture` owns shortcut validation/replacement rules, repeat suppression, request identity, stale-result rejection, OCR confirmation state, translation state, save-once behavior, persistence retry, and logical floating-window placement.
- `crates/platform-api` exposes portable capabilities, coordinates, `CaptureCandidate`, `OcrCandidate`, typed `PlatformError`, and provider traits.
- `crates/platform-contract-tests` contains reusable selection and unavailable-translation contract helpers.
- `apps/desktop/src-tauri` already composes storage, application services, a target-selected adapter, Tauri commands, shortcut events, capture events, and the two configured windows.
- `ui/src` already demonstrates Today, Vocabulary, Review, Settings, Manual Capture, word detail, shortcut recording, and a native capture card against one frontend backend interface.
- `.github/workflows/ci.yml` already has a `windows-latest` job that runs the Windows-target Rust workspace checks plus frontend check/test/build. It does not currently build or publish an NSIS artifact.

Evidence from the 2026-08-31 inspection:

- The Windows skeleton, application, capture, and storage Rust suites completed successfully: 49 tests passed across the selected packages.
- Frontend commands initially could not start in the managed sandbox because `esbuild` process creation returned `EPERM`. Exact-command reruns outside that sandbox passed: `pnpm check` reported 0 errors and 0 warnings, `pnpm test` passed 26 tests in 4 files, and `pnpm build` produced the production bundle. The initial `EPERM` remains environment evidence rather than a product failure.
- No native Windows runtime, UIA, OCR, focus, tray, notification, packaging, or physical-device result was established by that run.

### 1.2 Actual Windows skeleton state

`platform/windows` currently contains one implementation file and one capability test file. `WindowsPlatform::new()` returns `PlatformServices` with default capabilities, and every provider returns `PlatformError::Unsupported` for its capability.

The crate currently has no dependency on `windows-rs` and no implementation of:

- COM initialization or apartment ownership;
- UI Automation selection, TextPattern/TextPattern2, metadata, or bounds;
- bounded UIA tree traversal;
- Windows Graphics Capture or WinRT OCR;
- Windows-specific permission/status guidance;
- Win32 capture-window behavior;
- system tray, notifications, or launch-at-login;
- Windows diagnostic tooling.

The existing test proves only that the skeleton reports no capability and returns typed unsupported errors.

### 1.3 Reusable macOS and shared implementation

Reuse behavior and contracts, not the macOS native mechanism:

- Reuse `PlatformServices`, provider traits, portable errors, capability reporting, and shared adapter contract tests.
- Reuse `PlatformCaptureWorkflow` and `CaptureCoordinator` for request identity, confirmation, translation state, save-once behavior, and stale-result protection.
- Reuse logical placement in `crates/capture`; Windows supplies correctly normalized pointer, selection bounds, and work areas.
- Reuse frontend-facing command and event meanings, including `capture-ready`, `capture-error`, OCR confirmation, save, Undo, and hide.
- Reuse the macOS adapter's principle of moving blocking native work off the async runtime. Do not copy its Swift FFI, AppKit coordinates, permission semantics, or page-level UI.
- Reuse visual colors and proven component styling selectively. The Windows presentation must have its own behavior and regression tests under `ui/src` as required by ADR 0001.

### 1.4 Differences between documents and code

The implementation agent must begin from these observed differences:

| Topic | Documented intent | Actual code state |
|---|---|---|
| Windows build | Plan A said Windows was not compiled on macOS | The skeleton now compiles and its Rust tests pass on the current Windows host; physical-host status and native runtime remain unverified |
| OCR ranking | Pointer containment, distance, then confidence | `capture_with_ocr` currently selects only maximum confidence |
| OCR confirmation | Ambiguous candidates may be presented to the user | Desktop currently reduces the vector to one candidate before the UI sees it |
| OCR coordinates | Platform-neutral top-left logical coordinates at boundaries | Desktop OCR command contains an unconditional Cocoa coordinate conversion that is unsuitable for Windows |
| Windows UI | Independent Windows presentation after ADR 0001 | `ui/src/main.ts` currently selects only main versus capture window; the same page-level components are used for all OS targets |
| Settings | Visible settings should represent real behavior | Theme, reduced motion, launch at login, and review time are persisted but do not all drive desktop behavior |
| Progress | No Windows v2 requirement for analytics | Existing Progress content is static demonstration data and must not ship as real analytics |
| Translation | Provider contract exists | Windows reports translation unavailable; native capture UI currently offers retry/save-without-translation but not user-entered translation |
| Packaging | Bundling is enabled | No Windows-specific NSIS policy, WebView2 evidence, upgrade evidence, or uninstall-data behavior is configured and verified |
| Windows CI | A Windows lane was planned | A `windows-latest` lane exists and checks source/build contracts, but it does not build NSIS or establish runtime/physical evidence |
| Permissions | Portable status and request traits exist | Their Accessibility/ScreenRecording names reflect the macOS model; exact Windows semantics and user guidance remain to be validated |

---

## 2. Target Architecture

```text
Windows Svelte presentation
    | stable Backend methods + Tauri event DTOs
Tauri desktop composition
    | product workflows            | native presentation coordination
application + capture + storage    platform-api
                                      ^
                                      |
                               Windows adapter
                                      |
                   COM / UIA / Win32 / WinRT / OCR
```

### 2.1 Shared UI boundary

The shared UI surface is deliberately narrow:

- Keep `ui/src/lib/types.ts` and `ui/src/lib/backend.ts` as the frontend-facing contract unless a tested product use case requires an additive change.
- Share request identifiers, capture candidates, failures, capabilities, saved Capture Cards, Today views, Vocabulary Item views, Review inputs, and Settings shapes.
- Share visual tokens only when doing so does not couple page structure or OS behavior.
- Do not import Windows APIs into TypeScript or make page behavior branch on an OS string. Platform feature behavior is driven by capabilities and Windows presentation entry selection at the desktop boundary.
- The existing `App.svelte` and `FloatingCapture.svelte` are reference behavior, not automatically the Windows pages. The Windows implementation milestone must establish its presentation structure within `ui/src` without changing the macOS page behavior as a side effect.

### 2.2 Tauri desktop composition boundary

`apps/desktop/src-tauri` remains responsible for:

- choosing `vocab-platform-windows` at the existing target-specific bootstrap seam;
- opening the guest SQLite database and creating `AppState`;
- registering commands, events, global shortcuts, tray actions, autostart integration, notifications, and Tauri windows;
- reading Tauri pointer/monitor/window data and converting it to portable logical values before shared placement;
- coordinating raw OCR candidates with the Windows confirmation UI, then passing one resolved `CaptureCandidate` into the application workflow;
- mapping native/platform failures to stable `CaptureFailureCode` values;
- preserving request identity around show, hide, replacement, confirmation, save, and dismissal side effects.

It must not own UIA traversal, OCR recognition, raw screenshot processing, or HRESULT interpretation.

### 2.3 `platform-api` boundary

Prefer the existing portable types and traits. Change `crates/platform-api` only when a Windows tracer slice proves that an existing public contract cannot represent required behavior.

- `SelectionProvider` produces one normalized `CaptureCandidate` or a typed portable error.
- `OcrProvider` produces raw portable `Vec<OcrCandidate>`; it does not display UI or select on behalf of the user.
- `PermissionProvider` reports portable status. Windows-specific explanatory text belongs in desktop presentation mapping, not the trait.
- `WindowProvider` remains a narrow configuration capability. If tray, notification, or autostart need new seams, first determine whether they are product-level desktop services rather than platform capture providers.
- Do not add a generic native handle, HRESULT field, COM object, bitmap, WinRT async operation, or UIA element to a public DTO.

### 2.4 Windows adapter and native API boundary

All direct Windows API use belongs under `platform/windows/src` except Tauri-owned window/tray integration that must use a Tauri handle in the desktop crate.

- UI Automation: foreground/focused element discovery, TextPattern2/TextPattern, selection ranges, bounded ancestor/descendant traversal, UTF-16 conversion, bounding rectangles, process metadata, and error mapping.
- COM: initialization and teardown on the thread that owns native objects; native objects must not cross an unsupported apartment boundary.
- WinRT/Windows Graphics Capture: bounded in-memory capture near the pointer, user cancellation/access behavior, bitmap lifetime, and transfer to OCR.
- `Windows.Media.Ocr`: recognition into portable `OcrCandidate` values with logical top-left bounds and confidence when the API provides meaningful confidence.
- Win32: only the capture-window styles, focus/activation behavior, foreground process metadata, DPI/work-area conversion, and other native operations that Tauri cannot satisfy reliably.

Unsafe code, if `windows-rs` requires it, must be isolated within the smallest native module, documented with invariants, and never weaken `#![forbid(unsafe_code)]` outside the Windows adapter. Whether the crate-level lint must change is **Not run (pending validation)** until the COM/UIA spike.

---

## 3. UI Implementation Plan

### 3.1 Presentation structure

- Extend the existing `ui/src/main.ts` composition decision so the Windows target can select Windows main and capture presentations without placing OS conditions throughout features.
- Keep the existing backend contract as the single access point for browser/demo and Tauri operation.
- Split the current monolithic presentation by feature only as each Windows vertical slice is implemented. Do not pre-create an entire speculative component hierarchy.
- Keep Progress out of Windows navigation until real analytics are supplied by the application layer. Static bars and fabricated weekly values are not release content.

### 3.2 Main window and navigation

- Provide Windows navigation for Today, Vocabulary, Review entry, and Settings.
- Use a Windows-compatible title bar and window controls. The current macOS-oriented overlay configuration must not be assumed valid on Windows.
- Preserve the existing muted dark visual identity, accent colors, compact spacing, status language, and component proportions where they pass Windows accessibility and contrast checks.
- Support light, dark, and system themes without flash-of-wrong-theme during startup.
- Restore the previous main-window size and position only after clamping it to an available monitor work area; validation of the persistence location and format is **Not run** and belongs to the Windows shell milestone.

### 3.3 Today

- Load due count, estimated duration, recent captures, and Review queue through the existing backend contract.
- Represent initial loading, empty review, no recent captures, recoverable load failure, and retry.
- Never display stale counts after Manual Capture, native save, Undo, or completed Review.

### 3.4 Vocabulary

- Display persisted Vocabulary Items, translations when present, status, Encounter count, and last-seen time.
- Search by displayed text and translation with deterministic empty and no-result states.
- Open word detail with Encounter history and source metadata.
- Editing or deleting an existing Vocabulary Item is not present in the current application API and is outside this plan unless separately specified. User correction is supported before saving a new capture.

### 3.5 Review

- Start from the current due queue and submit only the existing `forgot` or `remembered` ratings.
- Provide progress, context, translation-unavailable treatment, close/resume behavior, final completion state, and refresh of Today.
- Keep review scheduling in `crates/domain`; the UI must not calculate due dates.

### 3.6 Settings

- Persist source/target language, shortcut, review time, daily limit, launch at login, appearance, and reduced motion through the existing Settings model.
- Apply theme and reduced motion immediately after a successful save.
- Register a replacement shortcut before removing the current shortcut, retaining existing rollback behavior.
- Make launch-at-login drive actual Windows autostart registration.
- Make review time drive a Windows notification schedule or an equivalent process-owned scheduler. Validation of the notification API and persistence across logout/reboot is **Not run** and belongs to Milestone 4.
- Surface registration, autostart, notification, and persistence failures beside the affected setting without discarding other saved values.

### 3.7 Manual Capture and native capture result

- Manual Capture accepts selected text, context, and optional user translation, then uses the existing application capture path.
- Native capture displays selected text, reconstructed sentence/context, source metadata when available, origin, and optional translation.
- Because automatic translation is unavailable, the Windows capture result permits user correction of selected text and context plus optional manual translation before save.
- The application/desktop contract for a user-confirmed correction must be added at the highest existing workflow seam; do not mutate provider-owned native objects or bypass save-once behavior.
- After save, show the returned Capture Card, duplicate/Encounter count, Undo, and timed dismissal. Hover/focus pauses dismissal.

### 3.8 OCR confirmation

- Rank candidates by pointer containment, distance to the rectangle, then confidence.
- A clearly dominant candidate receives one-item confirmation. Close scores produce a keyboard-navigable candidate list.
- Confirmation permits correction of selected text, reconstructed context, and optional manual translation.
- Cancel returns to the user's previous application without persistence.
- The confirmation view never receives or retains the screenshot itself.
- Dominance thresholds and OCR context reconstruction quality are **Not run** until recorded content-free geometry fixtures and the physical application matrix exist.

### 3.9 Floating window

- Remain topmost, absent from the taskbar, and non-activating during initial presentation.
- Do not steal focus from the source application merely to show a UIA result. Explicit editing or candidate selection may activate/focus the capture window; focus must return predictably after save/cancel.
- Position relative to selection bounds when available and pointer otherwise, using shared placement after Windows coordinate normalization.
- Clamp the complete card to the target monitor's work area at 100%, 125%, 150%, and 200% scale, including monitors with negative origins.
- Do not resize or move unexpectedly as loading, error, confirmation, and saved states change.

### 3.10 Accessibility and input

- Every command is keyboard reachable with visible focus.
- Dialogs trap focus only while genuinely modal; Escape cancels or closes the current layer.
- Candidate lists expose selection and active-item semantics to screen readers.
- Buttons use familiar icons plus accessible names; Windows system conventions govern Enter, Space, Tab, Shift+Tab, and Alt+F4.
- Meet WCAG AA text contrast and honor system reduced motion, text scaling, high contrast, and keyboard-only operation.
- Verify layout at the Tauri minimum size and Windows display/text scaling; text must not overlap or clip.

---

## 4. Product Function Plan

### Startup and database initialization

- Validate toolchain and target, start Tauri, select `WindowsPlatform`, open the existing guest SQLite database, run migrations, load Settings, apply theme, register the shortcut, configure tray/autostart/notifications, and show the main window.
- A database or migration failure must prevent unsafe writes and show a recoverable startup error with a diagnostic path that does not expose captured content.
- Closing the main window hides it to the tray. Tray Open restores it; tray Exit unregisters active resources and terminates.

### Global shortcut and UIA selection capture

- Reuse the Tauri global-shortcut plugin and shared validation/repeat suppression.
- Perform UIA work on a correctly initialized COM worker rather than the async runtime thread.
- Try the focused element first, then a bounded nearby ancestor/descendant search.
- Prefer TextPattern2, then TextPattern; return exact Unicode selected text, contextual sentence where available, bounds, process/source name, title, and URL only when reliably exposed.
- Map no selection, unsupported control, privilege boundary, native cancellation, and operation failures into the existing portable error surface.
- A diagnostic mode may print selected text only after an explicit developer flag; default diagnostics report lengths and metadata only.

### Clipboard fallback

Clipboard fallback is a conditional post-preview milestone, not part of the default `UIA -> OCR` path.

- First complete the compatibility matrix and identify applications where UIA fails and OCR is unacceptable.
- Enable the work only if the evidence names a material use case that clipboard capture solves.
- If approved, preserve all clipboard formats where Windows APIs permit, detect a new clipboard sequence/value rather than accepting stale text, restore the previous clipboard, and report restoration failure explicitly.
- Never send `Ctrl+C` without a reliable foreground target and user-triggered request.
- Keep it behind its own capability/strategy decision so it can be disabled without changing application workflows.

### OCR fallback

- Offer OCR after UIA returns an eligible selection failure; do not start continuous or background screen capture.
- Capture only a bounded region near the pointer after explicit user action.
- Convert recognition output to portable candidates, release every native image object, rank in shared portable logic when semantics are platform-neutral, and present confirmation through desktop/UI.
- Record `CaptureOrigin::Ocr` for the final Encounter.

### Translation and save

- Configure Windows translation capability as unavailable and reuse the typed unavailable provider contract.
- Replace the current automatic retry emphasis with manual translation entry and explicit save-without-translation.
- Preserve one request ID through correction, OCR confirmation, optional translation entry, save, Undo, and dismissal.
- Store corrected content only after confirmation; providers never write SQLite directly.

### Review and Settings

- Reuse `AppService` for Today, list/detail, Review submissions, and Settings persistence.
- Desktop composition implements the side effects implied by Settings; SQLite remains the source of truth for values.
- Partial side-effect failure must be visible. For example, a persisted shortcut must not claim success when registration rolled back.

### Permissions and recovery

- UIA normally has no macOS-style global Accessibility consent prompt; failures caused by elevated target processes, disabled accessibility exposure, or application incompatibility need Windows-specific guidance. Validation of exact mappings is **Not run**.
- Validation of Windows Graphics Capture consent/cancellation behavior and whether a reusable permission status exists is **Not run** against the selected API.
- Error UI offers only actions valid for its typed code: retry, choose OCR, open relevant system guidance, save without translation, or cancel.
- Never display raw HRESULT values as the primary user message. Retain sanitized native diagnostics for development evidence.

---

## 5. Phased Milestones

Each milestone ends with a runnable or independently verifiable slice and a documentation checkpoint. Commands are run from the repository root in Developer PowerShell unless stated otherwise.

### Milestone 0: Establish the Windows baseline

**Goal:** Prove the current skeleton, shared core, frontend, Tauri build inputs, and toolchain on the target machine before native work.

**Scope:** `platform/windows`, workspace manifests, `apps/desktop`, `ui`, CI configuration, and Windows development evidence. Do not add native capabilities.

**Dependencies:** Physical Windows 11 x64; Visual Studio 2022 Build Tools with Desktop C++ and Windows SDK; WebView2 Runtime; Rust 1.98 MSVC; Node.js 22; pnpm 11.19.0.

**Acceptance:** All applicable automated gates pass; `pnpm tauri dev` opens the skeleton application; SQLite, Today, Vocabulary, Review, Settings persistence, and Manual Capture are exercised; every native capability still reports false.

**Verification:**

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings
cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
cargo build --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
pnpm check
pnpm test
pnpm build
pnpm tauri dev
```

### Milestone 1: Windows application shell tracer bullet

**Goal:** Launch a Windows-owned presentation through the real Tauri backend and complete one Manual Capture-to-SQLite-to-Today round trip.

**Scope:** Existing UI entry and backend files under `ui/src`, Tauri window configuration and startup under `apps/desktop/src-tauri`, existing application/storage commands. Do not implement UIA or OCR.

**Dependencies:** Milestone 0 and ADR 0001.

**Acceptance:** Windows main UI has Today, Vocabulary, Review entry, Settings, loading/empty/error behavior, and Manual Capture; restart preserves data; the existing macOS presentation is not changed by Windows page behavior; static Progress is absent.

**Verification:**

```powershell
pnpm check
pnpm test
pnpm build
cargo test -p vocab-application -p vocab-storage
cargo test -p vocab-desktop --test command_contract
pnpm tauri dev
```

Manual tracer: save a new Vocabulary Item, capture it again as a second Encounter, inspect word detail, Undo the second Encounter, restart, and confirm the first Encounter remains.

### Milestone 2: COM and focused UIA selection tracer bullet

**Goal:** Press the global shortcut in Notepad, capture selected Unicode text through UIA, normalize it to `CaptureCandidate`, display it, and save it without automatic translation.

**Scope:** `platform/windows/Cargo.toml`, implementation within `platform/windows/src`, Windows tests under `platform/windows/tests`, existing selection command/event path under `apps/desktop/src-tauri`.

**Dependencies:** Milestone 1; exact `windows-rs` features verified against the pinned dependency version before coding.

**Acceptance:** COM lifetime is deterministic; focused TextPattern/TextPattern2 selection works in Notepad; Unicode including surrogate pairs is preserved; source app/title and bounds are present when available; typed failures replace HRESULT leakage; `selection_capture` becomes true only after tests and physical evidence.

**Verification:**

```powershell
cargo test -p vocab-platform-windows
cargo test -p vocab-platform-contract-tests
cargo test -p vocab-desktop --test command_contract
cargo clippy -p vocab-platform-windows --all-targets -- -D warnings
pnpm tauri dev
```

### Milestone 3: Bounded UIA compatibility and diagnostics

**Goal:** Expand the native-selection slice from Notepad to browsers, editors, terminal, Office, and PDF readers without application-name hacks.

**Scope:** Windows UIA implementation and tests, developer diagnostics using the existing debug tooling conventions, compatibility evidence in Windows documentation.

**Dependencies:** Milestone 2.

**Acceptance:** Focused fast path plus bounded traversal is measurable and terminates; empty/unsupported controls are typed; UTF-16 range conversion, multiple rectangles, context reconstruction, and unavailable URL/title behavior have fixtures; diagnostics default to content-free output.

**Verification:**

```powershell
cargo test -p vocab-platform-windows
cargo test -p vocab-platform-contract-tests
cargo test -p vocab-application --test platform_fakes
```

Complete the UIA columns of the physical compatibility matrix before proceeding.

### Milestone 4: Tray, Settings side effects, and Windows notifications

**Goal:** Make the Windows application useful between captures without keeping the main window visible.

**Scope:** Tauri startup/lifecycle, Settings commands and Windows presentation, tray, launch-at-login, theme, reduced motion, review-time notifications, and error recovery.

**Dependencies:** Milestone 1. It may proceed in parallel with Milestone 3 only if edits do not overlap.

**Acceptance:** Close hides to tray; Open restores; Exit terminates; shortcut remains active while hidden; autostart matches Settings; theme/reduced motion apply; review notification fires once at the configured behavior; failed registrations are reported and rolled back.

**Verification:**

```powershell
pnpm check
pnpm test
cargo test -p vocab-capture --test shortcut
cargo test -p vocab-desktop --test command_contract
pnpm tauri dev
```

Physical checks include login/restart, tray lifecycle, Explorer restart, shortcut conflict, notification disabled in Windows Settings, and daylight-saving/time-zone behavior. Notification scheduling semantics remain **Not run** until this milestone records the chosen behavior.

### Milestone 5: Non-activating floating window and mixed-DPI placement

**Goal:** Present the UIA result beside the source selection without stealing focus or leaving the visible work area.

**Scope:** Existing shared placement code/tests, desktop monitor normalization and Tauri window operations, Windows adapter window configuration, Windows capture presentation.

**Dependencies:** Milestones 2 and 4.

**Acceptance:** Initial result does not activate; editing activates only after explicit interaction; save/cancel restores predictable focus; taskbar/Alt+Tab behavior matches the product decision; placement passes negative-origin and mixed-scale monitors; capability becomes true only after physical evidence.

**Verification:**

```powershell
cargo test -p vocab-capture --test placement
cargo test -p vocab-platform-windows
cargo test -p vocab-desktop --test command_contract
pnpm test
pnpm tauri dev
```

### Milestone 6: OCR capture tracer bullet

**Goal:** After an eligible UIA failure, explicitly capture a bounded region, recognize candidates in memory, confirm one candidate, and save an OCR-origin Encounter.

**Scope:** Windows Graphics Capture and OCR inside `platform/windows`, portable ranking in `crates/capture` if proven cross-platform, desktop OCR coordination, Windows confirmation UI, application correction/manual-translation seam.

**Dependencies:** Milestones 3 and 5.

**Acceptance:** No screenshot is persisted or logged; Cocoa conversion is removed from the Windows path; pointer-aware ordering replaces confidence-only selection; dominant and ambiguous flows are tested; cancel saves nothing; correction and optional manual translation reach SQLite once; `screenshot_ocr` becomes true only after tests and physical evidence.

**Verification:**

```powershell
cargo test -p vocab-platform-windows
cargo test -p vocab-capture
cargo test -p vocab-application --test platform_fakes
cargo test -p vocab-desktop --test command_contract
pnpm test
pnpm tauri dev
```

### Milestone 7: Windows UI completion and accessibility

**Goal:** Finish the Windows-owned Today, Vocabulary, Review, Settings, Manual Capture, native result, and OCR confirmation experience across all states.

**Scope:** Windows presentation under `ui/src`, stable backend/types, and additive desktop commands required by proven workflows.

**Dependencies:** Milestones 4 and 6.

**Acceptance:** Every page has loading, empty, success, and error behavior; keyboard-only use works; screen-reader semantics are present; high contrast, reduced motion, themes, minimum size, and text scaling are verified; colors and components remain recognizably consistent with the existing macOS visual direction without sharing page-level implementation.

**Verification:**

```powershell
pnpm check
pnpm test
pnpm build
cargo test -p vocab-desktop --test command_contract
pnpm tauri dev
```

### Milestone 8: Clipboard evidence gate

**Goal:** Decide from recorded compatibility evidence whether synthetic Copy is necessary and safe enough for a separate implementation increment.

**Scope:** Compatibility report and, only if approved by that report, Windows adapter/desktop clipboard behavior. The default release path remains UIA then OCR.

**Dependencies:** Milestones 3 and 6 plus completed application matrix.

**Acceptance without implementation:** The matrix names no material case where clipboard is better than UIA/OCR; the release documents clipboard as unsupported and the milestone closes with no code.

**Acceptance with implementation:** Plain text is accepted only after a new clipboard sequence/update; existing formats are restored; stale values, delayed owners, large payloads, no selection, and restoration failure are tested; the feature can be disabled independently.

**Verification if implemented:**

```powershell
cargo test -p vocab-platform-windows
cargo test -p vocab-platform-contract-tests
cargo test -p vocab-desktop --test command_contract
pnpm tauri dev
```

### Milestone 9: NSIS preview and upgrade tracer bullet

**Goal:** Install, run, upgrade, and uninstall a per-user Windows preview while preserving user data by default.

**Scope:** Existing Tauri bundle configuration, icons, Windows CI artifact generation, installer evidence, release documentation.

**Dependencies:** Milestone 7; Milestone 8 decision recorded.

**Acceptance:** x64 per-user NSIS installs on a clean Windows 11 user account; WebView2 strategy succeeds; first launch initializes SQLite; installing a newer build preserves data/settings; uninstall removes binaries, tray/autostart integration, and shortcuts; local database remains unless the user selects explicit deletion; running app is handled before upgrade/uninstall.

**Verification:**

```powershell
pnpm check
pnpm test
pnpm build
cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
pnpm tauri build --bundles nsis
```

### Milestone 10: Release candidate and handoff

**Goal:** Produce one traceable Windows preview candidate with green automation and complete physical evidence.

**Scope:** Windows CI lane, all Windows documentation, compatibility matrix, known limitations, and release checklist.

**Dependencies:** Milestone 9.

**Acceptance:** CI is green; no claimed capability lacks provider tests and physical evidence; matrix and installer lifecycle are complete; privacy audit finds no captured content/screenshots in logs; documentation names unsupported apps and deferred automatic translation honestly.

**Verification:** Run every command in Section 7 and complete every applicable row in Section 6.5 on the release artifact, not only a development build.

---

## 6. Test Plan

### 6.1 Rust unit tests

Prefer pure inputs and observable portable outputs. Add tests at the highest existing seam that owns the rule:

- `crates/capture/tests`: candidate ranking, ambiguity ordering, coordinate/placement, request identity, correction/confirmation state, save-once, shortcut repeat suppression.
- `crates/application/tests`: corrected native candidate plus optional manual translation, unavailable translation, persistence retry, stale request, OCR confirmation, Today refresh.
- `crates/storage/tests`: only if a new persisted behavior is required; existing capture/settings/Undo behavior should be reused.
- `platform/windows/tests`: UTF-16 conversion, range/rectangle normalization, error mapping, bounded traversal policy, COM lifecycle wrappers, OCR fixture conversion, and clipboard rules if approved.

Do not unit-test private call order when the public provider or workflow result expresses the same behavior.

### 6.2 Platform contract tests

- Reuse `crates/platform-contract-tests` for selection and unavailable translation.
- Extend this crate only for behavior genuinely common to multiple adapters.
- Windows capability tests assert false before implementation and true only for verified providers.
- Contract fixtures include Unicode, empty selection, unsupported element, cancellation, permission/privilege failures, and exact portable metadata.

### 6.3 Frontend tests

- Test Windows pages against the `Backend` interface, not Tauri internals.
- Cover loading, empty, retry, Manual Capture, duplicate Encounter, Undo, Review completion, Settings rollback messaging, tray-driven refresh events, translation unavailable, user-entered translation, OCR single/list confirmation, cancel, stale events, focus order, Escape, and reduced motion.
- Preserve existing backend and capture race tests as prior art.
- Use semantic queries and accessible names rather than CSS selectors or snapshots as the primary assertion.

### 6.4 Tauri integration tests

- Extend the existing command-contract seam for additive commands/events, capability mapping, request identity, corrected candidate submission, and typed errors.
- Test desktop lifecycle logic behind injectable closures/services where physical window handles are unnecessary.
- Leave actual HWND activation, tray shell behavior, notification delivery, and installer behavior to physical tests.

### 6.5 Windows physical-device matrix

Record application version/commit, Windows build, architecture, WebView2 version, display layout/scales, application version, selected text kind, and result for every row.

| Application/scenario | UIA text | Bounds | Context | Source metadata | OCR | Focus preserved | Notes |
|---|---:|---:|---:|---:|---:|---:|---|
| Notepad plain text | Not run | Not run | Not run | Not run | Not run | Not run | Required |
| Windows Terminal | Not run | Not run | Not run | Not run | Not run | Not run | Required |
| VS Code editor | Not run | Not run | Not run | Not run | Not run | Not run | Required Electron case |
| Edge static page | Not run | Not run | Not run | Not run | Not run | Not run | Required Chromium case |
| Edge textarea/contenteditable | Not run | Not run | Not run | Not run | Not run | Not run | Required |
| Chrome static/editable page | Not run | Not run | Not run | Not run | Not run | Not run | Required |
| Firefox static/editable page | Not run | Not run | Not run | Not run | Not run | Not run | Required |
| Microsoft Word | Not run | Not run | Not run | Not run | Not run | Not run | Required if licensed on test host |
| PDF reader | Not run | Not run | Not run | Not run | Not run | Not run | Record product/version |
| Elevated target process | Not run | Not run | Not run | Not run | Not run | Not run | Error guidance required |

Additional physical suites:

- Shortcut: default registration, conflict, replacement rollback, repeat suppression, tray-hidden capture, restart persistence.
- OCR/privacy: consent/cancel/deny, no readable text, multiple candidates, correction, screenshot memory release, no file/log artifacts.
- Windowing: primary/secondary display, negative coordinates, 100/125/150/200%, mixed scales, taskbar on different edges, focus, Alt+Tab, Win+D, lock/unlock, display disconnect.
- UI/accessibility: keyboard-only, Narrator, high contrast, system light/dark, text scaling, minimum window size, reduced motion.
- Persistence: fresh database, migration from prior schema, duplicate Encounter, Undo, crash/restart, upgrade preservation.
- Lifecycle: close-to-tray, Open, Exit, Explorer restart, login launch, notification enabled/disabled, time-zone and daylight-saving changes.
- Packaging: clean install, WebView2 absent/present, launch without developer tools, in-place upgrade, uninstall with data retained, uninstall with explicit data deletion.

---

## 7. Release Plan

### NSIS and WebView2

- Build a Windows 11 x64 per-user NSIS installer through Tauri.
- Select and document a WebView2 installation mode supported by the pinned Tauri version. Validation of the exact bootstrapper/offline strategy is **Not run** on a clean VM or user account.
- Keep the existing application identifier stable so upgrades target the same installation and data directory.
- Do not add an in-app updater in v2.

### CI Windows lane

The existing `windows-latest` job already runs install, format, Clippy, workspace tests/build, Windows adapter tests, and frontend check/test/build. Extend that job, or add a release-only Windows job, so the complete release gate runs:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings
cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
cargo build --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
pnpm install --frozen-lockfile
pnpm check
pnpm test
pnpm build
pnpm tauri build --bundles nsis
```

Hosted CI proves compilation, automated contracts, and artifact creation only.

### Release readiness checklist

- [ ] Version, commit, Rust/Node/pnpm/Windows SDK/WebView2 versions recorded.
- [ ] All automated Windows gates pass from a clean checkout.
- [ ] Every capability marked true has provider tests and physical evidence.
- [ ] Application compatibility matrix is complete for required rows.
- [ ] Tray, shortcut, notification, autostart, focus, and mixed-DPI suites pass.
- [ ] NSIS clean install, in-place upgrade, and both uninstall-data choices pass.
- [ ] Upgrade preserves SQLite, Settings, and Review state.
- [ ] Privacy audit finds no screenshots, captured text, translations, URLs, or clipboard contents in production logs.
- [ ] Known limitations name unsupported applications and automatic translation unavailability.
- [ ] Rollback build and database compatibility are documented.
- [ ] Release artifact checksum and source commit are recorded.

---

## 8. Risks and Pending Validation

| Risk | Required mitigation/evidence |
|---|---|
| COM threading and object affinity | Own native operations on initialized worker threads; test initialization/teardown; never move COM objects into portable async state |
| UIA variability | Capability detection, bounded traversal, typed unsupported results, diagnostic tooling, and the physical application matrix |
| UTF-16 and range geometry | Fixture tests for surrogate pairs, composed characters, multiple ranges/rectangles, empty or invalid ranges |
| Clipboard side effects | Keep disabled by default; require the Milestone 8 evidence gate; preserve formats and reject stale values if approved |
| OCR privacy | Explicit user action, bounded memory-only capture, immediate release, content-free logs, artifact audit |
| OCR quality/context | Pointer-aware ranking, ambiguity confirmation, editable result, application matrix; dominance-threshold validation is **Not run** |
| DPI and coordinate spaces | Define one portable logical top-left space; test negative origins and mixed scales; remove Windows use of Cocoa conversion |
| Focus and activation | Separate passive display from explicit edit; verify HWND/Tauri behavior physically across source applications |
| Windows permission semantics | Map privilege/capture behaviors through portable errors after API experiments; do not copy macOS instructions |
| Notifications and autostart | Verify chosen Tauri/plugin or Windows mechanism, reboot behavior, disabled-notification recovery, and cleanup on uninstall |
| WebView2 deployment | Test clean account with runtime absent and present; document selected Tauri-supported mode |
| Independent UI drift | Share stable tokens/contracts, not pages; require Windows-specific behavior tests and periodic visual comparison rather than coupled implementation |
| Non-Windows verification limits | macOS/Linux/hosted CI cannot prove Windows runtime behavior; mark such rows `Not run` until physical Windows evidence exists |

No plan item may turn a **Not run** statement into a release claim without recording the command, environment, result, and date.

---

## 9. Definition of Done

Windows Platform v2 is done only when all of the following are true:

1. The Windows 11 x64 application installs from NSIS and starts without Rust, Node, pnpm, or preconfigured developer tooling.
2. Startup initializes or migrates SQLite and restores Settings without losing prior data.
3. The Windows-owned presentation provides Today, Vocabulary, Review, Settings, Manual Capture, word detail, native result, and OCR confirmation with complete loading/empty/error states.
4. Closing the main window keeps capture available in the tray; Open and Exit behave predictably; launch-at-login and review notifications match Settings.
5. The global shortcut handles registration, conflicts, replacement, persistence, repeat suppression, and tray-hidden capture.
6. UIA captures exact selected text in the required compatibility matrix where the application exposes usable selection, with portable metadata and errors.
7. UIA failure can lead to explicit, bounded, memory-only OCR; candidates use pointer-aware ranking and user confirmation; cancellation persists nothing.
8. Users can correct captured text/context, provide an optional manual translation, save without translation, Undo, and create repeat Encounters without duplicate Vocabulary Items.
9. The floating window is non-activating on presentation, keyboard accessible on interaction, correctly positioned on mixed-DPI/negative-origin displays, and absent from the taskbar as designed.
10. No Windows-specific type or conditional leaks into `crates/domain`, `crates/application`, `crates/storage`, or `crates/capture`.
11. Windows reports automatic translation unavailable honestly; no network translation or hidden external executable is introduced.
12. Automated Rust, frontend, Tauri contract, build, and NSIS gates pass on the release commit.
13. Physical-device, compatibility, accessibility, lifecycle, privacy, and installer matrices are complete for the release artifact.
14. Upgrade preserves user data and Settings; uninstall behavior matches the retained/delete-data choice.
15. Handoff documents contain no unsupported capability claim and no unexplained `Not run` row for a release requirement.

---

## 10. Agent Handoff Rules

### Required reading before work

Every implementation agent must read, in order:

1. `CONTEXT.md`
2. `docs/adr/0001-independent-windows-presentation.md`
3. `docs/windows-platform-plan-v2.md`
4. `docs/windows-development.md`, including its current uncommitted diff
5. `docs/architecture.md`
6. `docs/cross-platform-porting-guide.md`
7. `docs/native-capture-development.md`
8. The public contracts in `crates/platform-api`, `crates/application`, and `crates/capture`
9. The current Windows adapter, desktop command/event code, UI backend/types, and tests relevant to the assigned milestone

Do not assume a path or API named in an older plan still exists; confirm with `rg --files` and public definitions before editing.

### Documentation update after every milestone

- Update `docs/windows-development.md` with date, commit, machine/Windows build, tools, commands, results, capability changes, known failures, and next milestone. Preserve earlier evidence; do not rewrite failures into successes.
- Update the compatibility/manual matrix in this plan or move it to a dedicated Windows evidence document in the first milestone that needs repeated matrix edits. If moved, leave a stable link here.
- Update `docs/architecture.md` only when an actual cross-platform boundary changes.
- Update `docs/cross-platform-porting-guide.md` only when a reusable rule is learned, not for Windows-only details.
- Add an ADR only for a hard-to-reverse, surprising decision with real alternatives. Do not create ADRs as progress logs.
- Update `CONTEXT.md` only when domain vocabulary changes; never place implementation notes there.

### Evidence vocabulary

Use exactly these statuses:

- **Verified automated:** command, environment, and passing result recorded.
- **Verified Windows physical:** device/Windows build, application version, steps, and observed result recorded.
- **Not run:** no evidence exists yet; include the reason and owner milestone.
- **Blocked:** the check was attempted but cannot proceed; include the exact blocker and next action.
- **Unsupported:** deliberately not offered; include the user-visible fallback.

Never use “done,” “supported,” or a true capability flag for work that is only compiled, mocked, reviewed, or verified on a non-Windows host.

### Working rules

- Start each milestone by running its baseline tests and reviewing `git status --short`; preserve unrelated user changes.
- Use tests before implementation for portable behavior, then perform the physical checks that cannot be automated.
- Keep native diagnostics content-free by default.
- Record deviations from this plan in the handoff before proceeding. If a deviation changes architecture, privacy, release scope, or a public contract, obtain explicit approval first.
- End each milestone with a focused commit or documented reason no commit was made, updated evidence, and a clear list of remaining `Not run` checks.
