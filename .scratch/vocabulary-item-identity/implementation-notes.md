# Implementation and validation notes

Review baseline: `aab3fedae4d0ed8ff370efb82e5cda265a2a725e`.

## Data upgrade

- Local schema 8 expands saved translation history and optional Encounter translation snapshots, then consolidates identity. Startup runs the upgrade inside a savepoint; malformed data aborts without publishing half-migrated keys or a newer version.
- Identity is scoped to the owner. Active keys contain normalized lemma and source language; tombstones have distinct keys and are not resurrected. Case/whitespace normalization is unchanged.
- The lexicographically smallest UUID survives consolidation. Learning precedes Paused, then Mastered, then Achieved. Among equal states, the earlier due schedule wins, with UUID as a stable tie-breaker. All-Achieved groups preserve the later deletion deadline.
- Available legacy translations are imported in update-time/UUID order; this is a deterministic migration fallback, not proof of historical save order. New translation history follows committed save order. Missing legacy Encounter translations remain absent.
- Only the recognizable `qa-fixture-<number>-<normalized display text>` pattern is repaired. There is no general display-form identity fallback.
- Capture snapshots and per-item revision tracking support complete Undo and detect subsequent mutations even if caller timestamps are identical. Successful-save counters are updated in the same transaction, on the original local save date.
- New clients reject unknown future schema versions. Old binaries do not understand schema 8 and must not open an upgraded database; downgrade requires restoring a pre-upgrade backup. No mixed-version local writing is supported.
- Supabase changes describe future sync payloads and identity constraints only. They deliberately do not silently consolidate unknown remote duplicates. There is no cloud transport implementation or verified online deployment in this scope.

## Verification record

- Application service tests cover cross-target identity, translation updates and snapshots, preferred-language fallback, unambiguous auto reconciliation, Achieved review reset, stale/deleted matches, complete Undo, conflicting mutations, and unrelated-word isolation.
- Real SQLite upgrade tests cover historical consolidation, preserved counts/references, repeated startup, older schemas, and rollback after failed upgrade.
- Presentation tests cover the notice before Save, unchanged Save controls, retained drafts, OCR confirmation, multilingual detail, and conflict-aware demo behavior.
- Full validation and the two-axis review are recorded below as they finish. Physical macOS/Linux capture and credentialed provider calls cannot be claimed from this Windows session.

- Windows CI Rust suite passed: `cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux --no-fail-fast`. Physical UIA and credentialed provider tests remain explicitly ignored.
- Windows CI Clippy passed with `-D warnings`.
- UI typecheck and production build passed. Full Vitest run covered 110 tests; two legacy Undo assertions were updated to the agreed refusal policy, and their six-test Vocabulary log file passed on rerun.
- A SQLite backup copy of the user's default database upgraded from schema 4 to 8: 35 Vocabulary Items, 40 Encounters, and 3 review records retained; integrity_check returned ok and foreign_key_check returned zero violations. Reopening did not repeat migration writes. The robust QA fixture normalized to robust/en. The original database was not modified.

- Final post-review UI run: all 112 tests across 11 files passed; Svelte check reported zero errors/warnings and production build passed.
- Post-review Rust storage/application suites passed, including 32 application-flow tests, 15 native-workflow tests, 11 SQLite repository tests, and four Vocabulary log tests. Windows CI Clippy passed again with warnings denied.
- Native Selection and confirmed Region OCR were tested through the real AppService and SQLite using platform fakes: cross-target Achieved recapture shares the original identity, records both translations, and complete Undo restores the prior Achieved item.

## Standards

Initial review found no documented-standard violations and three heuristic maintainability findings. All were addressed and passed targeted re-review:

- `ReviewState::initial` now supplies both new-item and restarted-learning defaults.
- `WordReference` names the entity and parent identity, preserving existing tuple snapshot serialization.
- Merge and Undo share reference collection/remapping, including Review result payloads and pending outbox references.

Final targeted review found no remaining documented-standard violations or blocking new findings. Reviewers performed static inspection; validation commands above were run by the implementing agent.

## Spec

Initial review found two P2 interaction issues. Both were fixed and passed targeted re-review:

- An Achieved match discovered by the final lookup completes within the same explicit Save click in all four Capture presentations. The generic floating window's automatic path still pauses for explicit Save.
- Achieved status no longer hides Retry translation after a provider failure. Retry does not automatically restart learning.

Dedicated regression tests cover both cases. No remaining definite specification regression was found in the targeted re-review.

Review totals: Standards 3 heuristic findings resolved, 0 remaining; Spec 2 P2 findings resolved, 0 remaining. All six local tickets are complete. The original running app and its database were not switched to the new build during implementation.

## Notion recapture follow-up

A temporary identity-only trace from the running desktop connection confirmed schema 8 and two live `notion` records: an unresolved-source Learning item and an English Achieved item. `foster` had only the English Achieved item. Earlier external database reads did not reflect this runtime state and are not evidence that the running app was on schema 4.

The concrete failure was preview consolidation: Learning won over Achieved before the notice check. Capture lookup now returns a matched Achieved record before state consolidation. Save revalidates that record against the candidate set, allowing a different canonical ID to survive the merge; ordinary save without explicit recapture is rejected whenever a matched item is Achieved. The existing Learning-first consolidation policy remains unchanged. Complete Undo restores both pre-save records.

An additional native workflow defect was fixed: applying a translation draft for an unchanged word preserves the detected source/target language rather than overwriting it with settings (especially `auto`).

Both regression tests failed before their fixes and passed afterward. The runtime-shape regression also covers an older auto ID surviving the merge, normal-save refusal, consolidation and complete Undo. All storage/application tests and Clippy passed. Temporary runtime tracing and its output files were removed; no direct edits were made to user vocabulary data. Native visual confirmation still requires recapturing in the rebuilt app.

## Resolved external schema 4 discrepancy

Windows file-handle inspection established the source of the contradictory reads. The external Python process requested the standard Roaming path, but `GetFinalPathNameByHandleW` resolved its handle to:

`C:\Users\yifan\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Roaming\app.vocabcollector.desktop\guest.db`

That package-local file is 176128 bytes, volume serial 160692624, file ID 6473924464764085. Opening that explicit location read schema 4 and 35 words.

The running desktop process's duplicated database handle resolved to:

`C:\Users\yifan\AppData\Roaming\app.vocabcollector.desktop\guest.db`

It is 458752 bytes, on the same volume but file ID 281474977489805. The runtime export already confirmed schema 8. These are distinct physical files, caused by the external process's packaged-app filesystem redirection, not a single SQLite database supporting simultaneous schemas or a stale SQLite transaction cache.

Consequently the earlier schema-4 backup migration check validated the package-local historical copy, not a backup of the live desktop database. Future external investigations must verify the handle's final path and file identity, or export through the running app. No database was deleted, merged, or upgraded during this source investigation. The historical action that first created the package-local copy was not determined.
