# Vocab Collector

Vocab Collector helps a reader retain vocabulary encountered while reading. The product treats every sighting as useful context, even when the vocabulary item already exists.

## Language

**Vocabulary Item**:
A normalized word or phrase kept for later review, with an optional translation and learning status.
_Avoid_: Card, entry, capture

**Encounter**:
One occurrence of a vocabulary item in its original sentence and source context. Repeated encounters belong to the same vocabulary item.
_Avoid_: Duplicate, capture record

**Capture Candidate**:
Selected text and its surrounding context that have been resolved enough for the user to confirm or save.
_Avoid_: OCR candidate, raw selection

**Manual Capture**:
A capture entered directly by the user rather than obtained from another application's current selection.
_Avoid_: Fallback capture

**Native Capture**:
A capture initiated by the global shortcut and resolved from the active application through platform-native selection or OCR.
_Avoid_: Automatic capture, background monitoring

**OCR Confirmation**:
The explicit user decision that turns an OCR-derived suggestion into an accepted capture candidate. Nothing is translated or saved before this decision.
_Avoid_: OCR permission, automatic OCR save

**Automatic Translation**:
A translation proposed by a configured Translation Provider for the selected vocabulary text. The surrounding context is not sent for translation. An automatic translation remains an editable part of the capture draft and is not saved until the user explicitly saves the capture.
_Avoid_: Guaranteed translation, background translation

**Manual Translation**:
Translation text entered or edited by the user. Selected text, context, and translation are independently editable; the user confirms their final combination when saving.
_Avoid_: Corrected automatic translation

**Translation Provider**:
A replaceable service that translates selected vocabulary text between a requested source and target language. Provider availability does not imply that its credentials, network connection, quota, or requested language pair are currently usable.
_Avoid_: Windows translator, translation engine

**Review**:
A scheduled recall decision for a vocabulary item, recorded as either forgotten or remembered.
_Avoid_: Quiz, test
