# Cross-Platform Core and OS Adapter Design

## Purpose

Restructure Vocab Collector so one shared Rust backend owns product behavior while macOS, Linux, and Windows integrations remain replaceable operating-system adapters. The immediate work is split into two implementation plans: Plan A is performed on macOS and stabilizes the shared multi-platform structure and existing macOS product boundary; Plan B contains independent Linux and Windows implementation tracks performed on Ubuntu 24.04 and Windows 11 physical machines.

The split is based on the machine and operating-system capability required to complete and verify the work. Work that requires AT-SPI, a GNOME Wayland session, XDG Desktop Portal, Windows UI Automation, Win32 window behavior, native packaging, or target-OS runtime verification belongs to Plan B even when its source skeleton can be prepared on macOS.

## Goals

- Keep domain, application, capture orchestration, persistence, synchronization, and frontend behavior shared.
- Prevent platform code from directly owning vocabulary, review, storage, or synchronization behavior.
- Allow macOS, Linux, and Windows adapters to be developed in separate directories without long-lived platform branches.
- Make changes to the shared core safe through fake providers, reusable platform contract tests, and multi-OS CI.
- Preserve the current macOS state before restructuring it.
- Allow known macOS defects to be fixed inside a well-defined adapter instead of expanding Tauri's composition root.
- Deliver Ubuntu 24.04 GNOME Wayland and Windows 11 x64 support incrementally, with explicit capability and compatibility claims.

## Non-Goals

- Claiming support for every Linux distribution, desktop environment, or display server.
- Claiming Windows 10, Windows on ARM, or older Windows versions before separate validation.
- Replacing Tauri or Svelte.
- Reimplementing shared business rules for each operating system.
- Requiring the initial Linux or Windows port to ship OCR and translation before the shared application can launch and operate.
- Using a long-lived `macos` or `linux` Git branch as the platform boundary.

## Target Platforms

- Existing platform: macOS 15 or newer.
- First Linux target: Ubuntu 24.04 on a physical machine, GNOME, and Wayland.
- First Linux package target: Debian package (`.deb`).
- First Windows target: Windows 11 x64 on a physical machine.
- First Windows package target: Tauri NSIS installer.
- X11, KDE, Flatpak, AppImage, RPM, and other distributions are later compatibility increments.

## Architecture

The dependency direction is fixed:

```text
Svelte UI
    |
Tauri commands and events
    |
Shared Rust application and capture orchestration
    |                         |
Domain and repository ports  Platform capability ports
    ^                         ^
SQLite and sync adapters      OS adapters
                              |- macOS Rust adapter -> Swift native package
                              |- Linux Rust adapter -> AT-SPI / Portal / Linux APIs
                              `- Windows Rust adapter -> UI Automation / Win32
```

Shared crates must not depend on an operating-system adapter. Platform adapters may depend on shared contracts. Conditional compilation for operating-system selection is restricted to the desktop composition root and platform adapters.

## Repository Structure

```text
apps/
  desktop/
    src-tauri/
      src/
        main.rs
        lib.rs
        commands/
        events.rs
        bootstrap.rs

crates/
  domain/
  application/
  capture/
  platform-api/
  platform-contract-tests/
  storage/
  sync/

platform/
  macos/
    rust/
      Cargo.toml
      src/
    native/
      Package.swift
      Sources/
      Tests/
  linux/
    Cargo.toml
    src/
      accessibility/
      screenshot/
      translation/
      capabilities.rs
      permissions.rs
      session.rs
      window.rs
    tests/
  windows/
    Cargo.toml
    src/
      accessibility/
      screenshot/
      translation/
      capabilities.rs
      com.rs
      permissions.rs
      window.rs
    tests/

ui/
supabase/
docs/
```

The existing `crates/platform` becomes `crates/platform-api`. It defines contracts and portable data types only. The current Swift package remains native macOS code but moves under the macOS adapter boundary. The current `apps/desktop/src-tauri/src/macos_bridge.rs` becomes part of the macOS Rust adapter rather than the Tauri application. Linux and Windows integrations are separate Rust crates that implement the same contracts.

## Shared Core Responsibilities

The shared Rust backend consists of:

- `domain`: entities, normalization, deduplication, review rules, domain events, and repository contracts.
- `application`: capture, Undo, Today, Vocabulary, Review, Settings, and other product use cases.
- `capture`: request IDs, cancellation, stale-result rejection, save-once behavior, OCR fallback policy, shortcut validation, and portable placement.
- `platform-api`: small platform capability traits and portable DTOs.
- `storage`: SQLite repositories, migrations, and transactional outbox.
- `sync`: account synchronization and conflict behavior.

These crates must not contain Tauri APIs, Swift FFI, AT-SPI, XDG Portal, Win32, or operating-system-specific product behavior. Shared UI should branch on reported capabilities, not operating-system names.

## Platform Contracts

Platform capabilities are kept separate rather than combined into one large trait:

```rust
pub trait SelectionProvider {}
pub trait TranslationProvider {}
pub trait OcrProvider {}
pub trait PermissionProvider {}
pub trait WindowProvider {}
```

The composition root injects a set of provider objects into the shared application runtime. Portable contracts cover selection candidates, logical screen coordinates, monitor work areas, permission status, request cancellation, typed platform errors, and capability reporting.

All coordinates exposed through `platform-api` use logical units and a top-left virtual-desktop origin. Platform adapters normalize native coordinate systems before returning values.

Capabilities are reported explicitly. A platform that cannot provide selection bounds, OCR, offline translation, or a non-activating window reports that fact rather than imitating support or causing the UI to test the operating-system name.

## Adapter Isolation

The macOS adapter owns:

- Safe Rust wrappers around the Swift JSON/C ABI.
- Accessibility, ScreenCaptureKit, Vision, Apple Translation, and AppKit integration.
- macOS permission and error mapping.
- macOS-native window behavior and coordinate normalization.

The Linux adapter owns:

- GNOME/Wayland session and capability detection.
- AT-SPI2 selection access and available selection geometry.
- XDG Desktop Portal screenshot interactions.
- Linux OCR and translation provider integration selected in later focused designs.
- Linux permission guidance, window behavior, and coordinate normalization.

The Windows adapter owns:

- COM initialization and threading required by Windows integrations.
- UI Automation TextPattern/TextPattern2 selection access and available geometry.
- Windows Graphics Capture and the selected OCR provider integration.
- Win32/Tauri native window behavior, Windows permission guidance, and coordinate normalization.
- UTF-16 conversion without splitting surrogate pairs.

No adapter may write SQLite directly, decide review behavior, deduplicate words, or implement save and Undo semantics.

## Tauri Composition Root

The Tauri crate creates storage, selects the adapter for the build target, constructs the shared application runtime, and registers commands and events. Commands such as capture, permission request, translation, OCR, and save retain the same names and DTOs across operating systems. The adapter behind each command differs; the UI-facing API does not.

Target-specific Cargo dependencies select the macOS or Linux adapter. The Tauri crate should not directly call Swift or AT-SPI APIs.

## Testing and Change Safety

Four test layers protect shared development:

1. Shared core unit tests run on any operating system.
2. Application tests use fake platform providers and never require native permissions.
3. Reusable platform contract tests verify each adapter's portable behavior with fixtures.
4. Native integration and manual tests run on the corresponding operating system.

Contract coverage includes selection preservation, Unicode handling, empty and unsupported selections, permission denial, explicit OCR confirmation, coordinate normalization, cancellation, save-once behavior, and content-free diagnostics.

When debugging a platform, a change is first classified as a core behavior defect, an adapter defect, or an incomplete platform contract. Core changes begin with a shared regression test. Contract changes update fake providers, contract tests, macOS, and Linux adapters together. Unsupported optional capabilities return an explicit portable result.

Shared contracts evolve additively where practical. Cargo features may control optional dependencies, but they must not create different vocabulary or review rules per operating system.

## Git and CI Model

The repository uses one continuously integrated main branch and short-lived branches such as `refactor/platform-api`, `feature/linux-atspi-selection`, or `fix/macos-window-placement`. An immutable tag records the paused macOS baseline. Git worktrees may isolate simultaneous tasks, but long-lived platform branches are not used.

CI is divided into shared, macOS, Linux, and Windows jobs:

- Shared Rust and UI tests run on Ubuntu.
- macOS adapter tests and builds run on macOS.
- Linux adapter tests and builds run on Ubuntu 24.04.
- Windows adapter tests and builds run on Windows 11 runners.
- A change to shared core, platform contracts, Tauri commands, or shared UI runs macOS, Linux, and Windows jobs.
- A platform-only change runs shared checks and the affected platform job.
- Native permissions, AT-SPI, Portal, UI Automation, Win32 window-focus behavior, and mixed-monitor behavior retain physical-machine manual test matrices.

## Plan A Boundary: Work Performed on macOS

Plan A contains the macOS baseline and structural work needed to leave the macOS implementation safely encapsulated.

### A0: Preserve the macOS baseline

- Record toolchain, permissions, tested applications, known defects, and current build behavior.
- Run the complete Rust, Svelte, Swift, Tauri, and bundle verification suite.
- Commit a clean baseline and create an immutable macOS MVP tag.
- Configure a private Git remote or create a verified Git bundle before moving machines.

### A1: Extract and encapsulate the macOS adapter

- Establish `platform-api` and capability-specific traits.
- Move macOS Rust FFI and native integration out of the Tauri crate.
- Reorganize the Swift package beneath the macOS adapter boundary.
- Convert Tauri commands to adapter-independent calls.
- Keep SQLite, product rules, save/Undo, and UI state outside the adapter.
- Add macOS adapter tests, fake-provider application tests, and reusable contract tests.
- Fix macOS defects needed to prove the new boundary, while recording unrelated product defects rather than expanding the restructuring scope.

### A3-prep: Prepare the shared application for Linux and Windows startup

The runtime acceptance criteria, "the application launches on Ubuntu or Windows and shared business behavior works," cannot be completed or claimed on macOS. Plan A performs only the platform-neutral preparation:

- Add target-specific adapter selection in Cargo and the composition root.
- Add Linux and Windows adapter skeletons that report unsupported capabilities honestly, to the extent they can be checked without target-OS system libraries.
- Make shared commands and UI independent of macOS-specific command availability.
- Add Ubuntu and Windows CI configuration for checks that can run on hosted runners.
- Document Ubuntu 24.04 and Windows 11 setup and verification commands for Plan B.

Plan A completes when the multi-platform repository boundaries are established, macOS behavior remains usable, the macOS implementation is isolated behind `platform-api`, shared tests pass, and no Linux or Windows runtime success is claimed without target-machine verification.

## Plan B Boundary: Work Performed on Linux and Windows Physical Machines

Plan B contains a shared contract track plus independent Linux and Windows tracks. The Linux and Windows tracks may proceed in either order and must not depend on each other's native implementation. Both start from the same main branch produced by Plan A and depend only on shared contracts.

### B2: Complete cross-platform contract coverage

- Run and extend platform contract tests on Ubuntu and Windows.
- Add Linux and Windows fixtures for coordinates, errors, permissions, Unicode selection, and cancellation.
- Ensure core defects discovered on either platform receive OS-neutral regression tests.
- Require macOS, Linux, and Windows CI for changes to shared contracts or core behavior.

### BL3: Complete Ubuntu startup and shared behavior validation

- Install the pinned Rust, Node, pnpm, Tauri, GTK/WebKit, AT-SPI, and Portal development dependencies.
- Build and run the desktop application in a real GNOME Wayland session.
- Verify SQLite, Today, Vocabulary, Review, Settings, manual Quick Capture, and browser/Tauri frontend contracts.
- Correct Linux build and runtime assumptions through the smallest appropriate adapter or shared-contract change.

### BL4: Implement AT-SPI selection capture

- Probe and document behavior in Firefox, Chromium, GNOME Text Editor, LibreOffice, and one PDF reader.
- Implement selection and available geometry without clipboard mutation or focus changes.
- Return explicit unsupported and empty-selection results.

### BL5: Implement Linux shortcut and floating-window behavior

- Verify global shortcut behavior under GNOME Wayland.
- Implement capability-aware failure guidance where registration is unavailable.
- Verify non-activating behavior, work areas, scale factors, multiple displays, and placement.

### BL6: Implement the Linux translation provider

- Perform a focused provider selection before implementation.
- Keep provider details behind `TranslationProvider`.
- Preserve save-without-translation as a supported shared workflow.

### BL7: Implement Portal screenshot and explicit OCR

- Use XDG Desktop Portal and comply with Wayland permission behavior.
- Start OCR only after explicit user confirmation.
- Keep screenshot data in memory and out of logs.
- Keep the OCR engine replaceable behind `OcrProvider`.

### BL8: Produce an Ubuntu preview package

- Build a `.deb` that runs without development tools.
- Verify install, launch, upgrade, and uninstall behavior.
- Document runtime dependencies and the tested application/session matrix.

### BW3: Complete Windows startup and shared behavior validation

- Install the pinned Rust MSVC toolchain, Node, pnpm, Tauri, Visual Studio Build Tools, and WebView2 requirements.
- Build and run the desktop application on a Windows 11 x64 physical machine.
- Verify SQLite, Today, Vocabulary, Review, Settings, manual Quick Capture, and browser/Tauri frontend contracts.
- Correct Windows build and runtime assumptions through the smallest appropriate adapter or shared-contract change.

### BW4: Implement Windows UI Automation selection capture

- Implement COM initialization on the correct threads.
- Use UI Automation TextPattern/TextPattern2 for current selection, context, and available bounding rectangles.
- Preserve UTF-16 and Unicode boundaries without clipboard mutation or focus changes.
- Probe and document behavior in Edge, Chrome, Firefox, Notepad, Microsoft Word, and one PDF reader.
- Return explicit unsupported and empty-selection results for inaccessible controls.

### BW5: Implement Windows shortcut and floating-window behavior

- Verify shortcut registration, conflict handling, persistence, and repeat suppression.
- Use the minimum Win32/Tauri integration necessary for non-activating, always-on-top, task-switcher-aware behavior.
- Verify work areas, per-monitor DPI, mixed-scale displays, negative coordinates, and placement.

### BW6: Implement the Windows translation provider

- Perform a focused provider selection before implementation.
- Keep provider details behind the same `TranslationProvider` contract used on macOS and Linux.
- Preserve save-without-translation as a supported shared workflow.

### BW7: Implement explicit Windows screenshot OCR

- Use Windows Graphics Capture or another approved permission-respecting Windows API.
- Start capture and OCR only after explicit user confirmation.
- Keep screenshot data in memory and out of logs.
- Keep Windows Media OCR, Tesseract, or another selected engine replaceable behind `OcrProvider`.

### BW8: Produce a Windows preview package

- Build an NSIS installer that runs without development tools.
- Verify install, launch, upgrade, uninstall, and WebView2 behavior.
- Document runtime dependencies and the tested application matrix.

### B9: Three-platform regression

- Run the complete Linux suite on Ubuntu and Windows suite on Windows.
- Push shared and platform changes to the common repository rather than copying core code between machines.
- Use macOS CI for every shared change and perform physical macOS regression before a multi-platform release.
- Record platform capability differences and unsupported applications without forking shared product rules.

## Plan A and Plan B Handoff

Plan A produces a tagged macOS baseline, an isolated macOS adapter, stable shared contracts, contract tests, Linux and Windows skeletons, and target-machine setup documentation. Plan B starts from that same main branch on the Ubuntu and Windows machines. Neither platform copies or forks the core.

When Linux or Windows debugging requires a core change, Plan B adds an OS-neutral regression test and updates every affected adapter contract in the same short-lived branch. macOS, Linux, and Windows CI must pass before that change reaches main. Changes that affect only AT-SPI, Portal, or Linux packaging remain under the Linux adapter; changes that affect only UI Automation, Win32, Windows capture, or NSIS packaging remain under the Windows adapter.

## Acceptance Criteria

Plan A is ready for handoff when:

- The current macOS state is reproducibly tagged and backed up.
- macOS native code is accessible only through the macOS Rust adapter.
- The Tauri composition root and UI-facing commands are platform-neutral.
- Shared crates contain no operating-system-specific implementation.
- Linux and Windows adapter skeletons depend on `platform-api`, not macOS code.
- Core, fake-provider, contract, macOS adapter, UI, and macOS bundle checks pass.
- Linux and Windows runtime work is clearly identified as unverified rather than assumed complete.

Plan B is ready for multi-platform previews when:

- The application launches from an installable `.deb` on Ubuntu 24.04 GNOME Wayland.
- The application launches from an NSIS installer on Windows 11 x64.
- Shared capture, storage, review, settings, and Undo behavior matches the same contracts on all three platforms.
- AT-SPI and UI Automation selection work in their agreed application matrices or report documented limitations.
- Shortcut, window, translation, and explicit OCR states are represented honestly on each platform.
- Linux, Windows, and shared automated tests pass, and macOS CI remains green after shared-core changes.
- Known desktop-environment, Windows application, and platform capability limitations are documented without universal support claims.
