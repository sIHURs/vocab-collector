# Region OCR Capture specification

## Problem Statement

Vocab Collector currently treats OCR as a fallback after Selection Capture and automatically chooses a fixed region around the pointer. The pointer may have moved before OCR starts, the chosen region may not match the vocabulary item, and users cannot intentionally choose Selection Capture or OCR before beginning. The single shortcut setting also cannot express the two distinct user intentions.

## Solution

Provide two explicit Native Capture modes. Selection Capture remains the recommended way to collect text already selected in an application. Region OCR Capture has its own shortcut and lets the user draw a single-display screen region around one vocabulary item. Selection failure recommends Region OCR Capture but never starts it automatically.

After recognition, the user edits a Vocabulary field and an optional Context sentence field. OCR Confirmation authorizes translation. The translated draft remains editable and is saved only through an explicit Save Capture action.

## User Stories

1. As a reader, I want Selection Capture to remain the recommended capture method, so that ordinary text selection stays fast.
2. As a reader, I want a separate Region OCR Capture shortcut, so that I can choose OCR before capture begins.
3. As an existing user, I want my saved capture shortcut to become my Selection Capture shortcut, so that an upgrade does not change my habit.
4. As a reader, I want to disable either shortcut independently, so that the application does not reserve combinations I do not use.
5. As a reader, I want shortcut conflicts reported independently, so that one failed shortcut does not disable the other capture mode.
6. As a reader, I want Selection Capture failures to recommend OCR, so that I know how to continue.
7. As a reader, I want a Start OCR button after Selection Capture fails, so that OCR remains usable without a registered shortcut.
8. As a reader, I want unsupported OCR omitted from recovery choices, so that the UI does not promise unavailable behavior.
9. As a reader, I want an overlay to tell me to draw around one vocabulary item, so that the required action is clear.
10. As a reader, I want to drag a rectangular screen region, so that OCR examines the text I intended.
11. As a multi-monitor user, I want one capture region constrained to the display where dragging began, so that scaling and coordinates remain predictable.
12. As a reader, I want the instruction overlay excluded from recognition, so that application UI does not contaminate OCR text.
13. As a reader, I want Escape to cancel Region OCR Capture, so that no partial capture is retained.
14. As a reader, I want tiny or click-only regions rejected without ending selection, so that accidental input is recoverable.
15. As a reader, I want recognized text placed into an editable Vocabulary input, so that I can correct OCR mistakes.
16. As a reader, I want multiple recognized words called out, so that I can retain only the vocabulary item I intended.
17. As a reader, I want an optional Context sentence input, so that I can paste or type useful context when the region contains only one word.
18. As a privacy-conscious reader, I want recognition to remain local and in memory, so that screenshots are not written to files or content logs.
19. As a reader, I want translation blocked until OCR Confirmation, so that unconfirmed recognized text is not sent to a provider.
20. As a reader, I want translation shown after confirmation, so that I can review it before saving.
21. As a reader, I want Vocabulary, Context sentence, and Translation editable until save, so that the Encounter contains the information I intend.
22. As a reader, I want a Translate Again action after changing Vocabulary, so that translation can be brought back into sync deliberately.
23. As a reader, I want translation failure to preserve my OCR draft, so that I can retry, enter a translation, or save without one.
24. As a reader, I want failed recognition to offer Try Again and Manual Capture, so that I can recover without losing control.
25. As a reader, I want one explicit Save Capture action, so that confirmation and persistence remain distinct decisions.
26. As a reader, I want stale and duplicate actions ignored, so that overlapping capture attempts cannot save the wrong Encounter.

## Implementation Decisions

- Native Capture has two explicit modes: Selection Capture and Region OCR Capture. Neither automatically invokes the other.
- Selection Capture remains recommended and defaults to `Alt+Shift+V`. Region OCR Capture defaults to `Alt+Shift+O`.
- Both shortcuts are independently configurable and disableable, cannot be equal, and have independent registration, conflict, rollback, and startup recovery behavior.
- The legacy capture shortcut migrates to the Selection Capture shortcut. Failure to register the new OCR default does not prevent startup and does not select an undisclosed replacement.
- Both a Region OCR shortcut and the Selection failure Start OCR action start the same request mode.
- Region selection uses a full-desktop overlay and a non-captured, non-focusable instruction strip. The rectangle is constrained to the display where dragging begins.
- Region OCR consumes an explicit logical screen rectangle rather than a pointer. The platform adapter captures only that region in memory and returns recognized text without screenshot artifacts or content diagnostics.
- The intended region contains one vocabulary item. All recognized text initially populates Vocabulary. Context sentence is optional and initially empty. Multiple recognized words produce guidance rather than automatic importance ranking.
- OCR Confirmation changes the request from an editable unconfirmed OCR draft to a translation-eligible Capture Candidate. Translation and persistence are forbidden before that transition.
- Editing Vocabulary after translation marks the translation potentially stale. Translation is refreshed only through an explicit Translate Again action.
- Save Capture is the only final persistence action. Translation can be edited, omitted, or retried before saving.
- OCR availability remains capability-gated. Windows advertises it only after physical resource, privacy, DPI, multi-monitor, and workflow verification.
- Existing request identity, stale-result rejection, duplicate-action suppression, save-once, and Undo rules remain in force.

## Testing Decisions

- Tests assert external state and user-visible behavior rather than Svelte internals, WinRT implementation details, or exact private state representations.
- Windows presentation tests cover the highest user-facing seam: separate capture starts, overlay instructions, rectangle interaction, cancellation, editable OCR draft, confirmation, translation recovery, and save.
- Desktop command-contract tests cover capture mode routing, screen-region serialization, capability gates, request identity, OCR Confirmation, translation authorization, and persistence.
- Application and coordinator tests cover state transitions, stale requests, duplicate confirmation, translation retry, corrections, and save-once behavior.
- Settings tests reuse existing domain, storage, application, desktop transaction, and Windows Settings seams to cover migration, independent registration, independent rollback, validation, and disabling.
- Windows adapter tests cover coordinate conversion, display clamping, crop geometry, native resource ownership, timeout cleanup, and content-free diagnostics. Direct3D behavior is not duplicated in presentation tests.
- Physical verification covers shortcut conflicts, real overlay interaction, Windows Graphics Capture, mixed DPI, multiple displays, cancellation, no screenshot artifacts, and resource release.

## Out of Scope

- Automatic OCR fallback after Selection Capture failure.
- Pointer-centred or fixed-size automatic OCR regions.
- Cross-display rectangles or stitched screenshots.
- Automatic selection of the most important word from a region.
- Automatic acquisition of a context sentence outside the selected region.
- Clipboard capture or clipboard restoration.
- Cloud OCR, stored screenshots, or captured-content diagnostics.
- Enabling OCR capability without required physical Windows evidence.

## Further Notes

The full workflow is: start Region OCR Capture, draw a region, edit Vocabulary and optional Context sentence, confirm OCR, review or edit translation, and explicitly save. A Selection Capture failure may recommend and launch this workflow only after a separate user action.
