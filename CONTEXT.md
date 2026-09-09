# Vocab Collector

Vocab Collector helps a reader retain vocabulary encountered while reading. The product treats every sighting as useful context, even when the vocabulary item already exists.

## Language

**Vocabulary Item**:
A normalized word or phrase in a particular source language kept for later review, with one shared learning status across its target-language translations. Choosing a different target translation language does not create a different Vocabulary Item.
_Avoid_: Card, entry, capture

**Encounter**:
One occurrence of a vocabulary item in its original sentence and source context, optionally retaining the translation saved on that occasion. Repeated encounters belong to the same vocabulary item regardless of target translation language.
_Avoid_: Duplicate, capture record

**Capture Candidate**:
Selected text and its surrounding context that have been resolved enough for the user to confirm or save.
_Avoid_: OCR candidate, raw selection

**Manual Capture**:
A capture entered directly by the user rather than obtained from another application's current selection.
_Avoid_: Fallback capture

**Native Capture**:
A capture initiated by an explicitly chosen Selection Capture or Region OCR Capture action.
_Avoid_: Automatic capture, background monitoring

**Selection Capture**:
A recommended Native Capture mode that reads the selection already made in the active application through platform-native accessibility APIs.
_Avoid_: Word Capture, normal capture, primary capture

**Region OCR Capture**:
A Native Capture mode in which the user draws a screen region containing a vocabulary item before text is recognized from that region.
_Avoid_: OCR fallback, automatic OCR, pointer OCR

**OCR Confirmation**:
The explicit user decision that accepts the editable vocabulary and optional context produced during Region OCR Capture. Nothing is translated or saved before this decision.
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

**Review Insight**:
A short explanation or actionable suggestion derived from completed reviews and encounter history. It appears only after an answer is revealed and never substitutes for a review rating.
_Avoid_: Hint, score, AI explanation

**Vocabulary log**:
A daily history of successful vocabulary saves that have not been undone, including repeated saves of the same Vocabulary Item. It distinguishes known daily counts from incomplete or unknown history.
_Avoid_: Unique-word count, review activity, streak

**Mastered Vocabulary Item**:
A vocabulary item whose current learning status indicates mastery. Mastery makes the item eligible for Achieve but does not itself remove or delete the item.
_Avoid_: Achieved item, completed item

**Achieved Vocabulary Item**:
A Mastered Vocabulary Item that the user has intentionally placed in a temporary, reviewable list before permanent deletion. The intent may be expressed directly for one or more items or granted in advance through the automatic Achieve setting.
_Avoid_: Mastered item, deleted item, trash item

**Unachieve**:
The user decision that returns an Achieved Vocabulary Item to Mastered status before permanent deletion.
_Avoid_: Restore to learning, undo review

**Recapture an Achieved Vocabulary Item**:
The explicit capture-window decision that returns an Achieved Vocabulary Item to Learning and records the new Encounter atomically. This is distinct from Unachieve because the new Encounter is evidence that active learning should resume.
_Avoid_: Automatic restore, duplicate capture
