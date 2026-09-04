# 0003: Use explicit Selection Capture and Region OCR Capture modes

## Status

Accepted

## Context

Pointer-centred OCR was designed as a fallback after Selection Capture. By the time OCR starts, the pointer can be unrelated to the intended vocabulary item. Automatic fallback also combines two different user intentions and makes one shortcut responsible for incompatible interactions.

## Decision

Native Capture exposes two explicit modes. Selection Capture reads an existing application selection and remains recommended. Region OCR Capture is started independently and requires the user to draw a single-display rectangle containing one vocabulary item.

Selection Capture may recommend Region OCR Capture after failure, but OCR begins only through a separate shortcut or explicit Start OCR action. OCR produces an editable vocabulary draft and optional context. OCR Confirmation is required before translation, and Save Capture remains a separate final decision.

## Consequences

- Settings and desktop shortcut registration must support two independent actions.
- OCR no longer depends on a pointer coordinate or automatic fixed-size region.
- A screen-region selection presentation and region-based platform contract are required.
- Selection failures remain recoverable without silently capturing the screen.
- Users perform one additional spatial action in exchange for predictable OCR intent.
