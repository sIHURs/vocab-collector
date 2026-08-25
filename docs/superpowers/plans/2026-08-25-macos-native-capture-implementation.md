# macOS Native Capture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver system-wide selected-text capture on macOS with a configurable `⌥ Space V` shortcut, native Accessibility and Translation integration, a separately positioned floating Tauri window, automatic saving and dismissal, and an explicit Vision OCR fallback.

**Architecture:** Shared Rust contracts and a new capture coordinator own the portable state machine, cancellation, placement, and persistence. A Swift Package exposes macOS Accessibility, Translation, ScreenCaptureKit, Vision, permission, and native panel functions through a JSON C ABI; Tauri composes these adapters and emits typed events to one shared Svelte floating-window UI.

**Tech Stack:** Rust 1.98+, Tauri 2, Svelte 5, Swift 6, Xcode 16+, macOS Accessibility, Natural Language, Translation, ScreenCaptureKit, Vision, SQLite.

**Spec:** `docs/superpowers/specs/2026-08-25-macos-native-capture-design.md`

## Global Constraints

- Deployment target is macOS 15 or newer; development requires Xcode 16 or newer.
- Capture runs only after the user presses the configured shortcut.
- Default canonical shortcut is `Alt+Space+V`; Settings displays `⌥ Space V` and permits atomic replacement.
- Accessibility is the primary selection path; OCR requires explicit confirmation after selection capture fails.
- Capture, OCR, language analysis, and translation remain on device.
- Selected text, context, translations, URLs, and screenshots never enter logs or telemetry.
- The manual Quick Capture workflow remains operational throughout implementation.
- The floating card follows the approved 380×220/380×280 Inkdrop-inspired design and dismisses four seconds after a successful save unless paused.
- Every product behavior follows red-green-refactor TDD. Generated bindings and declarative bundle configuration are reviewed through compilation and integration checks.
- Each task ends in a meaningful Git commit.

---

### Task 1: Portable Capture Contracts and Placement

**Files:**
- Create: `crates/capture/Cargo.toml`
- Create: `crates/capture/src/lib.rs`
- Create: `crates/capture/src/placement.rs`
- Create: `crates/capture/tests/placement.rs`
- Modify: `Cargo.toml`
- Modify: `crates/platform/src/lib.rs`
- Modify: `crates/domain/src/models.rs`
- Modify: `crates/domain/tests/domain_behavior.rs`

**Interfaces:**
- Consumes: existing `CaptureCard`, `ApplicationError`, and platform `CaptureProvider` concepts.
- Produces: `ScreenPoint`, `ScreenRect`, `MonitorWorkArea`, `CaptureOrigin`, `PermissionKind`, `PermissionStatus`, `OcrCandidate`, `OcrProvider`, expanded `TranslationProvider`, and `place_floating_window(anchor, pointer, monitors, card_size) -> ScreenPoint`.

- [ ] **Step 1: Write failing placement and serialization tests**

```rust
#[test]
fn places_above_selection_and_flips_below_at_top_edge() {
    let monitor = MonitorWorkArea::new(-1440.0, 0.0, 1440.0, 900.0, 2.0);
    let card = ScreenSize::new(380.0, 220.0);
    assert_eq!(place_floating_window(rect(-900.0, 500.0, 120.0, 24.0), point(-840.0, 512.0), &[monitor], card), point(-1030.0, 268.0));
    assert_eq!(place_floating_window(rect(-900.0, 8.0, 120.0, 24.0), point(-840.0, 20.0), &[monitor], card), point(-1030.0, 44.0));
}

#[test]
fn capture_origin_round_trips_through_json() {
    let value = serde_json::to_string(&CaptureOrigin::Ocr).unwrap();
    assert_eq!(serde_json::from_str::<CaptureOrigin>(&value).unwrap(), CaptureOrigin::Ocr);
}
```

- [ ] **Step 2: Run tests and confirm missing types/functions fail**

Run: `cargo test -p vocab-capture --test placement && cargo test -p vocab-domain`

Expected: compilation fails because `vocab-capture`, geometry types, and `CaptureOrigin` do not exist.

- [ ] **Step 3: Implement the minimal portable contracts and placement algorithm**

```rust
pub fn place_floating_window(
    anchor: ScreenRect,
    pointer: ScreenPoint,
    monitors: &[MonitorWorkArea],
    card: ScreenSize,
) -> ScreenPoint {
    let monitor = monitor_for_anchor(anchor, pointer, monitors);
    let centered_x = anchor.x + (anchor.width - card.width) / 2.0;
    let preferred_y = anchor.y - card.height - 12.0;
    let y = if preferred_y >= monitor.y + 8.0 {
        preferred_y
    } else {
        anchor.y + anchor.height + 12.0
    };
    clamp_to_work_area(ScreenPoint { x: centered_x, y }, card, monitor, 8.0)
}
```

Add `capture_origin: CaptureOrigin` to the application-side capture request while keeping the database schema backward compatible by defaulting old callers to `Accessibility` or `Manual` as defined in the final enum.

- [ ] **Step 4: Run focused and workspace verification**

Run: `cargo fmt --all && cargo test -p vocab-capture && cargo test -p vocab-domain && cargo clippy -p vocab-capture --all-targets -- -D warnings`

Expected: all tests pass with no warnings.

- [ ] **Step 5: Commit contracts**

```bash
git add Cargo.toml Cargo.lock crates/capture crates/platform crates/domain
git commit -m "feat(capture): add portable capture contracts and placement"
```

### Task 2: Configurable Global Shortcut

**Files:**
- Create: `crates/capture/src/shortcut.rs`
- Create: `crates/capture/tests/shortcut.rs`
- Create: `apps/desktop/src-tauri/capabilities/default.json`
- Create: `ui/src/components/ShortcutRecorder.svelte`
- Create: `ui/src/components/ShortcutRecorder.test.ts`
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `crates/domain/src/models.rs`
- Modify: `ui/src/App.svelte`
- Modify: `ui/src/lib/types.ts`
- Modify: `ui/package.json`

**Interfaces:**
- Consumes: `UserSettings.capture_shortcut` and Tauri application setup.
- Produces: `ShortcutManager::replace(&self, candidate: &str) -> Result<RegisteredShortcut, ShortcutError>`, commands `get_shortcut()` and `replace_shortcut(candidate)`, and `ShortcutRecorder` events `recorded`/`cancelled`.

- [ ] **Step 1: Write failing Rust shortcut-policy tests**

```rust
#[test]
fn validates_default_and_rejects_unmodified_or_reserved_shortcuts() {
    assert_eq!(parse_shortcut("Alt+Space+V").unwrap().display_macos(), "⌥ Space V");
    assert_eq!(parse_shortcut("V").unwrap_err(), ShortcutError::ModifierRequired);
    assert_eq!(parse_shortcut("Meta+Q").unwrap_err(), ShortcutError::Reserved);
}

#[test]
fn replacement_registers_candidate_before_removing_current() {
    let backend = RecordingShortcutBackend::with_registered("Alt+Space+V");
    ShortcutManager::new(backend.clone(), "Alt+Space+V").replace("Control+Shift+W").unwrap();
    assert_eq!(backend.operations(), ["register:Control+Shift+W", "unregister:Alt+Space+V"]);
}
```

- [ ] **Step 2: Run shortcut tests and confirm failure**

Run: `cargo test -p vocab-capture --test shortcut`

Expected: compilation fails because shortcut parser and manager are missing.

- [ ] **Step 3: Implement parser, policy, backend trait, and duplicate-press gate**

```rust
pub trait ShortcutBackend: Send + Sync {
    fn register(&self, canonical: &str) -> Result<(), ShortcutError>;
    fn unregister(&self, canonical: &str) -> Result<(), ShortcutError>;
}

pub struct PressGate { pressed: AtomicBool }
impl PressGate {
    pub fn accept(&self, state: ShortcutState) -> bool {
        match state {
            ShortcutState::Pressed => !self.pressed.swap(true, Ordering::SeqCst),
            ShortcutState::Released => { self.pressed.store(false, Ordering::SeqCst); false }
        }
    }
}
```

- [ ] **Step 4: Write failing Svelte recorder tests**

```ts
it("records a modified key and cancels with Escape", async () => {
  render(ShortcutRecorder, { value: "Alt+Space+V" });
  await fireEvent.click(screen.getByRole("button", { name: "Record shortcut" }));
  await fireEvent.keyDown(window, { key: "w", ctrlKey: true, shiftKey: true });
  expect(screen.getByText("⌃ ⇧ W")).toBeVisible();
});
```

- [ ] **Step 5: Run UI test and confirm missing component failure**

Run: `pnpm --dir ui test -- ShortcutRecorder.test.ts`

Expected: import/component resolution fails.

- [ ] **Step 6: Implement recorder, Tauri plugin, capabilities, commands, and atomic persistence**

Initialize `tauri-plugin-global-shortcut` from Rust, react only to `Pressed`, and emit `capture-shortcut-pressed`. `replace_shortcut` must register first, save settings second, and unregister the old shortcut last; rollback the new registration if saving fails.

- [ ] **Step 7: Verify shortcut milestone**

Run: `cargo test -p vocab-capture && cargo test --workspace && pnpm check && pnpm test`

Expected: Rust and UI suites pass with no warnings.

- [ ] **Step 8: Commit shortcut milestone**

```bash
git add Cargo.toml Cargo.lock crates/capture crates/domain apps/desktop ui
git commit -m "feat(shortcut): add configurable global capture shortcut"
```

### Task 3: Swift Bridge, Accessibility Permission, and Selection Capture

**Files:**
- Create: `platform/macos/Package.swift`
- Create: `platform/macos/Sources/VocabMacBridge/Bridge.swift`
- Create: `platform/macos/Sources/VocabMacBridge/AccessibilityCapture.swift`
- Create: `platform/macos/Sources/VocabMacBridge/PermissionService.swift`
- Create: `platform/macos/Sources/VocabMacBridge/SentenceExtractor.swift`
- Create: `platform/macos/Tests/VocabMacBridgeTests/SentenceExtractorTests.swift`
- Create: `platform/macos/Tests/VocabMacBridgeTests/BridgeSerializationTests.swift`
- Create: `crates/platform/src/macos.rs`
- Create: `crates/platform/tests/macos_bridge.rs`
- Create: `apps/desktop/src-tauri/build/macos.rs`
- Modify: `apps/desktop/src-tauri/build.rs`
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Modify: `apps/desktop/src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: Task 1 `CaptureProvider`, `PermissionStatus`, `CaptureCandidate`, and geometry DTOs.
- Produces: C functions `vocab_mac_permission_status`, `vocab_mac_request_accessibility`, `vocab_mac_capture_selection`, and Rust `MacAccessibilityProvider`.

- [ ] **Step 1: Write failing Swift sentence/range tests**

```swift
@Test func extractsSentenceForUnicodeSelection() throws {
    let text = "A calm beginning. Café ☕️ brings clarity. Final thought."
    let range = try #require(text.range(of: "Café ☕️"))
    #expect(SentenceExtractor.sentence(in: text, selection: range) == "Café ☕️ brings clarity.")
}
```

- [ ] **Step 2: Run Swift tests and confirm missing implementation failure**

Run: `swift test --package-path platform/macos`

Expected: compilation fails because `SentenceExtractor` and bridge DTOs do not exist.

- [ ] **Step 3: Implement Swift package, permission adapter, Accessibility capture, and JSON ownership**

Use `AXIsProcessTrustedWithOptions`, focused application/element attributes, `kAXSelectedTextAttribute`, selected range plus `kAXValueAttribute`, and `kAXBoundsForRangeParameterizedAttribute`. Expose returned JSON through an allocated C string and a matching `vocab_mac_free_string` function.

- [ ] **Step 4: Write failing Rust bridge decoding tests**

```rust
#[test]
fn decodes_success_and_maps_ax_errors_without_content_logging() {
    let provider = MacAccessibilityProvider::with_bridge(FakeBridge::selection_fixture());
    let candidate = block_on(provider.capture_selected_text()).unwrap();
    assert_eq!(candidate.selected_text, "serendipity");
    assert_eq!(candidate.sentence, "A moment of serendipity.");
}
```

- [ ] **Step 5: Implement Rust FFI adapter and Tauri permission commands**

Add `get_permission_status`, `request_accessibility_permission`, and `open_permission_settings`. Isolate all `unsafe` FFI calls inside `crates/platform/src/macos.rs` with safe return types and ownership tests.

- [ ] **Step 6: Verify Swift/Rust integration**

Run: `swift test --package-path platform/macos && cargo test -p vocab-platform && cargo test --workspace`

Expected: Swift fixtures and Rust bridge tests pass.

- [ ] **Step 7: Commit native selection milestone**

```bash
git add platform/macos crates/platform apps/desktop Cargo.lock
git commit -m "feat(macos): add accessibility selection bridge"
```

### Task 4: Capture Coordinator and Floating Window Lifecycle

**Files:**
- Create: `crates/capture/src/coordinator.rs`
- Create: `crates/capture/src/dismissal.rs`
- Create: `crates/capture/tests/coordinator.rs`
- Create: `crates/capture/tests/dismissal.rs`
- Create: `apps/desktop/src-tauri/src/capture_runtime.rs`
- Create: `ui/src/floating/CaptureWindow.svelte`
- Create: `ui/src/floating/CaptureWindow.test.ts`
- Create: `ui/src/floating/main.ts`
- Create: `ui/floating.html`
- Modify: `apps/desktop/src-tauri/tauri.conf.json`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `ui/vite.config.ts`

**Interfaces:**
- Consumes: Tasks 1–3 providers, shortcut event, placement, and existing `AppService::capture`/`undo_capture`.
- Produces: `CaptureCoordinator::start() -> CaptureRequestId`, typed `CaptureEvent`, prewarmed Tauri window label `capture`, and floating Svelte entry point.

- [ ] **Step 1: Write failing coordinator ordering/cancellation tests**

```rust
#[test]
fn newer_request_cancels_stale_translation_and_saves_once() {
    let harness = CoordinatorHarness::new();
    let first = harness.start_selection("first");
    let second = harness.start_selection("second");
    harness.complete_translation(first, "erste");
    harness.complete_translation(second, "zweite");
    assert_eq!(harness.saved_words(), ["second"]);
    assert_eq!(harness.events_for(first).last(), Some(&CaptureEventKind::Cancelled));
}
```

- [ ] **Step 2: Run tests and confirm missing coordinator failure**

Run: `cargo test -p vocab-capture --test coordinator`

Expected: compilation fails for missing coordinator and events.

- [ ] **Step 3: Implement coordinator, request cancellation, and save-once gate**

Inject capture, translation, OCR, persistence, event sink, and clock traits. Do not call native or Tauri globals from coordinator tests.

- [ ] **Step 4: Write failing placement/dismissal and Svelte state tests**

```ts
it("shows saved state and pauses dismissal while hovered", async () => {
  const clock = new FakeClock();
  render(CaptureWindow, { event: savedEvent, clock });
  expect(screen.getByText("Saved · Undo")).toBeVisible();
  await fireEvent.mouseEnter(screen.getByRole("dialog"));
  clock.advance(5000);
  expect(screen.getByRole("dialog")).toBeVisible();
});
```

- [ ] **Step 5: Implement the dedicated Tauri window and shared floating states**

Configure `capture` as hidden, transparent, undecorated, shadowed, always-on-top, 380×220, and non-resizable. Prewarm it during setup, emit events by request ID, normalize coordinates, resize only for expanded states, and hide rather than destroy on close.

- [ ] **Step 6: Verify lifecycle milestone**

Run: `cargo test -p vocab-capture && cargo test --workspace && pnpm check && pnpm test && pnpm build`

Expected: all coordinator, dismissal, UI, and existing tests pass.

- [ ] **Step 7: Commit floating workflow**

```bash
git add crates/capture apps/desktop ui
git commit -m "feat(capture): add floating capture window lifecycle"
```

### Task 5: Apple Translation and Automatic Persistence

**Files:**
- Create: `platform/macos/Sources/VocabMacBridge/AppleTranslation.swift`
- Create: `platform/macos/Tests/VocabMacBridgeTests/TranslationAdapterTests.swift`
- Create: `crates/platform/tests/translation_contract.rs`
- Modify: `platform/macos/Sources/VocabMacBridge/Bridge.swift`
- Modify: `crates/platform/src/macos.rs`
- Modify: `crates/capture/src/coordinator.rs`
- Modify: `crates/capture/tests/coordinator.rs`
- Modify: `ui/src/floating/CaptureWindow.svelte`
- Modify: `ui/src/floating/CaptureWindow.test.ts`

**Interfaces:**
- Consumes: `TranslationProvider`, coordinator draft, configured target language, and existing transactional capture service.
- Produces: Swift C functions `vocab_mac_translation_availability`, `vocab_mac_prepare_translation`, `vocab_mac_translate`; complete loading/download/unavailable/saved event path.

- [ ] **Step 1: Write failing Swift availability mapping tests**

```swift
@Test func maps_installed_supported_and_unsupported_pairs() async throws {
    let adapter = AppleTranslationAdapter(session: FakeTranslationSession.ready("Zufall"))
    #expect(try await adapter.availability(source: "en", target: "de") == .ready)
    #expect(try await adapter.translate("serendipity", source: "en", target: "de") == "Zufall")
}
```

- [ ] **Step 2: Run Swift tests and confirm failure**

Run: `swift test --package-path platform/macos --filter TranslationAdapterTests`

Expected: compilation fails because the adapter is missing.

- [ ] **Step 3: Implement Apple Translation session reuse and status mapping**

Wrap `LanguageAvailability` and `TranslationSession`; reuse sessions per language pair, require explicit `prepare` for model downloads, and discard results when the request cancellation token is set.

- [ ] **Step 4: Add failing coordinator tests for ready, model-required, offline, unsupported, and save-without-translation paths**

Run: `cargo test -p vocab-capture --test coordinator translation`

Expected: assertions fail because translation states are not routed.

- [ ] **Step 5: Implement translation events, automatic save, and save-without-translation action**

Ensure the application capture service is invoked exactly once only after successful translation or explicit save-without-translation. Emit `Saved` with the returned `CaptureCard` so duplicate encounter and Undo behavior remain unchanged.

- [ ] **Step 6: Verify translation milestone**

Run: `swift test --package-path platform/macos && cargo test --workspace && pnpm test && pnpm check`

Expected: all tests pass without warnings.

- [ ] **Step 7: Commit translation milestone**

```bash
git add platform/macos crates/platform crates/capture ui
git commit -m "feat(macos): add on-device translation capture flow"
```

### Task 6: Screen Recording Permission and Vision OCR Fallback

**Files:**
- Create: `platform/macos/Sources/VocabMacBridge/ScreenCaptureOcr.swift`
- Create: `platform/macos/Sources/VocabMacBridge/OcrCandidateRanker.swift`
- Create: `platform/macos/Tests/VocabMacBridgeTests/OcrCandidateRankerTests.swift`
- Create: `platform/macos/Tests/VocabMacBridgeTests/Fixtures/ocr-reading.png`
- Create: `crates/platform/tests/ocr_contract.rs`
- Modify: `platform/macos/Sources/VocabMacBridge/Bridge.swift`
- Modify: `crates/platform/src/macos.rs`
- Modify: `crates/capture/src/coordinator.rs`
- Modify: `crates/capture/tests/coordinator.rs`
- Modify: `ui/src/floating/CaptureWindow.svelte`
- Modify: `ui/src/floating/CaptureWindow.test.ts`

**Interfaces:**
- Consumes: `OcrProvider`, Accessibility failure event, pointer and window-exclusion data.
- Produces: C functions `vocab_mac_screen_recording_status`, `vocab_mac_request_screen_recording`, `vocab_mac_ocr_near_pointer`, ranked `OcrCandidate` values, and explicit confirmation flow.

- [ ] **Step 1: Write failing OCR ranking tests from deterministic observations**

```swift
@Test func ranks_word_under_pointer_and_discards_low_confidence_text() {
    let result = OcrCandidateRanker.rank(observations: fixtures, pointer: CGPoint(x: 412, y: 220))
    #expect(result.first?.text == "serendipity")
    #expect(result.allSatisfy { $0.confidence >= 0.55 })
    #expect(result.count <= 5)
}
```

- [ ] **Step 2: Run Swift OCR test and confirm missing ranker failure**

Run: `swift test --package-path platform/macos --filter OcrCandidateRankerTests`

Expected: compilation fails for missing ranker.

- [ ] **Step 3: Implement permission checks, bounded ScreenCaptureKit screenshot, Vision OCR, ranking, and image disposal**

Capture a 900×420 logical-point region around the pointer, clamped to the display. Exclude both Vocab Collector windows. Configure `VNRecognizeTextRequest` for accurate recognition, English preference, automatic language detection, and language correction.

- [ ] **Step 4: Write failing coordinator/UI tests proving OCR is never automatic**

```rust
#[test]
fn accessibility_failure_offers_ocr_but_does_not_capture_until_confirmed() {
    let harness = CoordinatorHarness::accessibility_unsupported();
    harness.start();
    assert_eq!(harness.ocr_calls(), 0);
    assert!(harness.events().contains(&CaptureEventKind::OcrAvailable));
    harness.confirm_ocr();
    assert_eq!(harness.ocr_calls(), 1);
}
```

- [ ] **Step 5: Implement permission, candidate selection, retry-region, Enter/arrows/Escape, and confirmed save**

Hide the floating card during screenshot capture. Show up to five alternatives, require **Use this word** or Enter, and return `NoReadableText` with **Try another region** when no candidate reaches 0.55 confidence.

- [ ] **Step 6: Verify OCR milestone**

Run: `swift test --package-path platform/macos && cargo test --workspace && pnpm test && pnpm check && pnpm build`

Expected: all native, Rust, and UI tests pass.

- [ ] **Step 7: Commit OCR milestone**

```bash
git add platform/macos crates/platform crates/capture ui
git commit -m "feat(macos): add explicit Vision OCR fallback"
```

### Task 7: Developer Documentation, Packaging, and Release Verification

**Files:**
- Create: `docs/native-capture-development.md`
- Create: `docs/native-capture-testing.md`
- Create: `apps/desktop/src-tauri/Info.plist`
- Create: `apps/desktop/src-tauri/Entitlements.plist`
- Modify: `README.md`
- Modify: `docs/development.md`
- Modify: `docs/architecture.md`
- Modify: `apps/desktop/src-tauri/tauri.conf.json`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: all completed native providers and project commands.
- Produces: reproducible local setup, permission-testing procedure, compatibility matrix, signed/notarized packaging configuration, and CI coverage for Swift/Rust/Svelte.

- [ ] **Step 1: Write the developer setup and testing documents**

Document Xcode selection, `swift test`, Rust/UI checks, Tauri development, permission reset commands scoped to `app.vocabcollector.desktop`, manual application matrix, privacy log inspection, and release bundling. Explicitly state that end users do not need Xcode, Node.js, pnpm, or Rust.

- [ ] **Step 2: Add bundle metadata and native CI checks**

Add the minimum macOS version, permission purpose strings, required linked frameworks/resources, and CI commands:

```yaml
- run: swift test --package-path platform/macos
- run: cargo fmt --all --check
- run: cargo clippy --workspace --all-targets -- -D warnings
- run: cargo test --workspace
- run: pnpm check
- run: pnpm test
- run: pnpm build
```

- [ ] **Step 3: Run the complete automated verification suite**

Run:

```bash
swift test --package-path platform/macos
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check
pnpm test
pnpm build
pnpm tauri build -- --bundles app
```

Expected: every command exits 0; the `.app` exists under `target/release/bundle/macos/` and launches without a development server.

- [ ] **Step 4: Execute and record the manual macOS matrix**

Record results for Safari, Chrome, Firefox, Preview, Books, Mail, Notes, and a scanned PDF; Accessibility denied/granted/revoked; Screen Recording denied/granted; translation model ready/missing/offline; shortcut conflict/replacement/restart; Retina and secondary-display placement. Any unsupported case is documented rather than marked successful.

- [ ] **Step 5: Verify Git diff and privacy constraints**

Run: `git diff --check && rg -n "selected_text|sentence|source_url|screenshot" apps crates platform | rg "(info!|debug!|warn!|error!|print)"`

Expected: no whitespace errors and no logging statements containing captured content fields.

- [ ] **Step 6: Commit documentation and packaging**

```bash
git add README.md docs apps/desktop/src-tauri .github/workflows/ci.yml
git commit -m "docs: add native capture development and release guide"
```

## Final Completion Gate

- [ ] Re-read `docs/superpowers/specs/2026-08-25-macos-native-capture-design.md` and map every requirement to a completed task.
- [ ] Run the complete automated suite again from a clean working tree.
- [ ] Confirm the seven milestone commits exist in order.
- [ ] Use `superpowers:verification-before-completion` before claiming success.
- [ ] Use `superpowers:finishing-a-development-branch` and preserve the feature branch unless the user chooses another integration option.
