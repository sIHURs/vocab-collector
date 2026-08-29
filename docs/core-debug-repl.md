# Core debug REPL

The core debug REPL is a lightweight Rust example for inspecting shared domain,
application, review, and SQLite behavior without Tauri or an OS adapter.

## Start

From a Visual Studio Developer PowerShell on Windows:

```powershell
cargo run -p vocab-application --example core_debug
```

This starts an in-memory SQLite database. Its state lasts for the REPL session
and is discarded on exit. To inspect the same database across sessions, choose
the file explicitly:

```powershell
cargo run -p vocab-application --example core_debug -- --db .\core-debug.db
```

The REPL never locates or opens the desktop application's `guest.db`
automatically.

## Commands

```text
help
capture <DebugCaptureInput JSON>
list
show <word-uuid>
today [RFC3339 timestamp]
review <word-uuid> forgot|remembered [RFC3339 timestamp]
undo <encounter-uuid>
settings
settings set <UserSettings JSON>
db summary
db outbox
quit
```

Copy UUIDs from `capture`, `list`, or `show` output into later commands. Invalid
input prints one `error:` line and returns to the prompt. The REPL accepts no
SQL commands.

## Case 1: Trace a complete capture

Paste this as one line:

```text
capture {"selectedText":"Serendipity","lemma":null,"sentence":"A fortunate discovery made by accident.","sourceLanguage":"en","targetLanguage":"de","translation":"glücklicher Zufall","partOfSpeech":"noun","sourceApp":"core-debug","sourceTitle":"Manual test","sourceUrl":null,"captureOrigin":"manual","capturedAt":"2026-08-29T12:00:00Z"}
```

The four numbered sections are stable core boundaries:

1. Input accepted.
2. Domain normalization.
3. Application result.
4. Persistence readback and database delta.

Check that the domain observation contains `serendipity|en|de`, the application
result reports `isExistingWord: false`, the persisted word has one encounter,
and the outbox changes from zero to two mutations.

## Case 2: Verify normalization and deduplication

Capture the same lemma with different whitespace and language casing:

```text
capture {"selectedText":" serendipity ","lemma":null,"sentence":"  Another   lucky discovery.  ","sourceLanguage":"EN","targetLanguage":"DE","translation":"glücklicher Zufall","partOfSpeech":"noun","sourceApp":"core-debug","sourceTitle":"Duplicate test","sourceUrl":null,"captureOrigin":"manual","capturedAt":"2026-08-29T12:05:00Z"}
```

Then run:

```text
list
db summary
db outbox
```

Expected invariants:

- normalized lemma: `serendipity`;
- normalized sentence: `Another lucky discovery.`;
- the second result has `isExistingWord: true`;
- one word and two active encounters exist;
- three outbox mutations exist: one word upsert and two encounter upserts.

## Case 3: Inspect one word

```text
show <word-uuid>
```

Check the display form, normalized lemma, translation, status, review state,
encounter timeline, and empty initial review log.

## Case 4: Exercise the review schedule

The first capture is immediately due:

```text
today 2026-08-29T12:10:00Z
review <word-uuid> remembered 2026-08-29T12:10:00Z
show <word-uuid>
today 2026-08-29T12:11:00Z
```

After the first `remembered` review, check that stability is `3.0`, the next
review is due three days later, the review log contains one entry, and the word
is no longer in the queue at `12:11`.

For the forgotten path, capture a second word and run:

```text
review <second-word-uuid> forgot 2026-08-29T13:10:00Z
show <second-word-uuid>
```

Check difficulty `5.5`, stability `0.5`, lapse count `1`, and a due time one day
later.

## Case 5: Verify Undo

Copy the second encounter ID from the duplicate capture and run:

```text
undo <second-encounter-uuid>
show <word-uuid>
db summary
db outbox
```

The word remains, only one active encounter remains, and the outbox gains an
encounter delete mutation. This demonstrates that Undo soft-deletes the capture
encounter rather than deleting the vocabulary word.

## Case 6: Inspect and update settings

```text
settings
settings set {"sourceLanguage":"en","targetLanguage":"zh","captureShortcut":"Alt+Shift+V","reviewTime":"20:00","dailyLimit":9,"launchAtLogin":false,"appearance":"system","reducedMotion":false}
settings
```

The final output should contain target language `zh` and daily limit `9`.

## Case 7: Verify error isolation

```text
show not-a-uuid
review not-a-uuid remembered
capture {"selectedText":
db delete
select * from words
db summary
```

Each invalid command prints an error, the REPL remains active, and the final
database summary is unchanged.

## Scope

This tool validates the current shared core linked into the workspace. Its
stable API is `CoreDebugSession`; core internals may change without changing
the REPL as long as that API and its behavior remain compatible. It does not
validate native selection, OCR, translation providers, shortcuts, permissions,
or floating-window behavior.

