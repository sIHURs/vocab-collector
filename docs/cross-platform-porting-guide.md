# Cross-platform adapter porting guide

## Current handoff state

Vocab Collector has one shared Rust application and one shared Svelte UI. The
macOS adapter is implemented under `platform/macos`; `platform/linux` and
`platform/windows` are static Plan B skeletons. The skeletons report every
capability as unavailable and return typed `PlatformError::Unsupported`
results. They were not compiled, tested, or run on macOS. Runtime support is
established only by the target-machine work in `docs/linux-development.md` and
`docs/windows-development.md`.

## Dependency direction

```text
Svelte UI
    |
target-independent Tauri commands and typed events
    |
crates/application::PlatformCaptureWorkflow
    |                         |
crates/capture coordinator   crates/platform-api provider traits
    |                         ^
domain + storage              |
                              +-- platform/macos/rust -> platform/macos/native
                              +-- platform/linux      (Plan B skeleton)
                              `-- platform/windows    (Plan B skeleton)
```

Shared crates never depend on an OS adapter. Target-specific Cargo dependency
tables and `apps/desktop/src-tauri/src/bootstrap.rs` select exactly one adapter.
An adapter may normalize native data and implement native operations, but it
must not own SQLite, vocabulary normalization, review, save-once, Undo,
stale-result rejection, or UI policy.

## Portable boundary

`crates/platform-api` contains portable DTOs, `PlatformCapabilities`,
`PlatformError`, and five small traits:

- `SelectionProvider::capture_selection`
- `OcrProvider::recognize_near`
- `TranslationProvider::translate`
- `PermissionProvider::{status, request}`
- `WindowProvider::configure_capture_window`

`PlatformServices` bundles trait objects for those providers plus the reported
capabilities. The UI asks for capabilities and uses the same commands on every
target; it does not test an OS name.

The present `PlatformError` variants are:

- `PermissionRequired(PermissionKind)` and `PermissionDenied(PermissionKind)`
- `EmptySelection`, `UnsupportedElement`, and `InvalidSelectionRange`
- `Unsupported(Capability)` and `Cancelled`
- `Operation(String)` for a content-free adapter diagnostic

The desktop boundary maps these into stable capture failure codes:
`permission_required`, `permission_denied`, `empty_selection`,
`unsupported_element`, `translation_unavailable`, `translation_failed`,
`cancelled`, and `operation`. Do not expose captured text, context,
translations, URLs, screenshots, or raw native bridge payloads in errors or
logs.

## Shared capture workflow

There is one request state machine in `crates/capture::CaptureCoordinator`,
used by `crates/application::PlatformCaptureWorkflow`:

1. A shortcut starts a request UUID; a newer request makes older work stale.
2. The selection provider returns a portable `CaptureCandidate`.
3. An Accessibility candidate can enter translation. An OCR candidate remains
   blocked until explicit confirmation.
4. Translation success becomes ready-to-save. Translation failure permits only
   the explicit save-without-translation transition.
5. The coordinator serializes publication and persistence, rejects stale
   completion, and admits a request to storage once.
6. The shared application service persists the encounter. The shared UI owns
   presentation, Undo, and conditional dismissal.

Provider implementations return data and errors to this workflow; they do not
create a second coordinator or copy product behavior.

## Coordinates

The portable `ScreenPoint`, `ScreenRect`, and `MonitorWorkArea` convention is
logical units in one virtual-desktop coordinate space with the primary
display's top-left as origin. Negative X/Y values are valid for displays left
of or above the primary display. The current macOS OCR call is normalized at
the composition/adapter boundary: the Tauri point is converted to Cocoa before
Swift receives it, and Swift converts recognized bounds back to the portable
primary-top-left convention before returning them.

Normalize native pointer, selection/OCR bounds, monitor origins, work areas,
and scale factors at the adapter/composition boundary. After normalization,
`crates/capture` alone chooses and clamps the floating-card position. Every
implemented adapter needs literal fixture coverage for primary, above, below,
negative-coordinate, and mixed-scale arrangements supported by its OS.

## Adapter layout

```text
crates/
  platform-api/
  platform-contract-tests/

platform/
  macos/
    rust/                  Rust provider implementations and isolated FFI
    native/                SwiftPM package, sources, and native tests
  linux/                   Rust skeleton; native modules arrive in Plan B
  windows/                 Rust skeleton; native modules arrive in Plan B

apps/desktop/src-tauri/src/
  bootstrap.rs             storage + target adapter composition
  commands/                stable library and capture commands
  events.rs                typed capture events/errors
  lib.rs                   Tauri setup and registration
```

Reusable fake providers and contract assertions live in
`crates/platform-contract-tests`. Core behavior regressions belong in shared
crate tests. Native API and coordinate regressions belong in the responsible
adapter tests.

## Implementing a Plan B adapter

1. On the physical target machine, probe the native APIs and record supported
   OS/session/application combinations before claiming a capability.
2. Replace one unsupported skeleton provider at a time. Keep capabilities
   `false` until its implementation, automated contract tests, and relevant
   physical-machine checks exist.
3. Preserve exact selected Unicode and context without clipboard mutation,
   focus changes, continuous polling, or private-content diagnostics.
4. Keep OCR explicit and permission-respecting. Capture the smallest practical
   region, keep images in memory, and return candidates with normalized bounds.
5. Keep translation replaceable behind `TranslationProvider`; unsupported
   translation must preserve the shared save-without-translation path.
6. Limit native window code to behavior Tauri cannot provide consistently.
   Shared Rust retains placement policy.
7. Run shared, contract, UI, adapter, installer, permission, application-matrix,
   and mixed-monitor checks on that OS. Record unsupported cases rather than
   silently substituting behavior.

When a target reveals a missing portable concept, add the smallest shared
contract and an OS-neutral regression test, then update every affected adapter
in the same short-lived branch. Do not fork the core or maintain long-lived OS
branches.

## Plan B references

- Ubuntu 24.04 GNOME Wayland: `docs/linux-development.md`
- Windows 11 x64: `docs/windows-development.md`
- Cross-platform architecture and milestone boundary:
  `docs/superpowers/specs/2026-08-26-cross-platform-core-and-os-adapters-design.md`

No Linux or Windows startup, native capture, permission, OCR, translation,
window, or package claim is complete until its corresponding physical-machine
check is recorded.
