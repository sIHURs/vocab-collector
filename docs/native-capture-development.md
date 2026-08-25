# macOS native capture development

## Scope and requirements

The native capture feature targets macOS 15 or newer and requires Xcode 16+, Swift 6, Rust 1.98+, Node.js 22+, and pnpm 11+. Xcode does not need to remain open: Cargo invokes SwiftPM through `swift-rs` during Tauri builds.

The default shortcut is `Alt+Shift+V`, displayed as `⌥ ⇧ V`. Users can record another accelerator in Settings. Accelerators require at least one modifier and one ordinary key.

## Architecture

- `crates/platform` owns portable capture, permission, translation, OCR, and geometry DTOs.
- `crates/capture` owns shortcut policy, repeat suppression, and deterministic multi-monitor placement.
- `platform/macos` is a static Swift package. It contains Accessibility selection capture, sentence extraction, Apple Translation, ScreenCaptureKit screenshots, Vision OCR, and permission calls.
- `apps/desktop/src-tauri/src/macos_bridge.rs` is the only Rust file containing native FFI. It immediately copies and frees bridge JSON and exposes safe Rust results.
- `apps/desktop/src-tauri/src/lib.rs` composes providers, registers the shortcut, positions the capture window, and exposes narrow commands.
- `ui/src/FloatingCapture.svelte` is the shared capture card. The Tauri `capture` window is independent from the main window, transparent, always on top, and hidden by default.

No captured text, translation, URL, context, OCR image, or screenshot is written to logs. OCR screenshots live only in memory for the duration of the Vision request.

## Runtime flow

1. The global-shortcut plugin receives a `Pressed` transition for the persisted accelerator.
2. Swift queries the focused Accessibility element for selected text, full text, range bounds, application metadata, and document URL.
3. Rust chooses the correct monitor, places the 380×280 card above the selection when possible, and shows the hidden capture window.
4. The capture UI asks the local Apple Translation framework for the configured language pair.
5. The encounter and available translation are stored through the existing application service and SQLite transaction.
6. The card dismisses after four seconds. Hover pauses the timer; Undo soft-deletes the new encounter.
7. When Accessibility cannot obtain a selection, the card offers OCR. OCR runs only after explicit user action and Screen Recording approval, then selects the Vision text observation nearest the pointer.

If an Apple language pack is unavailable or translation fails, the encounter is still saved with `Translation pending`. A later enrichment/sync worker can fill the translation without changing the capture boundary.

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
swift test --package-path platform/macos
cargo test -p vocab-capture
cargo test -p vocab-desktop --lib
pnpm check
pnpm test
```

Full verification and a local application bundle:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check
pnpm test
pnpm build
pnpm tauri build --bundles app
```

The `.app` output is created under `target/release/bundle/macos/`. Distribution to other Macs additionally requires an Apple Developer signing identity and notarization.

## Porting boundary

Windows and Linux implementations replace only the native selection, permission, OCR, and window adapters. They reuse the capture coordinator contracts, placement tests, shortcut policy, Svelte card, application service, and storage. See `docs/cross-platform-porting-guide.md` for the per-OS adapter checklist.
