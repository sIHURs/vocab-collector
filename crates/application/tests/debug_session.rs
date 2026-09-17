use chrono::{Duration, TimeZone, Utc};
use uuid::Uuid;
use vocab_application::debug::{CoreDebugSession, DebugCaptureInput, DebugStage};
use vocab_domain::{CaptureOrigin, ReviewRating};

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
    let trace = session
        .capture(input(" Serendipity ", "  A   lucky moment. "))
        .unwrap();

    assert_eq!(trace.observations.len(), 4);
    assert_eq!(trace.observations[0].stage, DebugStage::Input);
    assert_eq!(trace.observations[1].stage, DebugStage::Domain);
    assert_eq!(trace.observations[1].data["normalizedLemma"], "serendipity");
    assert_eq!(
        trace.observations[1].data["normalizedSentence"],
        "A lucky moment."
    );
    assert_eq!(trace.observations[1].data["dedupeKey"], "serendipity|en");
    assert_eq!(trace.observations[2].stage, DebugStage::Application);
    assert_eq!(trace.observations[3].stage, DebugStage::Persistence);
    assert_eq!(trace.observations[3].data["outboxBefore"], 0);
    assert_eq!(trace.observations[3].data["outboxAfter"], 2);
    assert!(!trace.result.is_existing_word);
    assert_eq!(trace.persisted.detail.item.encounter_count, 1);
    assert_eq!(
        trace.persisted.detail.encounters[0].selected_text,
        "Serendipity"
    );
}

#[test]
fn session_exposes_duplicate_review_and_undo_through_core_apis() {
    let session = CoreDebugSession::in_memory(Uuid::nil()).unwrap();
    let first = session.capture(input("Serendipity", "First.")).unwrap();
    let mut second_input = input("serendipity", "Second.");
    second_input.captured_at = Utc.with_ymd_and_hms(2026, 8, 29, 12, 5, 0).unwrap();
    let second = session.capture(second_input).unwrap();

    assert!(!first.result.is_existing_word);
    assert!(second.result.is_existing_word);
    assert_eq!(second.persisted.detail.item.encounter_count, 2);
    assert_eq!(session.list_words().unwrap().len(), 1);

    let reviewed_at = Utc.with_ymd_and_hms(2026, 8, 29, 12, 10, 0).unwrap();
    let reviewed = session
        .review(second.result.word_id, ReviewRating::Remembered, reviewed_at)
        .unwrap();
    assert_eq!(reviewed.reviews.len(), 1);
    assert_eq!(
        reviewed.detail.item.next_review_at,
        Some(reviewed_at + Duration::days(3))
    );

    assert!(session.undo(second.result.encounter_id).is_err());
    let newest = session
        .capture(input("serendipity", "After review."))
        .unwrap();
    let summary = session.undo(newest.result.encounter_id).unwrap();
    assert_eq!(summary.active_encounter_count, 2);
    assert_eq!(
        session
            .inspect_word(second.result.word_id)
            .unwrap()
            .detail
            .item
            .encounter_count,
        2
    );
}

#[test]
fn session_exposes_settings_today_and_read_only_database_diagnostics() {
    let session = CoreDebugSession::in_memory(Uuid::nil()).unwrap();
    let trace = session
        .capture(input("Ephemeral", "It was brief."))
        .unwrap();

    let mut settings = session.settings().unwrap();
    settings.daily_limit = 9;
    settings.target_language = "zh".into();
    let updated = session.update_settings(settings).unwrap();
    assert_eq!(updated.daily_limit, 9);
    assert_eq!(updated.target_language, "zh-Hans");

    let today = session
        .today(Utc.with_ymd_and_hms(2026, 8, 29, 12, 1, 0).unwrap())
        .unwrap();
    assert_eq!(today.total_due_count, 1);
    assert_eq!(today.review_queue[0].word_id, trace.result.word_id);

    let summary = session.database_summary().unwrap();
    assert_eq!(summary.word_count, 1);
    assert_eq!(summary.active_encounter_count, 1);
    assert_eq!(summary.pending_outbox_count, 2);
    assert_eq!(session.outbox().unwrap().len(), 2);
}
