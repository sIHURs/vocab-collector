use std::sync::Arc;

use chrono::{Duration, TimeZone, Utc};
use uuid::Uuid;
use vocab_application::{AppService, CaptureRequest};
use vocab_domain::{
    CaptureOrigin, ReviewRating, ReviewRepository, SettingsRepository, WordRepository, WordStatus,
};
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
    assert_eq!(today.total_due_count, 1);
    assert_eq!(today.planned_review_count, 1);
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
    assert_eq!(service.get_today(now).unwrap().total_due_count, 0);
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
fn user_can_achieve_one_mastered_vocabulary_item() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    let card = service.capture(request("Achieve", "erreichen")).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 0).unwrap();
    let mut word = WordRepository::get(store.as_ref(), card.word_id)
        .unwrap()
        .unwrap();
    word.enter_mastered(now);
    WordRepository::save(store.as_ref(), &word).unwrap();

    let achieved = service.achieve_word(card.word_id, now).unwrap();

    assert_eq!(achieved.delete_after, now + Duration::days(30));
    assert_eq!(achieved.remaining_days, 30);
    assert!(service.list_words().unwrap().is_empty());
    assert_eq!(service.list_achieved_words().unwrap()[0].id, card.word_id);
}

#[test]
fn achieved_capture_requires_consent_then_atomically_returns_the_item_to_learning() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    let original = service.capture(request("Achieve", "erreichen")).unwrap();
    let achieved_at = Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 0).unwrap();
    let mut word = WordRepository::get(store.as_ref(), original.word_id)
        .unwrap()
        .unwrap();
    word.enter_mastered(achieved_at);
    WordRepository::save(store.as_ref(), &word).unwrap();
    service.achieve_word(original.word_id, achieved_at).unwrap();
    let repeated = request("achieve", "erreichen");

    let conflict = service
        .find_achieved_capture(&repeated)
        .unwrap()
        .expect("Achieved capture should require consent");
    assert_eq!(conflict.word_id, original.word_id);
    assert_eq!(conflict.display_form, "Achieve");
    assert_eq!(service.list_achieved_words().unwrap().len(), 1);

    let saved = service
        .restore_achieved_and_capture(conflict.word_id, repeated)
        .unwrap();
    let restored = WordRepository::get(store.as_ref(), original.word_id)
        .unwrap()
        .unwrap();
    assert_eq!(restored.status, WordStatus::Learning);
    assert_eq!(restored.mastered_at, None);
    assert_eq!(restored.achieved_at, None);
    assert_eq!(restored.delete_after, None);
    assert_eq!(saved.encounter_count, 2);
}

#[test]
fn achieved_capture_matches_an_auto_detected_source_when_recapture_resolves_the_language() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    let mut original_request = request("Validate", "验证");
    original_request.source_language = "auto".into();
    original_request.target_language = "zh-Hans".into();
    let original = service.capture(original_request).unwrap();
    let achieved_at = Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 0).unwrap();
    let mut word = WordRepository::get(store.as_ref(), original.word_id)
        .unwrap()
        .unwrap();
    word.enter_mastered(achieved_at);
    WordRepository::save(store.as_ref(), &word).unwrap();
    service.achieve_word(original.word_id, achieved_at).unwrap();
    let mut recapture = request("validate", "验证");
    recapture.source_language = "en".into();
    recapture.target_language = "zh-Hans".into();

    let conflict = service
        .find_achieved_capture(&recapture)
        .unwrap()
        .expect("auto source should match its later resolved language");
    assert_eq!(conflict.word_id, original.word_id);

    service
        .restore_achieved_and_capture(conflict.word_id, recapture)
        .unwrap();
    assert_eq!(service.list_words().unwrap().len(), 1);
    assert_eq!(service.list_words().unwrap()[0].encounter_count, 2);
}

#[test]
fn global_insight_preserves_lifetime_totals_after_an_achieved_item_is_purged() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    let now = Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 0).unwrap();
    let card = service.capture(request("Archive", "Archiv")).unwrap();
    service
        .submit_review(card.word_id, ReviewRating::Remembered, now)
        .unwrap();
    let mut word = WordRepository::get(store.as_ref(), card.word_id)
        .unwrap()
        .unwrap();
    word.enter_mastered(now);
    WordRepository::save(store.as_ref(), &word).unwrap();
    service.achieve_word(card.word_id, now).unwrap();

    let before = service.get_global_insight().unwrap();
    service.delete_achieved_words(&[card.word_id], now).unwrap();
    let after = service.get_global_insight().unwrap();

    assert_eq!(
        after.lifetime_vocabulary_count,
        before.lifetime_vocabulary_count
    );
    assert_eq!(
        after.lifetime_encounter_count,
        before.lifetime_encounter_count
    );
    assert_eq!(after.lifetime_review_count, before.lifetime_review_count);
    assert_eq!(after.lifetime_remembered_count, 1);
    assert_eq!(after.lifetime_forgotten_count, 0);
    assert_eq!(after.current_vocabulary_count, 0);
    assert_eq!(after.current_achieved_count, 0);
}

#[test]
fn user_can_unachieve_or_permanently_delete_achieved_items_in_batches() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    let now = Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 0).unwrap();
    let first = service.capture(request("First", "erste")).unwrap();
    let second = service.capture(request("Second", "zweite")).unwrap();
    for id in [first.word_id, second.word_id] {
        let mut word = WordRepository::get(store.as_ref(), id).unwrap().unwrap();
        word.enter_mastered(now);
        WordRepository::save(store.as_ref(), &word).unwrap();
        service.achieve_word(id, now).unwrap();
    }

    assert_eq!(
        service
            .unachieve_words(&[first.word_id], now + Duration::days(1))
            .unwrap(),
        1
    );
    let restored = WordRepository::get(store.as_ref(), first.word_id)
        .unwrap()
        .unwrap();
    assert!(!restored.is_achieved());
    assert_eq!(restored.mastered_at, Some(now + Duration::days(1)));

    assert_eq!(
        service
            .delete_achieved_words(&[second.word_id, second.word_id], now + Duration::days(1))
            .unwrap(),
        1
    );
    assert!(
        WordRepository::get(store.as_ref(), second.word_id)
            .unwrap()
            .is_none()
    );
    let summary = store.database_summary().unwrap();
    assert_eq!(summary.archived_vocabulary_count, 1);
    assert_eq!(summary.archived_encounter_count, 1);
}

#[test]
fn opted_in_lifecycle_sweep_achieves_at_thirty_days_and_purges_expired_items() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    let mastered_at = Utc.with_ymd_and_hms(2026, 8, 1, 12, 0, 0).unwrap();
    let card = service
        .capture(request("Automatic", "automatisch"))
        .unwrap();
    let mut word = WordRepository::get(store.as_ref(), card.word_id)
        .unwrap()
        .unwrap();
    word.enter_mastered(mastered_at);
    WordRepository::save(store.as_ref(), &word).unwrap();
    let mut settings = service.get_settings().unwrap();
    settings.automatic_achieve_enabled = true;
    settings.achieved_retention_days = 10;
    service.update_settings(settings).unwrap();

    let achieved = service
        .run_lifecycle_sweep(mastered_at + Duration::days(30))
        .unwrap();
    assert_eq!(achieved.achieved_count, 1);
    assert_eq!(achieved.purged_count, 0);
    let purged = service
        .run_lifecycle_sweep(mastered_at + Duration::days(40))
        .unwrap();
    assert_eq!(purged.purged_count, 1);
    assert!(
        WordRepository::get(store.as_ref(), card.word_id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn automatic_achieve_stays_off_without_opt_in_and_existing_deadlines_do_not_follow_settings() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    let mastered_at = Utc.with_ymd_and_hms(2026, 8, 1, 12, 0, 0).unwrap();
    let card = service.capture(request("Consent", "Zustimmung")).unwrap();
    let mut word = WordRepository::get(store.as_ref(), card.word_id)
        .unwrap()
        .unwrap();
    word.enter_mastered(mastered_at);
    WordRepository::save(store.as_ref(), &word).unwrap();
    assert_eq!(
        service
            .run_lifecycle_sweep(mastered_at + Duration::days(60))
            .unwrap()
            .achieved_count,
        0
    );

    let achieved = service
        .achieve_word(card.word_id, mastered_at + Duration::days(60))
        .unwrap();
    let original_deadline = achieved.delete_after;
    let mut settings = service.get_settings().unwrap();
    settings.achieved_retention_days = 10;
    service.update_settings(settings).unwrap();
    assert_eq!(
        service.list_achieved_words().unwrap()[0].delete_after,
        original_deadline
    );
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
fn retrying_one_logical_review_submission_is_idempotent() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    let card = service.capture(request("Durable", "beständig")).unwrap();
    let reviewed_at = Utc.with_ymd_and_hms(2026, 8, 25, 12, 5, 0).unwrap();
    let submission_id = Uuid::now_v7();

    let first = service
        .submit_review_once(
            submission_id,
            card.word_id,
            ReviewRating::Remembered,
            reviewed_at,
        )
        .unwrap();
    let word_after_first = WordRepository::get(store.as_ref(), card.word_id)
        .unwrap()
        .unwrap();
    let retry = service
        .submit_review_once(
            submission_id,
            card.word_id,
            ReviewRating::Remembered,
            reviewed_at,
        )
        .unwrap();
    let word_after_retry = WordRepository::get(store.as_ref(), card.word_id)
        .unwrap()
        .unwrap();

    assert_eq!(retry.word_id, first.word_id);
    assert_eq!(retry.rating, first.rating);
    assert_eq!(retry, first);
    assert_eq!(word_after_retry.review_state, word_after_first.review_state);
    assert_eq!(
        ReviewRepository::list_for_word(store.as_ref(), card.word_id)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn session_insight_counts_successful_results_and_next_day_workload() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store, Uuid::now_v7());
    let first = service.capture(request("Lucid", "klar")).unwrap();
    let second = service.capture(request("Nuance", "Feinheit")).unwrap();
    let reviewed_at = Utc.with_ymd_and_hms(2026, 8, 25, 12, 5, 0).unwrap();
    let remembered = service
        .submit_review(first.word_id, ReviewRating::Remembered, reviewed_at)
        .unwrap();
    service
        .submit_review(
            second.word_id,
            ReviewRating::Forgot,
            reviewed_at - chrono::Duration::days(2),
        )
        .unwrap();
    service
        .submit_review(
            second.word_id,
            ReviewRating::Forgot,
            reviewed_at - chrono::Duration::days(1),
        )
        .unwrap();
    let forgot = service
        .submit_review(second.word_id, ReviewRating::Forgot, reviewed_at)
        .unwrap();

    let insight = service
        .get_review_session_insight(
            &[
                remembered.submission_id,
                remembered.submission_id,
                Uuid::now_v7(),
                forgot.submission_id,
            ],
            Utc.with_ymd_and_hms(2026, 8, 27, 0, 0, 0).unwrap(),
        )
        .unwrap();

    assert_eq!(insight.reviewed_count, 2);
    assert_eq!(insight.remembered_count, 1);
    assert_eq!(insight.forgotten_count, 1);
    assert_eq!(insight.attention_word_ids, vec![second.word_id]);
    assert_eq!(insight.next_day_due_count, 1);
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
fn today_separates_the_complete_due_backlog_from_the_daily_plan() {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let service = AppService::new(store.clone(), Uuid::now_v7());
    for index in 0..7 {
        service
            .capture(request(&format!("due-{index}"), "translation"))
            .unwrap();
    }
    let mut settings = SettingsRepository::get(store.as_ref()).unwrap();
    settings.daily_limit = 5;
    service.update_settings(settings).unwrap();

    let today = service
        .get_today(Utc.with_ymd_and_hms(2026, 8, 25, 12, 5, 0).unwrap())
        .unwrap();

    assert_eq!(today.total_due_count, 7);
    assert_eq!(today.planned_review_count, 5);
    assert_eq!(today.review_queue.len(), 5);
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
