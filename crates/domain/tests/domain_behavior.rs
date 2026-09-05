use chrono::{Duration, TimeZone, Utc};
use uuid::Uuid;
use vocab_domain::{
    Appearance, CaptureOrigin, Encounter, OwnerScope, ReviewRating, ReviewState, UserSettings,
    Word, WordStatus, apply_review, build_review_queue, dedupe_key, normalize_context,
    normalize_lemma,
};

fn word(lemma: &str, due_offset_hours: i64) -> Word {
    let now = Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap();
    Word {
        id: Uuid::now_v7(),
        owner_scope: OwnerScope::Guest,
        lemma: lemma.into(),
        display_form: lemma.into(),
        source_language: "en".into(),
        target_language: "de".into(),
        translation: Some(format!("{lemma}-de")),
        part_of_speech: None,
        status: WordStatus::Learning,
        created_at: now,
        updated_at: now,
        deleted_at: None,
        mastered_at: None,
        achieved_at: None,
        delete_after: None,
        review_state: Some(ReviewState {
            difficulty: 5.0,
            stability: 1.0,
            due_at: now + Duration::hours(due_offset_hours),
            last_reviewed_at: None,
            lapse_count: 0,
        }),
    }
}

#[test]
fn normalization_preserves_meaning_while_making_dedupe_stable() {
    assert_eq!(normalize_lemma("  Serendipity\n"), "serendipity");
    assert_eq!(
        normalize_context("A   happy\n accident."),
        "A happy accident."
    );
    assert_eq!(
        dedupe_key("  Serendipity ", "EN", "de"),
        "serendipity|en|de"
    );
}

#[test]
fn daily_review_queue_is_due_ordered_and_capped_at_five() {
    let now = Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap();
    let words = [
        word("six", -1),
        word("two", -5),
        word("future", 1),
        word("four", -3),
        word("one", -6),
        word("five", -2),
        word("three", -4),
    ];

    let queue = build_review_queue(words.iter(), now, 5);
    let lemmas: Vec<_> = queue.iter().map(|item| item.lemma.as_str()).collect();

    assert_eq!(lemmas, vec!["one", "two", "three", "four", "five"]);
}

#[test]
fn mastered_and_deleted_words_never_enter_review() {
    let now = Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap();
    let mut mastered = word("mastered", -2);
    mastered.status = WordStatus::Mastered;
    let mut deleted = word("deleted", -3);
    deleted.deleted_at = Some(now);

    assert!(build_review_queue([&mastered, &deleted], now, 5).is_empty());
}

#[test]
fn encounter_keeps_exact_selection_and_normalized_sentence() {
    let now = Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap();
    let encounter = Encounter::new(
        Uuid::now_v7(),
        "Serendipity".into(),
        "A   moment of\nserendipity.".into(),
        Some("Safari".into()),
        now,
    );

    assert_eq!(encounter.selected_text, "Serendipity");
    assert_eq!(encounter.sentence, "A moment of serendipity.");
    assert_eq!(encounter.capture_origin, CaptureOrigin::Manual);
}

#[test]
fn settings_defaults_match_the_five_word_product_promise() {
    let settings = UserSettings::default();

    assert_eq!(settings.source_language, "en");
    assert_eq!(settings.selection_capture_shortcut, "Alt+Shift+V");
    assert_eq!(settings.region_ocr_capture_shortcut, "Alt+Shift+O");
    assert_eq!(settings.daily_limit, 5);
    assert_eq!(settings.recent_captures_limit, 20);
    assert_eq!(settings.review_time, "18:00");
    assert_eq!(settings.appearance, Appearance::System);
    assert!(!settings.reduced_motion);
}

#[test]
fn legacy_capture_shortcut_migrates_to_selection_and_adds_region_ocr_default() {
    let settings: UserSettings = serde_json::from_str(
        r#"{"sourceLanguage":"en","targetLanguage":"de","captureShortcut":"Control+Shift+W","reviewTime":"18:00","dailyLimit":5,"launchAtLogin":false,"appearance":"system","reducedMotion":false}"#,
    )
    .unwrap();

    assert_eq!(settings.selection_capture_shortcut, "Control+Shift+W");
    assert_eq!(settings.region_ocr_capture_shortcut, "Alt+Shift+O");
    assert!(!settings.automatic_achieve_enabled);
    assert_eq!(settings.achieved_retention_days, 30);
}

#[test]
fn only_mastered_vocabulary_items_can_be_achieved() {
    let now = Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 0).unwrap();
    let mut item = word("achieve", 0);
    assert!(item.achieve(now, 30).is_err());

    item.enter_mastered(now);
    item.achieve(now, 30).unwrap();

    assert_eq!(item.status, WordStatus::Mastered);
    assert_eq!(item.achieved_at, Some(now));
    assert_eq!(item.delete_after, Some(now + Duration::days(30)));
    assert!(item.achieve(now, 30).is_err());
}

#[test]
fn achieved_retention_accepts_only_product_options() {
    let now = Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 0).unwrap();
    let mut item = word("achieve", 0);
    item.enter_mastered(now);

    assert!(item.achieve(now, 15).is_err());
    assert!(UserSettings::retention_days_are_valid(10));
    assert!(UserSettings::retention_days_are_valid(60));
    assert!(!UserSettings::retention_days_are_valid(15));
}

#[test]
fn automatic_achieve_requires_thirty_uninterrupted_mastered_days() {
    let mastered_at = Utc.with_ymd_and_hms(2026, 8, 1, 12, 0, 0).unwrap();
    let mut item = word("durable", 0);
    item.enter_mastered(mastered_at);

    assert!(
        !item.is_automatic_achieve_due(mastered_at + Duration::days(30) - Duration::seconds(1))
    );
    assert!(item.is_automatic_achieve_due(mastered_at + Duration::days(30)));
    item.unmaster(mastered_at + Duration::days(10), WordStatus::Learning);
    assert!(!item.is_automatic_achieve_due(mastered_at + Duration::days(40)));
}

#[test]
fn review_ratings_expose_the_two_v1_actions() {
    assert_eq!(ReviewRating::Forgot.fsrs_grade(), 1);
    assert_eq!(ReviewRating::Remembered.fsrs_grade(), 3);
}

#[test]
fn review_updates_are_deterministic_and_forgotten_words_return_soon() {
    let reviewed_at = Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap();
    let prior = ReviewState {
        difficulty: 5.0,
        stability: 4.0,
        due_at: reviewed_at,
        last_reviewed_at: None,
        lapse_count: 0,
    };

    let forgot = apply_review(Some(&prior), ReviewRating::Forgot, reviewed_at);
    let remembered = apply_review(Some(&prior), ReviewRating::Remembered, reviewed_at);

    assert_eq!(forgot.due_at, reviewed_at + Duration::days(1));
    assert_eq!(forgot.lapse_count, 1);
    assert!(remembered.due_at > forgot.due_at);
    assert_eq!(remembered.last_reviewed_at, Some(reviewed_at));
}
