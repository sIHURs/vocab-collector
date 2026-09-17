# 06: Complete OCR translation, save, and recovery

**What to build:** Complete the confirmed OCR draft through editable translation, retry, explicit save, and recoverable recognition or translation failures.

**Blocked by:** 05: Deliver editable OCR Confirmation.

**Status:** complete

- [x] Confirmed Vocabulary can be translated, edited, retried, or saved without translation.
- [x] Editing Vocabulary never silently retranslates and exposes Translate Again.
- [x] Recognition failure offers Try Again and Manual Capture.
- [x] Save Capture persists once and existing Undo behavior remains available.
- [x] The pointer-centred OCR product path is removed.

**Verification:** `pnpm --dir ui check`; 62 focused shared/Windows capture tests; `cargo test -p vocab-desktop` (38 tests).
