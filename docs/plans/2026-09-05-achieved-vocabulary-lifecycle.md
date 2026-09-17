# Achieved Vocabulary Lifecycle Plan

## Goal

Let users intentionally retire Mastered Vocabulary Items, review them in an
Achieved list, restore or permanently delete multiple items, and optionally
authorize automatic Achieve after 30 days of uninterrupted mastery. Permanently
purge detailed item history after a user-selected retention period so the local
SQLite database does not retain every learned item indefinitely.

All lifecycle rules, timestamps, batch semantics, and purge transactions belong
to the cross-platform Rust core. This branch implements the Windows
presentation; a later macOS presentation must be able to reuse the same command
and data contracts.

## Confirmed Product Decisions

- Achieve is available only for a non-deleted Mastered Vocabulary Item.
- A user may Achieve one item from its Vocabulary row menu or detail page.
- Manual Achieve remains available whether automatic Achieve is enabled or not.
- Automatic Achieve is opt-in and disabled by default. Enabling it is the
  user's advance authorization to Achieve items after 30 uninterrupted days in
  Mastered status.
- Entering Mastered records `masteredAt`. Leaving Mastered cancels that period;
  entering it again starts a new 30-day period.
- Legacy Mastered items without `masteredAt` receive the migration time, so an
  upgrade cannot immediately Achieve them.
- Achieved items are excluded from active Vocabulary results, Today, due Review
  queues, and future Review scheduling.
- Unachieve returns an item to Mastered and starts a new 30-day automatic
  eligibility period. It never silently returns the item to Learning.
- Achieved items are retained for 10, 20, 30, or 60 days. The default is 30.
- The retention choice is captured into each item's `deleteAfter` when it is
  Achieved. Later Settings changes affect only newly Achieved items.
- An expired Achieved item is permanently purged. The purge retains compact,
  non-word-level aggregates needed to prevent Lifetime Insight totals from
  decreasing.
- Achieve itself has no bulk operation in the first version.
- The Achieved list supports search, multi-selection, selection of the entire
  filtered result, batch Unachieve, and confirmed batch permanent deletion.

## Domain Model

Keep learning status and retirement lifecycle separate. `WordStatus` remains
`Learning | Mastered | Paused`; do not add `Achieved` to that enum.

Extend a Vocabulary Item with portable timestamps:

- `masteredAt`: beginning of its current uninterrupted Mastered period;
- `achievedAt`: time at which direct or pre-authorized Achieve occurred;
- `deleteAfter`: immutable purge deadline captured from Settings at Achieve.

An item is an Achieved Vocabulary Item when it is Mastered, not deleted, and
has both `achievedAt` and `deleteAfter`. Domain constructors and transition
methods enforce the invariant rather than relying on disabled UI controls.

Valid transitions are:

```text
Learning/Paused -> Mastered
Mastered -> Achieved                 direct user action
Mastered -> Achieved                 enabled automatic policy, after 30 days
Achieved -> Mastered                 Unachieve before purge
Achieved -> Permanently Purged       deadline or confirmed delete
```

Any command against a missing, non-Mastered, already Achieved, or already
purged item returns a typed, idempotency-safe result. Repeating a successful
automatic sweep must not create another transition or deletion.

## Settings Contract

Extend shared `UserSettings` with:

- `automaticAchieveEnabled: bool`, default `false`;
- `achievedRetentionDays`, restricted to `10 | 20 | 30 | 60`, default `30`.

The 30-day Mastered threshold is a product rule, not a Windows scheduler
setting. Settings deserialization supplies defaults for legacy JSON, and the
application boundary rejects unsupported retention values.

The Windows Settings Review section renders:

- `Automatically achieve Mastered words after 30 days` toggle;
- `Keep achieved words for` select with 10, 20, 30, and 60 days;
- explanatory copy that retention changes apply to newly Achieved words.

## Portable Lifecycle Service

Add application commands whose inputs include an explicit UTC `now`:

- Achieve one eligible Vocabulary Item;
- Unachieve one or more Achieved Vocabulary Items;
- permanently delete one or more Achieved Vocabulary Items;
- list/search Achieved Vocabulary Items with stable ordering;
- run one automatic lifecycle sweep.

The lifecycle sweep performs two phases through one shared service:

1. If automatic Achieve is enabled, Achieve eligible items whose uninterrupted
   `masteredAt` period is at least 30 days, capturing the current retention
   value for each new deadline.
2. Permanently purge Achieved items whose `deleteAfter <= now`.

Each phase returns only a bounded summary such as achieved count, purged count,
and failures. It does not expose raw histories to the presentation.

## Execution and Platform Boundary

Invoke the shared sweep:

- once during application startup;
- before returning refreshed Vocabulary or Achieved data;
- once per local day while the desktop process remains alive;
- on the next startup after any offline interval.

The Rust application service evaluates all timestamps and eligibility. SQLite
performs the transaction. Windows-specific code may schedule the daily wake-up
and deliver one summary notification, but it must not query columns to decide
which item transitions or expires. A process that is not running is not a
background service; overdue work is caught up at the next invocation.

Automatic Achieve reports one summary such as `5 mastered words moved to
Achieved`. Automatic purge may report a compact deleted count and never opens a
confirmation dialog. Manual permanent deletion always requires confirmation in
the presentation before the command is sent.

## SQLite Migration and Transactions

Add nullable lifecycle columns and indexes supporting bounded eligibility and
deadline queries. Backfill `masteredAt` with migration time only for active
Mastered rows; other rows remain null. Preserve existing databases, foreign-key
enforcement, and default Settings decoding.

Achieve and Unachieve update the Vocabulary Item and enqueue the corresponding
portable mutation atomically. Batch operations are all-or-nothing for the
validated selection, with a bounded maximum enforced by the application
contract.

Before permanent purge:

1. Fold the item's contributions into compact historical aggregates required
   by the implemented Global Insight contract.
2. Enqueue a deletion tombstone if synchronization requires it.
3. Delete dependent Encounter and Review rows.
4. Delete the Vocabulary Item.
5. Commit all steps in one transaction.

No orphan rows or partially aggregated deletions may survive a failure. Exact
word-level history becomes unrecoverable after commit. Compact aggregates must
not contain the lemma, display form, translation, sentence, source URL, or
other data that could reconstruct the purged item.

## Duplicate Capture Behavior

Capturing a normalized word that is still in the Achieved list must not create
a duplicate Vocabulary Item. Return a typed result indicating that the item is
Achieved so the presentation can offer `Unachieve` before recording a new
Encounter. After permanent purge, capturing the same normalized word creates a
new Learning Vocabulary Item with no link to the deleted item-level history.

## Windows Vocabulary UI

Keep Achieved inside the existing Vocabulary area:

```text
Vocabulary
  Active | Mastered | Achieved (count)
```

This is not part of Review because it manages Vocabulary Item lifecycle rather
than recall sessions. The Achieved view contains:

- search by display form or normalized lemma;
- stable default ordering by nearest deletion deadline, then word;
- achieved date, deletion date, and whole-day countdown;
- yellow treatment at 3-7 days and red treatment at 0-2 days, with text and
  accessible labels so color is not the only signal;
- a row checkbox and a header checkbox selecting the entire filtered result;
- a sticky selection bar: `N selected`, `Unachieve`, `Delete permanently`;
- empty, no-search-results, loading, stale-request, and command-error states.

Batch Unachieve executes without confirmation and offers a short-lived UI Undo.
The Undo sends a new Achieve command and therefore receives a fresh deletion
deadline from the current retention setting. Batch permanent deletion shows the
exact selected count, states that word-level history cannot be recovered, and
requires explicit confirmation.

The Mastered row menu and detail page each expose a single-item `Achieve`
action. Confirmation shows the calculated deletion date. No checkbox or bulk
Achieve action appears in the Mastered view.

## Delivery Work Packages

1. **Lock lifecycle rules in the domain**
   - Add lifecycle timestamps, retention value, eligibility calculations, and
     typed transition outcomes.
   - Test all valid and rejected transitions with an injected clock value.
2. **Migrate Settings and SQLite safely**
   - Add legacy-compatible Settings defaults, lifecycle columns, indexes, and
     migration-time Mastered backfill.
   - Verify migration from every schema version represented in fixtures.
3. **Implement atomic repositories and purge aggregation**
   - Add bounded queries, atomic batch transitions, aggregate folding,
     dependent-row deletion, and tombstone behavior.
   - Verify rollback under failures at every transaction stage.
4. **Expose the portable application contract**
   - Add single-item Achieve, batch Unachieve/delete, filtered list, and sweep
     commands using platform-neutral DTOs.
   - Enforce batch limits, Settings validation, and idempotent retries.
5. **Connect desktop triggers without moving policy outward**
   - Run startup, refresh, and daily wake-up paths through the same application
     command.
   - Keep OS notification and timer APIs behind the desktop adapter.
6. **Build Windows Settings and Vocabulary presentation**
   - Add Settings controls, Vocabulary subviews, individual confirmation,
     Achieved search/selection, bulk action bar, countdowns, and accessible
     states.
7. **Verify portability, migration, and physical behavior**
   - Run shared Rust, SQLite, desktop command, and Windows UI suites.
   - Compile non-Windows targets/contracts and perform Windows physical tests
     without claiming macOS UI completion.

## Test Matrix

- Learning and Paused items cannot be manually or automatically Achieved.
- A Mastered item can be manually Achieved while automatic Achieve is disabled.
- Automatic Achieve never runs unless the persisted opt-in setting is enabled.
- Eligibility is false immediately before 30 days and true at the exact
  boundary.
- Leaving and re-entering Mastered resets `masteredAt`.
- Legacy Mastered rows receive migration time and are not immediately eligible.
- Changing retention affects new Achieve actions but not existing deadlines.
- Each 10/20/30/60-day setting produces the correct stored deadline.
- Unachieve before the deadline restores Mastered and resets the 30-day period.
- Expiration at `deleteAfter` purges once; repeated and concurrent sweeps are
  idempotent.
- Startup catch-up processes deadlines missed while the app was closed.
- Achieved items never appear in Today, due Review, Active, or Mastered lists.
- Search and select-all operate on the complete filtered Achieved result.
- Batch commands reject mixed eligible/ineligible IDs without partial writes.
- Purge removes item-level rows while aggregate Insight totals remain stable.
- Transaction failure leaves the item and aggregates unchanged.
- Capturing a still-Achieved word offers Unachieve and creates no duplicate.
- Capturing the same word after purge creates a new Learning item.
- Keyboard, screen reader, forced-colors, and reduced-motion behavior remain
  usable in the Windows list and dialogs.
- Shared lifecycle tests run without Win32 APIs; Windows-only tests cover only
  presentation, wake-up, and notification adaptation.

## Verification Commands

Run the repository's focused package commands discovered during implementation,
followed at minimum by:

```powershell
cargo test -p vocab-domain -p vocab-storage -p vocab-application
cargo test -p vocab-desktop-lib
pnpm --dir ui test
pnpm --dir ui check
cargo check --workspace
```

Build or run the Windows desktop application for manual verification on a
physical Windows machine. Do not mark startup scheduling, native notification,
WebView selection behavior, or restart catch-up as physically verified from a
non-Windows environment.

## Completion Criteria

- Only a user action or persisted user opt-in can authorize Achieve.
- Only Mastered Vocabulary Items can cross into Achieved.
- Achieved items remain searchable and reversible until their captured purge
  deadline.
- Batch Unachieve and confirmed batch deletion are atomic and accessible.
- Expired detailed data is actually removed from SQLite without decreasing
  promised Lifetime Insight totals.
- All product rules and destructive transactions remain in shared Rust crates;
  no Windows presentation code owns eligibility or deletion policy.
- Legacy databases migrate without immediate unexpected Achieve or data loss.
- Windows physical verification is reported separately from automated and
  cross-platform checks.

## Out of Scope

- Batch Achieve from the Mastered list.
- Automatic Achieve without explicit Settings opt-in.
- User-configurable automatic-Mastered threshold other than 30 days.
- Recovery after confirmed permanent deletion or elapsed retention.
- A Windows service that runs while the application is closed.
- macOS Achieved-list presentation in this branch.
- Cloud analytics or a redesign of the existing synchronization protocol
  beyond the deletion mutation required by this lifecycle.
