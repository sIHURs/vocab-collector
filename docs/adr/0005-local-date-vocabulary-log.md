---
status: accepted
---

# Keep Vocabulary log on the original local save date

Accepted by the user on 2026-09-09. Vocabulary log counts successful vocabulary saves, including repeated words, on the system-local date when the save occurs; that date remains fixed after timezone changes. Successful Undo subtracts once from the original save date, including across midnight; failed or repeated Undo does not subtract again, matching the existing exclusion of undone Encounters from lifetime statistics.

Retain anonymous daily dates and counts indefinitely while displaying only the rolling past 12 months. Achieve, Unachieve and permanent deletion do not reduce these counts, and retaining them must not preserve deleted vocabulary or context. This extends anonymous lifetime statistics to daily history without retaining item-level data governed by ADR 0004.

Do not reconstruct supposedly complete daily history from surviving Encounters or lifetime totals. For an upgraded database, dates before reliable coverage are unknown, and the upgrade day is partial unless completeness can be proven; complete coverage begins with the first full local day after migration. This deliberately accepts missing historical cells instead of misleading zeros or exact totals. Save/Undo and count updates must be atomic, with explicit coverage information exposed to the presentation.
