# Review History and Global Insights Plan

## Goal

Use real persisted Vocabulary Item, Encounter, and Review history to fill the
unused Windows Review page with an honest view of learning activity. All
statistics and comparison semantics live in the cross-platform Rust core. This
branch delivers only the Windows presentation; macOS can later reuse the same
query contract.

## Dependency

Implement this plan after the Review Recall and Session Insights plan. It may
reuse that plan's explicit total/planned due counts and review submission
results, but it must derive historical metrics from persisted source-of-truth
records rather than UI session state.

## Product Decisions

- Global Insight appears on the existing Review page whenever an active Review
  card or just-completed Session Insight is not occupying the primary area.
- The page defaults to the last 30 local calendar days and supports 7 days,
  30 days, and all history.
- Statistics are descriptive. They do not claim scientific retention,
  intelligence, mastery, or learning quality.
- The ratio of `Remembered` responses to completed Reviews is named
  **Remembered response rate**, not retention rate.
- Charts and comparisons appear only when backed by persisted data.
- Empty and sparse histories explain what will appear after the user completes
  Reviews; they do not display fabricated example values.

## First-Version Metrics

### Current state

- total active Vocabulary Items;
- total non-deleted Encounters;
- complete currently due count;
- today's planned Review count;
- next local calendar day's expected due count.

### Selected period

- completed Review count;
- Remembered and Forgot counts;
- Remembered response rate with its denominator;
- Vocabulary Items captured;
- Encounters recorded;
- daily captured and reviewed series.

### Actionable comparisons

- Vocabulary Items with repeated forgetting, ordered by a deterministic core
  rule;
- Remembered response rate for items with one Encounter versus items with two
  or more Encounters, with both sample sizes shown;
- a neutral insufficient-data state when either comparison group is too small.

No metric may silently exclude failed writes, deleted records, or historical
ratings without documenting the rule in its field name or contract.

## Cross-Platform Architecture Gate

- Domain types define range, bucket, count, ratio, and attention-item concepts
  without UI labels or chart-library types.
- Storage performs bounded aggregate queries over SQLite and returns portable
  records; it does not depend on an operating-system clock or time-zone API.
- Application code validates ranges, assembles the dashboard, applies
  denominator and insufficient-data rules, and exposes one coherent result.
- Callers supply explicit UTC range boundaries and local-day bucket boundaries.
  This keeps daylight-saving and locale decisions at the application boundary
  without consulting Win32 or AppKit inside shared crates.
- The desktop command and TypeScript contract are identical on all targets.
- Windows rendering stays under `ui/src/windows`. macOS UI work is deferred.
- Any proposed query or DTO that contains `windows`, native handles, platform
  capability flags, or presentation colors fails review.

## Data Semantics

### Range definitions

- `7 days` and `30 days` are local calendar-day ranges including today, not
  rolling hour counts.
- `All` begins at the earliest relevant persisted event and ends at the query
  time.
- Each request carries ordered UTC bucket boundaries derived from the user's
  local calendar. The core validates that they are contiguous and within a
  documented maximum.
- Review counts use `reviewedAt`; capture activity uses Encounter `capturedAt`.
- Current due and next-day due counts are point-in-time projections and are not
  filtered by the selected historical range.

### Inclusion rules

- Soft-deleted Vocabulary Items and Encounters do not contribute to current
  totals or capture series.
- Persisted Review Logs remain historical facts even if an Encounter is later
  removed; the plan must explicitly test the repository's chosen behavior when
  an entire Vocabulary Item is soft-deleted.
- Remembered response rate is `Remembered / completed Reviews` for the selected
  range. A zero denominator produces no percentage, not zero percent.
- Encounter-group comparisons classify each Vocabulary Item using its current
  non-deleted Encounter count and show the number of Reviews and items behind
  each result.
- “Needs attention” uses a deterministic lapse/history threshold shared with
  the Review Recall plan and never uses a hidden UI heuristic.

## Portable Query Contract

Add one application-level dashboard query accepting:

- range kind (`sevenDays`, `thirtyDays`, or `all`);
- query timestamp;
- explicit local-day UTC bucket boundaries;
- a bounded attention-item limit.

Return a versionable dashboard DTO containing:

- current-state summary;
- selected-period totals;
- daily activity buckets;
- Encounter-group comparison with denominators and availability state;
- bounded attention items;
- effective range boundaries.

Do not return raw Review Logs or all Encounters to the WebView to compute the
dashboard. Aggregation belongs below the presentation boundary for privacy,
performance, consistency, and future macOS reuse.

## Windows Review Page

When Review is idle, arrange the existing free space in this order:

1. **Review readiness**: complete due backlog, today's planned count, next-day
   expected count, and Start/Resume Review action.
2. **Period controls**: 7 days, 30 days, all history.
3. **Activity summary**: Reviews completed, Remembered response rate,
   Vocabulary Items captured, and Encounters recorded.
4. **Daily rhythm**: one accessible chart or compact table showing captures and
   Reviews per day.
5. **Context comparison**: one versus multiple Encounter groups, including
   denominators and cautious explanatory copy.
6. **Needs attention**: a short actionable list that can open Vocabulary detail
   or start the appropriate Review flow.

An active Review replaces or de-emphasizes the dashboard so charts do not
compete with recall. Session completion shows Session Insight first; returning
to the idle Review state refreshes the global dashboard.

## Chart and Accessibility Rules

- Every chart has an equivalent textual summary or accessible table.
- Color is not the only distinction between captures and Reviews.
- Hover is never required to read exact values.
- Large totals include their time range and denominator where applicable.
- Loading, empty, insufficient-data, stale-refresh, and query-error states are
  distinct.
- Reduced-motion settings disable animated chart transitions.
- The UI must not imply causation from the Encounter-group comparison. Approved
  language is “In your history, items with multiple contexts had a higher/lower
  Remembered response rate,” not “More contexts improve memory.”

## Delivery Work Packages

1. **Specify metric semantics with domain tests**
   - Lock range inclusion, zero denominators, deletion behavior, due projection,
     lapse thresholds, and Encounter-group classification.
2. **Add efficient storage aggregation**
   - Add repository queries and indexes only when query plans demonstrate need.
   - Keep all queries bounded and avoid loading complete history into memory.
   - Add migration and legacy-database tests if an index or schema change is
     introduced.
3. **Assemble the portable application dashboard**
   - Validate explicit bucket boundaries.
   - Calculate ratios and availability states centrally.
   - Return a single internally consistent snapshot from one read boundary when
     supported by the repository.
4. **Expose the desktop command**
   - Add typed Rust/TypeScript DTO coverage and command-contract tests on every
     compiled target.
5. **Build the Windows Review dashboard**
   - Integrate idle, active Review, Session complete, empty, loading, and error
     states.
   - Refresh after capture, Undo, Review submission, settings changes affecting
     the daily limit, and range changes.
6. **Verify performance and portability**
   - Seed a representative large local history and set a measurable dashboard
     query/render budget before declaring completion.
   - Run shared Rust, SQLite, desktop contract, and Windows presentation suites.
   - Keep physical Windows evidence separate from automated cross-platform
     evidence.

## Test Matrix

- 7-day and 30-day ranges include the correct local calendar boundaries across
  a daylight-saving transition.
- All-history works for an empty database and for legacy records.
- Zero Reviews returns an unavailable rate rather than `0%`.
- Daily capture and Review buckets reconcile exactly with selected-period
  totals.
- Remembered plus Forgot equals completed Reviews.
- One-Encounter and multi-Encounter comparisons expose correct denominators.
- Insufficient samples suppress comparative claims.
- Deleted Encounters do not inflate current capture totals.
- Total due may exceed today's planned count, and both match the Review plan's
  queue semantics.
- Dashboard refreshes after successful Review and does not incorporate failed
  submissions.
- Range switching rejects stale asynchronous responses.
- Charts remain understandable by keyboard and screen reader and under forced
  colors.
- Shared aggregation and command tests run without native Windows APIs.
- macOS continues to compile against the portable DTO even though it does not
  render the dashboard in this branch.

## Completion Criteria

- Every displayed number reconciles with persisted local data and a documented
  inclusion rule.
- The Windows Review page provides useful history without distracting from an
  active recall task.
- No UI copy claims retention, mastery, causation, or AI analysis.
- The WebView does not receive raw complete histories to calculate aggregates.
- Shared Rust crates remain platform-independent and pass non-Windows contract
  checks.
- Empty, sparse, and large histories have verified behavior.
- macOS UI implementation is not required, but no Windows-specific concept is
  embedded in the reusable query contract.

## Out of Scope

- AI-generated learning advice or semantic analysis of vocabulary.
- Scientific retention or memory-strength claims.
- Streaks, leaderboards, social comparison, and pressure-oriented goals.
- Cloud analytics, telemetry, account-level aggregation, or cross-device merge
  behavior beyond the existing synchronization boundary.
- Editing, pausing, mastering, deleting, or merging Vocabulary Items from the
  dashboard.
- macOS dashboard presentation.
- Changes to the Review scheduling algorithm or rating scale.
