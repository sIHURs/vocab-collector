# Ubuntu 24.04 development handoff

## Current status

Plan A defines the Linux crate, desktop target selection, unsupported provider contracts, and the Ubuntu CI lane. The Linux adapter is a static skeleton: every native capability reports `false`, and every provider returns a typed `PlatformError::Unsupported` result. It contains no AT-SPI, Portal, OCR, translation, permission, session, or native window implementation.

No Linux code or target was compiled, tested, or run during Plan A on macOS. Hosted CI is intended to establish only build and automated contract status on its Ubuntu runner. All native runtime, permission, desktop-session, and packaging results below remain unverified until Plan B runs on an Ubuntu 24.04 physical machine with GNOME Wayland.

## Plan B machine setup

Target environment:

- Ubuntu 24.04 on a physical machine
- GNOME with a Wayland session
- Rust 1.98 or newer
- Node.js 22
- pnpm 11.19.0

Install the baseline Tauri build dependencies:

```bash
sudo apt-get update
sudo apt-get install --yes build-essential curl wget file libssl-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev patchelf
```

Install Rust 1.98+, Node.js 22, and pnpm 11.19.0 using the machine's approved toolchain managers, then install repository dependencies:

```bash
pnpm install --frozen-lockfile
```

These setup instructions are prepared for Plan B and were not executed on Ubuntu during Plan A.

## First target-machine checks

Run these commands on Ubuntu, not on macOS:

```bash
cargo test -p vocab-platform-linux
cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-windows -- -D warnings
cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-windows
cargo build --workspace --exclude vocab-platform-macos --exclude vocab-platform-windows
pnpm check
pnpm test
pnpm build
pnpm tauri dev
```

Passing the automated commands will confirm the skeleton and shared application compile on that Ubuntu environment; it will not prove native capture support. `pnpm tauri dev` is the first Plan B startup check and is currently unverified.

## Deferred physical-machine verification

Every item in this section is explicitly deferred to Plan B and remains unverified:

- [ ] Application startup and shared SQLite, Today, Vocabulary, Review, Settings, and manual capture behavior
- [ ] GNOME Wayland session detection and global shortcut behavior
- [ ] AT-SPI selection text, Unicode, geometry, and unsupported-element behavior
- [ ] XDG Desktop Portal screenshot consent and cancellation
- [ ] Explicit OCR confirmation, in-memory image handling, and OCR results
- [ ] Translation provider selection and offline/error behavior
- [ ] Permission guidance and recovery
- [ ] Non-activating floating-window focus, placement, work areas, and mixed-scale displays
- [ ] Firefox, Chromium, GNOME Text Editor, LibreOffice, and PDF-reader compatibility matrix
- [ ] `.deb` packaging, installation, launch, upgrade, and uninstall

Do not mark a Linux capability `true` until its provider and matching Ubuntu tests exist and the relevant physical-machine checks have been recorded.
