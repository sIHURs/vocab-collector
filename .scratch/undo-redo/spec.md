# Windows Undo / Redo

Status: design discussion; implementation not yet authorized by this discussion.

## Confirmed decisions

- Undo targets completed, saved data operations. Search, navigation and filtering do not enter this history; text editing keeps its own Undo behavior. The exact supported data operations remain to be decided.
- Undo and Redo must update affected Insights data as well as other affected displays consistently.
- User-visible history exists only during the current application run. Navigation does not clear it; a true application exit does. Tray/window lifecycle details remain open.

## Existing constraints

- ADR 0005 requires capture Undo to decrement the original local save date atomically. Redo date semantics remain open.
- ADR 0004 governs Achieve and permanent purge. Permanent deletion is currently presented as irreversible; making it reversible would require an explicit design change.
- Current Windows UI has separate short-lived Undo notifications for capture, Learning Status and Unachieve, rather than chronological history. Existing backend Undo tokens are consumed and guarded by revisions; multi-step Undo and Redo require more than adding buttons.

## Open decisions

- Chronological multi-step history versus the existing one-step actions.
- Supported operations, including Native Capture, Review, Achieve, Unachieve, settings and permanent deletion.
- History ordering across windows, batch grouping, capacity, conflicts and failures.
- Redo semantics for dates and derived Insights.
- Keyboard focus, dialogs, button location and disabled states.

## Discussion log

- 2026-09-13: User accepted saved-data scope and run-only history, and required Insights to update with the affected data.
