# 04: Deliver the restrained Insights learning summary

**What to build:** Deliver Insights as a first-class navigation destination that summarizes the learning information the application can currently provide. Present Captured, Reviewed, Due, Weekly Rhythm, and Review Insights in a calm reading-companion style without inventing new analytics behavior or changing how existing values are calculated.

**Blocked by:** 01: Establish the visual system through Settings and Manual Capture.

**Status:** ready-for-agent

- [ ] Before designing or implementing the slice, inventory the metrics and Review Insights actually available from the current application; labels, calculations, periods, and empty conditions remain faithful to those sources.
- [ ] Insights is present in the fixed sidebar and replaces the prior Progress naming consistently at the presentation level without altering unrelated routes or application services.
- [ ] Captured, Reviewed, and Due summaries have restrained hierarchy and do not resemble oversized revenue-dashboard metrics.
- [ ] Weekly Rhythm uses a compact, accessible visualization with an equivalent textual interpretation and no dependence on color alone.
- [ ] Review Insights use the project's canonical terminology and do not introduce scores, streaks, fabricated recommendations, or pressure-oriented language.
- [ ] Real-data, loading, empty-data, partial-data, and recoverable-error states are defined using only actions supported by the current application.
- [ ] The page works at 1040×720 and 840×600, at 150% text scaling, in Light and Dark Theme, with keyboard navigation and reduced motion.
- [ ] Existing data and presentation tests continue to pass, with focused tests confirming that the redesign does not alter metric meaning or Review behavior.
