# 03: Route explicit Native Capture modes

**What to build:** Start Selection Capture and Region OCR Capture as distinct requests, and let a failed Selection Capture recommend but never automatically invoke OCR.

**Blocked by:** 02: Deliver dual Capture shortcut settings.

**Status:** ready-for-agent

- [ ] Each shortcut starts only its configured Capture mode.
- [ ] Selection failure offers the OCR shortcut and Start OCR when OCR is available.
- [ ] Start OCR remains usable when its shortcut is disabled or conflicted.
- [ ] Request identity protects both modes from stale work.
