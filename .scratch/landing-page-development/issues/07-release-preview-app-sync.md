# 07: Deliver a release-ready preview and App synchronization workflow

**What to build:** The owner receives a reproducible preview with finalized publication content and a repeatable way to update the site when the released App changes.

**Blocked by:** 06 — Complete phone, expanded-demo and accessible exploration.

**Status:** ready-for-agent

- [ ] Measure production page/demo loading, establish justified budgets, and optimize loading without breaking static content or interactions.
- [ ] Verify supported browsers, metadata/social preview, assets, direct navigation and demo failure fallback.
- [ ] Replace placeholders with approved reading samples, branding, installer URL and contact/privacy/license destinations; missing inputs block publication, not independent verification work.
- [ ] Record released App version/commit, fixture and demo-flow versions, and build date in a release manifest; build public previews from an approved App revision.
- [ ] Add appropriate checks for shared UI/contracts and document how to rebuild, review and release website changes without automatically publishing every App commit.
- [ ] Provide a preview artifact, full journey evidence, known limitations and hosting-specific readiness checks once a host is chosen. Production deployment requires a separate explicit release request.

## Scope and verification

Reuse actual Windows App UI and contracts with browser-local sample data. Preserve desktop defaults and Windows/macOS presentation boundaries. No native commands, production data, provider credentials or translation requests. English interface and Simplified Chinese translations; approved Paper appearance and current native UI remain authoritative. Include meaningful behavioral checks and relevant shared-App regression tests. Implementation does not authorize production deployment.

