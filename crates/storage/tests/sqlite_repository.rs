use chrono::{TimeZone, Utc};
use uuid::Uuid;
use vocab_domain::{
    Appearance, CaptureOrigin, EncounterRepository, SettingsRepository, WordRepository,
};
use vocab_storage::{CaptureRecord, SqliteStore};

fn capture(selected_text: &str, sentence: &str) -> CaptureRecord {
    CaptureRecord {
        selected_text: selected_text.into(),
        lemma: selected_text.into(),
        sentence: sentence.into(),
        source_language: "en".into(),
        target_language: "de".into(),
        translation: Some("glücklicher Zufall".into()),
        part_of_speech: Some("noun".into()),
        source_app: Some("Safari".into()),
        source_title: Some("An essay".into()),
        source_url: Some("https://example.com/essay".into()),
        capture_origin: CaptureOrigin::Accessibility,
        captured_at: Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap(),
    }
}

#[test]
fn capture_origin_survives_sqlite_and_outbox_round_trips() {
    let path = std::env::temp_dir().join(format!("vocab-origin-{}.db", Uuid::now_v7()));
    {
        let store = SqliteStore::open(&path).unwrap();
        let stored = store
            .capture(&capture("Serendipity", "A lucky moment."))
            .unwrap();
        assert_eq!(
            stored.encounter.capture_origin,
            CaptureOrigin::Accessibility
        );
        assert_eq!(
            store.list_for_word(stored.word.id).unwrap()[0].capture_origin,
            CaptureOrigin::Accessibility
        );
    }
    let connection = rusqlite::Connection::open(&path).unwrap();
    let payload: String = connection.query_row(
        "SELECT payload FROM outbox WHERE entity_type = 'encounter' ORDER BY created_at DESC LIMIT 1",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&payload).unwrap()["captureOrigin"],
        "accessibility"
    );
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn version_one_database_migrates_existing_encounters_to_manual_origin() {
    let path = std::env::temp_dir().join(format!("vocab-v1-{}.db", Uuid::now_v7()));
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch(
        "CREATE TABLE encounters (
           id TEXT PRIMARY KEY, word_id TEXT NOT NULL, selected_text TEXT NOT NULL,
           sentence TEXT NOT NULL, source_app TEXT, source_title TEXT, source_url TEXT,
           captured_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT
         );
         INSERT INTO encounters VALUES ('e1','w1','word','A word.',NULL,NULL,NULL,'2026-01-01','2026-01-01',NULL);
         PRAGMA user_version = 1;",
    ).unwrap();
    drop(connection);

    let store = SqliteStore::open(&path).unwrap();
    assert_eq!(store.schema_version().unwrap(), 4);
    drop(store);
    let connection = rusqlite::Connection::open(&path).unwrap();
    let origin: String = connection
        .query_row(
            "SELECT capture_origin FROM encounters WHERE id = 'e1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(origin, "\"manual\"");
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn legacy_mastered_items_receive_migration_time_without_becoming_achieved() {
    let path = std::env::temp_dir().join(format!("vocab-v3-mastered-{}.db", Uuid::now_v7()));
    let id = Uuid::now_v7();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch(
        "CREATE TABLE words (
           id TEXT PRIMARY KEY, dedupe_key TEXT NOT NULL UNIQUE, owner_scope TEXT NOT NULL,
           lemma TEXT NOT NULL, display_form TEXT NOT NULL, source_language TEXT NOT NULL,
           target_language TEXT NOT NULL, translation TEXT, part_of_speech TEXT, status TEXT NOT NULL,
           created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT, review_state_json TEXT
         ); PRAGMA user_version = 3;"
    ).unwrap();
    connection.execute(
        "INSERT INTO words VALUES(?1,'legacy|en|de','\"guest\"','legacy','Legacy','en','de',NULL,NULL,'\"mastered\"','2026-01-01T00:00:00+00:00','2026-01-01T00:00:00+00:00',NULL,'null')",
        [id.to_string()],
    ).unwrap();
    drop(connection);

    let store = SqliteStore::open(&path).unwrap();
    let word = WordRepository::get(&store, id).unwrap().unwrap();
    assert!(word.mastered_at.is_some());
    assert!(word.achieved_at.is_none());
    assert!(word.delete_after.is_none());
    drop(store);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn duplicate_capture_keeps_one_word_and_all_contexts() {
    let store = SqliteStore::open_in_memory().unwrap();

    let first = store
        .capture(&capture("Serendipity", "A surprising discovery."))
        .unwrap();
    let second = store
        .capture(&capture(" serendipity ", "Another lucky moment."))
        .unwrap();

    assert_eq!(first.word.id, second.word.id);
    assert!(!first.is_existing_word);
    assert!(second.is_existing_word);
    assert_eq!(store.list().unwrap().len(), 1);
    assert_eq!(store.list_for_word(first.word.id).unwrap().len(), 2);
    assert_eq!(store.pending_outbox_count().unwrap(), 3);
}

#[test]
fn undo_soft_deletes_only_the_new_encounter() {
    let store = SqliteStore::open_in_memory().unwrap();
    let result = store
        .capture(&capture("Serendipity", "A surprising discovery."))
        .unwrap();
    let deleted_at = Utc.with_ymd_and_hms(2026, 8, 25, 12, 1, 0).unwrap();

    store.soft_delete(result.encounter.id, deleted_at).unwrap();

    assert!(store.list_for_word(result.word.id).unwrap().is_empty());
    assert_eq!(store.list().unwrap().len(), 1);
    assert_eq!(store.pending_outbox_count().unwrap(), 3);
}

#[test]
fn settings_round_trip_without_an_account() {
    let store = SqliteStore::open_in_memory().unwrap();
    let mut settings = SettingsRepository::get(&store).unwrap();
    settings.target_language = "fr".into();
    settings.appearance = Appearance::Dark;
    settings.reduced_motion = true;

    SettingsRepository::save(&store, &settings).unwrap();

    assert_eq!(SettingsRepository::get(&store).unwrap(), settings);
}

#[test]
fn databases_are_initialized_with_foreign_keys_and_schema_version() {
    let store = SqliteStore::open_in_memory().unwrap();

    assert!(store.foreign_keys_enabled().unwrap());
    assert_eq!(store.schema_version().unwrap(), 4);
}

#[test]
fn predefined_diagnostics_report_database_state_without_mutating_it() {
    let store = SqliteStore::open_in_memory().unwrap();
    let first = store
        .capture(&capture("Serendipity", "A lucky moment."))
        .unwrap();
    store
        .capture(&capture("serendipity", "Another moment."))
        .unwrap();

    let summary = store.database_summary().unwrap();
    assert_eq!(summary.schema_version, 4);
    assert!(summary.foreign_keys_enabled);
    assert_eq!(summary.word_count, 1);
    assert_eq!(summary.active_encounter_count, 2);
    assert_eq!(summary.review_log_count, 0);
    assert_eq!(summary.pending_outbox_count, 3);

    let outbox = store.outbox_debug_entries().unwrap();
    assert_eq!(outbox.len(), 3);
    assert_eq!(
        outbox
            .iter()
            .filter(|entry| entry.entity_type == "word")
            .count(),
        1
    );
    assert_eq!(
        outbox
            .iter()
            .filter(|entry| entry.entity_type == "encounter")
            .count(),
        2
    );
    assert_eq!(store.list_for_word(first.word.id).unwrap().len(), 2);
}

#[test]
fn ids_are_client_generated_uuid_v7_values() {
    let store = SqliteStore::open_in_memory().unwrap();
    let result = store
        .capture(&capture("Serendipity", "A surprising discovery."))
        .unwrap();

    assert_eq!(result.word.id.get_version_num(), 7);
    assert_eq!(result.encounter.id.get_version_num(), 7);
    assert_ne!(result.encounter.id, Uuid::nil());
}
