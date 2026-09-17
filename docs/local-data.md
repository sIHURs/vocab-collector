# Local data

## Check data

Settings → Local data → Check data runs a read-only check of database structure, foreign-key relationships and basic application formats. It does not change or repair data. One check runs at a time; Cancel check stops it, and navigating away does not discard the running check. The check has a 30-second work budget and a short lock wait; a busy or unreadable database is reported as incomplete.

“No issues found” means only that the listed checks passed. The quick check does not verify all index contents or prove that past data has never been lost. If the app cannot start, this Settings entry is unavailable; startup recovery is a separate future feature.

Save diagnostic report writes a JSON report containing check stages, issue codes, versions, counts and timing. Vocabulary text, context, source URLs, credentials and absolute paths are excluded. Nothing is uploaded automatically. The database size is sampled separately from the consistent data read.

## Export vocabulary CSV

Settings → Local data → Export vocabulary CSV exports every non-deleted, non-achieved Vocabulary Item, regardless of the current search or page. Each row includes vocabulary, source language, displayed translation and its language, Learning Status, Encounter count and last Encounter time. Items without an Encounter have a zero count and an empty time.

The displayed translation follows the current preferred language with the existing fallback. Other translations, context, Review history and settings are excluded. CSV is a readable list, not a backup. Text that could be interpreted as a spreadsheet formula is prefixed with an apostrophe. UTF-8 BOM, standard CSV quoting and CRLF support Excel and Unicode text.

Choose a local `.csv` file; diagnostic reports use `.json`. Cancelling the file dialog creates no export. A failed write preserves an existing destination file. The app reports success only after the completed file is published.

## Development, release and uninstall

The release application uses `app.vocabcollector.desktop.release`; debug builds keep the original `app.vocabcollector.desktop` identifier so existing development data stays available. Each uses its own OS app-data directory. The database filename remains `guest.db`. No existing development database is migrated into the release.

On Windows, explicit uninstall removes the release-owned Roaming and Local AppData directories, including the database and its journal files, settings, WebView data, cache and logs. Installer-driven replacement passes `/UPDATE` and preserves them. Exports and other manually saved files elsewhere are retained, as is the development directory. An unsuccessful cleanup reports remaining data rather than claiming complete removal. This is filesystem deletion, not a secure-erasure guarantee.

These installer behaviors are implemented and package-built; isolated Windows lifecycle execution remains a release gate. Do not perform the release test against a real user's profile.

Backup/restore and cloud synchronization remain future work.
