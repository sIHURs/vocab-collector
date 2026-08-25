# macOS Native Capture and Floating Window Design

## Purpose

Add the missing system-wide reading workflow to Vocab Collector on macOS. A user selects an English word in another application, presses a configurable global shortcut, and receives a compact floating translation card near the selection. The app stores the word and surrounding context without requiring the main window or interrupting reading. Accessibility capture is the primary path; on-device OCR is the explicit fallback.

## Scope

This increment delivers:

1. A global shortcut with default `Alt+Space+V`, displayed as `⌥ Space V`, and a recorder in Settings.
2. Accessibility permission status, prompting, System Settings navigation, and retry.
3. Selection capture from the focused macOS application using Accessibility APIs.
4. A separate, prewarmed, transparent, always-on-top Tauri WebView window for capture states.
5. Context extraction, source metadata, language analysis, and Apple Translation.
6. Placement next to the selection or pointer with multi-display clamping.
7. Automatic saving and conditional four-second dismissal.
8. Screen Recording permission and region OCR fallback using ScreenCaptureKit and Vision.

This increment does not add Windows/Linux capture providers, cloud translation, AI explanations, browser extensions, automatic continuous screen monitoring, or background screenshots. OCR begins only after a shortcut-triggered Accessibility failure and an explicit user action.

## Platform and Version Requirements

- Deployment target: macOS 15 or newer, required for the customized Apple Translation workflow.
- Development: Xcode 16 or newer and the existing Rust, Node.js, pnpm, Tauri 2, and Svelte 5 toolchain.
- Translation, OCR, language detection, and capture processing remain on device.
- Accessibility permission is requested only in response to onboarding or an explicit user action.
- Screen Recording permission is requested only when the user chooses OCR fallback.
- The main vocabulary database remains available without either permission.

## Architecture

```text
Global shortcut
      │
      ▼
Rust CaptureCoordinator ──────────────────────────────────────┐
      │                                                       │
      ├── Swift AccessibilityCaptureProvider                  │
      │      ├── focused application + focused element        │
      │      ├── selected text and selected range             │
      │      ├── surrounding value text                       │
      │      └── kAXBoundsForRangeParameterizedAttribute      │
      │                                                       │
      ├── Swift OcrCaptureProvider (explicit fallback)        │
      │      ├── ScreenCaptureKit region image                │
      │      └── Vision text observations + bounding boxes    │
      │                                                       │
      ├── Swift AppleTranslationProvider                      │
      ├── Rust application capture/save service               │
      └── Tauri FloatingWindowController ──events──> Svelte UI│
                                                              │
Settings <──typed commands── ShortcutManager + PermissionService
```

The Swift bridge exposes a narrow C ABI and serializes request/response payloads as UTF-8 JSON. Rust owns orchestration and cancellation, so macOS details do not enter domain or storage crates. The platform bridge performs calls on the required macOS actor or dispatch queue and never retains pointers owned by Rust.

## Core Contracts

The platform-neutral capture contract becomes:

```rust
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub enum CaptureOrigin {
    Accessibility,
    Ocr,
}

pub struct CaptureCandidate {
    pub selected_text: String,
    pub sentence: String,
    pub source_app: Option<String>,
    pub source_title: Option<String>,
    pub source_url: Option<String>,
    pub selection_bounds: Option<ScreenRect>,
    pub origin: CaptureOrigin,
}

pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
    Restricted,
}
```

`CaptureCoordinator::capture_selection()` returns a stream of `CaptureEvent` values rather than blocking until translation finishes:

```rust
pub enum CaptureEvent {
    Loading { request_id: Uuid, anchor: ScreenRect },
    PermissionRequired { request_id: Uuid, permission: PermissionKind },
    OcrAvailable { request_id: Uuid, anchor: ScreenRect },
    OcrCandidates { request_id: Uuid, candidates: Vec<OcrCandidate> },
    TranslationReady { request_id: Uuid, draft: CaptureDraft },
    Saved { request_id: Uuid, card: CaptureCard },
    Failed { request_id: Uuid, error: CaptureFailure },
}
```

Every shortcut press creates a request UUID. Events with a superseded request UUID are ignored, preventing stale translation or OCR results from replacing a newer capture card.

## Global Shortcut

Tauri’s official global-shortcut plugin registers the shortcut from Rust during application setup. The persisted setting uses the plugin’s canonical syntax, `Alt+Space+V`; the UI separately renders macOS symbols.

Settings replaces the plain text shortcut input with a recorder:

1. Click **Record shortcut**.
2. The control listens for one non-modifier key plus one or more modifiers.
3. Pressing only modifiers is ignored.
4. `Escape` cancels recording.
5. The app rejects shortcuts without a modifier, reserved application commands, and the current review/settings commands.
6. Rust attempts to register the candidate before unregistering the current shortcut.
7. On success, Rust persists the new value and unregisters the old value.
8. On conflict or registration failure, the existing shortcut remains active and the UI displays **Shortcut unavailable. Try another combination.**

The handler reacts only to the `Pressed` transition and suppresses a repeated key-down until the matching release. Shortcut changes take effect immediately without restarting the application.

## Accessibility Capture

The Swift provider follows this sequence:

1. Call `AXIsProcessTrustedWithOptions` without prompting to obtain status.
2. Resolve the system-wide focused application and focused UI element.
3. Read `kAXSelectedTextAttribute`. If absent, read `kAXSelectedTextRangeAttribute` and slice the element’s `kAXValueAttribute` by character range.
4. Preserve the exact selected surface form, normalize whitespace only in context, and reject empty or whitespace-only selections.
5. Read the full accessible value and extract the sentence containing the selected range. Sentence boundaries use `NLTokenizer(unit: .sentence)`; when the full value is unavailable, context equals the selected text.
6. Read `kAXBoundsForRangeParameterizedAttribute` for the selected range. If unavailable, return no bounds and use pointer placement.
7. Resolve the frontmost application name and active window title. Capture a URL only when the application exposes one through Accessibility; absence is not an error.

The capture must not synthesize Copy, modify the clipboard, type into the source application, or move focus. Safari, Chrome, Firefox, Preview, Books, Mail, Notes, and common PDF viewers are included in manual compatibility testing, but unsupported controls fail cleanly into OCR.

## Permission Experience

The Settings permissions section displays Accessibility and Screen Recording independently as **Granted**, **Not granted**, or **Restart required**.

For Accessibility:

- **Enable Accessibility** calls `AXIsProcessTrustedWithOptions` with the prompt option once.
- The floating card explains why capture cannot continue and offers **Open System Settings** and **Retry**.
- The app polls only while its permission instruction UI is visible, at one-second intervals, and stops after 60 seconds or when the UI closes.
- A newly granted permission is detected without requiring restart where macOS permits it.

For Screen Recording:

- No prompt occurs during startup or ordinary Accessibility capture.
- Choosing **Use OCR instead** calls the preflight API, explains the screenshot scope, and then requests access.
- If macOS requires restart, the app preserves the failed capture intent only until shutdown and shows **Restart Vocab Collector**.

## OCR Fallback

OCR is available when Accessibility returns unsupported element, empty selection, missing selected-text attributes, or invalid range. It is not silently executed.

1. The floating error card says **This app doesn’t expose the selected text** and offers **Use OCR instead**.
2. After permission succeeds, the app hides its own floating window and captures a region centered on the last pointer position. The initial region is 900×420 logical points, clamped to the active display.
3. ScreenCaptureKit captures only that display region and excludes the Vocab Collector windows.
4. Vision runs `VNRecognizeTextRequest` with accurate recognition, automatic language detection, language correction, and English preferred.
5. Observations below 0.55 confidence are discarded. Remaining lines retain bounding rectangles.
6. The provider selects the word whose bounding rectangle contains or is nearest to the pointer. It returns up to five alternatives from the same line.
7. The floating card displays the best candidate, its line as context, and **Use this word**. Arrow keys move through alternatives; Enter confirms; Escape cancels.
8. Only the confirmed candidate is translated and saved. The encounter records `origin = ocr` in the sync payload metadata while preserving the existing encounter schema in this increment.

If no credible text is found, the card says **No readable text found** and offers **Try another region** or **Close**. Retry uses a crosshair region selector and never captures continuously.

## Translation and Language Handling

English remains the configured source language for this version. Natural Language detects the selected text’s dominant language when sufficient text is available. A confident non-English selection produces **English capture only for now** and remains unsaved unless the user edits it manually in the main application.

Apple Translation checks the configured pair with `LanguageAvailability` and uses a reusable `TranslationSession` for requests of the same pair. The floating card enters a loading state immediately, before translation completes.

- Installed/supported model: translate and auto-save.
- Supported but model download required: show **Download German translation model** and start preparation only after the user clicks.
- Unsupported pair: show original word and context with **Translation unavailable** and a **Save without translation** action.
- Offline while a model is missing: show **Translation model needs a connection** without losing the capture candidate.
- Cancellation or a newer shortcut request: discard the obsolete response.

The app stores translations only after the provider identifies a successful response or the user explicitly saves without one.

## Floating Window

The `capture` Tauri window is created during setup with these properties:

- label `capture`
- 380×220 default size and 380×280 expanded size
- transparent, undecorated, shadowed, hidden at startup
- always on top and visible on all Spaces
- excluded from the Dock and window switcher where supported
- not resizable by the user
- initially non-activating so showing a loading or saved card does not steal focus from the reading application

The window uses the approved Inkdrop-inspired palette and existing component styling. UI states are:

1. Loading selection
2. Translating
3. Saved with Undo
4. Existing word/new encounter
5. Accessibility permission required
6. OCR offered
7. OCR candidate confirmation
8. Translation model download required
9. Translation unavailable/save without translation
10. Recoverable error

Keyboard input is enabled only for interactive states. When interaction is required, clicking the card activates it; the original application is not reactivated until dismissal.

## Placement

The Swift bridge returns Accessibility bounds in global screen pixels with a top-left coordinate convention. Rust normalizes them into Tauri logical coordinates using the target monitor’s scale factor.

Placement rules:

1. Prefer 12 logical pixels above the selection, horizontally centered.
2. If insufficient room above, place 12 pixels below.
3. Clamp the complete card rectangle to the active monitor’s visible work area with an 8-pixel inset.
4. If selection bounds are absent, place the card 16 pixels down and right from the pointer; flip left/up when close to an edge.
5. Recalculate for the expanded card without moving it to another display.

Unit tests cover top/bottom flipping, every screen edge, negative display coordinates, Retina scale factors, and an anchor spanning two displays.

## Saving, Undo, and Dismissal

Successful translation calls the existing application capture service exactly once and displays the returned `CaptureCard`. Duplicate words therefore append encounters using the existing transaction and outbox behavior.

The four-second dismissal timer begins only after `Saved` is rendered. It pauses while the window is hovered, keyboard-focused, expanded, downloading a model, confirming OCR, or showing an error. It resumes with the remaining duration when the pause condition ends. `Escape` closes immediately in every state except while the operating-system permission prompt owns focus. Undo calls the existing `undo_capture` command before closing.

Closing the floating window cancels in-flight translation/OCR work for that request and hides the window; it does not destroy the prewarmed WebView.

## Error Model

Errors are typed and mapped to user-facing actions:

```rust
pub enum CaptureFailure {
    AccessibilityPermissionDenied,
    ScreenRecordingPermissionDenied,
    UnsupportedElement,
    EmptySelection,
    InvalidSelectionRange,
    OcrNoText,
    OcrCaptureFailed,
    TranslationModelRequired,
    TranslationUnavailable,
    ShortcutUnavailable,
    Storage(String),
    Platform(String),
}
```

Permission and model errors remain visible until action or dismissal. Transient platform failures offer **Try again**. Storage failures never claim the word was saved. Logs include request IDs and error categories but never selected text, context, translations, screenshots, or URLs.

## Testing Strategy

### Rust

- Shortcut parsing, validation, atomic replacement, duplicate key-down suppression, and startup registration.
- Capture coordinator event ordering, request cancellation, Accessibility-to-OCR routing, and save-once behavior using fake providers.
- Placement across monitor layouts and scale factors.
- Dismissal timer pause/resume rules using a fake clock.
- Existing application and SQLite tests remain unchanged and green.

### Swift

- Sentence extraction and range conversion with Unicode, emoji, composed characters, and multiline selections.
- JSON C-ABI serialization and memory ownership.
- Accessibility error mapping using protocol-wrapped system calls.
- Vision candidate ranking from fixture images without taking real screenshots.
- Translation status mapping behind a protocol-wrapped adapter.

### Svelte

- Every floating state renders approved copy and controls.
- OCR candidate keyboard selection.
- Shortcut recorder cancellation, validation, conflict, and successful replacement.
- Permission cards and retry behavior.
- Dismissal pause on hover/focus and Undo.

### Manual macOS Matrix

- Safari, Chrome, Firefox, Preview, Books, Mail, Notes, and a scanned PDF.
- Standard and Retina displays, secondary display left of primary, and display scale changes.
- Accessibility denied/granted/revoked while running.
- Screen Recording denied/granted/restart-required.
- Translation model installed/missing, offline, and unsupported pair.
- Shortcut conflict, replacement, restart persistence, and rapid repeated presses.

## Delivery Sequence

1. Platform contracts, Swift build integration, and fixture-driven bridge tests.
2. Configurable global shortcut and Settings recorder.
3. Accessibility permission and selection/context capture.
4. Floating Tauri window, event protocol, placement, and dismissal.
5. Apple Translation states and automatic persistence.
6. Screen Recording permission, screenshot region capture, Vision OCR, and confirmation UI.
7. Compatibility matrix, privacy verification, packaging, and documentation.

Each delivery is independently committed and leaves the existing manual Quick Capture workflow operational as a recovery path.
