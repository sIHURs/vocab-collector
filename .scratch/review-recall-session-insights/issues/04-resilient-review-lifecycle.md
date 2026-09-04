# 04: Make the Review lifecycle resumable and single-submit

**What to build:** Keep Review trustworthy when a user pauses, closes, retries, or encounters a failed response. Recall and revealed cards can be resumed without an accidental rating; a failed submission remains on the same revealed card; and repeated actions or a lost response cannot produce duplicate Review Logs or skip a Vocabulary Item.

**Blocked by:** 01: Require active recall and return a Review result

**Status:** ready-for-agent

- [ ] Closing during Recall pauses the Review without revealing, rating, or removing the active Vocabulary Item.
- [ ] Closing after reveal but before rating creates no Review Log and resumes at a safe, explicitly defined card state.
- [ ] A submission failure retains the revealed answer, selected Vocabulary Item, and retry action while showing a recoverable error.
- [ ] Rating controls are disabled while a submission is in flight.
- [ ] Repeated clicks, stale asynchronous responses, and close/navigation races cannot advance the queue twice.
- [ ] Retrying after an ambiguous or lost response cannot create a second Review Log for the same logical submission.
- [ ] A successful result is added to the current session exactly once and the user explicitly advances from Result to the next card.
- [ ] Pause/resume reports the correct remaining planned count and never reintroduces a successfully completed card.
- [ ] Focus returns to a meaningful action after errors, resume, reveal, and result transitions.
- [ ] Domain/application idempotency checks and Windows lifecycle tests cover one-card, multi-card, error, close, retry, and stale-response scenarios without platform logic leaking into the shared core.

