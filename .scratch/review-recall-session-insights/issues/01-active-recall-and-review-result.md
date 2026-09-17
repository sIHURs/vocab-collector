# 01: Require active recall and return a Review result

**What to build:** Make one complete Windows Review card require the user to reveal its answer before rating it, then return and display the scheduling result produced by the shared Rust core. Before reveal, the translation is absent visually and from assistive-technology content. Reveal alone records no Review. After `Forgot` or `Remembered`, the persisted Review result shows the submitted rating and next due time without changing the existing scheduling policy.

**Blocked by:** None (can start immediately)

**Status:** completed

- [ ] A Windows Review card initially shows the Vocabulary Item, saved context, progress, and a `Show answer` action without exposing the translation.
- [ ] `Forgot` and `Remembered` are unavailable until the user explicitly reveals the answer.
- [ ] Revealing the answer creates no Review Log, changes no Review State, and survives closing the card without being mistaken for a rating.
- [ ] A successful rating persists exactly one Review Log and the updated Review State before returning success.
- [ ] The shared result contains portable product facts including Vocabulary Item identity, submitted rating, review timestamp, next due timestamp, and the prior/resulting scheduling facts needed to describe the interval change.
- [ ] The Windows result state confirms the rating and displays the persisted next due time before offering `Next`.
- [ ] `Forgot` remains due after one day and `Remembered` preserves the current stability-based schedule.
- [ ] Focus moves predictably to revealed content and then to the result action; newly revealed content is announced without announcing the hidden answer early.
- [ ] Shared domain, application, storage, and desktop command tests cover the result contract without importing or branching on Windows APIs.
- [ ] Existing macOS and Linux compilation boundaries continue to consume the same platform-neutral Review command contract even though their presentations do not implement the new flow in this ticket.
