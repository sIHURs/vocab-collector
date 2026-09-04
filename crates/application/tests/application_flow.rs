use std::sync::Arc;

use chrono::{TimeZone, Utc};
use uuid::Uuid;
use vocab_application::{AppService, CaptureRequest};
use vocab_domain::{CaptureOrigin, ReviewRating, SettingsRepository};
use vocab_storage::SqliteStore;

fn request(word: &str, translation: &str) -> CaptureRequest {
    CaptureRequest {
        selected_text: word.into(),
        lemma: Some(word.into()),
        sentence: format!("This sentence contains {word}."),
        source_language: "en".into(),
        target_language: "de".into(),
        translation: Some(translation.into()),
        part_of_speech: Some("noun".into()),
        source_app: Some("Safari".into()),
        source_title: Some("Reading".into()),
        source_url: Some("https://example.com".into()),
        capture_origin: CaptureOrigin::Manual,
        captured_at: Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap(),
    }
}

#[test]
fn capture_today_review_and_vocabulary_flow_share_one_source_of_truth() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    let card = service
        .capture(request("Serendipity", "glücklicher Zufall"))
        .unwrap();

    assert_eq!(card.encounter_count, 1);
    assert!(!card.is_existing_word);

    let now = Utc.with_ymd_and_hms(2026, 8, 25, 12, 5, 0).unwrap();
    let today = service.get_today(now).unwrap();
    assert_eq!(today.due_count, 1);
    assert_eq!(today.review_queue[0].word_id, card.word_id);
    assert_eq!(today.recent_captures.len(), 1);

    let result = service
        .submit_review(card.word_id, ReviewRating::Remembered, now)
        .unwrap();
    assert_eq!(result.word_id, card.word_id);
    assert_eq!(result.rating, ReviewRating::Remembered);
    assert_eq!(result.reviewed_at, now);
    assert_eq!(result.previous_stability, 1.0);
    assert_eq!(result.stability, 3.0);
    assert_eq!(
        result.next_due_at,
        Utc.with_ymd_and_hms(2026, 8, 28, 12, 5, 0).unwrap()
    );
    assert_eq!(service.get_today(now).unwrap().due_count, 0);
    assert_eq!(service.list_words().unwrap()[0].encounter_count, 1);
}

#[test]
fn repeated_capture_and_undo_return_frontend_ready_counts() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store, Uuid::now_v7());
    service.capture(request("Serendipity", "Zufall")).unwrap();
    let repeated = service.capture(request("serendipity", "Zufall")).unwrap();

    assert!(repeated.is_existing_word);
    assert_eq!(repeated.encounter_count, 2);

    service.undo_capture(repeated.encounter_id).unwrap();
    assert_eq!(service.list_words().unwrap()[0].encounter_count, 1);
}

#[test]
fn review_result_reports_encounters_and_repeated_forgetting_after_three_consecutive_lapses() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store, Uuid::now_v7());
    let first = service.capture(request("Nuance", "Feinheit")).unwrap();
    service.capture(request("nuance", "Feinheit")).unwrap();
    let start = Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap();

    let one = service
        .submit_review(first.word_id, ReviewRating::Forgot, start)
        .unwrap();
    let two = service
        .submit_review(
            first.word_id,
            ReviewRating::Forgot,
            start + chrono::Duration::days(1),
        )
        .unwrap();
    let three = service
        .submit_review(
            first.word_id,
            ReviewRating::Forgot,
            start + chrono::Duration::days(2),
        )
        .unwrap();

    assert_eq!(three.encounter_count, 2);
    assert!(!one.repeated_forgetting);
    assert!(!two.repeated_forgetting);
    assert!(three.repeated_forgetting);
}

#[test]
fn settings_update_is_visible_to_today_view() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    let mut settings = SettingsRepository::get(store.as_ref()).unwrap();
    settings.target_language = "fr".into();

    service.update_settings(settings.clone()).unwrap();

    let now = Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap();
    assert_eq!(service.get_today(now).unwrap().settings, settings);
}

#[test]
fn automatic_source_and_explicit_target_round_trip_through_settings() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store, Uuid::now_v7());
    let mut settings = service.get_settings().unwrap();
    settings.source_language = "auto".into();
    settings.target_language = "zh-Hant".into();

    service.update_settings(settings).unwrap();

    let saved = service.get_settings().unwrap();
    assert_eq!(saved.source_language, "auto");
    assert_eq!(saved.target_language, "zh-Hant");
}

#[test]
fn automatic_or_blank_target_language_is_rejected() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store, Uuid::now_v7());
    let mut settings = service.get_settings().unwrap();
    settings.target_language = "auto".into();

    assert_eq!(
        service.update_settings(settings).unwrap_err().to_string(),
        "target language must be explicit"
    );
}

#[test]
fn legacy_chinese_language_codes_are_normalized_at_the_application_boundary() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let mut settings = SettingsRepository::get(store.as_ref()).unwrap();
    settings.source_language = "zh".into();
    settings.target_language = "zh".into();
    SettingsRepository::save(store.as_ref(), &settings).unwrap();
    let service = AppService::new(store, Uuid::now_v7());

    let normalized = service.get_settings().unwrap();

    assert_eq!(normalized.source_language, "zh-Hans");
    assert_eq!(normalized.target_language, "zh-Hans");
}

#[test]
fn recent_captures_use_the_configured_limit() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    for index in 0..25 {
        service
            .capture(request(&format!("word-{index}"), "translation"))
            .unwrap();
    }

    let now = Utc.with_ymd_and_hms(2026, 8, 25, 12, 5, 0).unwrap();
    assert_eq!(service.get_today(now).unwrap().recent_captures.len(), 20);

    let mut settings = SettingsRepository::get(store.as_ref()).unwrap();
    settings.recent_captures_limit = 7;
    service.update_settings(settings).unwrap();
    assert_eq!(service.get_today(now).unwrap().recent_captures.len(), 7);
}

#[test]
fn word_detail_contains_the_original_context_timeline() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store, Uuid::now_v7());
    let card = service.capture(request("Serendipity", "Zufall")).unwrap();

    let detail = service.get_word(card.word_id).unwrap();

    assert_eq!(detail.lemma, "serendipity");
    assert_eq!(detail.encounters.len(), 1);
    assert!(detail.encounters[0].sentence.contains("Serendipity"));
}

#[test]
fn unknown_word_detail_is_a_not_found_error() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store, Uuid::now_v7());

    assert!(service.get_word(Uuid::now_v7()).is_err());
}
