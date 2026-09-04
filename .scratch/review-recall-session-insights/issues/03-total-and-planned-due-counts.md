# 03: Separate the complete due backlog from today's plan

**What to build:** Give users an honest view of Review workload by returning the complete number of due Learning Vocabulary Items separately from the daily-limit plan. Windows Today and idle Review show both values, while the Review Queue remains capped and ordered exactly as before.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The shared application result exposes unambiguous complete-due and planned-review counts instead of treating one capped value as both.
- [ ] The complete due count includes every non-deleted, due Learning Vocabulary Item before the daily limit is applied.
- [ ] The planned count equals the length of the ordered Review Queue and never exceeds the configured daily limit.
- [ ] When the complete backlog exceeds the plan, Windows Today and Review communicate both values without implying that the remaining items are not due.
- [ ] Empty, below-limit, exactly-at-limit, and above-limit queues produce correct counts and copy.
- [ ] Changing the daily limit changes the planned count and queue length but not the complete due count.
- [ ] Existing due-date ordering remains unchanged.
- [ ] The contract migration updates every current caller and demo/test backend without leaving two ambiguous count meanings.
- [ ] Shared application and command-contract tests run without an operating-system adapter.
- [ ] Windows presentation tests verify the user-visible distinction and preserve the existing Start Review path.

