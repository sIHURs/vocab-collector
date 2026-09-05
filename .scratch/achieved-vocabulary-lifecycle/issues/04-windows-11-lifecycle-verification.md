# 04: Verify the Achieved lifecycle on a Windows 11 physical machine

**What to build:** Demonstrate the complete Achieved Vocabulary Item lifecycle
in the real Windows desktop application, fix any acceptance-blocking defects,
and record evidence that distinguishes Windows 11 physical behavior from
automated and unverified behavior.

**Blocked by:** 01: Manually Achieve one Mastered Vocabulary Item; 02: Manage
and permanently purge Achieved Vocabulary Items; 03: Run user-authorized
automatic Achieve and expiry

**Status:** blocked — native WebView automation is unavailable to this Codex session

**Involved directories:** `apps/desktop`, `ui/src/windows`, `docs`; shared
`crates` only when a discovered defect requires a portable fix

- [x] Upgrading a populated legacy database preserves data and does not
      immediately Achieve existing Mastered items.
- [ ] Manual Achieve, restart persistence, Achieved search/selection, batch
      Unachieve, and confirmed batch deletion work through the real WebView.
- [ ] Countdown warnings, keyboard navigation, focus management, confirmation,
      and notification behavior are usable on Windows 11.
- [ ] Enabling automation and simulating elapsed deadlines verifies daily
      processing and restart catch-up without a Windows background service.
- [ ] SQLite inspection proves item-level detail is removed and promised
      Lifetime Insight aggregates remain stable.
- [x] The development log separately records Windows 11 physical, automated,
      not run, and not verified evidence.

## Verification commands

```powershell
cargo test --workspace
pnpm --dir ui test
pnpm --dir ui check
pnpm --dir ui build
pnpm --dir ui tauri dev
```

The final runtime acceptance items require a confirmed Windows 11 physical
machine. If the environment cannot be confirmed as physical, leave those items
unchecked and do not mark this ticket complete.
