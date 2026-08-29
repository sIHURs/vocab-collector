# Core Debug REPL Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a lightweight Rust REPL example that exposes and explains the real shared core workflow, with safe session-local storage and predefined database inspection.

**Architecture:** A public `vocab_application::debug::CoreDebugSession` composes the existing `AppService` and `SqliteStore` and returns structured diagnostic views. A thin example binary parses REPL commands and formats those views; it never depends on Tauri, an OS adapter, raw SQLite access, or a CLI framework.

**Tech Stack:** Rust 1.98, existing `chrono`, `serde`, `serde_json`, `uuid`, `rusqlite`, and workspace crates.

**Spec:** `docs/superpowers/specs/2026-08-29-core-debug-repl-design.md`

## Global Constraints

- Keep the tool under `crates/application`; do not add a workspace crate or product app.
- Default to session-lifetime in-memory SQLite; open a file only through explicit `--db <path>`.
- Route every mutation through existing application/core APIs.
- Expose predefined read-only diagnostics only; never expose a raw SQLite connection or arbitrary SQL.
- Do not depend on Tauri or macOS, Linux, or Windows adapter crates.
- Do not add a CLI framework, terminal UI framework, colors, history, or completion.
- Preserve the existing uncommitted Windows SQLite test-lifecycle fix in `crates/storage/tests/sqlite_repository.rs`.

---

### Task 1: Add read-only storage diagnostics

**Files:**
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/tests/sqlite_repository.rs`

**Interfaces:**
- Consumes: existing SQLite tables and repository serialization.
- Produces: `DatabaseSummary`, `OutboxDebugEntry`, `SqliteStore::database_summary()`, and `SqliteStore::outbox_debug_entries()`.

- [ ] **Step 1: Write failing diagnostic tests**

Add imports and a test that captures one word, appends a second encounter, records a review through the existing repositories, and checks literal counts:

```rust
#[test]
fn predefined_diagnostics_report_database_state_without_mutating_it() {
    let store = SqliteStore::open_in_memory().unwrap();
    let first = store.capture(&capture("Serendipity", "A lucky moment.")).unwrap();
    store.capture(&capture("serendipity", "Another moment.")).unwrap();

    let summary = store.database_summary().unwrap();
    assert_eq!(summary.schema_version, 2);
    assert!(summary.foreign_keys_enabled);
    assert_eq!(summary.word_count, 1);
    assert_eq!(summary.active_encounter_count, 2);
    assert_eq!(summary.review_log_count, 0);
    assert_eq!(summary.pending_outbox_count, 3);

    let outbox = store.outbox_debug_entries().unwrap();
    assert_eq!(outbox.len(), 3);
    assert_eq!(outbox[0].entity_type, "word");
    assert_eq!(store.list_for_word(first.word.id).unwrap().len(), 2);
}
```

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```powershell
cargo test -p vocab-storage --test sqlite_repository predefined_diagnostics_report_database_state_without_mutating_it -- --exact
```

Expected: compilation fails because `database_summary` and `outbox_debug_entries` do not exist.

- [ ] **Step 3: Add the minimal diagnostic types and queries**

Add serializable public types:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSummary {
    pub schema_version: u32,
    pub foreign_keys_enabled: bool,
    pub word_count: usize,
    pub active_encounter_count: usize,
    pub review_log_count: usize,
    pub pending_outbox_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboxDebugEntry {
    pub mutation_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub attempt_count: u32,
}
```

Implement `database_summary()` with scalar `COUNT(*)` queries and `outbox_debug_entries()` with:

```sql
SELECT mutation_id, entity_type, entity_id, operation, payload, created_at, attempt_count
FROM outbox
ORDER BY created_at, mutation_id
```

Parse UUIDs, timestamps, and payloads through the existing conversion helpers. Convert every signed count with `usize::try_from` and map failures through `repo_error`.

- [ ] **Step 4: Verify storage GREEN**

Run:

```powershell
cargo test -p vocab-storage
cargo clippy -p vocab-storage --all-targets -- -D warnings
```

Expected: all storage tests pass and Clippy emits no warning.

- [ ] **Step 5: Commit the storage diagnostic boundary**

```powershell
git add crates/storage/src/lib.rs crates/storage/tests/sqlite_repository.rs
git commit -m "feat(storage): add read-only debug snapshots"
```

### Task 2: Expose a structured core debug session

**Files:**
- Create: `crates/application/src/debug.rs`
- Modify: `crates/application/src/lib.rs`
- Create: `crates/application/tests/debug_session.rs`

**Interfaces:**
- Consumes: `AppService`, `CaptureRequest`, domain normalization functions, repository traits, `DatabaseSummary`, and `OutboxDebugEntry`.
- Produces: `DebugCaptureInput`, `DebugStage`, `DebugObservation`, `CaptureTrace`, `WordDebugView`, and `CoreDebugSession` with the exact signatures in the spec.

- [ ] **Step 1: Write a failing end-to-end capture trace test**

Create `debug_session.rs` with a literal input fixture and assertions independent of the implementation:

```rust
fn input(text: &str, sentence: &str) -> DebugCaptureInput {
    DebugCaptureInput {
        selected_text: text.into(),
        lemma: None,
        sentence: sentence.into(),
        source_language: "EN".into(),
        target_language: "DE".into(),
        translation: Some("glücklicher Zufall".into()),
        part_of_speech: Some("noun".into()),
        source_app: Some("core-debug".into()),
        source_title: Some("manual fixture".into()),
        source_url: None,
        capture_origin: CaptureOrigin::Manual,
        captured_at: Utc.with_ymd_and_hms(2026, 8, 29, 12, 0, 0).unwrap(),
    }
}

#[test]
fn capture_trace_explains_normalization_application_and_storage() {
    let session = CoreDebugSession::in_memory(Uuid::nil()).unwrap();
    let trace = session.capture(input(" Serendipity ", "  A   lucky moment. ")).unwrap();

    assert_eq!(trace.observations.len(), 4);
    assert_eq!(trace.observations[0].stage, DebugStage::Input);
    assert_eq!(trace.observations[1].stage, DebugStage::Domain);
    assert_eq!(trace.observations[1].data["normalizedLemma"], "serendipity");
    assert_eq!(trace.observations[1].data["normalizedSentence"], "A lucky moment.");
    assert_eq!(trace.observations[1].data["dedupeKey"], "serendipity|en|de");
    assert_eq!(trace.observations[2].stage, DebugStage::Application);
    assert_eq!(trace.observations[3].stage, DebugStage::Persistence);
    assert_eq!(trace.observations[3].data["outboxBefore"], 0);
    assert_eq!(trace.observations[3].data["outboxAfter"], 2);
    assert!(!trace.result.is_existing_word);
    assert_eq!(trace.persisted.detail.item.encounter_count, 1);
    assert_eq!(trace.persisted.detail.encounters[0].selected_text, "Serendipity");
}
```

- [ ] **Step 2: Verify the capture trace test is RED**

Run:

```powershell
cargo test -p vocab-application --test debug_session capture_trace_explains_normalization_application_and_storage -- --exact
```

Expected: compilation fails because `vocab_application::debug` is absent.

- [ ] **Step 3: Implement capture tracing through `AppService`**

In `lib.rs`, add `pub mod debug;`. In `debug.rs`, define serde-enabled `DebugCaptureInput` and this conversion:

```rust
impl From<DebugCaptureInput> for CaptureRequest {
    fn from(value: DebugCaptureInput) -> Self {
        Self {
            selected_text: value.selected_text,
            lemma: value.lemma,
            sentence: value.sentence,
            source_language: value.source_language,
            target_language: value.target_language,
            translation: value.translation,
            part_of_speech: value.part_of_speech,
            source_app: value.source_app,
            source_title: value.source_title,
            source_url: value.source_url,
            capture_origin: value.capture_origin,
            captured_at: value.captured_at,
        }
    }
}
```

`CoreDebugSession` retains `Arc<SqliteStore>` and `AppService`. Its `capture` computes observation values with `normalize_lemma`, `normalize_context`, and `dedupe_key`, records the initial outbox count, calls `AppService::capture`, reads the word back with `get_word`, and records the final outbox count. Do not call `SqliteStore::capture` directly.

- [ ] **Step 4: Add RED tests for duplicate, review, Undo, settings, and diagnostics**

Add focused tests asserting:

```rust
let first = session.capture(input("Serendipity", "First.")).unwrap();
let second = session.capture(input("serendipity", "Second.")).unwrap();
    assert!(!first.result.is_existing_word);
    assert!(second.result.is_existing_word);
    assert_eq!(second.persisted.detail.item.encounter_count, 2);

    let reviewed = session.review(second.result.word_id, ReviewRating::Remembered, at).unwrap();
assert_eq!(reviewed.reviews.len(), 1);
assert_eq!(reviewed.detail.item.next_review_at, Some(at + chrono::Duration::days(3)));

    session.undo(second.result.encounter_id).unwrap();
    assert_eq!(session.inspect_word(second.result.word_id).unwrap().detail.item.encounter_count, 1);

let mut settings = session.settings().unwrap();
settings.daily_limit = 9;
assert_eq!(session.update_settings(settings).unwrap().daily_limit, 9);
assert_eq!(session.database_summary().unwrap().word_count, 1);
assert!(!session.outbox().unwrap().is_empty());
```

Expected RED: methods and `WordDebugView` fields do not exist yet.

- [ ] **Step 5: Implement the remaining thin session methods**

Define:

```rust
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WordDebugView {
    pub detail: WordDetail,
    pub reviews: Vec<ReviewLog>,
}
```

Implement every method by delegating to `AppService` or repository traits. `inspect_word` obtains `WordDetail` from `AppService` and review logs from `ReviewRepository`. `review` delegates to `submit_review` then calls `inspect_word`. `undo` delegates to `undo_capture` then calls `database_summary`. `update_settings` delegates to `AppService` then reads settings back. No method executes SQL.

- [ ] **Step 6: Verify the debug session is GREEN**

Run:

```powershell
cargo test -p vocab-application --test debug_session
cargo test -p vocab-application
cargo clippy -p vocab-application --all-targets -- -D warnings
```

Expected: all application tests pass without warnings.

- [ ] **Step 7: Commit the core debug API**

```powershell
git add crates/application/src/debug.rs crates/application/src/lib.rs crates/application/tests/debug_session.rs
git commit -m "feat(application): expose core debug session"
```

### Task 3: Add the lightweight command parser and REPL

**Files:**
- Create: `crates/application/examples/core_debug.rs`
- Create: `crates/application/tests/debug_repl.rs`
- Modify: `crates/application/Cargo.toml`

**Interfaces:**
- Consumes: `CoreDebugSession` and its structured results.
- Produces: `Command`, `parse_command(&str) -> Result<Command, String>`, `run<R: BufRead, W: Write>(input: R, output: W, session: CoreDebugSession) -> io::Result<()>`, and the `core_debug` example entry point.

- [ ] **Step 1: Write failing parser tests**

Place the parser and runner in the example, and import it into the integration test with:

```rust
#[path = "../examples/core_debug.rs"]
mod core_debug;
```

Add literal tests for every command:

```rust
assert!(matches!(parse_command("help").unwrap(), Command::Help));
assert!(matches!(parse_command("list").unwrap(), Command::List));
assert!(matches!(parse_command("db summary").unwrap(), Command::DatabaseSummary));
assert!(matches!(parse_command("db outbox").unwrap(), Command::Outbox));
assert!(matches!(parse_command("quit").unwrap(), Command::Quit));

let command = parse_command(concat!(
    "capture {\"selectedText\":\"word\",\"lemma\":null,\"sentence\":\"A word.\",",
    "\"sourceLanguage\":\"en\",\"targetLanguage\":\"de\",\"translation\":null,",
    "\"partOfSpeech\":null,\"sourceApp\":\"core-debug\",\"sourceTitle\":null,",
    "\"sourceUrl\":null,\"captureOrigin\":\"manual\",",
    "\"capturedAt\":\"2026-08-29T12:00:00Z\"}",
)).unwrap();
assert!(matches!(command, Command::Capture(_)));
assert!(parse_command("show not-a-uuid").is_err());
assert!(parse_command("db delete").is_err());
assert!(parse_command("select * from words").is_err());
```

Use the complete JSON literal above containing every `DebugCaptureInput` field.

- [ ] **Step 2: Verify parser RED**

Run:

```powershell
cargo test -p vocab-application --test debug_repl parser
```

Expected: compilation fails because `Command` and `parse_command` are absent.

- [ ] **Step 3: Implement parsing with the standard library**

Define `Command` variants matching the spec. Parse the first token with `split_once(' ')`; pass the untouched remainder of `capture` and `settings set` to `serde_json::from_str`. Parse UUIDs with `Uuid::parse_str`, timestamps with `DateTime::parse_from_rfc3339(value).map(|parsed| parsed.with_timezone(&Utc))`, and review ratings with an exact match on `forgot` and `remembered`.

Add `serde_json` as an application dependency rather than a dev dependency because the example uses it:

```toml
serde_json.workspace = true
```

- [ ] **Step 4: Write a failing scripted REPL test**

Use `Cursor<&[u8]>` as input and `Vec<u8>` as output:

```rust
let input = Cursor::new(concat!(
    "help\n",
    "capture {\"selectedText\":\"Serendipity\",\"lemma\":null,\"sentence\":\"A lucky moment.\",",
    "\"sourceLanguage\":\"en\",\"targetLanguage\":\"de\",\"translation\":\"glücklicher Zufall\",",
    "\"partOfSpeech\":\"noun\",\"sourceApp\":\"core-debug\",\"sourceTitle\":null,",
    "\"sourceUrl\":null,\"captureOrigin\":\"manual\",\"capturedAt\":\"2026-08-29T12:00:00Z\"}\n",
    "db summary\n",
    "quit\n",
));
let mut output = Vec::new();
run(input, &mut output, CoreDebugSession::in_memory(Uuid::nil()).unwrap()).unwrap();
let output = String::from_utf8(output).unwrap();
assert!(output.contains("1. Input accepted"));
assert!(output.contains("2. Domain normalization"));
assert!(output.contains("3. Application result"));
assert!(output.contains("4. Persistence readback and database delta"));
assert!(output.contains("word count: 1"));
```

- [ ] **Step 5: Implement REPL dispatch and formatting**

`run` writes `core-debug> ` before each read, ignores blank lines, prints one error prefixed with `error:` and continues, and returns on EOF or `Quit`. Use `serde_json::to_string_pretty` for structured results. Capture formatting must use the four exact boundary labels asserted above. `help` prints every command plus one complete capture JSON example.

`main` accepts only zero arguments or `--db <path>`. For zero arguments, construct `CoreDebugSession::in_memory(Uuid::now_v7())` and print `database: in-memory (discarded on exit)`. For `--db`, resolve with `std::path::absolute`, print the path, and call `CoreDebugSession::open`. Other arguments print usage and exit with code 2.

- [ ] **Step 6: Verify parser and REPL GREEN**

Run:

```powershell
cargo test -p vocab-application --test debug_repl
@('help','quit') | cargo run -p vocab-application --example core_debug
```

Expected: tests pass; the example prints the in-memory safety banner, help, and exits successfully.

- [ ] **Step 7: Commit the REPL**

```powershell
git add crates/application/Cargo.toml crates/application/examples/core_debug.rs crates/application/tests/debug_repl.rs Cargo.lock
git commit -m "feat(application): add lightweight core debug REPL"
```

### Task 4: Document the core walkthrough and run the Windows gate

**Files:**
- Create: `docs/core-debug-repl.md`
- Modify: `docs/development.md`

**Interfaces:**
- Consumes: the completed REPL commands and current Windows verification commands.
- Produces: a copyable manual core-verification walkthrough and interpretation guide.

- [ ] **Step 1: Write the user-facing walkthrough**

Document:

```powershell
cargo run -p vocab-application --example core_debug
cargo run -p vocab-application --example core_debug -- --db .\core-debug.db
```

Include one complete capture JSON line, then `list`, `show`, a duplicate capture, `review`, `today`, `undo`, `settings`, `db summary`, and `db outbox`. Explain the expected invariant after each command: one word/two encounters after duplicate capture, one remaining encounter after Undo, review due date changes, and outbox mutations increase only through core write operations.

- [ ] **Step 2: Link the guide from development documentation**

Add a `Core debugging` subsection to `docs/development.md` linking `docs/core-debug-repl.md` and state that the REPL validates shared behavior only, not native adapters.

- [ ] **Step 3: Run the complete Windows verification gate**

From a Visual Studio Developer PowerShell, run:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --exclude vocab-platform-macos --exclude vocab-platform-linux -- -D warnings
cargo test --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
cargo build --workspace --exclude vocab-platform-macos --exclude vocab-platform-linux
pnpm check
pnpm test
pnpm build
```

Expected: every command exits 0; frontend reports 26 passing tests unless this plan intentionally adds frontend tests, which it does not.

- [ ] **Step 4: Perform the manual persistent-database smoke test**

Run the REPL with `--db .\core-debug-smoke.db`, perform one capture, quit, restart with the same path, and run `list` and `db summary`. Expected: the word remains and summary reports one word. Close the REPL before removing the smoke database on Windows.

- [ ] **Step 5: Commit documentation**

```powershell
git add docs/core-debug-repl.md docs/development.md
git commit -m "docs: add core debug walkthrough"
```

## Plan Self-Review

- Spec coverage: Tasks 1–4 cover safe database modes, structured debug API, complete capture input, five-stage trace, list/show/today/review/Undo/settings, predefined database inspection, error recovery, documentation, and Windows verification.
- Type consistency: `CoreDebugSession`, `DebugCaptureInput`, `CaptureTrace`, `WordDebugView`, `DatabaseSummary`, and `OutboxDebugEntry` retain the same names and ownership across tasks.
- Scope: native providers, arbitrary SQL, Tauri, platform adapters, terminal polish, and new product behavior remain excluded.
