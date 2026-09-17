# Vocabulary Item identity and Achieved recapture

Status: confirmed by the user on 2026-09-10; ready for implementation planning.

## Intended outcome

Within a user's vocabulary, one normalized lemma in one source language identifies one Vocabulary Item regardless of target translation language. Captures add Encounters and saved translations to that item. Recapturing an Achieved Vocabulary Item uses the existing Save action to restart its learning, without creating a parallel Learning item.

This document defines the product target, not a claim that the current implementation already meets it.

## Confirmed goals

- When a reader captures an Achieved Vocabulary Item, show a brief notice inside the existing capture window.
- Keep the normal capture controls and workflow. Saving the capture is the explicit decision to return the same Vocabulary Item to Learning and record a new Encounter; do not require a separate confirmation step.
- Merely opening or cancelling the capture must not change the item's learning state.
- Saving must remove the item from the Achieved list and restart learning, rather than leave an Achieved item alongside a newly created Learning item.
- The user regards the two robust examples with different translation languages as the same vocabulary being learned.

## Verified current behavior and limits of diagnosis

- Production deduplication includes normalized lemma, source language, and target language.
- The current Word model stores one target language and one translation.
- Achieved lookup and restore validate target language. Ordinary storage capture guards against an exact-key Achieved match.
- The Windows capture window currently checks for Achieved on Save and offers a separate Return to Learning action.
- Restore clears mastery and achievement timestamps but retains the prior review state.
- The Supabase schema includes target language in its per-user uniqueness constraint.
- A local default-database record displays robust but uses lemma qa-fixture-26-robust. An isolated in-memory reproduction confirmed this mismatch can cause the reported duplicate behavior. That local record was not Achieved when inspected, so it does not establish the cause of the user's screenshots.
- Screenshots alone do not establish stored language fields.

## Decision tree

### Round 1: settled decisions

1. Adopt normalized lemma plus source language for every Vocabulary Item, independently of target translation language. Encounter counts and learning state are shared.
2. Retain multiple target-language translations under one Vocabulary Item. The Vocabulary detail sidebar should display translations the user has actually saved through capture; this does not authorize automatically generating other translations. Translation display and history follow Round 2.
3. Include consolidation of existing duplicate Vocabulary Items, preserving Encounter and review history. Conflict policies follow Round 2.

### Round 2: settled decisions

- Vocabulary lists and revealed Review answers prefer the current target-language translation. If absent, show the most recently saved nonempty translation with its language. Do not generate translations or Encounters merely because settings change.
- Per target language, show the latest explicitly saved nonempty translation. Preserve the translation saved with each new Encounter as historical context. Saving without translation does not erase an existing translation. Legacy per-Encounter translation history may be unavailable and must not be fabricated.
- Existing duplicate consolidation retains an active learning state over Achieved, with the earlier due schedule among conflicting active learning schedules. Preserve all history. If all candidates are Achieved, retain Achieved with the later deletion deadline. Consolidation is not a new capture.
- Existing data is test data. Prioritize valid data structure; migration must not cause app errors or break startup, saving, or review. This is not authorization to discard the database.
- Recapturing an Achieved Vocabulary Item resets current review parameters to new-item defaults, due from now, while retaining prior Encounter and review history.
- Undo should withdraw the new Encounter and restore prior learning state, review schedule, and deletion deadline. Translation rollback and subsequent-change protection follow Round 4.

### Additional verified facts

- Current normalization only folds case and normalizes whitespace; it is not morphological lemmatization.
- Without a translation result, source language can remain auto. Ordinary capture uses exact identity, whereas Achieved lookup has a limited auto fallback. This inconsistency must be addressed explicitly.
- Cloud sync transport is not implemented: the repository contains a placeholder sync crate and future Supabase schema. Updating that schema is different from migrating a verified live cloud deployment.

### Round 3: settled decisions

- Preserve current case/whitespace normalization. No morphological lemmatization, new punctuation/Unicode normalization, or sense-based splitting of same-source-language homographs in this change.
- Keep saving without translation when source language is unresolved. Reuse a same-lemma item only when source attribution is unique and unambiguous across candidate items; never silently merge distinct known source languages. Ambiguous saves remain language-unresolved until later evidence permits reconciliation. Temporary unresolved records are an accepted tradeoff. Ordinary capture and Achieved detection must share matching rules without adding a mandatory confirmation step.
- Cover the shared data model, Manual Capture, Selection Capture, Region OCR Capture, Vocabulary lists/detail sidebars, and revealed Review answers. Preserve OCR Confirmation before entering the normal capture flow.
- Update the repository's future sync schema consistently, but do not implement cloud sync.

### Round 4: settled decisions

- Undo reverses all effects of this save together, including its Encounter, default translation changes, learning state, review schedule, and deletion deadline. If a subsequent save, translation edit, review, or lifecycle change prevents safe full restoration, refuse the entire Undo with an explanation; do not partially revert or overwrite later changes.
- Preserve available legacy item-level translations. Do not infer historical translations for legacy Encounters; omit unavailable historical translation fields. New Encounters retain their actual saved translation snapshot.
- Revalidate the candidate against current backend state at Save and apply the result atomically. If the same item remains Achieved, reset it to Learning and append the Encounter. If it is now Learning or Mastered, perform an ordinary repeated capture without resetting its review progress. If the previously matched item was deleted or attribution has become ambiguous, retain the draft and ask the user to recheck; do not silently create a replacement or save to another item.

## Acceptance scenarios

| Scenario | Required result |
| --- | --- |
| Save English robust with Chinese, then German translation | One Vocabulary Item, two successful-save Encounters, both translations available in the detail sidebar, one learning state. |
| Save the same translation five times | Five Encounters; translation count is not the save count. |
| Change the target-language setting | Preferred display changes; no new Vocabulary Item, Encounter, translation request, or learning reset. |
| Preferred translation is absent | Display the most recently saved nonempty translation with its language; preserve existing empty-translation behavior when none exists. |
| Save a revised Chinese translation | Chinese default becomes the new nonempty value; prior new-format Encounter snapshots remain unchanged. |
| Save without translation | Record the Encounter without erasing existing saved translations. |
| Open capture for an Achieved item, including through another target language | Show a brief inline notice before the normal Save decision; preserve editing/translation controls and allow cancellation without mutation. |
| Save that Achieved item | Same item identity, new Encounter, Learning with initial review parameters due now, no Achieved/deletion markers, historical reviews retained. |
| Cancel or fail that save | No partial transition, translation write, or Encounter write; failed saves retain the draft. |
| Fully undo an uncontested successful save | Restore its prior translations and lifecycle/review state; withdraw its Encounter and its contribution to successful-save counts. |
| Undo after a subsequent conflicting change | Explain refusal and leave all data unchanged. |
| Item is already active at Save | Ordinary repeated capture; no extra progress reset. |
| Previously matched item disappears or becomes ambiguous | Keep draft, report need to recheck, and do not silently replace or retarget. |
| auto source with unique unambiguous attribution | Reuse the same item under the common matching policy; do not create avoidable auto/en duplicates. |
| auto source with multiple possible known languages | Preserve unresolved attribution instead of guessing; later reconciliation must follow the same identity and history rules. |
| ROBUST versus robust; run versus running | First pair shares identity in the same source language; second pair remains distinct. |
| Known same-spelling words in different source languages | Remain distinct Vocabulary Items. |
| Migrate duplicates with Learning and Achieved states | Retain active learning and history, consolidate references/translations, and do not add a capture or retain a parallel Achieved item. |
| Migrate duplicates all in Achieved | Preserve Achieved with the later deletion deadline. |
| Legacy Encounter has no translation snapshot | Keep the Encounter; do not fabricate its historical translation. |
| Restart app after migration and repeat startup | Valid schema and references, stable consolidated data, normal startup/save/review behavior, no repeated merge side effects. |

## Migration and planning constraints

- Existing records are test data, but blanket deletion is not the agreed migration strategy. Preserve available Encounter/review history and translations; repair demonstrably invalid fixtures without inventing linguistic mappings for real words.
- Vocabulary Item counts can decrease after consolidation. Successful-save history and review history must not increase merely because of migration; keep known/unknown history distinctions.
- Identity changes must remain within the owning user's vocabulary. Deterministic surviving IDs, reference rewrites, transaction boundaries, rollback, and outbox/schema updates belong in the implementation plan.
- Old clients that write the former schema are not automatically compatible. The plan must specify the local upgrade boundary rather than claim mixed-version safety.
- The plan should make review-state precedence and tie-breaking explicit, consistent with preserving active learning and the earlier due schedule; do not combine incompatible review parameters arbitrarily.
- Legacy translation ordering may lack true save timestamps. Use a documented deterministic migration rule without claiming unavailable provenance.
- A previewed identity and its Save must not race into duplicate records or incorrect restoration. Include this in backend verification, not just mocked UI tests.
- Validate real database migration, repeated-save behavior, undo, and all three capture entry points, alongside sidebar/list/review presentation.

## Explicit exclusions

- Morphological lemmatization, sense disambiguation, and expanded text normalization.
- Automatically generating multilingual translations merely for display or on setting changes.
- Independent learning/review state per target translation language.
- Implementing authentication, cloud sync transport, or an unverified live-cloud migration.
- Redesigning capture controls, adding a mandatory second Achieved confirmation, or changing OCR Confirmation.
- Replacing the review algorithm beyond resetting the current schedule for explicitly recaptured Achieved items.
- Fabricating missing legacy history or silently clearing the database.

## Handoff

All interview questions Q1–Q15 have answers, and the user confirmed the consolidated scope as accurate on 2026-09-10. This document is the agreed input for the subsequent implementation plan and local tickets. No production implementation or data migration has been performed during this interview.

## Documentation policy for this interview

Update this document as decisions settle. Update CONTEXT.md only for agreed domain definitions. Record a durable identity/migration tradeoff in an ADR after agreement. No production code or user data is changed during this interview.
