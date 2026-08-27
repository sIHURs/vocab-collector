# Windows 11 development handoff

## Current status

Plan A defines the Windows crate, desktop target selection, unsupported provider contracts, and the Windows CI lane. The Windows adapter is a static skeleton: every native capability reports `false`, and every provider returns a typed `PlatformError::Unsupported` result. It contains no COM, UI Automation, Windows Graphics Capture, OCR, translation, permission, DPI, or Win32 window implementation.

No Windows code or target was compiled, tested, or run during Plan A on macOS. Hosted CI is intended to establish only build and automated contract status on its Windows runner. All native runtime, permission, application-compatibility, and packaging results below remain unverified until Plan B runs on a Windows 11 x64 physical machine.

## Plan B machine setup

Target environment:

- Windows 11 x64 on a physical machine
- Visual Studio 2022 Build Tools with Desktop development with C++ and the Windows SDK
- Microsoft WebView2 Runtime
- Rust 1.98 or newer using the MSVC toolchain
- Node.js 22
- pnpm 11.19.0

In a Developer PowerShell, select the Rust target and install repository dependencies after the tools above are available:

```powershell
rustup default 1.98.0-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc
corepack enable
corepack prepare pnpm@11.19.0 --activate
pnpm install --frozen-lockfile
```

These setup instructions are prepared for Plan B and were not executed on Windows during Plan A.

## First target-machine checks

Run these commands on Windows, not on macOS:

```powershell
cargo test -p vocab-platform-windows
cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings
cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
cargo build --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
pnpm check
pnpm test
pnpm build
pnpm tauri dev
```

Passing the automated commands will confirm the skeleton and shared application compile on that Windows environment; it will not prove native capture support. `pnpm tauri dev` is the first Plan B startup check and is currently unverified.

## Deferred physical-machine verification

Every item in this section is explicitly deferred to Plan B and remains unverified:

- [ ] Application startup and shared SQLite, Today, Vocabulary, Review, Settings, and manual capture behavior
- [ ] COM initialization and thread ownership
- [ ] UI Automation TextPattern/TextPattern2 selection, Unicode/UTF-16, geometry, and inaccessible-control behavior
- [ ] Global shortcut registration, conflicts, persistence, and repeat suppression
- [ ] Windows Graphics Capture consent and cancellation
- [ ] Explicit OCR confirmation, in-memory image handling, and OCR results
- [ ] Translation provider selection and offline/error behavior
- [ ] Permission guidance and recovery
- [ ] Non-activating floating-window focus, task-switcher behavior, per-monitor DPI, negative coordinates, and mixed-scale displays
- [ ] Edge, Chrome, Firefox, Notepad, Microsoft Word, and PDF-reader compatibility matrix
- [ ] NSIS packaging, installation, launch, WebView2 behavior, upgrade, and uninstall

Do not mark a Windows capability `true` until its provider and matching Windows tests exist and the relevant physical-machine checks have been recorded.
