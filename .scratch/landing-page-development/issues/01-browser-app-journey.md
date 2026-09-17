# 01: Prove the shared App journey in a browser

**What to build:** A minimal local browser route where a visitor saves one prepared word, sees its Vocabulary Item and Encounter, and completes the two-word Review using actual App UI.

**Blocked by:** None (can start immediately).

**Status:** completed

- [x] First make the minimum presentation/adapter separation needed for embedding; preserve desktop defaults and avoid duplicated App markup.
- [x] Choose and document scoped shared views or, if demonstrably necessary, same-origin frame isolation based on working evidence, including portals, theme and focus.
- [x] One prepared Capture Candidate can be explicitly saved; Vocabulary refreshes without reloading; detail shows sentence, saved translation and source.
- [x] Two previously due sample Vocabulary Items support reveal, rating and completion; the newly captured item is excluded.
- [x] Verify browser execution without native commands/provider requests and with isolated session data; demonstrate dark/light and a usable narrow viewport.
- [x] Run relevant existing App regressions after shared changes and record the chosen integration boundary and follow-up limitations. Full marketing styling and all sample branches are outside this slice.

## Scope and verification

Reuse actual Windows App UI and contracts with browser-local sample data. Preserve desktop defaults and Windows/macOS presentation boundaries. No native commands, production data, provider credentials or translation requests. English interface and Simplified Chinese translations; approved Paper appearance and current native UI remain authoritative. Include meaningful behavioral checks and relevant shared-App regression tests. Implementation does not authorize production deployment.


## Completion evidence — 2026-09-17

See [browser journey implementation and verification](../browser-journey-evidence.md) for the chosen same-origin frame boundary, source paths, local commands, 80 passing tests, browser checks and follow-up limitations. Ticket 02 is now unblocked.
