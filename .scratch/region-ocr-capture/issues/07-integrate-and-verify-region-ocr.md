# 07: Integrate and verify Region OCR Capture

**What to build:** Prove the complete dual-mode Capture experience through automated and physical Windows verification, enabling OCR capability only when its evidence gate passes.

**Blocked by:** 06: Complete OCR translation, save, and recovery.

**Status:** blocked-by-physical-windows-verification

- [x] All Windows-linkable Rust, frontend, formatting, and content-safety checks pass.
- [ ] Upgrade, shortcut conflict, cancellation, mixed-DPI, and multi-display behavior is verified. Upgrade, conflict, cancellation, and coordinate transforms have automated coverage; live mixed-DPI and multi-display behavior is not run.
- [ ] Native resources and screenshot artifact absence have physical evidence.
- [x] Capability and development status accurately reflect verified behavior.

**Automated evidence (2026-09-04):** `pnpm --dir ui check`; `pnpm --dir ui test` (70 tests); `pnpm --dir ui build`; `cargo fmt --all --check`; `cargo test --workspace --exclude vocab-platform-macos`; `cargo clippy --workspace --exclude vocab-platform-macos --all-targets -- -D warnings`; `git diff --check`. The unfiltered workspace test cannot link the macOS Swift FFI test binary on Windows.

**Physical evidence still required:** exercise both shortcuts and the recovery button on the target Windows build; draw/cancel/tiny-region flows on single, mixed-DPI, and multi-display layouts; inspect WGC/WinRT resource release across success, cancellation, timeout, and failure; audit the filesystem and logs for screenshot or captured-content artifacts. `screenshot_ocr` must remain false until these checks pass.
