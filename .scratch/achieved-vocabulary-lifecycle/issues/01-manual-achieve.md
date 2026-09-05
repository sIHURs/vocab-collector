# 01: Manually Achieve one Mastered Vocabulary Item

**What to build:** Let a user Achieve one Mastered Vocabulary Item from its row
menu or detail page, then immediately find it in `Vocabulary > Achieved` with
the captured deletion deadline. Deliver the complete path through portable
domain rules, SQLite migration, application and desktop commands, shared DTOs,
and the Windows presentation.

**Blocked by:** None (can start immediately)

**Status:** complete

**Involved directories:** `crates/domain`, `crates/storage`,
`crates/application`, `apps/desktop/src-tauri`, `ui/src/lib`, `ui/src/windows`

- [x] Only a non-deleted Mastered Vocabulary Item can be Achieved; Learning,
      Paused, and already Achieved items return typed outcomes.
- [x] `WordStatus` remains unchanged and lifecycle timestamps independently
      represent Mastered and Achieved.
- [x] SQLite and Settings migrate legacy data safely; legacy Mastered rows do
      not become immediately eligible for future automation.
- [x] Retention accepts only 10, 20, 30, or 60 days and defaults to 30 days.
- [x] A user can Achieve one item from the Mastered row menu or detail page
      after confirming the calculated deletion date.
- [x] The item leaves active/Review/Mastered results and appears in a basic
      Achieved view after the command succeeds.
- [x] No bulk Achieve control is introduced.

## Verification commands

```powershell
cargo test -p vocab-domain -p vocab-storage -p vocab-application
cargo test -p vocab-desktop-lib
pnpm --dir ui test
pnpm --dir ui check
cargo check --workspace
```

Automated implementation and verification can run in the current environment.
Real WebView interaction and restart persistence are not Windows 11 physical
verification until completed in Ticket 04.
