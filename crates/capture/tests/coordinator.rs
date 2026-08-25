use vocab_capture::{CaptureCoordinator, CoordinatorError};
use vocab_platform::{CaptureCandidate, CaptureOrigin, TranslationResult};

fn candidate(origin: CaptureOrigin) -> CaptureCandidate {
    CaptureCandidate {
        selected_text: "serendipity".into(),
        sentence: "It was pure serendipity.".into(),
        source_app: Some("Safari".into()),
        source_title: None,
        source_url: None,
        selection_bounds: None,
        origin,
    }
}

fn translation() -> TranslationResult {
    TranslationResult {
        translated_text: "glücklicher Zufall".into(),
        source_language: "en".into(),
        target_language: "de".into(),
    }
}

#[test]
fn starting_again_rejects_every_late_result_from_the_previous_request() {
    let coordinator = CaptureCoordinator::default();
    let first = coordinator.start();
    coordinator
        .set_candidate(first, candidate(CaptureOrigin::Accessibility))
        .unwrap();
    let second = coordinator.start();

    assert_eq!(
        coordinator.set_translation(first, translation()),
        Err(CoordinatorError::StaleRequest)
    );
    assert_ne!(first, second);
}

#[test]
fn ocr_requires_confirmation_before_translation_or_save() {
    let coordinator = CaptureCoordinator::default();
    let request = coordinator.start();
    coordinator
        .set_candidate(request, candidate(CaptureOrigin::Ocr))
        .unwrap();

    assert_eq!(
        coordinator.set_translation(request, translation()),
        Err(CoordinatorError::InvalidTransition)
    );
    assert_eq!(
        coordinator.save_with(request, false, |_| Ok::<_, &str>(())),
        Err(CoordinatorError::InvalidTransition)
    );

    coordinator.confirm_ocr(request).unwrap();
    coordinator.set_translation(request, translation()).unwrap();
    coordinator
        .save_with(request, false, |_| Ok::<_, &str>(()))
        .unwrap();
}

#[test]
fn a_request_can_reach_persistence_only_once() {
    let coordinator = CaptureCoordinator::default();
    let request = coordinator.start();
    coordinator
        .set_candidate(request, candidate(CaptureOrigin::Accessibility))
        .unwrap();
    coordinator.set_translation(request, translation()).unwrap();
    coordinator
        .save_with(request, false, |_| Ok::<_, &str>(()))
        .unwrap();

    assert_eq!(
        coordinator.save_with(request, false, |_| Ok::<_, &str>(())),
        Err(CoordinatorError::AlreadySaved)
    );
}

#[test]
fn failed_translation_requires_the_explicit_save_without_translation_path() {
    let coordinator = CaptureCoordinator::default();
    let request = coordinator.start();
    coordinator
        .set_candidate(request, candidate(CaptureOrigin::Accessibility))
        .unwrap();
    coordinator.translation_failed(request).unwrap();

    assert_eq!(
        coordinator.save_with(request, false, |_| Ok::<_, &str>(())),
        Err(CoordinatorError::TranslationRequired)
    );
    coordinator
        .save_with(request, true, |_| Ok::<_, &str>(()))
        .unwrap();
}

#[test]
fn persistence_failure_rolls_back_so_the_user_can_retry() {
    let coordinator = CaptureCoordinator::default();
    let request = coordinator.start();
    coordinator
        .set_candidate(request, candidate(CaptureOrigin::Accessibility))
        .unwrap();
    coordinator.set_translation(request, translation()).unwrap();

    let result = coordinator.save_with(request, false, |_| Err::<(), _>("disk full"));
    assert_eq!(
        result,
        Err(CoordinatorError::Persistence("disk full".into()))
    );
    coordinator
        .save_with(request, false, |_| Ok::<_, &str>(()))
        .unwrap();
}
