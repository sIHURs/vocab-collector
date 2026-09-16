# Backups retain historical data until cleanup

Permanent deletion removes item-level data from the current vocabulary database under ADR 0004; existing backups retain their historical snapshot until rotated or explicitly deleted. Restoring an older backup may therefore bring back previously deleted Vocabulary Items. This preserves the ability to recover an earlier state without rewriting historical backups; the product must explain this boundary and allow users to clear app-managed backups, while manually saved copies remain under user control. Anonymous statistics retain the semantics of ADR 0005.

After restoring a backup, automatic expiry-based permanent deletion stays paused until the user has inspected the result and explicitly resumes cleanup. Preserve the original deadlines rather than restarting retention periods. Persist the pause across app restarts so that restarting cannot bypass this review opportunity.
