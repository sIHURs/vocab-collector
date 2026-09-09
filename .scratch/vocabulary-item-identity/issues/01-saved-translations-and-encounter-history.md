# 01: Preserve saved translations and Encounter translation history in Vocabulary detail

**What to build:** After capture, a reader can inspect the saved translation by target language and see the translation actually saved with a new Encounter. Repeated same-language saves update the default translation without rewriting historical snapshots.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Expand storage and application contracts to retain translations per target language and optional immutable Encounter translation snapshots, while preserving existing identity semantics until Ticket 02.
- [x] Adapt Manual Capture, Selection Capture, Region OCR Capture, and their presentation/backend adapters to persist final explicitly saved values. Blank translation does not erase an existing translation.
- [x] Vocabulary detail displays the saved translations grouped by language and available Encounter snapshots; it does not generate additional translations.
- [x] Migrate legacy item translations into the expanded representation without fabricating historical Encounter snapshots. Legacy unknowns remain omitted.
- [x] Maintain working list/review/default-translation compatibility until Ticket 04. Existing Undo must not leave newly introduced translation mutations behind; retain enough operation provenance for the complete lifecycle Undo in Ticket 06.
- [x] Keep future sync schema and outgoing representations compatible with the expanded model; do not implement cloud transport.
- [x] Verify persistence across restart, same-language replacement, empty-translation saves, historical snapshot immutability, capture/translation Undo, and legacy migration using real storage tests plus detail presentation tests.

## Scope and completion

Follow the confirmed Vocabulary Item identity and Achieved recapture specification (Q1–Q15) and the approved ticket breakdown. This ticket is one slice of the complete feature; do not expand it into cloud sync, linguistic lemmatization, or a capture redesign.

Run this ticket only when its blockers are complete. If this is the last remaining slice, run the full integrated acceptance matrix, including migration/restart/save/review/Undo and all capture presentations; report platform validation limitations explicitly.

## Completion evidence

Implemented and reviewed on `codex/windows-platform-v2`. See [implementation and validation notes](../implementation-notes.md) for the shared acceptance results, migration evidence, and platform limitations.
