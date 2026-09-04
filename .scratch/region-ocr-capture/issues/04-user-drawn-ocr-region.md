# 04: Deliver a user-drawn OCR region

**What to build:** Let a user draw one single-display screen rectangle and recognize only that in-memory region.

**Blocked by:** 03: Route explicit Native Capture modes.

**Status:** ready-for-agent

- [ ] A non-captured overlay instructs the user and supports drag and Escape.
- [ ] Tiny selections remain recoverable and cross-display dragging is clamped.
- [ ] The platform OCR contract consumes a rectangle rather than a pointer.
- [ ] Windows capture releases native resources and never stores or logs captured content.
