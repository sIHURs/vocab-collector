# Plan B: Linux and Windows Platform Ports Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement and verify independent Ubuntu 24.04 and Windows 11 adapters against the shared Rust contracts established by Plan A, without forking core product behavior.

**Architecture:** `vocab-platform-linux` and `vocab-platform-windows` implement the same capability-specific traits from `vocab-platform-api`. Linux integrates AT-SPI2 and XDG Desktop Portal; Windows integrates UI Automation, COM, Windows Graphics Capture, and minimal Win32 window behavior. Both are selected only in the Tauri composition root and are protected by shared contract tests and three-OS CI.

**Tech Stack:** Rust 1.98+, Tauri 2, Svelte 5, SQLite, Ubuntu 24.04 GNOME Wayland, AT-SPI2, XDG Desktop Portal, Windows 11 x64, Windows UI Automation/Win32/WinRT, pnpm 11, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-08-26-cross-platform-core-and-os-adapters-design.md`

## Global Constraints

- Begin only after Plan A passes its macOS handoff gate and is pushed to the common repository.
- Execute `BL` tasks on an Ubuntu 24.04 GNOME Wayland physical machine and `BW` tasks on a Windows 11 x64 physical machine.
- Linux and Windows adapters may depend on shared contracts; shared crates may not depend on either adapter.
- Do not mutate the clipboard, synthesize Copy, move source focus, poll the screen, or run OCR without explicit confirmation.
- Preserve selected text exactly and normalize native coordinates to logical top-left virtual-desktop coordinates.
- Selected text, context, URLs, translations, and screenshots must not enter logs or crash diagnostics.
- A native capability that is not verified returns explicit `Unsupported`; it must not be silently emulated.
- Shared-core or contract changes require OS-neutral regression tests and green macOS, Linux, and Windows CI.
- Translation success is not required for the first native preview; save-without-translation must remain available until a separately approved offline-provider design is implemented.

---

### Task 1: Validate the Plan A handoff on both target machines

**Files:**
- Modify: `docs/linux-development.md`
- Modify: `docs/windows-development.md`
- Create: `docs/platform-test-matrix.md`

**Interfaces:**
- Consumes: Plan A main branch, `LinuxPlatform::new()`, `WindowsPlatform::new()`, shared commands, and contract tests.
- Produces: Reproducible target-machine environments and a three-platform test matrix.

- [ ] **Step 1: Clone the same repository revision on Ubuntu and Windows**

Run on both machines:

On the Mac, run `git remote get-url origin` and copy the exact result. On each target machine, assign that exact value to `VOCAB_REPOSITORY_URL`, then run:

```bash
git clone "$VOCAB_REPOSITORY_URL" vocab-collector-app
cd vocab-collector-app
git rev-parse HEAD
```

Expected: both hashes equal the Plan A handoff commit. If using `git bundle`, clone the verified bundle and configure the eventual private remote before platform work diverges.

- [ ] **Step 2: Install and record target toolchains**

On Ubuntu install the documented Rust, Node, pnpm, Tauri GTK/WebKit, AT-SPI, D-Bus, and Portal development packages. On Windows install Rust MSVC, Node, pnpm, Visual Studio Build Tools, Windows SDK, and WebView2. Append exact versions and install commands to the corresponding development document.

- [ ] **Step 3: Run the shared baseline on each machine**

```bash
cargo fmt --all --check
cargo test -p vocab-domain
cargo test -p vocab-application
cargo test -p vocab-capture
cargo test -p vocab-platform-api
cargo test -p vocab-platform-contract-tests
pnpm install --frozen-lockfile
pnpm check
pnpm test
pnpm build
```

Expected: PASS on both machines before native adapter work.

- [ ] **Step 4: Create the platform matrix**

Record rows for Ubuntu/Windows launch, manual capture, native selection applications, shortcut, permissions, OCR, translation-unavailable save, Undo, focus, DPI/scaling, multiple displays, installer, and upgrade. Each cell uses only `Not run`, `Pass`, `Fail`, or `Unsupported`, with a linked issue for failures.

- [ ] **Step 5: Commit environment documentation**

```bash
git add docs/linux-development.md docs/windows-development.md docs/platform-test-matrix.md
git commit -m "docs: record Linux and Windows port environments"
```

### Task 2: Complete Ubuntu startup and shared behavior

**Files:**
- Modify: `platform/linux/Cargo.toml`
- Modify: `platform/linux/src/lib.rs`
- Create: `platform/linux/src/session.rs`
- Create: `platform/linux/src/capabilities.rs`
- Modify: `apps/desktop/src-tauri/src/bootstrap.rs`
- Modify: `apps/desktop/src-tauri/tauri.conf.json`
- Test: `platform/linux/tests/capabilities.rs`

**Interfaces:**
- Consumes: `PlatformCapabilities` and `PlatformServices` from Plan A.
- Produces: `LinuxSession::detect() -> Result<LinuxSession, PlatformError>` and a launchable Linux desktop runtime.

- [ ] **Step 1: Write failing session-detection tests**

Use injected environment values to assert `XDG_SESSION_TYPE=wayland` and `XDG_CURRENT_DESKTOP=GNOME` produce `LinuxSession { desktop: Gnome, display: Wayland }`; missing values produce typed `Unknown` values rather than panic.

- [ ] **Step 2: Run and verify failure**

Run: `cargo test -p vocab-platform-linux --test capabilities`

Expected: FAIL because session detection is absent.

- [ ] **Step 3: Implement detection and honest capabilities**

Implement pure parsing separately from process environment reads. Initially report manual capture and shared storage as available; selection, OCR, and native-window refinements remain false until their tasks pass physical tests.

- [ ] **Step 4: Launch on the physical Ubuntu session**

```bash
cargo test -p vocab-platform-linux
pnpm tauri dev
```

Expected: main window launches; SQLite, Today, Vocabulary, Review, Settings, manual Quick Capture, and Undo work. Native shortcut capture may still return `Unsupported`.

- [ ] **Step 5: Commit**

```bash
git add platform/linux apps/desktop/src-tauri docs/platform-test-matrix.md
git commit -m "feat(linux): launch shared desktop application"
```

### Task 3: Implement Linux AT-SPI selection

**Files:**
- Modify: `platform/linux/Cargo.toml`
- Create: `platform/linux/src/accessibility/mod.rs`
- Create: `platform/linux/src/accessibility/atspi.rs`
- Create: `platform/linux/src/accessibility/text_range.rs`
- Create: `platform/linux/tests/atspi_fixtures.rs`
- Create: `platform/linux/tests/selection_contract.rs`
- Modify: `platform/linux/src/capabilities.rs`

**Interfaces:**
- Consumes: `SelectionProvider`, `CaptureCandidate`, `ScreenRect`, `PlatformError`.
- Produces: `AtSpiSelectionProvider` implementing `capture_selection()`.

- [ ] **Step 1: Write failing pure range tests**

Fixtures must cover ASCII, composed Unicode, emoji, empty ranges, phrase selections, missing extents, and context containing the exact selected surface form. Expected failures use `EmptySelection`, `UnsupportedElement`, or `InvalidSelectionRange`.

- [ ] **Step 2: Run and verify failure**

Run: `cargo test -p vocab-platform-linux --test atspi_fixtures`

Expected: FAIL because range conversion is absent.

- [ ] **Step 3: Implement AT-SPI access behind the provider**

Resolve the focused accessible object, query Text/Selection interfaces, read the selected range and smallest useful surrounding text, and convert available character extents into portable logical coordinates. Keep D-Bus and AT-SPI types private to this module.

- [ ] **Step 4: Run reusable contract tests**

Run:

```bash
cargo test -p vocab-platform-linux --test atspi_fixtures
cargo test -p vocab-platform-linux --test selection_contract
```

Expected: PASS for fixtures and provider error mapping.

- [ ] **Step 5: Run the physical application matrix**

Verify Firefox, Chromium, GNOME Text Editor, LibreOffice, and one PDF reader. Record exact `Pass`, `Unsupported`, or linked failure results. Confirm the clipboard and source focus do not change.

- [ ] **Step 6: Enable the capability and commit**

Enable `selection_capture` only when AT-SPI is reachable; enable `selection_bounds` only when geometry is actually returned.

```bash
git add platform/linux docs/platform-test-matrix.md
git commit -m "feat(linux): capture selected text through AT-SPI"
```

### Task 4: Implement Linux shortcut, placement, Portal OCR, and `.deb`

**Files:**
- Create: `platform/linux/src/window.rs`
- Create: `platform/linux/src/permissions.rs`
- Create: `platform/linux/src/screenshot/mod.rs`
- Create: `platform/linux/src/screenshot/portal.rs`
- Create: `platform/linux/src/screenshot/tesseract.rs`
- Create: `platform/linux/tests/coordinates.rs`
- Create: `platform/linux/tests/portal_fixtures.rs`
- Modify: `apps/desktop/src-tauri/tauri.conf.json`
- Modify: `platform/linux/src/capabilities.rs`
- Modify: `docs/linux-development.md`

**Interfaces:**
- Consumes: shared shortcut gate, placement function, `OcrProvider`, `WindowProvider`, and typed permission errors.
- Produces: GNOME Wayland shortcut/window behavior, explicit Portal OCR, and a `.deb` preview.

- [ ] **Step 1: Write failing coordinate and Portal response tests**

Cover 100%/200% scaling, negative monitor positions, absent selection bounds, user-cancelled Portal requests, denied screenshots, empty OCR, and ranked OCR candidates. Portal parsing tests use recorded content-free D-Bus response fixtures.

- [ ] **Step 2: Run and verify failure**

```bash
cargo test -p vocab-platform-linux --test coordinates
cargo test -p vocab-platform-linux --test portal_fixtures
```

Expected: FAIL before implementations.

- [ ] **Step 3: Implement window and shortcut integration**

Use Tauri behavior where it satisfies the contract and keep GNOME/Wayland-specific fallbacks in `window.rs`. Registration conflicts remain typed failures. Confirm the passive card does not steal focus; if GNOME prevents a capability, report it instead of compositor-specific bypasses.

- [ ] **Step 4: Implement explicit Portal OCR**

Create a Portal screenshot request only from the shared confirmed-OCR path, hide Vocab Collector windows first, process the returned image with packaged Tesseract language data, return ranked candidates, and release image memory immediately. Do not persist Portal URIs or image paths.

- [ ] **Step 5: Run automated and physical verification**

```bash
cargo test -p vocab-platform-linux
cargo test -p vocab-capture
pnpm check
pnpm test
pnpm tauri dev
```

Verify shortcut replacement, OCR cancellation, permission denial, two displays, 100%/200% scaling, focus, save-without-translation, save-once, Undo, and dismissal.

- [ ] **Step 6: Build and test the Debian package**

Run: `pnpm tauri build --bundles deb`

Expected: installable `.deb`; install it on a clean Ubuntu user account and verify launch without Rust, Node, or pnpm.

- [ ] **Step 7: Commit**

```bash
git add platform/linux apps/desktop docs
git commit -m "feat(linux): add native capture and Debian preview"
```

### Task 5: Complete Windows startup and shared behavior

**Files:**
- Modify: `platform/windows/Cargo.toml`
- Modify: `platform/windows/src/lib.rs`
- Create: `platform/windows/src/capabilities.rs`
- Create: `platform/windows/src/com.rs`
- Modify: `apps/desktop/src-tauri/src/bootstrap.rs`
- Test: `platform/windows/tests/capabilities.rs`

**Interfaces:**
- Consumes: `PlatformCapabilities`, `PlatformServices`, and target-specific Cargo selection.
- Produces: `ComApartmentGuard`, `WindowsPlatform::new()`, and a launchable Windows runtime.

- [ ] **Step 1: Write failing capability and COM lifecycle tests**

Assert default capabilities are false until initialized, COM initialization errors are typed, and apartment cleanup happens once through RAII.

- [ ] **Step 2: Run and verify failure**

Run in PowerShell: `cargo test -p vocab-platform-windows --test capabilities`

Expected: FAIL because Windows initialization is absent.

- [ ] **Step 3: Implement Windows initialization**

Use target-scoped `windows` crate features for Foundation, UI Automation, Graphics Capture, and required Win32 APIs. Keep all Windows types private to the adapter. Initialize COM on the threads that call UI Automation.

- [ ] **Step 4: Launch on the physical Windows machine**

```powershell
cargo test -p vocab-platform-windows
pnpm tauri dev
```

Expected: main window launches and shared SQLite, Today, Vocabulary, Review, Settings, manual capture, and Undo work.

- [ ] **Step 5: Commit**

```powershell
git add platform/windows apps/desktop/src-tauri docs/platform-test-matrix.md
git commit -m "feat(windows): launch shared desktop application"
```

### Task 6: Implement Windows UI Automation selection

**Files:**
- Create: `platform/windows/src/accessibility/mod.rs`
- Create: `platform/windows/src/accessibility/uia.rs`
- Create: `platform/windows/src/accessibility/text_range.rs`
- Create: `platform/windows/tests/uia_fixtures.rs`
- Create: `platform/windows/tests/selection_contract.rs`
- Modify: `platform/windows/src/capabilities.rs`

**Interfaces:**
- Consumes: `SelectionProvider`, portable candidates/errors, and `ComApartmentGuard`.
- Produces: `UiaSelectionProvider` implementing `capture_selection()`.

- [ ] **Step 1: Write failing UTF-16 and rectangle tests**

Cover BMP text, surrogate pairs, composed characters, phrases, multiple bounding rectangles, empty selections, unsupported controls, and per-monitor DPI conversion. Never slice a Rust string by a UI Automation UTF-16 offset.

- [ ] **Step 2: Run and verify failure**

Run: `cargo test -p vocab-platform-windows --test uia_fixtures`

Expected: FAIL before conversion and provider code.

- [ ] **Step 3: Implement UI Automation access**

Resolve the focused element, request TextPattern2 then TextPattern, obtain selection ranges, read selected text and available surrounding document range, and normalize bounding rectangles. Map HRESULTs into content-free portable categories.

- [ ] **Step 4: Run contracts and physical matrix**

```powershell
cargo test -p vocab-platform-windows --test uia_fixtures
cargo test -p vocab-platform-windows --test selection_contract
```

Verify Edge, Chrome, Firefox, Notepad, Microsoft Word, and one PDF reader. Confirm no clipboard or focus mutation.

- [ ] **Step 5: Enable capabilities and commit**

```powershell
git add platform/windows docs/platform-test-matrix.md
git commit -m "feat(windows): capture text through UI Automation"
```

### Task 7: Implement Windows shortcut, window, OCR, and NSIS packaging

**Files:**
- Create: `platform/windows/src/window.rs`
- Create: `platform/windows/src/permissions.rs`
- Create: `platform/windows/src/screenshot/mod.rs`
- Create: `platform/windows/src/screenshot/graphics_capture.rs`
- Create: `platform/windows/src/screenshot/media_ocr.rs`
- Create: `platform/windows/tests/coordinates.rs`
- Create: `platform/windows/tests/capture_fixtures.rs`
- Modify: `apps/desktop/src-tauri/tauri.conf.json`
- Modify: `platform/windows/src/capabilities.rs`
- Modify: `docs/windows-development.md`

**Interfaces:**
- Consumes: shared shortcut, placement, OCR, permission, and window contracts.
- Produces: Windows native capture behavior and an NSIS preview installer.

- [ ] **Step 1: Write failing DPI, HRESULT, and OCR tests**

Cover 100%/125%/150%/200% scaling, negative monitor coordinates, taskbar work areas, shortcut conflicts, capture cancellation, access denial, no readable text, and ranked OCR candidates.

- [ ] **Step 2: Run and verify failure**

```powershell
cargo test -p vocab-platform-windows --test coordinates
cargo test -p vocab-platform-windows --test capture_fixtures
```

Expected: FAIL before implementations.

- [ ] **Step 3: Implement minimal native window behavior**

Use Tauri first; apply only the Win32 extended styles required for non-activating, always-on-top, task-switcher-aware behavior. Keep HWND operations inside `window.rs` and feed normalized work areas to the shared placement algorithm.

- [ ] **Step 4: Implement explicit screenshot OCR**

Use Windows Graphics Capture only after shared OCR confirmation, exclude or hide Vocab Collector windows, recognize the bounded image with Windows Media OCR, return portable candidates, and release image objects immediately.

- [ ] **Step 5: Run automated and physical verification**

```powershell
cargo test -p vocab-platform-windows
cargo test -p vocab-capture
pnpm check
pnpm test
pnpm tauri dev
```

Verify shortcuts, focus, task switcher, two displays, mixed DPI, permission/cancellation paths, save-without-translation, save-once, Undo, and dismissal.

- [ ] **Step 6: Build and test NSIS**

Run: `pnpm tauri build --bundles nsis`

Expected: installer succeeds on a clean Windows user account, launches with WebView2, upgrades over the previous preview, and uninstalls cleanly.

- [ ] **Step 7: Commit**

```powershell
git add platform/windows apps/desktop docs
git commit -m "feat(windows): add native capture and NSIS preview"
```

### Task 8: Define the offline translation follow-up without blocking ports

**Files:**
- Create: `docs/superpowers/specs/2026-08-26-linux-windows-translation-provider-decision.md`
- Modify: `platform/linux/src/lib.rs`
- Modify: `platform/windows/src/lib.rs`
- Modify: `ui/src/FloatingCapture.test.ts`
- Modify: `docs/platform-test-matrix.md`

**Interfaces:**
- Consumes: `TranslationProvider`, `PlatformCapabilities.translation`, and the shared save-without-translation flow.
- Produces: An explicit unavailable provider for both previews and a decision document for a separately approved offline translation implementation.

- [ ] **Step 1: Add failing unavailable-translation tests**

Assert Linux and Windows return `PlatformError::Unsupported(Capability::Translation)`, report `translation: false`, retain the capture candidate, and allow exactly one save-without-translation operation.

- [ ] **Step 2: Run and verify failure**

```bash
cargo test -p vocab-platform-linux
cargo test -p vocab-platform-windows
pnpm test -- FloatingCapture.test.ts
```

Expected: FAIL until both adapters and UI state agree on the typed result.

- [ ] **Step 3: Implement the explicit unavailable providers**

Use the shared `UnavailableTranslationProvider`; do not send captured content to a network service and do not invoke external executables.

- [ ] **Step 4: Write the provider decision document**

Compare at least one shared offline engine and one OS-specific approach using fixed criteria: supported language pairs, model license, binary/model size, CPU/RAM, latency, Rust/FFI safety, offline model preparation, cancellation, Windows/Linux packaging, update strategy, and privacy. The document must recommend one bounded follow-up and state whether macOS keeps Apple Translation or adopts the shared engine.

- [ ] **Step 5: Verify preview behavior and commit**

```bash
cargo test -p vocab-platform-linux
cargo test -p vocab-platform-windows
pnpm test
git add platform ui docs
git commit -m "docs: define cross-platform translation provider decision"
```

Expected: both previews behave honestly without translation; translation engine implementation waits for approval of the new decision document and its own implementation plan.

### Task 9: Run three-platform regression and publish preview handoff

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `docs/platform-test-matrix.md`
- Modify: `docs/cross-platform-porting-guide.md`
- Create: `docs/platform-preview-release.md`

**Interfaces:**
- Consumes: macOS adapter from Plan A plus completed Linux and Windows adapters.
- Produces: Green three-platform CI, physical-machine evidence, and preview release instructions.

- [ ] **Step 1: Add path-aware three-platform CI**

Shared core, API, UI, command, or workspace configuration changes trigger macOS, Ubuntu, and Windows jobs. Platform-only changes trigger shared checks plus the affected target job. Keep a manual full-matrix workflow for release candidates.

- [ ] **Step 2: Run complete target suites**

On each target run formatting, Clippy, full Rust workspace tests, Svelte checks/tests/build, native adapter tests, and the target bundle command. Expected: PASS, with physical-only checks recorded in the matrix.

- [ ] **Step 3: Audit dependency direction and platform leakage**

Run:

```bash
rg '#\[cfg\(target_os' crates ui
rg 'vocab-platform-(macos|linux|windows)' crates
rg 'AT-SPI|Win32|UIAutomation|Swift|AppKit' crates/domain crates/application crates/capture crates/storage crates/sync
```

Expected: no platform implementation or OS branching in shared product crates; any legitimate `cfg` is documented and moved to a boundary when possible.

- [ ] **Step 4: Complete physical matrices**

On macOS rerun the Plan A baseline subset affected by shared changes. On Ubuntu and Windows complete application, shortcut, permissions, OCR, focus, scaling, multi-monitor, installer, and upgrade rows. No cell remains `Not run` for a claimed preview capability.

- [ ] **Step 5: Write preview release documentation**

Document supported OS versions, installers, verified applications, unsupported cases, translation-unavailable behavior, privacy rules, rollback tags, and database compatibility. Do not claim universal Linux or Windows application support.

- [ ] **Step 6: Commit and tag preview candidates**

```bash
git add .github docs
git commit -m "docs: prepare Linux and Windows preview handoff"
git tag multi-platform-preview-v0.2.0
git status --short --branch
```

Expected: clean worktree, green CI, `.deb` and NSIS artifacts traceable to the same commit, and macOS regression evidence recorded.
