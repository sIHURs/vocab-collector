# Achieved Vocabulary Lifecycle

Source plan: `docs/plans/2026-09-05-achieved-vocabulary-lifecycle.md`

Implement the confirmed Achieved Vocabulary Item lifecycle as four sequential
tracer-bullet tickets. Shared Rust owns eligibility, timestamps, persistence,
batch semantics, automatic processing, and permanent purge. Windows owns only
its presentation, wake-up integration, and notifications.

The ticket chain is `01 -> 02 -> 03 -> 04`. Windows 11 physical verification
must remain distinct from automated verification and cannot be claimed merely
because tests pass on a Windows-compatible build environment.

