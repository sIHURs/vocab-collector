# 02: Manage and permanently purge Achieved Vocabulary Items

**What to build:** Let a user search and select Achieved Vocabulary Items,
Unachieve the filtered selection, or permanently delete it after explicit
confirmation, while preserving compact Lifetime Insight totals and preventing
duplicate capture of an item that is still Achieved.

**Blocked by:** 01: Manually Achieve one Mastered Vocabulary Item

**Status:** ready-for-agent

**Involved directories:** `crates/domain`, `crates/storage`,
`crates/application`, `apps/desktop/src-tauri`, `ui/src/lib`, `ui/src/windows`

- [ ] Achieved results support search and stable ordering by nearest deletion
      deadline and then word.
- [ ] Row selection and the header checkbox select the complete filtered
      result and expose the exact selected count.
- [ ] Batch Unachieve atomically returns every selected item to Mastered,
      restarts its Mastered period, and offers short-lived UI Undo.
- [ ] Batch permanent deletion states the exact count, requires confirmation,
      and cannot be recovered after commit.
- [ ] Mixed invalid selections and injected transaction failures produce no
      partial writes.
- [ ] Permanent purge removes Vocabulary Item, Encounter, and Review detail
      while keeping non-word-level Lifetime Insight aggregates stable.
- [ ] Capturing a word that remains Achieved creates no duplicate and exposes
      an Unachieve recovery path.

## Verification commands

```powershell
cargo test -p vocab-domain -p vocab-storage -p vocab-application
cargo test -p vocab-desktop-lib
pnpm --dir ui test
pnpm --dir ui check
cargo check --workspace
```

Transactions, aggregation, and component interaction can be verified in the
current environment. Native dialog focus, keyboard flow, and physical database
inspection remain for Ticket 04.

