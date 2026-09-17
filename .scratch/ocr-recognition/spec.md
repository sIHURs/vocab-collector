# Region OCR Capture: English word recognition

Status: missing-pack installation prompt verified; English recognition tests require the English OCR pack

## Report

The user selects `consisting` on a GitHub page in Chrome at 100% Windows display scaling and receives `OCR did not find readable text`.

## Findings

- The Windows provider ignored the configured source language and selected the Windows profile recognizer (`zh-Hans-CN` on this machine). Only `zh-Hans-CN` and `de-DE` OCR packs are installed. The full synthetic word fixture was misrecognized with Chinese and recognized correctly with German.
- An accurately bounded 76×20 pixel crop of the synthetic word returned no candidates even with German. The same crop recognized `consisting` after 3× in-memory enlargement.
- The earlier overlay press-animation and source-window fixes remain in the working tree. These are separate from recognition quality.

## Changes

Pass the saved source language to the OCR provider. Match the requested language or a regional variant of that language. At the user's request, cross-language fallback has been removed: missing English now reports that the English Optical Character Recognition (OCR) feature must be installed through Windows language settings. Validate the language before capturing the source window.

Enlarge short image crops in memory, capped by Windows OCR's maximum image dimension. Map recognized bounds back through the enlargement factor. Do not save captured images or text to disk.

## Native reproduction

`cargo test -p vocab-platform-windows native_ocr_reads_consisting -- --ignored --nocapture`

This opt-in test creates a synthetic Windows window, runs the actual WGC → image preparation → Windows OCR pipeline on a tight word crop, and checks both the word and returned bounds. It passed with the previous German fallback. It now requires the English OCR pack to be installed. Without image enlargement, the identical crop produced zero candidates. Temporary image dumps and debug logging were removed.

The supplied screenshot's visible word crop (75×20 pixels) also passed Windows OCR with the previous fallback via `native_ocr_reads_consisting_from_reported_image`, with `VOCAB_OCR_PNG` pointing to the user-supplied PNG. This test now also requires the English pack. No copy of that image was retained. The user's original live Chrome region has not been replayed automatically.

Windows desktop, Windows/Linux adapter and shared application regression suites passed for the initial recognition changes. After removing fallback, all 15 Windows provider unit tests passed, and `native_missing_english_pack_reports_installation_instructions` passed against the actual installed languages (`de-DE`, `zh-Hans-CN`). macOS native tests require its native bridge and cannot link on Windows. Automatic source-language mode keeps the Windows profile recognizer.
