# 03: Resolve unknown source language consistently during capture

**What to build:** A capture with unresolved source language can still be saved without translation, reusing an existing Vocabulary Item only when attribution is unambiguous. Later language detection reconciles records without losing history or merging different known languages.

**Blocked by:** 02 — Share a Vocabulary Item across target languages and consolidate existing duplicates.

**Status:** done

- [x] Use one matching policy for ordinary capture, Achieved preview, and restore/save, considering candidates across lifecycle states.
- [x] Prefer explicit known-language identity; resolve auto only when attribution is unique. Do not treat one Achieved match as unambiguous when another active-language candidate exists.
- [x] Preserve unresolved attribution when ambiguous and allow the agreed temporary unresolved records without a mandatory new confirmation step.
- [x] Reconcile unresolved and resolved identities when later evidence permits, reusing Ticket 02 history/translation/state preservation rules and preventing duplicate Encounters or log counts.
- [x] Preserve draft values and visible language attribution through all three capture paths, including translation failure and OCR Confirmation.
- [x] Verify auto-to-known and known-to-auto cases, exact known matches alongside unresolved records, ambiguous homographs in different languages, mixed Achieved/active candidates, and concurrent reconciliation against actual storage.

## Scope and completion

Follow the confirmed Vocabulary Item identity and Achieved recapture specification (Q1–Q15) and the approved ticket breakdown. This ticket is one slice of the complete feature; do not expand it into cloud sync, linguistic lemmatization, or a capture redesign.

Run this ticket only when its blockers are complete. If this is the last remaining slice, run the full integrated acceptance matrix, including migration/restart/save/review/Undo and all capture presentations; report platform validation limitations explicitly.

## Completion evidence

Implemented and reviewed on `codex/windows-platform-v2`. See [implementation and validation notes](../implementation-notes.md) for the shared acceptance results, migration evidence, and platform limitations.
