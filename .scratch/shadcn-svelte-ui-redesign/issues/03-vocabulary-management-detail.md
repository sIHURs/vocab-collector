# 03: Deliver Vocabulary management and detail

**What to build:** Provide a compact Notion-database-style Vocabulary experience for finding and inspecting Vocabulary Items. Active, Mastered, and Achieved views, search, counts, pagination, status, and the Vocabulary Detail Sheet must continue to expose the behavior and information the application already implements.

**Blocked by:** 01: Establish the visual system through Settings and Manual Capture.

**Status:** ready-for-agent

- [ ] Before designing or implementing the slice, document the current Vocabulary states, filters, actions, lifecycle terminology, pagination, detail loading, and Encounter presentation; no existing behavior is silently renamed, removed, or replaced.
- [ ] The primary view is a compact table with Word, Translation, Status, Encounters, and Last Seen, using readable text and icons in addition to color for status.
- [ ] Active, Mastered, and Achieved remain distinct and use the repository's canonical Vocabulary Item lifecycle language.
- [ ] Search, result count, pagination, no-results, empty-library, loading, and recoverable-error states retain their currently implemented behavior.
- [ ] Selecting a Vocabulary Item opens an accessible right-side Sheet whose hierarchy is Word, Translation, Status, then Encounter Timeline, with correct Escape, focus trap, close, and focus-return behavior.
- [ ] At 840×600 and 150% text scaling, essential identity, status, and actions remain available while lower-priority metadata may wrap, shorten, move, or be hidden without changing the underlying feature.
- [ ] Realistic long English, German, and Chinese Vocabulary Items, Translations, and Contexts do not overlap controls or become inaccessible.
- [ ] Existing Vocabulary Item lifecycle and detail tests continue to pass, with focused presentation and accessibility coverage added where needed.
