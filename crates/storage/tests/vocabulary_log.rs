use chrono::{NaiveDate, Utc};
use std::sync::{Arc, Mutex};
use vocab_domain::{CaptureOrigin, EncounterRepository, VocabularyLogRepository};
use vocab_storage::{CaptureRecord, SqliteStore};

fn day(value: &str) -> NaiveDate {
    value.parse().unwrap()
}
fn record() -> CaptureRecord {
    CaptureRecord {
        selected_text: "word".into(),
        lemma: "word".into(),
        sentence: "A word.".into(),
        source_language: "en".into(),
        target_language: "de".into(),
        translation: None,
        part_of_speech: None,
        source_app: None,
        source_title: None,
        source_url: None,
        capture_origin: CaptureOrigin::Manual,
        captured_at: "2000-01-01T00:00:00Z".parse().unwrap(),
    }
}
#[test]
fn successful_saves_use_local_save_day_and_undo_original_day_once() {
    let path = std::env::temp_dir().join(format!("vocab-log-{}.db", uuid::Uuid::now_v7()));
    let clock = Arc::new(Mutex::new(day("2026-03-28")));
    let source = clock.clone();
    let store = SqliteStore::open_with_local_date(&path, Arc::new(move || *source.lock().unwrap()))
        .unwrap();
    let a = store.capture(&record()).unwrap();
    let mut other = record();
    other.lemma = "other".into();
    other.selected_text = "other".into();
    store.capture(&other).unwrap();
    *clock.lock().unwrap() = day("2026-03-29"); // DST / next local date: no 24-hour arithmetic.
    store.capture(&other).unwrap();
    store.soft_delete(a.encounter.id, Utc::now()).unwrap();
    let _ = store.soft_delete(a.encounter.id, Utc::now());
    let log = store.vocabulary_log().unwrap();
    assert_eq!(
        log.days
            .iter()
            .find(|d| d.date == day("2026-03-28"))
            .unwrap()
            .count,
        Some(1)
    );
    assert_eq!(log.days.last().unwrap().count, Some(1));
    assert_eq!(
        log.days.last().unwrap().coverage,
        vocab_domain::LogCoverage::Complete
    );
    assert_eq!(log.days.first().unwrap().count, None);
    drop(store);
    let source = clock.clone();
    let reopened =
        SqliteStore::open_with_local_date(&path, Arc::new(move || *source.lock().unwrap()))
            .unwrap();
    assert_eq!(reopened.vocabulary_log().unwrap(), log);
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn failed_capture_and_failed_undo_roll_back_daily_count_with_encounter() {
    let path = std::env::temp_dir().join(format!("vocab-log-{}.db", uuid::Uuid::now_v7()));
    let store = SqliteStore::open(&path).unwrap();
    let a = store.capture(&record()).unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TRIGGER fail_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
    assert!(store.capture(&record()).is_err());
    assert!(store.soft_delete(a.encounter.id, Utc::now()).is_err());
    assert_eq!(
        store.vocabulary_log().unwrap().days.last().unwrap().count,
        Some(1)
    );
    assert!(
        store.list_for_word(a.word.id).unwrap()[0]
            .deleted_at
            .is_none()
    );
    drop(connection);
    drop(store);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn migration_does_not_reconstruct_history_and_timezone_changes_do_not_rebucket() {
    let path = std::env::temp_dir().join(format!("vocab-log-{}.db", uuid::Uuid::now_v7()));
    let store = SqliteStore::open(&path).unwrap();
    let old = store.capture(&record()).unwrap();
    drop(store);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("DROP TABLE vocabulary_daily_counts; DROP TABLE vocabulary_log_coverage; ALTER TABLE encounters DROP COLUMN saved_local_date; PRAGMA user_version = 5;").unwrap();
    drop(connection);
    let clock = Arc::new(Mutex::new(day("2026-09-09")));
    let source = clock.clone();
    let store = SqliteStore::open_with_local_date(&path, Arc::new(move || *source.lock().unwrap()))
        .unwrap();
    assert_eq!(
        store.vocabulary_log().unwrap().days.last().unwrap().count,
        Some(0)
    );
    store.soft_delete(old.encounter.id, Utc::now()).unwrap();
    store.capture(&record()).unwrap();
    *clock.lock().unwrap() = day("2026-09-08"); // Travel west across the date boundary.
    store.capture(&record()).unwrap();
    assert_eq!(
        store
            .vocabulary_log()
            .unwrap()
            .days
            .last()
            .unwrap()
            .coverage,
        vocab_domain::LogCoverage::Partial
    );
    *clock.lock().unwrap() = day("2026-09-10");
    let log = store.vocabulary_log().unwrap();
    assert_eq!(
        log.days
            .iter()
            .find(|d| d.date == day("2026-09-08"))
            .unwrap()
            .count,
        Some(1)
    );
    assert_eq!(
        log.days
            .iter()
            .find(|d| d.date == day("2026-09-09"))
            .unwrap()
            .count,
        Some(1)
    );
    assert_eq!(log.days.last().unwrap().count, Some(0));
    drop(store);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn achieved_recapture_counts_once_and_purge_keeps_only_anonymous_daily_totals() {
    use vocab_domain::{WordRepository, WordStatus};
    let path = std::env::temp_dir().join(format!("vocab-log-{}.db", uuid::Uuid::now_v7()));
    let store = SqliteStore::open(&path).unwrap();
    let first = store.capture(&record()).unwrap();
    let mut word = first.word;
    word.status = WordStatus::Mastered;
    word.mastered_at = Some(Utc::now());
    word.achieved_at = Some(Utc::now());
    word.delete_after = Some(Utc::now() + chrono::Duration::days(30));
    WordRepository::save(&store, &word).unwrap();
    assert!(store.capture(&record()).is_err());
    store
        .restore_achieved_and_capture(word.id, &record())
        .unwrap();
    assert_eq!(
        store.vocabulary_log().unwrap().days.last().unwrap().count,
        Some(2)
    );
    WordRepository::save(&store, &word).unwrap();
    store.delete_achieved_words(&[word.id], Utc::now()).unwrap();
    assert_eq!(
        store.vocabulary_log().unwrap().days.last().unwrap().count,
        Some(2)
    );
    assert!(store.list_for_word(word.id).unwrap().is_empty());
    assert!(WordRepository::get(&store, word.id).unwrap().is_none());
    assert!(store.soft_delete(first.encounter.id, Utc::now()).is_err());
    drop(store);
    std::fs::remove_file(path).unwrap();
}
