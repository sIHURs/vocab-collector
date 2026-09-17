# 02: Deliver Today and the focused Review flow

**What to build:** Redesign Today around the planned Review and provide a focused, single-column Review experience inside the main content area. Recent Captures remain available as a secondary section, and all currently implemented Review scheduling, answer, rating, progress, completion, and recovery behavior remains authoritative.

**Blocked by:** 01: Establish the visual system through Settings and Manual Capture.

**Status:** ready-for-agent

**Paper design:** Draft complete, revised 2026-09-08. [Current page](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/3-0): 18 Light/Dark artboards covering Today, focused Review, results, completion, recovery and Minimum layouts with unchanged font sizes. Root Today uses the Snapshot Capture icon at the right end of its heading row; nested Review screens omit it. See `../paper-ticket-01-02-handoff.md` for the behavior inventory and verification limits. Application implementation and its acceptance checkboxes below remain open.

- [ ] Before designing or implementing the slice, document the current Today and Review states, commands, transitions, counts, dismissal behavior, and error recovery; the new presentation has an explicit equivalent for each one.
- [ ] Today gives the planned Review count, estimated time, total due count, and Review start action the strongest visual hierarchy without introducing streaks, scores, or pressure-oriented copy.
- [ ] Recent Captures uses the shared Vocabulary Item presentation and continues to open the currently implemented detail experience without changing its data or navigation behavior.
- [ ] Review runs in the main content area with the sidebar retained and presents Vocabulary Item, Context, answer reveal, Translation, Review Insight, and Forgotten/Remembered actions in the agreed hierarchy.
- [ ] Question, revealed answer, submitting, remembered, forgotten, repeated-forgetting guidance, recoverable error, paused, empty, and complete states are visually defined wherever those states exist in current behavior.
- [ ] Loading and empty states preserve layout stability and provide accurate, non-invented actions.
- [ ] The flow remains usable at 1040×720 and 840×600, at 150% text scaling, with keyboard-only navigation, visible focus, and reduced motion.
- [ ] Existing Review and Today tests continue to pass, with focused tests covering presentation state transitions and focus behavior.
