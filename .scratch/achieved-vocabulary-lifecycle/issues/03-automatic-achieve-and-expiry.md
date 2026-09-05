# 03: Run user-authorized automatic Achieve and expiry

**What to build:** When the user opts in, automatically Achieve Vocabulary
Items after 30 uninterrupted days in Mastered and permanently purge Achieved
items at their captured deadline, using one idempotent shared Rust lifecycle
sweep invoked by desktop startup, refresh, and daily wake-up paths.

**Blocked by:** 02: Manage and permanently purge Achieved Vocabulary Items

**Status:** complete

**Involved directories:** `crates/domain`, `crates/storage`,
`crates/application`, `apps/desktop/src-tauri/src`, `ui/src/lib`,
`ui/src/windows`

- [x] Automatic Achieve defaults off, requires persisted user opt-in, and does
      not remove the manual Achieve feature.
- [x] Leaving and re-entering Mastered resets the uninterrupted period; the
      exact 30-day boundary is deterministic with an explicit UTC clock.
- [x] Each automatic transition captures the then-current retention setting;
      later Settings changes do not move existing deadlines.
- [x] Startup, Vocabulary refresh, and daily wake-up invoke the same portable
      lifecycle service; missed offline work is caught up next launch.
- [x] Expiry and repeated or concurrent sweeps are idempotent.
- [x] Windows owns only wake-up and summary notification adaptation, never
      eligibility or purge decisions.
- [x] Automatic actions expose bounded achieved/purged summary counts.

## Verification commands

```powershell
cargo test -p vocab-domain -p vocab-storage -p vocab-application
cargo test -p vocab-desktop-lib
pnpm --dir ui test
pnpm --dir ui check
cargo check --workspace
```

Injected-clock and adapter tests can run in the current environment. Native
daily timing, notifications, and restart catch-up require Ticket 04 on a
Windows 11 physical machine.
