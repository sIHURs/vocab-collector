# Plan A: macOS Platform Encapsulation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve the current macOS application, establish the final three-platform repository boundaries, and encapsulate all macOS-native behavior behind stable Rust platform contracts.

**Architecture:** Shared Rust crates own product behavior and depend only on capability-specific traits in `vocab-platform-api`. A new `vocab-platform-macos` crate owns safe Rust wrappers around the Swift C ABI; the Tauri crate is reduced to platform selection, application composition, commands, and events. Linux and Windows receive contract-compatible skeleton crates, but their runtime behavior is not claimed on macOS.

**Tech Stack:** Rust 1.98+, Cargo workspace, Tauri 2, Svelte 5, Swift 6/SwiftPM, SQLite, pnpm 11, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-08-26-cross-platform-core-and-os-adapters-design.md`

## Global Constraints

- Existing platform remains macOS 15 or newer.
- Shared crates must not depend on macOS, Linux, Windows, Tauri, Swift FFI, AT-SPI, Portal, UI Automation, or Win32 APIs.
- OS selection is permitted only in platform adapters and the desktop composition root.
- Native errors and logs must not contain selected text, context, translations, URLs, or screenshots.
- UI behavior depends on reported capabilities, never an OS-name conditional.
- Linux and Windows runtime success must not be claimed from macOS-only verification.
- Use one main branch with short-lived branches; preserve the pre-refactor state with an immutable tag.

---

### Task 1: Record and tag the macOS baseline

**Files:**
- Create: `docs/macos-baseline.md`
- Modify: `.gitignore`

**Interfaces:**
- Consumes: Current `mvp-foundation` behavior and macOS build commands.
- Produces: Tag `macos-mvp-v0.1.0` and a reproducible baseline record used by every later regression check.

- [ ] **Step 1: Verify the worktree and toolchain**

Run:

```bash
git status --short --branch
rustc --version
cargo --version
node --version
pnpm --version
swift --version
xcodebuild -version
sw_vers
```

Expected: clean tracked worktree; every required command reports a version.

- [ ] **Step 2: Run the pre-refactor baseline suite**

Run:

```bash
swift test --package-path platform/macos
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check
pnpm test
pnpm build
pnpm tauri build --bundles app
```

Expected: record each PASS or exact failure before changing structure. A failing baseline is documented, not silently treated as a refactor regression.

- [ ] **Step 3: Write the baseline record**

Create `docs/macos-baseline.md` with exact tool versions, command results, Accessibility and Screen Recording setup, tested applications, and known defects. Include a checklist for Safari, Chrome, Firefox, Preview, Books, shortcut replacement, OCR confirmation, translation failure, Undo, dismissal, and multi-monitor placement.

- [ ] **Step 4: Exclude Swift build output if necessary**

Ensure `.gitignore` contains:

```gitignore
platform/macos/native/.build/
platform/macos/.build/
```

- [ ] **Step 5: Commit and tag**

```bash
git add docs/macos-baseline.md .gitignore
git commit -m "docs: record macOS MVP baseline"
git tag macos-mvp-v0.1.0
git show --stat macos-mvp-v0.1.0
```

Expected: tag resolves to the baseline commit. Push the branch and tag to the private remote, or create and verify a `git bundle` before machine transfer.

### Task 2: Establish capability-specific platform contracts

**Files:**
- Move: `crates/platform/` → `crates/platform-api/`
- Modify: `Cargo.toml`
- Modify: `crates/platform-api/Cargo.toml`
- Modify: `crates/platform-api/src/lib.rs`
- Create: `crates/platform-api/src/capabilities.rs`
- Create: `crates/platform-api/src/errors.rs`
- Create: `crates/platform-api/src/providers.rs`
- Create: `crates/platform-api/tests/contracts.rs`
- Modify: dependent `Cargo.toml` and Rust imports from `vocab-platform` to `vocab-platform-api`

**Interfaces:**
- Consumes: Existing `CaptureCandidate`, geometry, permission, translation, OCR, and enrichment types.
- Produces: `PlatformCapabilities`, `SelectionProvider`, `OcrProvider`, `TranslationProvider`, `PermissionProvider`, `WindowProvider`, and typed `PlatformError`.

- [ ] **Step 1: Write failing serialization and capability tests**

Test these exact defaults in `crates/platform-api/tests/contracts.rs`:

```rust
#[test]
fn unavailable_capabilities_are_explicit() {
    let value = PlatformCapabilities::default();
    assert!(!value.selection_capture);
    assert!(!value.selection_bounds);
    assert!(!value.screenshot_ocr);
    assert!(!value.translation);
    assert!(!value.non_activating_window);
}

#[test]
fn unsupported_error_is_content_free() {
    assert_eq!(PlatformError::Unsupported(Capability::Selection).to_string(),
               "unsupported platform capability: selection");
}
```

- [ ] **Step 2: Run the tests and verify failure**

Run: `cargo test -p vocab-platform-api --test contracts`

Expected: FAIL because the renamed crate and new types do not exist.

- [ ] **Step 3: Move the crate and implement the contracts**

Define small async traits with these signatures:

```rust
#[async_trait]
pub trait SelectionProvider: Send + Sync {
    async fn capture_selection(&self) -> Result<CaptureCandidate, PlatformError>;
}

#[async_trait]
pub trait OcrProvider: Send + Sync {
    async fn recognize_near(&self, pointer: ScreenPoint) -> Result<Vec<OcrCandidate>, PlatformError>;
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn translate(&self, text: &str, source: &str, target: &str)
        -> Result<TranslationResult, PlatformError>;
}

#[async_trait]
pub trait PermissionProvider: Send + Sync {
    async fn status(&self, kind: PermissionKind) -> Result<PermissionStatus, PlatformError>;
    async fn request(&self, kind: PermissionKind) -> Result<PermissionStatus, PlatformError>;
}

pub trait WindowProvider: Send + Sync {
    fn configure_capture_window(&self) -> Result<(), PlatformError>;
}

pub struct PlatformServices {
    pub capabilities: PlatformCapabilities,
    pub selection: Arc<dyn SelectionProvider>,
    pub ocr: Arc<dyn OcrProvider>,
    pub translation: Arc<dyn TranslationProvider>,
    pub permissions: Arc<dyn PermissionProvider>,
    pub window: Arc<dyn WindowProvider>,
}
```

Define `Capability::{Selection, SelectionBounds, ScreenshotOcr, Translation, NonActivatingWindow}` and `PlatformError::{PermissionRequired, PermissionDenied, EmptySelection, UnsupportedElement, InvalidSelectionRange, Unsupported, Cancelled, Operation}`. Keep DTO definitions portable and preserve current camelCase serialization.

- [ ] **Step 4: Update all workspace dependencies and imports**

Use package name `vocab-platform-api`; update `vocab-capture`, `vocab-desktop`, and other direct consumers. Do not add target-specific code to shared crates.

- [ ] **Step 5: Run focused and workspace tests**

Run:

```bash
cargo test -p vocab-platform-api
cargo test -p vocab-capture
cargo check --workspace
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock crates apps/desktop/src-tauri/Cargo.toml apps/desktop/src-tauri/src
git commit -m "refactor: define portable platform capability contracts"
```

### Task 3: Add reusable provider contract tests and fakes

**Files:**
- Create: `crates/platform-contract-tests/Cargo.toml`
- Create: `crates/platform-contract-tests/src/lib.rs`
- Create: `crates/platform-contract-tests/src/selection.rs`
- Create: `crates/platform-contract-tests/src/translation.rs`
- Create: `crates/platform-contract-tests/src/privacy.rs`
- Create: `crates/application/tests/platform_fakes.rs`
- Modify: `Cargo.toml`

**Interfaces:**
- Consumes: Traits and DTOs from `vocab-platform-api`.
- Produces: `assert_selection_contract`, `assert_translation_contract`, `FakeSelectionProvider`, and `UnavailableTranslationProvider` for adapter and application tests.

- [ ] **Step 1: Write failing fake-provider application tests**

Cover exact surface-form preservation, empty selection, unsupported selection, translation unavailable, and save-without-translation. The fake selection provider returns a caller-supplied `Result<CaptureCandidate, PlatformError>`.

- [ ] **Step 2: Run and verify failure**

Run: `cargo test -p vocab-application --test platform_fakes`

Expected: FAIL because the fakes and provider-injected application path do not exist.

- [ ] **Step 3: Implement the contract-test crate**

Expose test helpers that accept a provider factory and fixture values. Tests must compare exact Unicode strings and typed errors; they must never inspect private provider internals.

- [ ] **Step 4: Inject fakes into application tests**

Add a provider bundle used by tests without Tauri or native permissions. Preserve the existing storage repository injection pattern.

- [ ] **Step 5: Run tests**

```bash
cargo test -p vocab-platform-contract-tests
cargo test -p vocab-application --test platform_fakes
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock crates/platform-contract-tests crates/application
git commit -m "test: add portable platform provider contracts"
```

### Task 4: Extract the macOS Rust adapter

**Files:**
- Create: `platform/macos/rust/Cargo.toml`
- Create: `platform/macos/rust/build.rs`
- Create: `platform/macos/rust/src/lib.rs`
- Move: `apps/desktop/src-tauri/src/macos_bridge.rs` → `platform/macos/rust/src/ffi.rs`
- Create: `platform/macos/rust/src/providers.rs`
- Create: `platform/macos/rust/tests/adapter_contract.rs`
- Modify: `Cargo.toml`
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Modify: `apps/desktop/src-tauri/build.rs`

**Interfaces:**
- Consumes: `vocab-platform-api` traits and the existing Swift C ABI.
- Produces: `MacPlatform::new() -> Result<PlatformServices, PlatformError>` and safe macOS provider implementations.

- [ ] **Step 1: Write failing adapter tests**

Move the current JSON bridge tests into the new crate and add tests asserting `noSelection` maps to `PlatformError::EmptySelection`, permission-required codes map to typed permission errors, and error displays contain no bridge payload.

- [ ] **Step 2: Run and verify failure**

Run: `cargo test -p vocab-platform-macos`

Expected: FAIL because the adapter crate is not yet wired.

- [ ] **Step 3: Move FFI and implement providers**

Keep all `unsafe extern "C"` declarations and pointer ownership in `ffi.rs`. Implement the portable traits in `providers.rs`; no Tauri type may appear in this crate's public API except a narrowly scoped native-window handle adapter where unavoidable.

- [ ] **Step 4: Move Swift linking into the adapter build**

Move `swift-rs` build dependency and rpath/link configuration from the desktop crate into `platform/macos/rust/build.rs`. Restrict the dependency with the macOS target configuration.

- [ ] **Step 5: Run focused checks**

```bash
cargo test -p vocab-platform-macos
cargo clippy -p vocab-platform-macos --all-targets -- -D warnings
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock platform/macos/rust apps/desktop/src-tauri
git commit -m "refactor: extract macOS Rust platform adapter"
```

### Task 5: Place the Swift package under the macOS native boundary

**Files:**
- Move: `platform/macos/Package.swift` → `platform/macos/native/Package.swift`
- Move: `platform/macos/Sources/` → `platform/macos/native/Sources/`
- Move: `platform/macos/Tests/` → `platform/macos/native/Tests/`
- Modify: `platform/macos/README.md`
- Modify: `platform/macos/rust/build.rs`
- Modify: `docs/native-capture-development.md`

**Interfaces:**
- Consumes: Existing exported `vocab_mac_*` C symbols.
- Produces: The same ABI from the new package path; no product behavior change.

- [ ] **Step 1: Record the exported ABI names**

Run: `rg 'vocab_mac_' platform/macos/Sources apps/desktop/src-tauri/src/macos_bridge.rs`

Expected: capture the exact symbol set before moving files.

- [ ] **Step 2: Move the package without changing symbols**

Use `git mv`; update the Rust build path to `../native` relative to the adapter crate. Do not rename Swift exported functions in this task.

- [ ] **Step 3: Verify Swift and Rust consumers**

```bash
swift test --package-path platform/macos/native
cargo test -p vocab-platform-macos
```

Expected: PASS with the same serialization fixtures.

- [ ] **Step 4: Update development documentation and commit**

```bash
git add platform/macos docs/native-capture-development.md
git commit -m "refactor: isolate macOS native Swift package"
```

### Task 6: Make the Tauri composition and commands platform-neutral

**Files:**
- Split: `apps/desktop/src-tauri/src/lib.rs`
- Create: `apps/desktop/src-tauri/src/bootstrap.rs`
- Create: `apps/desktop/src-tauri/src/commands/mod.rs`
- Create: `apps/desktop/src-tauri/src/commands/library.rs`
- Create: `apps/desktop/src-tauri/src/commands/capture.rs`
- Create: `apps/desktop/src-tauri/src/events.rs`
- Create: `apps/desktop/src-tauri/tests/command_contract.rs`
- Modify: `apps/desktop/src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: `PlatformServices` and shared application services.
- Produces: One command/event surface for every target and `build_app_state(repository, platform) -> AppState`.

- [ ] **Step 1: Write failing command-surface tests**

Assert the platform-neutral capture service exposes permission status, selection capture, OCR, translation, save, hide, and capability lookup without `#[cfg]` on individual command names.

- [ ] **Step 2: Run and verify failure**

Run: `cargo test -p vocab-desktop --test command_contract`

Expected: FAIL because commands still call macOS functions directly.

- [ ] **Step 3: Split composition from command handlers**

`bootstrap.rs` selects `MacPlatform` under the macOS target and constructs `AppState`. `commands/capture.rs` calls injected traits. `events.rs` owns `NativeCaptureEvent` and typed `NativeCaptureErrorEvent`. Keep `lib.rs` limited to builder configuration and module wiring.

- [ ] **Step 4: Remove direct Swift calls from Tauri**

Verify: `rg 'macos_bridge|vocab_mac_' apps/desktop/src-tauri/src`

Expected: no matches.

- [ ] **Step 5: Run tests**

```bash
cargo test -p vocab-desktop --lib
cargo test -p vocab-desktop --test command_contract
cargo test --workspace
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add apps/desktop/src-tauri
git commit -m "refactor: make desktop composition platform neutral"
```

### Task 7: Replace UI string matching with typed capabilities and failures

**Files:**
- Modify: `ui/src/lib/types.ts`
- Modify: `ui/src/lib/backend.ts`
- Modify: `ui/src/FloatingCapture.svelte`
- Modify: `ui/src/FloatingCapture.test.ts`

**Interfaces:**
- Consumes: `PlatformCapabilities` and typed failure codes from desktop commands/events.
- Produces: `PlatformCapabilities`, `CaptureFailureCode`, and deterministic UI actions independent of OS error strings.

- [ ] **Step 1: Write failing UI tests**

Add cases proving `permission_required` shows the permission action, `empty_selection` offers OCR only when `screenshotOcr` is true, `translation_unavailable` offers save without translation, and arbitrary diagnostic messages never control buttons.

- [ ] **Step 2: Run and verify failure**

Run: `pnpm test -- FloatingCapture.test.ts`

Expected: FAIL because the component currently uses `error.includes(...)`.

- [ ] **Step 3: Add typed frontend models and rendering**

Define:

```ts
export type CaptureFailureCode =
  | "permission_required" | "permission_denied" | "empty_selection"
  | "unsupported_element" | "translation_unavailable" | "cancelled" | "operation";
```

Render actions from the code plus `PlatformCapabilities`; retain the diagnostic message only as non-sensitive explanatory copy.

- [ ] **Step 4: Run UI verification**

```bash
pnpm check
pnpm test
pnpm build
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src
git commit -m "refactor: drive capture UI from typed platform state"
```

### Task 8: Add Linux and Windows adapter skeletons and CI selection

**Files:**
- Create: `platform/linux/Cargo.toml`
- Create: `platform/linux/src/lib.rs`
- Create: `platform/linux/tests/capabilities.rs`
- Create: `platform/windows/Cargo.toml`
- Create: `platform/windows/src/lib.rs`
- Create: `platform/windows/tests/capabilities.rs`
- Modify: `Cargo.toml`
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Modify: `apps/desktop/src-tauri/src/bootstrap.rs`
- Modify: `.github/workflows/ci.yml`
- Create: `docs/linux-development.md`
- Create: `docs/windows-development.md`

**Interfaces:**
- Consumes: `vocab-platform-api` and platform-neutral composition.
- Produces: `LinuxPlatform::new()` and `WindowsPlatform::new()` skeletons that return explicit unsupported capabilities.

- [ ] **Step 1: Write capability tests for both skeletons**

Assert every not-yet-verified native capability is false and every invoked unavailable provider returns `PlatformError::Unsupported`.

- [ ] **Step 2: Run target-independent skeleton tests**

```bash
cargo test -p vocab-platform-linux
cargo test -p vocab-platform-windows
```

Expected: FAIL before skeletons, then PASS after minimal implementations. Native OS imports remain target-gated.

- [ ] **Step 3: Add target-specific desktop dependencies**

Use Cargo target dependency tables for macOS, Linux, and Windows. `bootstrap.rs` selects exactly one adapter per target; unsupported targets fail at compile time with a clear message.

- [ ] **Step 4: Add three-OS CI jobs**

Configure `macos-latest`, `ubuntu-24.04`, and `windows-latest` jobs. Shared/API changes run all jobs; each job installs the required Rust, Node, and pnpm versions and runs formatting, Clippy, Rust tests, Svelte checks, and builds supported on that runner.

- [ ] **Step 5: Document target setup without claiming runtime success**

List install and verification commands in the Linux and Windows development documents, explicitly marking native runtime checks as Plan B physical-machine work.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock platform/linux platform/windows apps/desktop .github docs
git commit -m "build: scaffold Linux and Windows platform adapters"
```

### Task 9: Fix macOS boundary defects and run the handoff gate

**Files:**
- Modify: `platform/macos/rust/src/ffi.rs`
- Modify: `platform/macos/rust/src/providers.rs`
- Modify: `platform/macos/rust/tests/adapter_contract.rs`
- Modify: `crates/capture/tests/coordinator.rs`
- Modify: `docs/macos-baseline.md`
- Modify: `docs/cross-platform-porting-guide.md`

**Interfaces:**
- Consumes: Typed platform errors, cancellation, capability contracts, and baseline defect list.
- Produces: A macOS adapter whose bridge memory, error mapping, stale requests, and optional capabilities match the shared contracts.

- [ ] **Step 1: Convert each reproducible boundary defect into a focused failing test**

For each baseline issue, place the test at the lowest responsible layer: bridge decoding in the macOS adapter, stale/save-once behavior in `vocab-capture`, or presentation state in Svelte. Do not group unrelated defects in one test.

- [ ] **Step 2: Run each focused test and verify the reported defect**

Use the package-specific command and confirm failure matches the recorded defect, not an environment or permission failure.

- [ ] **Step 3: Apply the smallest boundary-preserving fix**

Reject any fix that moves SQLite, save/Undo, review, or UI policy into Swift or the macOS adapter. Update the known-issues record after each verified fix.

- [ ] **Step 4: Run the complete handoff verification**

```bash
swift test --package-path platform/macos/native
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check
pnpm test
pnpm build
pnpm tauri build --bundles app
```

Expected: PASS. Manually verify selection, permission denial/recovery, OCR confirmation, translation unavailable, save-once, Undo, dismissal, and multi-monitor placement against `docs/macos-baseline.md`.

- [ ] **Step 5: Commit the verified fixes and handoff documentation**

```bash
git add platform/macos crates/capture ui docs
git commit -m "fix: stabilize macOS adapter boundary"
git status --short --branch
```

Expected: clean worktree. Push all commits and the baseline tag, or produce a verified Git bundle for the Ubuntu and Windows machines.
