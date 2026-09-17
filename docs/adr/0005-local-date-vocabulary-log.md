---
status: accepted
---

# Keep Vocabulary log on the original local save date

Accepted by the user on 2026-09-09. Vocabulary log counts successful vocabulary saves, including repeated words, on the system-local date when the save occurs; that date remains fixed after timezone changes. Successful Undo subtracts once from the original save date, including across midnight; failed or repeated Undo does not subtract again, matching the existing exclusion of undone Encounters from lifetime statistics.

Retain anonymous daily dates and counts indefinitely while displaying only the rolling past 12 months. Achieve, Unachieve and permanent deletion do not reduce these counts, and retaining them must not preserve deleted vocabulary or context. This extends anonymous lifetime statistics to daily history without retaining item-level data governed by ADR 0004.

Do not reconstruct daily history from surviving Encounters or lifetime totals. Dates before tracking began without recorded counts remain unknown. From the first tracking day onward, show recorded daily totals normally, including zero. Dates with recorded counts also show those counts normally if a timezone change moves the local date before the tracking start date. Save/Undo and count updates must remain atomic.

Updated at the user's request on 2026-09-12: remove the partial-history state and all first-day completeness handling. The time of day when tracking begins does not change how daily totals are classified or presented.
