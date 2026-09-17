# 02: Deliver the three-section landing page with a working demo

**What to build:** A visitor can open the independently built website, understand the product, navigate its three sections and launch the working journey from 01.

**Blocked by:** 01 — Prove the shared App journey in a browser.

**Status:** ready-for-agent

- [ ] Separate website entry, build output and local preview from desktop startup, using existing Svelte/TypeScript/Vite dependencies.
- [ ] Implement Paper marketing composition, fonts, navigation, benefits, footer and download area; integrate the 01 demo rather than leaving only a disconnected placeholder.
- [ ] Dark initial theme and remembered theme selection work without wiping session data or leaking native styles into marketing content.
- [ ] Meaningful static/prerendered marketing content and download explanation remain usable without JavaScript or if the demo fails; choose and document the rendering mechanism.
- [ ] No page-wide overflow at 390px; navigation is keyboard usable and demo loading/failure states are accessible.
- [ ] Website and desktop builds both pass; incomplete public destinations are clearly marked in development and cannot masquerade as working downloads.

## Scope and verification

Reuse actual Windows App UI and contracts with browser-local sample data. Preserve desktop defaults and Windows/macOS presentation boundaries. No native commands, production data, provider credentials or translation requests. English interface and Simplified Chinese translations; approved Paper appearance and current native UI remain authoritative. Include meaningful behavioral checks and relevant shared-App regression tests. Implementation does not authorize production deployment.

