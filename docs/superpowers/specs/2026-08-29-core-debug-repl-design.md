# Core Debug REPL Design

## Purpose

Provide a lightweight executable guide for manually exercising and understanding the shared Rust core before Windows adapter work begins. The tool must call the same domain, application, capture, and storage code used by the desktop application, while remaining independent of Tauri and every operating-system adapter.

## Form

The REPL is a Rust example in `crates/application/examples/core_debug.rs`, launched with:

```powershell
cargo run -p vocab-application --example core_debug
cargo run -p vocab-application --example core_debug -- --db .\core-debug.db
```

It is not a product application, workspace crate, platform adapter, or general database console. The example owns terminal input and output. Reusable orchestration and diagnostic result types live in `vocab-application` so tests can call them without spawning a process.

## Database Safety

- Without `--db`, the session uses `SqliteStore::open_in_memory()`. State survives for the lifetime of the REPL and disappears on exit.
- With `--db <path>`, the session opens exactly that SQLite file and prints its resolved absolute path before accepting commands.
- The REPL never searches for or automatically opens the desktop application's `guest.db`.
- Database inspection is limited to predefined read-only methods. Arbitrary SQL is not accepted.

## Public Debug API

`vocab_application::debug` exposes:

```rust
pub struct CoreDebugSession;

impl CoreDebugSession {
    pub fn in_memory(device_id: Uuid) -> Result<Self, ApplicationError>;
    pub fn open(path: impl AsRef<Path>, device_id: Uuid) -> Result<Self, ApplicationError>;
    pub fn capture(&self, input: DebugCaptureInput) -> Result<CaptureTrace, ApplicationError>;
    pub fn list_words(&self) -> Result<Vec<WordListItem>, ApplicationError>;
    pub fn inspect_word(&self, word_id: Uuid) -> Result<WordDebugView, ApplicationError>;
    pub fn today(&self, now: DateTime<Utc>) -> Result<TodayView, ApplicationError>;
    pub fn review(&self, word_id: Uuid, rating: ReviewRating, at: DateTime<Utc>)
        -> Result<WordDebugView, ApplicationError>;
    pub fn undo(&self, encounter_id: Uuid) -> Result<DatabaseSummary, ApplicationError>;
    pub fn settings(&self) -> Result<UserSettings, ApplicationError>;
    pub fn update_settings(&self, settings: UserSettings) -> Result<UserSettings, ApplicationError>;
    pub fn database_summary(&self) -> Result<DatabaseSummary, ApplicationError>;
    pub fn outbox(&self) -> Result<Vec<OutboxDebugEntry>, ApplicationError>;
}
```

`DebugCaptureInput` contains the complete manual capture input: selected text, optional lemma, sentence, source and target languages, optional translation and part of speech, source application/title/URL, capture origin, and capture time. It converts into the existing `CaptureRequest`; it does not introduce a second capture model.

`CaptureTrace` keeps a stable outer API while allowing core internals to evolve:

```rust
pub struct CaptureTrace {
    pub input: DebugCaptureInput,
    pub observations: Vec<DebugObservation>,
    pub result: CaptureCard,
    pub persisted: WordDebugView,
}

pub struct DebugObservation {
    pub stage: DebugStage,
    pub data: serde_json::Value,
}

pub enum DebugStage {
    Input,
    Domain,
    Application,
    Persistence,
}
```

The stage enum represents durable architectural boundaries, not private state-machine phases. Domain data initially includes normalized lemma, normalized sentence, and dedupe key. Persistence data initially includes outbox counts before and after capture. Those JSON objects may gain or replace diagnostic fields without changing the REPL-facing method signatures.

`WordDebugView` combines the public word detail with its review log. `DatabaseSummary` reports schema version, foreign-key status, word count, active encounter count, review-log count, and pending outbox count. `OutboxDebugEntry` exposes mutation metadata and JSON payload for manual inspection. All inspection methods are read-only.

The debug API may retain an `Arc<SqliteStore>` alongside `AppService`; it must not add getters that expose a raw SQLite connection.

## REPL Commands

Commands are whitespace-delimited. JSON payloads occupy the remainder of the line and are parsed with the existing `serde_json` dependency.

```text
help
capture <DebugCaptureInput JSON object>
list
show <word-uuid>
today [RFC3339 timestamp]
review <word-uuid> forgot|remembered [RFC3339 timestamp]
undo <encounter-uuid>
settings
settings set <UserSettings JSON object>
db summary
db outbox
quit
```

`help` includes a complete copyable capture example. Empty lines are ignored. Invalid commands, UUIDs, timestamps, JSON, and core errors print one concise error and return to the prompt. EOF behaves like `quit`. Private capture data is printed only because the user explicitly entered it into this local debugging session; no background logging is added.

## Core Walkthrough

For a capture, the REPL prints these numbered boundaries:

1. Input accepted.
2. Domain normalization and dedupe key.
3. Application capture result.
4. Persistence readback and database/outbox delta.

Repeating a lemma demonstrates deduplication through `CaptureCard::is_existing_word` and the encounter count. Review prints before/after review state. Undo prints the new summary and leaves the word intact, matching current product behavior.

## Testing

- Application integration tests exercise `CoreDebugSession` against in-memory SQLite and verify capture trace, duplicate capture, review, Undo, settings, summary, and outbox inspection.
- Parser tests exercise every REPL command and malformed input without terminal I/O.
- A process smoke test pipes `help`, a complete capture, `db summary`, and `quit` into the example and verifies a successful exit plus stable section labels.
- The existing Windows workspace, Clippy, formatting, and frontend gates remain unchanged.

## Non-Goals

- Native selection, translation, OCR, permissions, shortcuts, or window behavior.
- Arbitrary SQL or direct database mutation.
- A polished terminal UI, command history, completion, colors, or third-party CLI framework.
- Replacing automated tests or proving platform adapters.
- Adding product behavior solely for the debugger.
