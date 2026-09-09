# 01: Establish the visual system through Settings and Manual Capture

**What to build:** Deliver the first usable shadcn-svelte-style vertical slice through the application shell, Settings, and Manual Capture. Introduce the approved Notion-inspired compact visual language, fixed sidebar, product mark, Lucide icons, semantic themes, and upper-right Capture icon on top-level Today/Vocabulary/Insights pages. Settings and nested pages do not show this entry. Preserve Settings and Manual Capture behavior except the explicitly requested removal of redundant dialog close controls.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Paper design:** Draft complete, revised 2026-09-08. [Current page](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/2-0): 16 Light/Dark artboards based on the user's Snapshot references, including Minimum Settings at both scroll positions and Manual Capture without a duplicate close icon. See `../paper-ticket-01-02-handoff.md` for the behavior inventory and verification limits. Application implementation and its acceptance checkboxes below remain open.

- [ ] Before designing or implementing the slice, document the current Settings and Manual Capture states, actions, validation, keyboard behavior, persistence, and feedback; the approved design maps each behavior to an explicit UI state without adding, removing, or reordering product behavior.
- [ ] The shared visual foundation provides semantic Light and Dark Theme tokens, a system font stack with English, German, and Chinese coverage, compact spacing, restrained radii, visible focus treatment, and a low-saturation accent approved from the Paper explorations.
- [ ] The main shell uses a fixed narrow sidebar with icon-and-text navigation for Today, Vocabulary, Insights, and Settings, plus a linear product mark and platform-safe title and drag regions.
- [ ] Settings is a single-column page divided into Languages, Review, Capture, and Appearance sections, and all existing values, constraints, shortcut-recording states, save behavior, and error behavior remain unchanged.
- [ ] Manual Capture opens from the accessible upper-right Capture icon on eligible top-level pages, uses the existing fields and validation, returns focus correctly when dismissed, and preserves save and Undo behavior. Its Cancel button replaces the duplicate X; Escape remains supported.
- [ ] Successful page-level saves use a brief toast, while validation and actionable errors remain next to the affected control without changing when or how the underlying operation occurs.
- [ ] The slice works at 1040×720 and 840×600, at 150% text scaling, with keyboard-only navigation, reduced motion, and both Light and Dark Theme.
- [ ] Existing automated tests continue to pass, with focused coverage added for any newly introduced presentation states or accessibility behavior.
