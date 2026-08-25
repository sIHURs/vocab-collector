use chrono::{TimeZone, Utc};
use uuid::Uuid;
use vocab_domain::{Appearance, EncounterRepository, SettingsRepository, WordRepository};
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
        captured_at: Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap(),
    }
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
    assert_eq!(store.schema_version().unwrap(), 1);
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
