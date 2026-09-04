# 02: Explain Encounter history and repeated forgetting

**What to build:** Enrich each successful Review result with an accurate Encounter count and a deterministic repeated-forgetting signal from the shared Rust core, then turn those facts into a concise Windows Review Insight. A user who repeatedly forgets a Vocabulary Item receives one neutral, actionable suggestion to inspect another Encounter or review its translation; other users see only relevant factual feedback.

**Blocked by:** 01: Require active recall and return a Review result

**Status:** ready-for-agent

- [ ] Every successful Review result includes the current non-deleted Encounter count for its Vocabulary Item.
- [ ] The shared core defines and tests one explicit repeated-forgetting threshold based on persisted Review history or the resulting lapse state.
- [ ] New, legacy, repeatedly encountered, and previously forgotten Vocabulary Items produce correct Insight facts.
- [ ] The Insight result contains facts and eligibility flags rather than Windows copy, styling, or inferred psychological judgments.
- [ ] The Windows result state reports when the Vocabulary Item has been encountered multiple times without overstating what that history proves.
- [ ] An eligible repeated-forgetting result presents at most one neutral recommendation with an action to inspect other saved context or the translation.
- [ ] Ineligible results do not show empty warnings, invented advice, retention claims, or failure language.
- [ ] Reading Insight facts does not create a Review, modify scheduling, or add a separate durable learning event.
- [ ] Storage and application tests prove the returned facts match the same committed Review and Vocabulary Item state.
- [ ] The feature remains portable: no native platform dependency enters the domain, application, or storage implementation.

