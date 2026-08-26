# macOS MVP baseline

Recorded on 2026-08-26 before the Plan A platform encapsulation refactor. This
record is the regression baseline for the `macos-mvp-v0.1.0` tag.

## Environment

| Tool | Version / result |
| --- | --- |
| Rust | `rustc 1.98.0 (88d9e12ae 2026-08-18)` |
| Cargo | `cargo 1.98.0 (797e8a9bc 2026-08-05)` |
| Node.js | `v24.19.0` |
| pnpm | `11.19.0` |
| Swift | `swift-driver version: 1.148.6 Apple Swift version 6.3.1 (swiftlang-6.3.1.1.2 clang-2100.0.123.102)`; target `arm64-apple-macosx26.0` |
| Xcode | `Xcode 26.4.1`, build `17E202` |
| macOS | `macOS 26.5` (`25F71`) |

The baseline worktree was clean (`## plan-a-macos-platform-encapsulation`) before
this record was added.

## Automated baseline

| Command | Result |
| --- | --- |
| `swift test --package-path platform/macos` | PASS |
| `cargo fmt --all --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS (34 integration/unit tests; all doc tests pass) |
| `pnpm check` | PASS: `svelte-check` found 0 errors and 0 warnings |
| `pnpm test` | PASS: 4 test files and 8 tests pass |
| `pnpm build` | PASS: Vite production bundle built |
| `pnpm tauri build --bundles app` | PASS: built `target/release/bundle/macos/Vocab Collector.app` |

Swift and Cargo commands were run with access to the existing SwiftPM and Clang
user caches. No repository cache location was changed.

The frontend and Tauri commands used the bundled Codex Node runtime on `PATH`.
The Tauri invocation also needed the pre-existing Cargo bin directory on `PATH`
so that Tauri could invoke Cargo; no repository configuration was changed.

## Permissions setup

Grant the built `Vocab Collector` executable these permissions in System
Settings → Privacy & Security:

- Accessibility, required to read selections in other applications.
- Screen & System Audio Recording, requested only when the user explicitly
  chooses the OCR fallback.

After a rebuild under another signing identity or executable path, remove the
old System Settings entry, relaunch the app, and grant the permission again.

## Manual macOS regression checklist

Not run for this command-line baseline; re-run and mark each item during a
signed application smoke test.

- [ ] Safari selection capture
- [ ] Chrome selection capture
- [ ] Firefox selection capture
- [ ] Preview selection capture
- [ ] Books selection capture
- [ ] Shortcut replacement and persistence
- [ ] OCR requires explicit confirmation
- [ ] Translation failure offers Save without translation
- [ ] Undo removes only the new encounter
- [ ] Capture card dismissal and hover/focus pause
- [ ] Primary and secondary-monitor card placement

## Known defects / constraints

- Manual app-level checks have not been performed in this baseline. Safari,
  Chrome, Firefox, Preview, Books, permission recovery, OCR confirmation,
  translation failure, Undo, dismissal, and multi-monitor placement remain
  explicitly unverified.
