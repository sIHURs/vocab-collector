# macOS MVP baseline

Recorded on 2026-08-26 before the Plan A platform encapsulation refactor. This
record is the regression baseline for the `macos-mvp-v0.1.0` tag.

## Baseline references

- `macos-mvp-v0.1.0` is the one canonical, immutable handoff baseline tag for
  this record.
- An earlier local alias using this spelling was moved during verification setup
  and discarded before the canonical handoff reference was established. The
  temporary `macos-mvp-v0.1.0-baseline` tag is retired at that point.
- On 2026-08-27 the repository's primary branch was named `main`. A subsequent
  `git filter-repo --force --path platform/macos/.build --invert-paths` removed
  committed SwiftPM build output from repository history and necessarily
  rewrote commit object IDs. After that rewrite, the canonical
  `macos-mvp-v0.1.0` tag resolves to
  `b06857e27b5a0ee96a74e0389d624c26bd999ae3`; pre-rewrite hashes are not valid
  handoff identifiers.

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

## Plan A handoff verification

Run on 2026-08-28 after macOS adapter encapsulation, final review fixes, and
the history rewrite.
Linux and Windows adapter packages and targets were not compiled, checked,
tested, or run on this Mac. `cargo fmt --all --check` is a static formatting
check only; every other Rust workspace command explicitly excluded
`vocab-platform-linux` and `vocab-platform-windows`.

| Command | Result |
| --- | --- |
| `swift test --package-path platform/macos/native` | PASS: 9 Swift tests |
| `cargo fmt --all --check` | PASS |
| `cargo clippy --workspace --all-targets --exclude vocab-platform-linux --exclude vocab-platform-windows -- -D warnings` | PASS |
| `cargo test --workspace --exclude vocab-platform-linux --exclude vocab-platform-windows` | PASS: 83 Rust tests; all doc tests pass |
| `cargo build --workspace --exclude vocab-platform-linux --exclude vocab-platform-windows` | PASS |
| `pnpm check` | PASS: 0 errors and 0 warnings |
| `pnpm test` | PASS: 4 test files and 26 tests |
| `pnpm build` | PASS: Vite production bundle built |
| `pnpm tauri build --bundles app` | PASS: built `target/release/bundle/macos/Vocab Collector.app` |

The Swift and Rust commands required access to the existing SwiftPM and Clang
user caches. Frontend and Tauri commands used the bundled Codex Node runtime
and the existing Cargo bin directory on `PATH`. Vite reported that no explicit
Svelte configuration file exists and used its default configuration; checks,
tests, frontend build, and bundle build still exited successfully.

The fresh local bundle contains a valid `Info.plist`, an executable arm64
Mach-O at `Contents/MacOS/vocab-desktop`, and the required `/usr/lib/swift`
runtime search path. This command-line gate did not distribution-sign, notarize,
launch, or interact with the bundle; signing and notarization remain separate
release steps.

The transferable backup `vocab-collector-app-plan-a.bundle` was created from
the final Plan A branch and verified with `git bundle verify`. It contains the
`plan-a-macos-platform-encapsulation` branch and the `macos-mvp-v0.1.0` tag.
Record its SHA-256 alongside the final handoff because regenerating the bundle
after a later commit necessarily changes that checksum.

Focused regressions now cover OCR result bounds on the primary display and on
secondary displays above, below, and left of a nonzero primary origin. Rust FFI
tests also cover copying and freeing each non-null native bridge string exactly
once before returning either a successful or invalid-JSON result, while null
returns typed no-data without invoking the free operation. These automated
fixtures do not replace the interactive multi-monitor or permission checks.

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
- [ ] Accessibility permission denial, grant, revocation, and recovery
- [ ] Screen Recording permission denial, grant, revocation, and recovery

## Known defects / constraints

- Manual app-level checks have not been performed in this baseline. Safari,
  Chrome, Firefox, Preview, Books, permission recovery, OCR confirmation,
  translation failure, Undo, dismissal, and multi-monitor placement remain
  explicitly unverified.
- The Plan A gate did not launch or interact with the signed application, so all
  checklist items above remain unchecked even though the automated and bundle
  gates pass.
