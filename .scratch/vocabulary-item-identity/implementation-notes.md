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
