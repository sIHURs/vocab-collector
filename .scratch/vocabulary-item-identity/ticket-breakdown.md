# Vocabulary Item identity: approved ticket breakdown

Status: approved by the user on 2026-09-10; published as six ready-for-agent local tickets.

Source: the confirmed Vocabulary Item identity and Achieved recapture specification, Q1–Q15, and ADRs on identity, Achieve, and the Vocabulary log.

## Delivery approach

Each ticket includes persistence, application/API behavior, affected presentations, and meaningful verification for its user-visible slice. Shared changes must keep existing callers working; any small prefactoring belongs at the start of the slice that needs it. No speculative standalone architecture rewrite is required.

Ticket 01 expands translation storage compatibly before Ticket 02 changes identity. This avoids attempting the translation representation change and duplicate identity migration in one oversized ticket. All tickets are required for the final feature; intermediate delivery is not a claim that the full specification is complete.

## 01: Preserve saved translations and Encounter translation history in Vocabulary detail

**Blocked by:** None (can start immediately).

**What to build:** After capture, a reader can inspect the saved translation by target language and see the translation actually saved with a new Encounter. Repeated same-language saves update the default translation without rewriting historical snapshots.

- Expand storage and application contracts to retain translations per target language and optional immutable Encounter translation snapshots, while preserving existing identity semantics until Ticket 02.
- Adapt Manual Capture, Selection Capture, Region OCR Capture, and their presentation/backend adapters to persist final explicitly saved values. Blank translation does not erase an existing translation.
- Vocabulary detail displays the saved translations grouped by language and available Encounter snapshots; it does not generate additional translations.
- Migrate legacy item translations into the expanded representation without fabricating historical Encounter snapshots. Legacy unknowns remain omitted.
- Maintain working list/review/default-translation compatibility until Ticket 04. Existing Undo must not leave newly introduced translation mutations behind; retain enough operation provenance for the complete lifecycle Undo in Ticket 06.
- Keep future sync schema and outgoing representations compatible with the expanded model; do not implement cloud transport.
- Verify persistence across restart, same-language replacement, empty-translation saves, historical snapshot immutability, capture/translation Undo, and legacy migration using real storage tests plus detail presentation tests.

## 02: Share a Vocabulary Item across target languages and consolidate existing duplicates

**Blocked by:** 01 — Preserve saved translations and Encounter translation history in Vocabulary detail.

**What to build:** Capturing English robust in Chinese and German produces one Vocabulary Item, two Encounters, two visible translations, and one learning state. Existing duplicates upgrade to that same model without breaking startup, saving, or review.

- Adopt normalized lemma plus known source language within the owning vocabulary. Retain current case/whitespace normalization; different known source languages remain distinct.
- Apply the identity rule to all capture entry points and existing Achieved lookup/restore paths; target language no longer prevents matching. Retain existing explicit Achieved safeguards until Ticket 05 changes the interaction.
- Migrate stored deduplication keys and consolidate duplicates atomically. Choose deterministic surviving identities; redirect Encounter/review/other references and maintain valid outbox/future schema representations.
- Preserve available translations and history. Active learning takes precedence over Achieved; conflicting active schedules retain the earlier due schedule without arbitrarily mixing review parameters. All-Achieved groups retain the later deletion deadline. Document deterministic tie-breaking and any available legacy translation ordering evidence.
- Consolidation does not create successful saves, inflate Vocabulary log/review totals, or fabricate unknown history. Item counts may decrease. Tombstones and deleted records must not resurrect through uniqueness changes.
- Handle demonstrably invalid test fixtures narrowly; do not equate display text with lemma globally or clear the database.
- Verify old-database upgrade, migration failure atomicity/recovery, repeat startup, referential integrity, review eligibility, same-word cross-target captures, and concurrent duplicate save prevention. Define the local schema upgrade boundary for older clients.

## 03: Resolve unknown source language consistently during capture

**Blocked by:** 02 — Share a Vocabulary Item across target languages and consolidate existing duplicates.

**What to build:** A capture with unresolved source language can still be saved without translation, reusing an existing Vocabulary Item only when attribution is unambiguous. Later language detection reconciles records without losing history or merging different known languages.

- Use one matching policy for ordinary capture, Achieved preview, and restore/save, considering candidates across lifecycle states.
- Prefer explicit known-language identity; resolve auto only when attribution is unique. Do not treat one Achieved match as unambiguous when another active-language candidate exists.
- Preserve unresolved attribution when ambiguous and allow the agreed temporary unresolved records without a mandatory new confirmation step.
- Reconcile unresolved and resolved identities when later evidence permits, reusing Ticket 02 history/translation/state preservation rules and preventing duplicate Encounters or log counts.
- Preserve draft values and visible language attribution through all three capture paths, including translation failure and OCR Confirmation.
- Verify auto-to-known and known-to-auto cases, exact known matches alongside unresolved records, ambiguous homographs in different languages, mixed Achieved/active candidates, and concurrent reconciliation against actual storage.

## 04: Display the preferred saved translation in Vocabulary and Review

**Blocked by:** 01 — Preserve saved translations and Encounter translation history in Vocabulary detail.

**What to build:** Vocabulary lists and revealed Review answers display the current target-language translation when saved, otherwise the most recently saved nonempty translation with its language. The detail sidebar continues to show all saved target languages.

- Apply the same selection rule to affected Vocabulary list states and revealed Review answers across presentations.
- Preserve existing empty-translation behavior when no translation is available. Do not expose the answer before reveal.
- Changing target-language settings changes selection only; it does not call translation providers, add Encounters, create items, or reset progress.
- Use deterministic ordering for legacy translations without claiming unavailable timestamps. Keep backend projections and presentation test doubles consistent.
- Verify preferred-language, fallback, empty, settings-change, and review-reveal behavior. Ticket 01's expanded model allows this slice to start without waiting for identity consolidation.

## 05: Recapture Achieved Vocabulary Items with the normal Save action

**Blocked by:** 03 — Resolve unknown source language consistently during capture.

**What to build:** Capturing an Achieved Vocabulary Item shows a short notice in the existing window before Save. Saving returns that same item to Learning with a fresh review schedule and new Encounter, using unchanged normal controls.

- Cover Manual Capture, Selection Capture, and Region OCR Capture; preserve OCR Confirmation, editing, translation, cancellation, and normal Save labels. Recheck after candidate/language edits and ignore stale asynchronous lookup responses.
- Opening/cancelling the window changes nothing. The ordinary Save action conveys consent; no separate Return to Learning confirmation.
- Revalidate the previewed identity at Save. If still Achieved, atomically clear mastery/achievement/deletion markers, initialize review parameters due now, store translations and Encounter, and update successful-save counts.
- If that same item is now Learning or Mastered, perform ordinary repeated capture without resetting progress. If it was deleted or attribution became ambiguous, retain the draft and require rechecking without silent replacement or retargeting.
- Preserve lifetime review/Encounter history; refresh affected list and review views after successful saves.
- Keep an operation snapshot/version sufficient for Ticket 06. Until complete rollback is supported for a state-changing save, fail safely rather than perform a destructive partial Undo.
- Verify original cross-target robust scenario, all capture entry points, candidate edits, cancellation, failures, stale preview/state transitions, and atomic persistence/count updates. Use storage/application tests and UI interaction tests, not UI mocks alone.

## 06: Completely undo capture changes without overwriting later work

**Blocked by:** 05 — Recapture Achieved Vocabulary Items with the normal Save action.

**What to build:** Undo fully reverses an uncontested save, including translation changes and Achieved-to-Learning restoration. If later work conflicts, it explains refusal and leaves data unchanged.

- Restore prior target-language defaults, learning state, review parameters, achievement/deletion timestamps, and withdraw the new Encounter and its successful-save contribution together.
- Preserve earlier translation snapshots and histories; respect Vocabulary log date and coverage semantics. Restore identity/references safely where this save performed language reconciliation, or refuse the entire operation if safe restoration is no longer possible.
- Detect subsequent save, translation edit, review, or lifecycle change, including deletion. Refuse the whole Undo rather than partially reverting or overwriting later state.
- Present success/refusal consistently through supported capture Undo controls; preserve valid UI state and do not close the flow as if a refused Undo succeeded.
- Verify successful rollback and every conflicting-change category with real storage transactions; cover translation creation/replacement, original new-item capture, auto reconciliation, Achieved restoration, and count invariants.
- Verify this slice end to end through upgrade/restart/save/Undo, alongside targeted UI checks. Confirm no obsolete target-language identity checks or stale single-translation write paths remain in the affected operation paths; record any platform validation limitations honestly.

## Dependency frontier

- Start: 01.
- After 01: 02 and 04 can proceed independently.
- Then: 02 → 03 → 05 → 06.
- Feature completion requires all six tickets, including 04. Ticket 06's local behavior does not depend on preferred-translation presentation, but full-feature verification must include it.
- Completion gate: whichever slice finishes last runs the full confirmed acceptance matrix on the integrated feature, including upgrade/restart/save/review/Undo and all presentations. This final verification does not block independent implementation of 04 and 06.

## Publication

The user approved this granularity and these blocking edges on 2026-09-10. Six independent ready-for-agent tickets have been published in the local issue tracker. Start with 01; ready-for-agent indicates a fully specified ticket, not permission to ignore its blockers. The confirmed specification is unchanged.
