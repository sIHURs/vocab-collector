# Cross-Platform Core and OS Adapter Design

## Purpose

Restructure Vocab Collector so one shared Rust backend owns product behavior while macOS, Linux, and future Windows integrations remain replaceable operating-system adapters. The immediate work is split into two implementation plans: Plan A is performed on macOS and stabilizes the existing macOS product boundary; Plan B is performed on an Ubuntu 24.04 physical machine and implements and verifies the Linux port.

The split is based on the machine and operating-system capability required to complete and verify the work. Work that requires AT-SPI, a GNOME Wayland session, XDG Desktop Portal, Linux packaging, or Ubuntu runtime verification belongs to Plan B even when its source skeleton can be prepared on macOS.

## Goals

- Keep domain, application, capture orchestration, persistence, synchronization, and frontend behavior shared.
- Prevent platform code from directly owning vocabulary, review, storage, or synchronization behavior.
- Allow macOS and Linux adapters to be developed in separate directories without long-lived platform branches.
- Make changes to the shared core safe through fake providers, reusable platform contract tests, and multi-OS CI.
- Preserve the current macOS state before restructuring it.
- Allow known macOS defects to be fixed inside a well-defined adapter instead of expanding Tauri's composition root.
- Deliver Ubuntu 24.04 GNOME Wayland support incrementally, with explicit capability and compatibility claims.

## Non-Goals

- Implementing Windows support in the first two plans.
- Claiming support for every Linux distribution, desktop environment, or display server.
- Replacing Tauri or Svelte.
- Reimplementing shared business rules for each operating system.
- Requiring the initial Linux port to ship OCR and translation before the shared application can launch and operate.
- Using a long-lived `macos` or `linux` Git branch as the platform boundary.

## Target Platforms

- Existing platform: macOS 15 or newer.
- First Linux target: Ubuntu 24.04 on a physical machine, GNOME, and Wayland.
- First Linux package target: Debian package (`.deb`).
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
                              `- future Windows adapter
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

ui/
supabase/
docs/
```

The existing `crates/platform` becomes `crates/platform-api`. It defines contracts and portable data types only. The current Swift package remains native macOS code but moves under the macOS adapter boundary. The current `apps/desktop/src-tauri/src/macos_bridge.rs` becomes part of the macOS Rust adapter rather than the Tauri application.

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

Neither adapter may write SQLite directly, decide review behavior, deduplicate words, or implement save and Undo semantics.

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

CI is divided into shared, macOS, and Linux jobs:

- Shared Rust and UI tests run on Ubuntu.
- macOS adapter tests and builds run on macOS.
- Linux adapter tests and builds run on Ubuntu 24.04.
- A change to shared core, platform contracts, Tauri commands, or shared UI runs both platform jobs.
- A platform-only change runs shared checks and the affected platform job.
- Native permissions, AT-SPI, Portal, window-focus behavior, and mixed-monitor behavior retain physical-machine manual test matrices.

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

### A3-prep: Prepare the shared application for Linux startup

The original M3 acceptance criterion, "the application launches on Ubuntu and shared business behavior works," cannot be completed or claimed on macOS. Plan A performs only its platform-neutral preparation:

- Add target-specific adapter selection in Cargo and the composition root.
- Add a compiling Linux adapter skeleton that reports unsupported capabilities honestly, to the extent it can be checked without Linux system libraries.
- Make shared commands and UI independent of macOS-specific command availability.
- Add Ubuntu CI configuration for checks that can run in hosted CI.
- Document the Ubuntu setup and verification commands that Plan B must execute.

Plan A completes when macOS behavior remains usable, the macOS implementation is isolated behind `platform-api`, shared tests pass, and no Ubuntu runtime success is claimed without Ubuntu verification.

## Plan B Boundary: Work Performed on Ubuntu 24.04

Plan B begins by completing the runtime portion of M3 and then implements the remaining Linux milestones.

### B2/M2: Complete cross-platform contract coverage

- Run and extend the platform contract tests on Ubuntu.
- Add Linux fixtures for coordinates, errors, permissions, Unicode selection, and cancellation.
- Ensure core changes discovered on Ubuntu remain covered by OS-neutral regression tests.

### B3: Complete Ubuntu startup and shared behavior validation

- Install the pinned Rust, Node, pnpm, Tauri, GTK/WebKit, AT-SPI, and Portal development dependencies.
- Build and run the desktop application in a real GNOME Wayland session.
- Verify SQLite, Today, Vocabulary, Review, Settings, manual Quick Capture, and browser/Tauri frontend contracts.
- Correct Linux build and runtime assumptions through the smallest appropriate adapter or shared-contract change.

### B4: Implement AT-SPI selection capture

- Probe and document behavior in Firefox, Chromium, GNOME Text Editor, LibreOffice, and one PDF reader.
- Implement selection and available geometry without clipboard mutation or focus changes.
- Return explicit unsupported and empty-selection results.

### B5: Implement shortcut and floating-window behavior

- Verify global shortcut behavior under GNOME Wayland.
- Implement capability-aware failure guidance where registration is unavailable.
- Verify non-activating behavior, work areas, scale factors, multiple displays, and placement.

### B6: Implement the Linux translation provider

- Perform a focused provider selection before implementation.
- Keep provider details behind `TranslationProvider`.
- Preserve save-without-translation as a supported shared workflow.

### B7: Implement Portal screenshot and explicit OCR

- Use XDG Desktop Portal and comply with Wayland permission behavior.
- Start OCR only after explicit user confirmation.
- Keep screenshot data in memory and out of logs.
- Keep the OCR engine replaceable behind `OcrProvider`.

### B8: Produce an Ubuntu preview package

- Build a `.deb` that runs without development tools.
- Verify install, launch, upgrade, and uninstall behavior.
- Document runtime dependencies and the tested application/session matrix.

### B9: Cross-platform regression

- Run the complete Linux suite on Ubuntu.
- Push shared and Linux changes to the common repository.
- Return to macOS or use macOS CI to compile and test the macOS adapter.
- Perform physical macOS regression testing before the next multi-platform release.

## Plan A and Plan B Handoff

Plan A produces a tagged macOS baseline, an isolated macOS adapter, stable shared contracts, contract tests, a Linux skeleton, and Ubuntu setup documentation. Plan B starts from that same main branch on the Ubuntu machine. It does not copy or fork the core.

When Ubuntu debugging requires a core change, Plan B adds an OS-neutral regression test and updates all affected adapter contracts in the same short-lived branch. macOS CI must pass before that change reaches main. Changes that affect only AT-SPI, Portal, Linux window integration, or Linux packaging remain inside `platform/linux` and Linux-specific packaging files.

## Acceptance Criteria

Plan A is ready for handoff when:

- The current macOS state is reproducibly tagged and backed up.
- macOS native code is accessible only through the macOS Rust adapter.
- The Tauri composition root and UI-facing commands are platform-neutral.
- Shared crates contain no operating-system-specific implementation.
- Core, fake-provider, contract, macOS adapter, UI, and macOS bundle checks pass.
- Ubuntu runtime work is clearly identified as unverified rather than assumed complete.

Plan B is ready for an Ubuntu preview when:

- The application launches from an installable `.deb` on Ubuntu 24.04 GNOME Wayland.
- Shared capture, storage, review, settings, and Undo behavior matches macOS contracts.
- AT-SPI selection works in the agreed application matrix or reports documented limitations.
- Shortcut, window, translation, and explicit Portal OCR states are represented honestly.
- Linux and shared automated tests pass, and macOS CI remains green after shared-core changes.
- Known desktop-environment and application limitations are documented without claiming universal Linux support.
