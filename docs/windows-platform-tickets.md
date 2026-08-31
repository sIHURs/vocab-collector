# Windows Platform Tracer-Bullet Tickets

Source: `docs/windows-platform-plan-v2.md`

Environment labels:

- **Current:** implementation and automated verification can be completed in the current managed workspace.
- **Hybrid:** implementation and automated verification can run in the current workspace, but the ticket is not complete until its listed Windows 11 physical-machine checks pass.
- **Physical:** the deciding work and evidence require a Windows 11 x64 physical machine.

Every ticket must update `docs/windows-development-log.md`. Native capability flags remain false until automated and physical evidence both exist.

## W-01: Establish the Windows automated baseline

**What to build:** Record a reproducible baseline for the existing Windows skeleton, shared product workflows, frontend, desktop contract, and CI without enabling native capabilities.

**Blocked by:** None.

**Environment:** Current for automated checks; `pnpm tauri dev` and runtime behavior are Physical.

**Directories:** workspace manifests; `crates/`; `platform/windows/`; `apps/desktop/`; `ui/`; `.github/workflows/`; `docs/`.

**Acceptance criteria:**

- [x] Required tool versions and the current Windows CI lane are recorded.
- [x] Existing Rust, frontend, and build gates have an exact pass/fail/block result.
- [x] Windows native capability flags remain false and no later-ticket behavior is implemented.
- [x] Runtime checks are marked `Not run` unless the host is confirmed to be a Windows 11 x64 physical machine.
- [x] `docs/windows-development-log.md` records scope, decisions, files, commands/results, unverified items, and the W-02 starting point.

**Verification:**

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings
cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
cargo build --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
pnpm check
pnpm test
pnpm build
cargo test -p vocab-platform-windows
```

Physical-only follow-up: `pnpm tauri dev` plus Manual Capture, SQLite restart persistence, Today, Vocabulary, Review, and Settings smoke checks.

## W-02: Select an independent Windows presentation

**What to build:** Make Windows select its own main and capture presentation while continuing to use the stable frontend backend contract and leaving macOS page behavior unchanged.

**Blocked by:** W-01.

**Environment:** Current.

**Directories:** `ui/src/`; `apps/desktop/src-tauri/`; `docs/`.

**Acceptance criteria:**

- [x] Windows presentation selection is centralized at composition, not scattered through feature components.
- [x] Browser/demo mode and macOS presentation remain usable.
- [x] Windows navigation contains no static Progress claim.

**Verification:** `pnpm check`, `pnpm test`, `pnpm build`, and `cargo test -p vocab-desktop --test command_contract`.

## W-03: Deliver Manual Capture through Today and Vocabulary

**What to build:** Let a Windows user save a Manual Capture, see it in Today and Vocabulary, add a repeat Encounter, inspect detail, Undo, and retain data after restart.

**Blocked by:** W-02.

**Environment:** Hybrid; browser behavior is Current, SQLite restart through Tauri is Physical.

**Directories:** `ui/src/`; `apps/desktop/src-tauri/`; `crates/application/`; `crates/storage/`; `docs/`.

**Acceptance criteria:**

- [x] One Vocabulary Item owns repeat Encounters.
- [x] Today, Vocabulary, detail, and Undo refresh from the backend contract.
- [x] Loading, empty, no-result, and recoverable failure states are observable.

**Verification:** `pnpm check`, `pnpm test`, `cargo test -p vocab-application -p vocab-storage`, `cargo test -p vocab-desktop --test command_contract`, and physical `pnpm tauri dev` restart smoke test.

## W-04: Deliver the Windows Review workflow

**What to build:** Let a user start Review from Today, submit forgotten/remembered outcomes, finish or close the queue, and see Today refresh.

**Blocked by:** W-02, W-03.

**Environment:** Current.

**Directories:** `ui/src/`; `crates/domain/`; `crates/application/`; `apps/desktop/src-tauri/`; `docs/`.

**Acceptance criteria:**

- [ ] Review uses the shared due queue and existing ratings.
- [ ] Empty, in-progress, closed, and completed states are covered.
- [ ] UI does not calculate review dates.

**Verification:** `pnpm check`, `pnpm test`, `cargo test -p vocab-domain -p vocab-application`, and desktop command-contract tests.

## W-05: Make Windows Settings and visual preferences effective

**What to build:** Persist Windows Settings and make language, daily limit, shortcut, theme, and reduced-motion choices observable.

**Blocked by:** W-02.

**Environment:** Current; OS shortcut behavior completes in W-06.

**Directories:** `ui/src/`; `crates/domain/`; `crates/application/`; `crates/storage/`; `apps/desktop/src-tauri/`; `docs/`.

**Acceptance criteria:**

- [ ] Settings round-trip through SQLite.
- [ ] System/light/dark and reduced motion apply without false success states.
- [ ] Windows presentation retains the established Vocab Collector visual identity.

**Verification:** `pnpm check`, `pnpm test`, storage/application tests, and desktop command-contract tests.

## W-06: Keep capture available through tray and system settings

**What to build:** Keep the app available after the main window closes, with tray Open/Exit, active shortcut, launch-at-login, and review-time notifications.

**Blocked by:** W-05.

**Environment:** Hybrid.

**Directories:** `apps/desktop/src-tauri/`; `ui/src/`; `crates/capture/`; `.github/workflows/`; `docs/`.

**Acceptance criteria:**

- [ ] Close hides to tray; Open restores; Exit terminates and cleans session resources.
- [ ] Shortcut replacement rolls back safely on conflicts.
- [ ] Autostart and notifications match Settings and expose failures.
- [ ] Login/restart, Explorer restart, disabled notifications, and time-zone behavior have physical evidence.

**Verification:** `pnpm check`, `pnpm test`, shortcut tests, desktop command-contract tests, `pnpm tauri dev`, and the physical lifecycle matrix.

## W-07: Capture selected text from Notepad through UIA

**What to build:** Press the global shortcut over a Notepad selection and receive a portable Capture Candidate through UI Automation.

**Blocked by:** W-01, W-02.

**Environment:** Hybrid.

**Directories:** `platform/windows/`; `crates/platform-api/`; `crates/platform-contract-tests/`; `apps/desktop/src-tauri/`; `ui/src/`; `docs/`.

**Acceptance criteria:**

- [ ] COM ownership is deterministic and native objects do not leak across boundaries.
- [ ] Exact Unicode text, context when available, source metadata, and bounds normalize into portable values.
- [ ] HRESULT details do not leak into normal product UI.
- [ ] Selection capability becomes true only after Notepad physical evidence.

**Verification:** Windows adapter and contract tests, desktop command-contract tests, Clippy, `pnpm tauri dev`, and physical Notepad Unicode capture.

## W-08: Expand UIA compatibility and safe diagnostics

**What to build:** Add bounded TextPattern2/TextPattern discovery and content-safe diagnostics for browsers, editors, Terminal, Office, and PDF readers.

**Blocked by:** W-07.

**Environment:** Hybrid.

**Directories:** `platform/windows/`; `crates/platform-contract-tests/`; `crates/application/`; `docs/`.

**Acceptance criteria:**

- [ ] Focused fast path and bounded traversal terminate predictably.
- [ ] UTF-16, multiple rectangles, empty selection, unsupported control, and missing metadata have fixture coverage.
- [ ] Default diagnostics print metadata/lengths, not captured content.
- [ ] Required compatibility-matrix UIA columns are recorded physically.

**Verification:** Windows adapter, platform-contract, and application fake-provider tests plus the physical UIA matrix.

## W-09: Present a non-activating mixed-DPI capture window

**What to build:** Show the capture result next to its source without initial focus theft and keep the complete window inside the selected monitor work area.

**Blocked by:** W-07.

**Environment:** Hybrid.

**Directories:** `crates/capture/`; `platform/windows/`; `apps/desktop/src-tauri/`; `ui/src/`; `docs/`.

**Acceptance criteria:**

- [ ] Passive presentation is non-activating and absent from the taskbar.
- [ ] Explicit editing can focus the window and save/cancel returns predictable focus.
- [ ] Negative origins and 100/125/150/200% mixed scales place correctly.

**Verification:** placement, Windows adapter, desktop contract, and frontend tests plus physical focus/multi-monitor checks.

## W-10: Correct and save a Native Capture manually

**What to build:** Let the user correct selected text/context, enter an optional translation, save once, Undo, or explicitly save without translation.

**Blocked by:** W-03, W-07, W-09.

**Environment:** Hybrid.

**Directories:** `crates/capture/`; `crates/application/`; `apps/desktop/src-tauri/`; `ui/src/`; `docs/`.

**Acceptance criteria:**

- [ ] Corrections enter through the shared application workflow, not the Windows adapter.
- [ ] One request ID protects correction, save, Undo, and dismissal.
- [ ] Windows reports automatic translation unavailable honestly.

**Verification:** capture/application tests, desktop command-contract tests, frontend tests, and physical floating-window interaction.

## W-11: Save one explicitly confirmed OCR candidate

**What to build:** After eligible UIA failure, explicitly capture a bounded region, recognize in memory, confirm one candidate, and save an OCR-origin Encounter.

**Blocked by:** W-08, W-09, W-10.

**Environment:** Hybrid.

**Directories:** `platform/windows/`; `crates/capture/`; `crates/application/`; `apps/desktop/src-tauri/`; `ui/src/`; `docs/`.

**Acceptance criteria:**

- [ ] Windows Graphics Capture and OCR release native image resources without files or content logs.
- [ ] Windows no longer uses the Cocoa coordinate conversion.
- [ ] Confirm saves once; cancel saves nothing.

**Verification:** Windows adapter, capture, application, desktop contract, and frontend tests plus physical consent/cancel/OCR checks.

## W-12: Resolve ambiguous OCR candidates

**What to build:** Rank OCR candidates by pointer containment, distance, then confidence and let the user choose among close candidates with the keyboard.

**Blocked by:** W-11.

**Environment:** Hybrid.

**Directories:** `crates/capture/`; `apps/desktop/src-tauri/`; `ui/src/`; `platform/windows/`; `docs/`.

**Acceptance criteria:**

- [ ] Dominant candidates use single confirmation; close candidates use an accessible list.
- [ ] Candidate correction and cancellation preserve request safety.
- [ ] Ranking uses portable geometry fixtures independent from OCR implementation.

**Verification:** capture ranking tests, application/desktop contract tests, frontend keyboard tests, and physical ambiguous OCR cases.

## W-13: Complete Windows UI states and accessibility

**What to build:** Finish all Windows pages and capture states with keyboard, Narrator, high-contrast, theme, reduced-motion, and text-scaling support.

**Blocked by:** W-03, W-04, W-06, W-10, W-12.

**Environment:** Hybrid.

**Directories:** `ui/src/`; `apps/desktop/src-tauri/`; `docs/`.

**Acceptance criteria:**

- [ ] Every page has loading, empty, success, and recoverable error states.
- [ ] Keyboard focus and semantic names are complete.
- [ ] UI matches existing colors/component style where Windows behavior permits.
- [ ] Narrator, high contrast, minimum size, and text scaling have physical evidence.

**Verification:** `pnpm check`, `pnpm test`, `pnpm build`, desktop contract tests, `pnpm tauri dev`, and the physical accessibility matrix.

## W-14: Decide clipboard fallback from compatibility evidence

**What to build:** Use the completed application matrix to close clipboard capture as unsupported or deliver a separately disableable safe fallback where it materially improves capture.

**Blocked by:** W-08, W-12.

**Environment:** Physical for the decision; Hybrid if implementation is approved.

**Directories:** `platform/windows/`; `apps/desktop/src-tauri/`; `crates/platform-contract-tests/`; `docs/`.

**Acceptance criteria:**

- [ ] The decision names the failing applications/use cases and evidence.
- [ ] If omitted, the fallback and user-visible alternative are documented.
- [ ] If implemented, new clipboard updates are distinguished from stale values and previous formats are restored or restoration failure is explicit.

**Verification:** Physical compatibility matrix; if implemented, Windows adapter, platform-contract, desktop contract tests, and physical clipboard-format checks.

## W-15: Produce an installable NSIS preview in CI

**What to build:** Produce a traceable Windows 11 x64 per-user NSIS artifact and verify clean install, WebView2 handling, upgrade, and uninstall-data choices.

**Blocked by:** W-13, W-14.

**Environment:** Hybrid.

**Directories:** `apps/desktop/src-tauri/`; `.github/workflows/`; `docs/`.

**Acceptance criteria:**

- [ ] CI builds and retains an NSIS artifact tied to a commit.
- [ ] Clean launch requires no developer tools.
- [ ] Upgrade preserves SQLite/Settings.
- [ ] Uninstall cleans application integration and follows the retain/delete-data choice.

**Verification:** full Windows workspace/frontend gates, `pnpm tauri build --bundles nsis`, and physical clean-install/upgrade/uninstall scenarios.

## W-16: Complete Windows release evidence

**What to build:** Run the release artifact through the complete physical compatibility, lifecycle, privacy, accessibility, DPI, and installer matrices and publish an honest readiness conclusion.

**Blocked by:** W-15.

**Environment:** Physical.

**Directories:** `docs/`; `.github/workflows/` only if evidence exposes a release-gate gap.

**Acceptance criteria:**

- [ ] Every Definition of Done item has automated and/or physical evidence appropriate to the claim.
- [ ] No true capability has an unexplained `Not run` row.
- [ ] Known limitations include unsupported applications, clipboard decision, and unavailable automatic translation.
- [ ] Artifact checksum, source commit, environment, and rollback information are recorded.

**Verification:** all W-15 commands plus the complete physical matrix in `docs/windows-platform-plan-v2.md`.
