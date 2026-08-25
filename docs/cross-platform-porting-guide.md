# Cross-Platform Capture Porting Guide

## Purpose

Vocab Collector is one shared application with small native adapters, not three separate desktop products. Rust owns vocabulary behavior, persistence, review, synchronization, and capture orchestration. Svelte owns the main and floating interfaces. Each operating system supplies only the capabilities that cannot be implemented portably: reading the current selection, obtaining its screen position, requesting permissions, capturing an OCR image, translating text, and applying native floating-window behavior.

The macOS implementation described in [`docs/superpowers/specs/2026-08-25-macos-native-capture-design.md`](superpowers/specs/2026-08-25-macos-native-capture-design.md) is the first adapter. Windows and Linux implementations must satisfy the same contracts rather than duplicating the application.

## Architectural Rule

```text
┌──────────────────────────────────────────────────────────────┐
│ Shared Svelte UI                                             │
│ Today · Vocabulary · Review · Settings · Floating card       │
├──────────────────────────────────────────────────────────────┤
│ Shared Rust application                                     │
│ Capture coordinator · Vocabulary service · Review · Sync     │
├──────────────────────────────────────────────────────────────┤
│ Shared Rust infrastructure                                  │
│ SQLite · outbox · account boundaries · typed Tauri commands  │
└────────────────────────────┬─────────────────────────────────┘
                             │ provider contracts
           ┌─────────────────┼─────────────────┐
           ▼                 ▼                 ▼
     macOS adapter      Windows adapter     Linux adapter
     Swift/Apple APIs   UI Automation       AT-SPI/portals
```

Platform code may depend on shared contracts. Shared domain and application code must never depend on Swift, Objective-C, Win32, COM, AT-SPI, X11, Wayland, or a particular translation engine.

## What Must Stay Shared

The following behavior has one implementation for every operating system:

- Word normalization and language-pair deduplication
- Encounter history and original context storage
- SQLite migrations and transactional sync outbox
- Review scheduling and review logs
- Guest and account data separation
- Capture request IDs, cancellation, and event ordering
- Accessibility-to-OCR fallback policy
- Save-once and Undo behavior
- Floating-card states, copy, layout, and keyboard controls
- Window placement algorithm after coordinates are normalized
- Four-second dismissal and pause conditions
- Settings data model and shortcut validation policy
- Supabase synchronization and conflict resolution
- Future AI enrichment interface and provenance rules

If a port needs different behavior in one of these areas, first determine whether the shared contract is incomplete. Do not copy the Rust service or Svelte component into an OS-specific directory.

## Recommended Repository Structure

```text
apps/
  desktop/
    src-tauri/                 Tauri setup, commands, events, windows

crates/
  domain/                      entities and product rules
  application/                 vocabulary and review use cases
  capture/                     cross-platform capture coordinator
  platform/                    provider traits and portable DTOs
  storage/                     SQLite and transactional outbox
  sync/                        account synchronization

platform/
  macos/
    Package.swift
    Sources/VocabMacBridge/
      AccessibilityCapture.swift
      AppleTranslation.swift
      ScreenCaptureOcr.swift
      PermissionService.swift
      NativePanel.swift
    Tests/VocabMacBridgeTests/

  windows/
    CMakeLists.txt
    src/
      UiAutomationCapture.cpp
      WindowsOcr.cpp
      PermissionService.cpp
      NativeWindow.cpp
    tests/

  linux/
    Cargo.toml
    src/
      atspi_capture.rs
      portal_screenshot.rs
      permission_service.rs
      native_window.rs
    tests/

ui/src/
  main-window/                 main application screens
  floating-window/             shared capture and review cards
  lib/backend.ts               typed Tauri/browser adapters
  lib/types.ts                 frontend view-model types
```

The exact native language is an implementation choice. Contract behavior is not.

## Shared Provider Contracts

The `vocab-platform` crate defines the portable boundary. All screen rectangles use logical coordinates with a top-left origin after adapter normalization.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureOrigin {
    Accessibility,
    Ocr,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureCandidate {
    pub selected_text: String,
    pub sentence: String,
    pub source_app: Option<String>,
    pub source_title: Option<String>,
    pub source_url: Option<String>,
    pub selection_bounds: Option<ScreenRect>,
    pub origin: CaptureOrigin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
    Restricted,
    RestartRequired,
}
```

### Selection capture

```rust
#[async_trait]
pub trait CaptureProvider: Send + Sync {
    async fn permission_status(&self) -> Result<PermissionStatus, PlatformError>;
    async fn request_permission(&self) -> Result<PermissionStatus, PlatformError>;
    async fn capture_selected_text(&self) -> Result<CaptureCandidate, PlatformError>;
}
```

`capture_selected_text` must:

- Run only after the user invokes the global shortcut.
- Return only the current selection and the smallest useful surrounding context.
- Preserve the exact selected surface form.
- Avoid changing the clipboard, typing into the source application, or moving focus.
- Return `UnsupportedElement` or `EmptySelection` when the selection is unavailable, allowing the shared coordinator to offer OCR.
- Avoid logging selected text, context, URLs, or screenshots.

### OCR

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrCandidate {
    pub text: String,
    pub line: String,
    pub bounds: ScreenRect,
    pub confidence: f32,
}

#[async_trait]
pub trait OcrProvider: Send + Sync {
    async fn permission_status(&self) -> Result<PermissionStatus, PlatformError>;
    async fn request_permission(&self) -> Result<PermissionStatus, PlatformError>;
    async fn recognize_near_pointer(
        &self,
        pointer: ScreenPoint,
    ) -> Result<Vec<OcrCandidate>, PlatformError>;
    async fn recognize_region(
        &self,
        region: ScreenRect,
    ) -> Result<Vec<OcrCandidate>, PlatformError>;
}
```

OCR must never run continuously or silently. It begins only after Accessibility capture fails and the user chooses **Use OCR instead**. The adapter captures the smallest practical region, excludes Vocab Collector windows, returns confidence-ranked candidates, and deletes image memory after recognition.

### Translation

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationAvailability {
    Ready,
    ModelDownloadRequired,
    NetworkRequired,
    Unsupported,
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn availability(
        &self,
        source_language: &str,
        target_language: &str,
    ) -> Result<TranslationAvailability, PlatformError>;

    async fn prepare(
        &self,
        source_language: &str,
        target_language: &str,
    ) -> Result<(), PlatformError>;

    async fn translate(
        &self,
        text: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<TranslationResult, PlatformError>;
}
```

The application does not assume Apple Translation exists on every platform. Each adapter can use an OS translation service, an offline library, or a separately approved provider. Provider identity must not leak into word or encounter entities.

### Native window behavior

Tauri creates and renders the shared floating WebView. A platform adapter supplies only behavior that Tauri cannot make consistent:

```rust
pub trait NativeFloatingWindow: Send + Sync {
    fn configure_non_activating(&self, window_label: &str) -> Result<(), PlatformError>;
    fn show_on_all_workspaces(&self, window_label: &str) -> Result<(), PlatformError>;
    fn exclude_from_task_switcher(&self, window_label: &str) -> Result<(), PlatformError>;
    fn pointer_position(&self) -> Result<ScreenPoint, PlatformError>;
    fn work_areas(&self) -> Result<Vec<MonitorWorkArea>, PlatformError>;
}
```

The shared Rust placement service decides where the card appears. Native adapters report normalized pointer, selection, monitor, scale-factor, and work-area data.

## Coordinate Normalization

Native APIs use incompatible coordinate systems:

- Different top-left or bottom-left origins
- Physical pixels or logical points
- Per-monitor scale factors
- Negative coordinates for monitors left or above the primary display

Each adapter converts native values into a shared virtual desktop of logical points with a top-left origin. The adapter must not place the card itself.

The shared placement algorithm then:

1. Identifies the monitor containing most of the anchor rectangle.
2. Places the card 12 logical points above the selection.
3. Places it below when insufficient space exists above.
4. Clamps the complete card to the monitor work area with an 8-point inset.
5. Uses a 16-point offset from the pointer when selection bounds are absent.
6. Keeps an expanded card on the same monitor.

Fixture tests for every adapter must demonstrate correct conversion for 100%, 125%, 150%, and 200% scaling where the OS supports those values.

## Platform Implementations

### macOS

Recommended native layer: Swift static library with a JSON C ABI.

| Capability | API |
|---|---|
| Selected text | macOS Accessibility `AXUIElement` |
| Selection bounds | `kAXBoundsForRangeParameterizedAttribute` |
| Context sentences | Natural Language `NLTokenizer` |
| Translation | Apple Translation |
| Screenshot region | ScreenCaptureKit |
| OCR | Vision `VNRecognizeTextRequest` |
| Permission status | Accessibility trust and screen-capture preflight APIs |
| Native panel behavior | AppKit `NSPanel`/`NSWindow` configuration |

Target macOS 15 or newer for the customized Apple Translation path. Ship a signed and notarized `.app` inside a `.dmg` for the first distribution channel.

### Windows

Recommended native layer: C++/WinRT or Rust with carefully isolated Windows bindings.

| Capability | Candidate API |
|---|---|
| Selected text | UI Automation `TextPattern`/`TextPattern2` |
| Selection bounds | `IUIAutomationTextRange::GetBoundingRectangles` |
| Context | enclosing document range plus shared sentence extraction |
| Translation | approved offline/provider adapter |
| Screenshot region | Windows Graphics Capture |
| OCR | Windows Media OCR or approved offline engine |
| Permission status | capability-specific Windows status and user guidance |
| Native panel behavior | Win32 extended window styles |

The adapter must initialize COM on the correct thread and convert UTF-16 ranges without splitting surrogate pairs. Windows application installers should use the existing Tauri MSI or NSIS packaging path. Do not require Microsoft Edge or a browser beyond the WebView2 runtime bundled or supplied by supported Windows versions.

### Linux

Recommended native layer: Rust, split between accessibility and desktop-session adapters.

| Capability | Candidate API |
|---|---|
| Selected text | AT-SPI2 Text and Selection interfaces |
| Selection bounds | AT-SPI character extents where supported |
| Context | AT-SPI text range plus shared sentence extraction |
| Translation | approved offline/provider adapter |
| Screenshot region | XDG Desktop Portal Screenshot API |
| OCR | Tesseract or another packaged offline engine |
| Permission status | portal result and desktop-specific guidance |
| Native panel behavior | Tauri plus X11/Wayland session adapter |

Linux must treat X11 and Wayland as distinct environments. Do not bypass Wayland security with compositor-specific automation by default. If AT-SPI does not expose a selection, offer the portal-backed OCR flow. Package targets can include AppImage, Debian, RPM, and Flatpak, but the first Linux milestone should select one tested format rather than claiming universal distribution.

## Global Shortcut Portability

The persisted shortcut uses Tauri’s canonical format, such as `Alt+Space+V`. UI formatting is platform-specific:

| Canonical | macOS | Windows/Linux |
|---|---|---|
| `Alt` | `⌥` | `Alt` |
| `Control` | `⌃` | `Ctrl` |
| `Meta` | `⌘` | `Win`/`Super` |
| `Shift` | `⇧` | `Shift` |

The shortcut manager is shared and performs atomic replacement: register the candidate, persist it, then unregister the old shortcut. The native adapter or Tauri plugin reports registration failure. A port must not silently substitute a different combination.

Default shortcuts may differ only when the original combination is impossible or conventionally reserved on that OS. Any change requires a product decision and a migration for existing settings.

## Capture Data Flow

Every adapter participates in the same workflow:

```text
User selects text
      ↓
User presses configured shortcut
      ↓
CaptureCoordinator creates request UUID
      ↓
CaptureProvider reads selection
      ├── success → context → translation → transactional save
      └── unsupported/empty → offer OCR
                                ↓ explicit confirmation
                              OCR candidates
                                ↓ user confirms
                              translation → transactional save
      ↓
Shared floating window displays Saved · Undo
      ↓
Dismiss after four seconds unless paused
```

A newer shortcut request cancels older capture, translation, and OCR results. Platform callbacks must return the request UUID so stale work cannot update the floating card.

## Error Mapping

Native errors map into portable categories:

```rust
pub enum PlatformError {
    PermissionRequired(PermissionKind),
    PermissionDenied(PermissionKind),
    RestartRequired(PermissionKind),
    UnsupportedElement,
    EmptySelection,
    InvalidSelectionRange,
    NoReadableText,
    TranslationModelRequired,
    TranslationUnavailable,
    ShortcutUnavailable,
    Cancelled,
    Operation { category: String },
}
```

Platform error strings are for diagnostics, not UI copy. The shared application maps categories to consistent card states and actions. Diagnostics may include OS version, provider name, request ID, error code, and timing. They must exclude selected text, context, translations, URLs, and screenshots.

## Adding a New Operating System

### Phase 1: Capability probe

- Confirm the supported OS versions and desktop sessions.
- Verify selected-text retrieval in the five most important reading applications.
- Verify whether selection bounds use physical or logical coordinates.
- Identify a permission-respecting screenshot API.
- Identify an on-device or explicitly approved translation provider.
- Record unsupported cases before committing to product claims.

Throwaway probes do not enter production crates.

### Phase 2: Implement providers

- Implement selection capture behind `CaptureProvider`.
- Implement permission status and explicit request actions.
- Implement coordinate conversion with fixtures.
- Implement translation availability, preparation, and execution.
- Implement screenshot OCR behind `OcrProvider`.
- Implement native floating-window behavior only where Tauri is insufficient.

Every provider must be replaceable without modifying `vocab-domain`, `vocab-application`, `vocab-storage`, or the floating Svelte components.

### Phase 3: Wire the desktop target

- Select providers with conditional compilation at the composition root.
- Register the global shortcut during Tauri setup.
- Preserve manual Quick Capture as a recovery path.
- Include required native libraries and permission metadata in the installer.
- Confirm the production app runs without Node.js, pnpm, Rust, or development tools installed.

### Phase 4: Compatibility and release

- Run the automated contract suites.
- Execute the OS-specific application matrix.
- Verify installer signing and update behavior.
- Test permission denial, revocation, and recovery.
- Test mixed-scale multi-monitor placement.
- Verify no private reading content appears in logs or crash reports.
- Document known unsupported applications and desktop environments.

## Required Contract Tests

Each operating-system adapter must pass a shared provider test suite with fakes or fixtures:

1. A selected word preserves exact casing and punctuation.
2. A selected phrase remains a phrase rather than being reduced to one token.
3. Unicode ranges do not split composed characters or surrogate pairs.
4. Context contains the selected text when source context is available.
5. Empty selections return `EmptySelection`.
6. Unsupported controls return `UnsupportedElement` and enable OCR fallback.
7. Permission denial does not invoke screenshot or translation work.
8. OCR does not start without explicit user confirmation.
9. OCR candidates are confidence-ranked and include bounds.
10. Native coordinates normalize correctly on mixed-scale, negative-coordinate displays.
11. A cancelled request cannot emit a saved event.
12. Provider logs contain no captured content.

The shared end-to-end suite must additionally verify duplicate encounters, automatic save, Undo, translation-unavailable saving, and four-second conditional dismissal.

## Port Acceptance Criteria

A new operating-system port is ready for preview when:

- The installable application launches without a development server.
- The global shortcut persists and can be replaced safely.
- Selected text works in the agreed application matrix.
- Failure to access selected text offers explicit OCR rather than silently capturing the screen.
- The floating card appears on the correct monitor without stealing focus during passive states.
- Translation success, missing model/provider, offline, and unsupported-pair states are represented honestly.
- Capture and review continue without an account or network connection.
- The existing Rust, Svelte, storage, and sync tests pass without OS-specific forks.
- Native bridge memory, thread, cancellation, and privacy behavior has dedicated tests.
- The installer is signed according to platform conventions and upgrade behavior is tested.

## Anti-Patterns

Do not:

- Reimplement vocabulary or review behavior inside a native wrapper.
- Give Swift, C++, or Linux accessibility code direct SQLite access.
- Let the Svelte UI call native selection APIs directly.
- Use clipboard mutation as the normal selection path.
- Poll or monitor the active screen before the shortcut is pressed.
- Run OCR without explicit user confirmation.
- Make screenshots, selected text, or context part of telemetry.
- Place windows using unnormalized native coordinates.
- Fork the floating-card UI by operating system unless a documented native limitation requires it.
- Claim application compatibility based only on the accessibility API existing; test real applications.

## Maintenance Rule

When a platform reveals a missing capability, evolve the smallest shared contract that describes the behavior. Implement the contract for existing supported platforms before merging the new port. This keeps Vocab Collector one product with multiple adapters rather than multiple products that gradually diverge.
