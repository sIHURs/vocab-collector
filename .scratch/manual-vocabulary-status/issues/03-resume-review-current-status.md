# 03: Resume Review using current Vocabulary Item learning statuses

**What to build:** When a user pauses Review, edits or undoes a Vocabulary Item's status, and resumes, rebuild the remaining queue from current persisted status and scheduling while retaining the results already completed in that session.

**Blocked by:** 01: Edit an individual Vocabulary Item's Learning Status in Windows.

**Status:** complete

- [x] Resuming a paused Review fetches/recomputes the remaining queue using current status and due times. Paused and Mastered are absent; eligible Learning items can enter under existing queue ordering and daily-limit behavior.
- [x] Completed session results remain available for the session summary and Review Insight. A refresh does not discard, duplicate, or resubmit completed Reviews.
- [x] When the refreshed remaining queue is empty, show session completion with the retained results rather than a stale card or unusable Resume action.
- [x] Clear stale card/reveal/submission UI state as needed when the remaining queue changes. A failed refresh exposes retry and prevents resuming the stale queue.
- [x] Queue handling depends on current persisted state rather than which action changed it, so the same refresh path also supports status Undo once ticket 02 is available.
- [x] Automated tests demonstrate pausing a session, editing a remaining item to Mastered or Paused, adding a due Learning item through a status transition, and resuming. Cover retained results, existing daily limit, empty completion, and refresh retry.

## Verification

Completed and reviewed. All 123 UI tests and 203 host-compatible Rust tests passed; Svelte typecheck and production build passed. See the feature implementation notes for the review findings, fixes, and macOS-on-Windows test limitation.
