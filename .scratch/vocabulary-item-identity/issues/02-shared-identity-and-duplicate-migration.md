# 02: Share a Vocabulary Item across target languages and consolidate existing duplicates

**What to build:** Capturing English robust in Chinese and German produces one Vocabulary Item, two Encounters, two visible translations, and one learning state. Existing duplicates upgrade to that same model without breaking startup, saving, or review.

**Blocked by:** 01 — Preserve saved translations and Encounter translation history in Vocabulary detail.

**Status:** done

- [x] Adopt normalized lemma plus known source language within the owning vocabulary. Retain current case/whitespace normalization; different known source languages remain distinct.
- [x] Apply the identity rule to all capture entry points and existing Achieved lookup/restore paths; target language no longer prevents matching. Retain existing explicit Achieved safeguards until Ticket 05 changes the interaction.
- [x] Migrate stored deduplication keys and consolidate duplicates atomically. Choose deterministic surviving identities; redirect Encounter/review/other references and maintain valid outbox/future schema representations.
- [x] Preserve available translations and history. Active learning takes precedence over Achieved; conflicting active schedules retain the earlier due schedule without arbitrarily mixing review parameters. All-Achieved groups retain the later deletion deadline. Document deterministic tie-breaking and any available legacy translation ordering evidence.
- [x] Consolidation does not create successful saves, inflate Vocabulary log/review totals, or fabricate unknown history. Item counts may decrease. Tombstones and deleted records must not resurrect through uniqueness changes.
- [x] Handle demonstrably invalid test fixtures narrowly; do not equate display text with lemma globally or clear the database.
- [x] Verify old-database upgrade, migration failure atomicity/recovery, repeat startup, referential integrity, review eligibility, same-word cross-target captures, and concurrent duplicate save prevention. Define the local schema upgrade boundary for older clients.

## Scope and completion

Follow the confirmed Vocabulary Item identity and Achieved recapture specification (Q1–Q15) and the approved ticket breakdown. This ticket is one slice of the complete feature; do not expand it into cloud sync, linguistic lemmatization, or a capture redesign.

Run this ticket only when its blockers are complete. If this is the last remaining slice, run the full integrated acceptance matrix, including migration/restart/save/review/Undo and all capture presentations; report platform validation limitations explicitly.

## Completion evidence

Implemented and reviewed on `codex/windows-platform-v2`. See [implementation and validation notes](../implementation-notes.md) for the shared acceptance results, migration evidence, and platform limitations.
