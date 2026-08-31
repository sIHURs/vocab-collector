# Windows Development Log

This log records implementation evidence for `docs/windows-platform-tickets.md`. Evidence uses the status vocabulary defined in `docs/windows-platform-plan-v2.md`: **Verified automated**, **Verified Windows physical**, **Not run**, **Blocked**, and **Unsupported**.

## 2026-08-31 - W-01 Windows automated baseline

### Implementation

- Published the approved tracer-bullet breakdown in `docs/windows-platform-tickets.md`.
- Established the Windows-target automated baseline without changing product code or enabling native capabilities.
- Confirmed the existing GitHub Actions Windows lane uses `windows-latest`, Rust 1.98.0, Node.js 22, and pnpm 11.19.0. It currently runs formatting, Clippy, workspace tests/build, the Windows adapter tests, and frontend check/test/build; it does not build an NSIS artifact.
- Confirmed the Windows adapter remains the static unsupported skeleton: all capability flags are false and all providers return capability-specific `PlatformError::Unsupported` results.

### Key decisions

- W-01 is an evidence/baseline ticket. It adds no core behavior, so no new TDD cycle or production test was warranted.
- Existing public seams are the baseline: Rust workspace tests, Windows adapter capability contracts, frontend typecheck/tests/build, and the desktop command contract.
- The current session runs on a Windows host, but the host has not been confirmed as a Windows 11 x64 physical machine. No runtime or manual result is classified as **Verified Windows physical**.
- Frontend commands that require the bundled `esbuild` helper were rerun outside the managed sandbox after their first attempt failed with `spawn EPERM`. The successful reruns used the exact ticket commands; the initial failures are environment restrictions rather than product failures.

### Main files changed

- `CONTEXT.md`
- `docs/adr/0001-independent-windows-presentation.md`
- `docs/windows-platform-plan-v2.md`
- `docs/windows-platform-tickets.md`
- `docs/windows-development-log.md`

The glossary, ADR, and v2 plan were approved planning inputs produced immediately before W-01 and committed with the ticket baseline. No product source, manifest, CI workflow, platform capability, or existing content in `docs/windows-development.md` was modified for W-01.

### Toolchain

| Tool | Observed version | Required/CI version | Status |
|---|---|---|---|
| `rustc` | 1.98.0 | 1.98.0 or newer | Verified automated |
| `cargo` | 1.98.0 | Rust 1.98 toolchain | Verified automated |
| Node.js | 24.19.0 | 22 in CI; 22 or newer in project docs | Verified automated; differs from CI major version |
| pnpm | 11.19.0 | 11.19.0 | Verified automated |

Baseline commit before W-01: `2844499583840bdc2d6c21f9ee36a7a8f36b53ea`.

### Tests and results

| Command | Result | Evidence |
|---|---|---|
| `cargo fmt --all --check` | PASS | Verified automated; exit code 0 |
| `cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings` | PASS | Verified automated; exit code 0 |
| `cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux` | PASS | Verified automated; 79 tests passed, independently recounted with the same package selection and `-- --list`, including domain, storage, application, capture, platform contracts, desktop command contracts, and Windows skeleton contracts |
| `cargo build --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux` | PASS | Verified automated; exit code 0 |
| `pnpm check` | PASS after sandbox-external rerun | Verified automated; `svelte-check` reported 0 errors and 0 warnings. Initial managed-sandbox attempt was Blocked by `esbuild` `spawn EPERM` |
| `pnpm test` | PASS after sandbox-external rerun | Verified automated; 4 files and 26 tests passed. Initial managed-sandbox attempt was Blocked by `esbuild` `spawn EPERM` |
| `pnpm build` | PASS after sandbox-external rerun | Verified automated; Vite transformed 119 modules and produced the production bundle. Initial managed-sandbox attempt was Blocked by `esbuild` `spawn EPERM` |
| `cargo test -p vocab-platform-windows` | PASS | Verified automated; 2 capability tests passed and confirmed that no native capability is enabled |

### Not yet verified

- **Not run:** `pnpm tauri dev`. The current session does not establish that the host is a Windows 11 x64 physical machine.
- **Not run:** main-window startup and WebView2 runtime behavior.
- **Not run:** guest SQLite initialization/restart persistence through the Tauri application.
- **Not run:** Today, Vocabulary, Review, Settings, and Manual Capture physical smoke tests.
- **Not run:** global shortcut registration and conflict behavior in a live Windows session.
- **Not run:** COM, UI Automation, OCR, permission/error recovery, capture-window activation/focus, mixed-DPI displays, tray, notifications, autostart, NSIS, upgrade, and uninstall.

The deferred runtime checks remain owned by their corresponding Hybrid or Physical tickets. No native Windows capability is claimed by W-01.

### Next ticket starting point

W-02 starts from a green automated baseline. It should centralize Windows presentation selection in the UI/desktop composition boundary, introduce a Windows-owned main and capture presentation without changing macOS page behavior, preserve the existing frontend backend contract, and omit the static Progress view. Before editing, rerun the narrow frontend and desktop command-contract baselines and review ADR 0001.
