# 04: Deliver the restrained Insights learning summary

**What to build:** Deliver Insights as a first-class navigation destination with Captured, Reviewed, Due, Vocabulary log, All-time detail and Review Insights. Preserve existing metric meanings. The user-requested Vocabulary log is a new daily successful-save series: every successful vocabulary save counts, including repeated words. Use the Snapshot GitHub-style heatmap with blue intensity levels in the second row, above All-time detail.

**Blocked by:** 01: Establish the visual system through Settings and Manual Capture.

**Status:** ready-for-agent

**Paper design:** Complete, revised 2026-09-08. [Insights & Learning Summary](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459/5-0) contains 8 Light/Dark artboards: overview, Minimum overview, Minimum scrolled/day-tooltip proofs and data/coverage/recovery states. Vocabulary log replaces Weekly Rhythm. Current interfaces expose no daily series, so populated heatmaps are design samples and the unavailable fallback remains specified. See `../paper-ticket-03-05-handoff.md`. Application implementation and runtime acceptance remain open.

- [ ] Before designing or implementing the slice, inventory the metrics and Review Insights actually available from the current application; labels, calculations, periods, and empty conditions remain faithful to those sources.
- [ ] Insights is present in the fixed sidebar and replaces the prior Progress naming consistently at the presentation level without altering unrelated routes or application services.
- [ ] Captured, Reviewed, and Due summaries have restrained hierarchy and do not resemble oversized revenue-dashboard metrics.
- [ ] Implement phase 4a before connecting phase 4b's real graph: SQLite/application/Tauri/TypeScript Backend and DemoBackend daily series with explicit coverage, following accepted `docs/adr/0005-local-date-vocabulary-log.md` and `docs/plans/2026-09-09-paper-ui-implementation.md`.
- [ ] Vocabulary log counts successful vocabulary saves, including repeats, with no Review or failed-save contributions. Assign the system-local date at save time and freeze that assignment across timezone changes.
- [ ] Successful Undo subtracts once from the original save date; failure and repeated Undo do not subtract again. Save/Undo and daily updates are atomic, including Achieved recapture.
- [ ] Retain anonymous daily dates/counts indefinitely; display the rolling past 12 months. Achieve, Unachieve and permanent deletion do not reduce counts or cause deleted item/context retention.
- [ ] Do not synthesize history from lifetime totals or surviving Encounters. Upgraded databases expose unknown prior history and a partial upgrade day unless completeness is proven; complete coverage begins with the first full local day after migration. Distinguish unknown/partial dates from known zero.
- [ ] Test migration, rollback, repeated saves, idempotent Undo, midnight, timezone/DST changes, Achieved recapture and permanent deletion before graph integration. User acceptance of this plan on 2026-09-09 does not authorize implementation during the planning phase.
- [ ] Vocabulary log uses blue intensity levels, full-date/exact-count tooltips, keyboard navigation and accessible labels. Minimum preserves typography and cell size, using horizontal and vertical scrolling as shown in Paper.
- [ ] Review Insights use the project's canonical terminology and do not introduce scores, streaks, fabricated recommendations, or pressure-oriented language.
- [ ] Real-data, loading, empty-data, partial-data, and recoverable-error states are defined using only actions supported by the current application.
- [ ] The page works at 1040×720 and 840×600, at 150% text scaling, in Light and Dark Theme, with keyboard navigation and reduced motion.
- [ ] Existing data and presentation tests continue to pass, with focused tests confirming that the redesign does not alter metric meaning or Review behavior.
