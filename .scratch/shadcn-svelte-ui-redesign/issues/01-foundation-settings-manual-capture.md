# 01: Establish the visual system through Settings and Manual Capture

**What to build:** Deliver the first usable shadcn-svelte-style vertical slice through the application shell, Settings, and Manual Capture. Introduce the approved Notion-inspired compact visual language, fixed sidebar, product mark, Lucide icons, semantic themes, and bottom-right `+ Capture` action while preserving every currently implemented Settings and Manual Capture behavior.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Before designing or implementing the slice, document the current Settings and Manual Capture states, actions, validation, keyboard behavior, persistence, and feedback; the approved design maps each behavior to an explicit UI state without adding, removing, or reordering product behavior.
- [ ] The shared visual foundation provides semantic Light and Dark Theme tokens, a system font stack with English, German, and Chinese coverage, compact spacing, restrained radii, visible focus treatment, and a low-saturation accent approved from the Paper explorations.
- [ ] The main shell uses a fixed narrow sidebar with icon-and-text navigation for Today, Vocabulary, Insights, and Settings, plus a linear product mark and platform-safe title and drag regions.
- [ ] Settings is a single-column page divided into Languages, Review, Capture, and Appearance sections, and all existing values, constraints, shortcut-recording states, save behavior, and error behavior remain unchanged.
- [ ] Manual Capture opens from an accessible bottom-right `+ Capture` floating action, uses the existing fields and validation, returns focus correctly when closed, and preserves the current save and Undo behavior.
- [ ] Successful page-level saves use a brief toast, while validation and actionable errors remain next to the affected control without changing when or how the underlying operation occurs.
- [ ] The slice works at 1040×720 and 840×600, at 150% text scaling, with keyboard-only navigation, reduced motion, and both Light and Dark Theme.
- [ ] Existing automated tests continue to pass, with focused coverage added for any newly introduced presentation states or accessibility behavior.
