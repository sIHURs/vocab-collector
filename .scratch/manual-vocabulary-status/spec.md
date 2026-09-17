# Manual Vocabulary Item learning status

Status: implemented

## Confirmed decisions

- Keep the existing Learning, Mastered, and Paused statuses. Users select a status for an individual Vocabulary Item; custom status definitions are out of scope.
- Reviews continue to change scheduling only, without automatically changing learning status.
- After a successful status change, Learning and Paused appear in Active, and Mastered appears in Mastered. Refresh affected lists immediately without navigating to another view; show a destination message. Keep an open detail view on the same item with its updated status.
- The only new editing entry point is a status dropdown in the individual Vocabulary Item detail view. Selecting a different status saves immediately.
- The feature is intended for all operating systems. This implementation exposes it in the Windows presentation only, with shared domain/application semantics.
- Only individual Vocabulary Items are edited. Bulk status editing is out of scope.
- A Vocabulary Item continues to share one learning status across its target-language translations.
- Paused to Learning preserves review progress and the existing due time; overdue items become eligible for the due queue.
- Mastered to Learning resets scheduling to the initial state and is immediately due. Historical Review records and Encounters are retained.
- Mastered to Paused also resets scheduling to the initial state, due at the transition time, while excluding the item from review. Returning to Learning subsequently makes it immediately eligible for review.
- Entering Paused or Mastered removes the item from the due queue and retains history.
- Entering Mastered manually starts a new mastery period and participates in automatic Achieve after 30 uninterrupted days if that setting is enabled. Leaving Mastered clears that period; re-entering starts it again. Selecting the existing status does not reset it.
- Achieved items do not expose the new editor. Unachieve first returns an item to Mastered, where it can be edited. Explicit recapture of Achieved items continues to return them to Learning under existing rules.
- A successful change offers one Undo through the existing bottom-right notification design. Undo restores the prior learning status, review scheduling state, and mastery start time, rather than applying a reverse transition. Its validity ends when the same item is changed again or receives a review submission.
- Disable the dropdown while saving. On save failure, retain the original state and show a retryable error.
- Resuming a paused Review session recomputes the remaining queue from current learning statuses and due times, excluding Paused and Mastered and including eligible Learning items under the existing daily limit. Preserve completed Review results; if no items remain, show session completion.

## Transition matrix

| From | To | Scheduling | Mastery period |
| --- | --- | --- | --- |
| Learning | Paused | Preserve; exclude from review | None |
| Learning | Mastered | Preserve; exclude from review | Start now |
| Paused | Learning | Preserve; eligible when due | None |
| Paused | Mastered | Preserve; exclude from review | Start now |
| Mastered | Learning | Reset to initial state; due now | Clear |
| Mastered | Paused | Reset to initial state; exclude until resumed | Clear |
| Any editable status | Same status | No change | No change |

Historical Reviews and Encounters are preserved in every transition. Undo restores the snapshot instead of following this matrix.

## Acceptance scenarios

- In Windows detail, selecting a new status persists it and updates the badge and the corresponding Active/Mastered list without switching tabs or closing detail.
- No bulk editor, custom status definitions, or non-Windows presentation controls are introduced.
- Pausing a Learning item preserves its scheduling; resuming after its due time makes it eligible for review.
- Returning from Mastered directly or via Paused resets scheduling while preserving historical records.
- Manual mastery participates in the existing opt-in automatic-Achieve policy; same-status selection does not restart its timer.
- The bottom-right Undo restores exact prior scheduling and mastery time and cannot overwrite a subsequent change or Review of that item.
- Failed saves retain the original state; saving disables repeat selection.
- Resuming a paused Review uses the refreshed queue and preserves completed results.
- Achieved remains outside the status editor and follows existing Unachieve/recapture behavior.

## Current implementation findings

- New items start Learning. Ordinary recapture preserves status.
- Review submission does not change status. There is no production action that sets an item to Paused, and no production caller of enter_mastered.
- Pausing a review session is presentation state, not a change to any Vocabulary Item's learning status.
- Windows Active currently includes Learning and Paused; Mastered has its own view. Achieved is a separate lifecycle view.
- Only Learning items whose review schedule is due enter the normal review queue.
- Achieve, Unachieve, and explicit recapture of Achieved items follow ADR 0004.

## Confirmation

All interview questions Q1-Q11 and the ticket breakdown were confirmed by the user. The user authorized sequential implementation of tickets 01-03; implementation, final verification, and code review are complete.

## Discussion

2026-09-10: User confirmed the three existing statuses, user-controlled learning stage with scheduling-only reviews, Windows-only implementation for an eventual cross-platform feature, and no bulk editing.

2026-09-10: User accepted Q4-Q8 recommendations: existing view grouping without automatic navigation, detail-only immediate-save dropdown, transition-specific scheduling, manual mastery participating in automatic Achieve, and keeping Achieved outside the editor.

2026-09-10: User accepted Q9-Q11 recommendations: reset scheduling on Mastered to Paused, exact Undo with the existing bottom-right UI design, and refresh the remaining queue when resuming Review while preserving completed results.

