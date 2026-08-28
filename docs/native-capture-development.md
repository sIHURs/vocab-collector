# macOS native capture development

## Scope and requirements

The native capture feature targets macOS 15 or newer and requires Xcode 16+, Swift 6, Rust 1.98+, Node.js 22+, and pnpm 11+. Xcode does not need to remain open: Cargo invokes SwiftPM through `swift-rs` during Tauri builds.

The default shortcut is `Alt+Shift+V`, displayed as `⌥ ⇧ V`. Users can record another accelerator in Settings. Accelerators require at least one modifier and one ordinary key.

## Architecture

- `crates/platform-api` owns portable capture, permission, translation, OCR, and geometry DTOs, typed `PlatformError` values, capability reporting, and the small `SelectionProvider`, `OcrProvider`, `TranslationProvider`, `PermissionProvider`, and `WindowProvider` traits. `PlatformServices` is the injected provider bundle.
- `crates/capture` owns the single capture coordinator, stale-result rejection, save-once gate, shortcut policy, repeat suppression, and deterministic multi-monitor placement.
- `crates/application::PlatformCaptureWorkflow` combines that coordinator with shared application persistence and the injected providers. It is the one product workflow for both Accessibility and explicitly confirmed OCR candidates.
- `platform/macos/native` is a static Swift package. It contains Accessibility selection capture, sentence extraction, Apple Translation, ScreenCaptureKit screenshots, Vision OCR, and permission calls.
- `platform/macos/rust` is the only Rust crate containing native FFI. It immediately copies and frees bridge JSON and exposes safe Rust results.
- `apps/desktop/src-tauri/src/bootstrap.rs` selects the target adapter and injects its `PlatformServices`; `commands/` and `events.rs` expose a target-independent command/event surface. `lib.rs` performs only Tauri setup, shortcut registration, and composition.
- `ui/src/FloatingCapture.svelte` is the shared capture card. The Tauri `capture` window is independent from the main window, transparent, always on top, and hidden by default.

No captured text, translation, URL, context, OCR image, or screenshot is written to logs. OCR screenshots live only in memory for the duration of the Vision request.

## Runtime flow

1. The global-shortcut plugin receives a `Pressed` transition for the persisted accelerator.
2. `PlatformCaptureWorkflow` uses the injected selection provider. The macOS Rust adapter calls Swift, which queries the focused Accessibility element for selected text, context, range bounds, application metadata, and document URL.
3. The same coordinator admits the candidate and rejects stale request UUIDs. Shared Rust chooses the monitor, places the 380×280 card, and publishes the typed event only while the request is current.
4. The capture UI invokes the target-independent translation command. The workflow authorizes the transition before calling the injected provider and accepts the result only while the same request remains current.
5. The coordinator applies the save-once gate, then the application service stores the encounter, origin, and available translation in one SQLite transaction.
6. The card dismisses after four seconds. Hover pauses the timer; Undo soft-deletes the new encounter.
7. When Accessibility cannot obtain a selection, the card offers OCR only when the reported capability permits it. OCR capture runs after explicit user action and Screen Recording approval; the suggested candidate must then be explicitly confirmed before translation or save. Swift converts Vision bounds into the shared primary-top-left logical coordinate convention.

If an Apple language pack is unavailable or translation fails, the card offers an explicit **Save without translation** action. A later enrichment/sync worker can fill the translation without changing the capture boundary.

Commands return a stable `CaptureFailure` object. Its `code` is one of
`permission_required`, `permission_denied`, `empty_selection`,
`unsupported_element`, `translation_unavailable`, `translation_failed`,
`cancelled`, or `operation`; UI copy does not branch on native Swift strings
or an OS name.

## Permissions

Accessibility is required to read selections from other applications. Screen Recording is required only for OCR. During development, grant permissions to the built `Vocab Collector` executable in:

- System Settings → Privacy & Security → Accessibility
- System Settings → Privacy & Security → Screen & System Audio Recording

After rebuilding with a different signing identity or executable path, macOS may treat it as a new application. Remove the old entry, relaunch, and grant it again.

## Local commands

```bash
pnpm install
pnpm tauri dev
```

The browser-only `pnpm dev` mode retains manual Quick Capture but cannot use system selection, native translation, or OCR.

Focused verification:

```bash
swift test --package-path platform/macos/native
cargo test -p vocab-platform-macos
pnpm check
pnpm test
```

Full verification and a local application bundle:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --exclude vocab-platform-linux --exclude vocab-platform-windows -- -D warnings
cargo test --workspace --exclude vocab-platform-linux --exclude vocab-platform-windows
cargo build --workspace --exclude vocab-platform-linux --exclude vocab-platform-windows
pnpm check
pnpm test
pnpm build
pnpm tauri build --bundles app
```

The `.app` output is created under `target/release/bundle/macos/`. Distribution to other Macs additionally requires an Apple Developer signing identity and notarization.

## Porting boundary

Windows and Linux replace only the provider implementations selected in the
desktop composition root. Their Plan A crates are static skeletons with all
capabilities set to `false` and typed `Unsupported` results; they have not been
compiled or run on this Mac. Plan B implements and verifies each native adapter
on its physical target machine. See `docs/cross-platform-porting-guide.md`.
