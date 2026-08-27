use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use vocab_capture::{CaptureCoordinator, CoordinatorError};
use vocab_platform_api::{CaptureCandidate, CaptureOrigin, TranslationResult};

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
    assert!(!coordinator.is_current(first));
    assert!(coordinator.is_current(second));
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
    coordinator.begin_translation(request).unwrap();
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
    coordinator.begin_translation(request).unwrap();
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
    coordinator.begin_translation(request).unwrap();
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
fn translation_must_be_authorized_before_calling_a_provider() {
    let coordinator = CaptureCoordinator::default();
    let request = coordinator.start();
    coordinator
        .set_candidate(request, candidate(CaptureOrigin::Ocr))
        .unwrap();

    assert_eq!(
        coordinator.begin_translation(request),
        Err(CoordinatorError::InvalidTransition)
    );

    coordinator.confirm_ocr(request).unwrap();
    coordinator.begin_translation(request).unwrap();
}

#[test]
fn only_one_translation_provider_call_can_be_in_flight() {
    let coordinator = CaptureCoordinator::default();
    let request = coordinator.start();
    coordinator
        .set_candidate(request, candidate(CaptureOrigin::Accessibility))
        .unwrap();

    coordinator.begin_translation(request).unwrap();
    assert_eq!(
        coordinator.begin_translation(request),
        Err(CoordinatorError::InvalidTransition)
    );
}

#[test]
fn stale_request_publication_never_runs_its_closure() {
    let coordinator = CaptureCoordinator::default();
    let stale = coordinator.start();
    let current = coordinator.start();
    let published = AtomicBool::new(false);

    assert_eq!(
        coordinator.publish_if_current(stale, || published.store(true, Ordering::SeqCst)),
        Err(CoordinatorError::StaleRequest)
    );
    assert!(!published.load(Ordering::SeqCst));
    assert!(coordinator.is_current(current));
}

#[test]
fn starting_a_new_request_waits_for_current_publication_to_finish() {
    let coordinator = Arc::new(CaptureCoordinator::default());
    let request = coordinator.start();
    let publication_entered = Arc::new((Mutex::new(false), Condvar::new()));
    let release_publication = Arc::new((Mutex::new(false), Condvar::new()));

    let publishing_coordinator = coordinator.clone();
    let publishing_entered = publication_entered.clone();
    let publishing_release = release_publication.clone();
    let publication = thread::spawn(move || {
        publishing_coordinator
            .publish_if_current(request, || {
                let (entered, signal) = &*publishing_entered;
                *entered.lock().unwrap() = true;
                signal.notify_one();

                let (released, signal) = &*publishing_release;
                let mut released = released.lock().unwrap();
                while !*released {
                    released = signal.wait(released).unwrap();
                }
            })
            .unwrap();
    });

    let (entered, signal) = &*publication_entered;
    let mut entered = entered.lock().unwrap();
    while !*entered {
        entered = signal.wait(entered).unwrap();
    }
    drop(entered);

    let (started_tx, started_rx) = mpsc::channel();
    let starting_coordinator = coordinator.clone();
    let start = thread::spawn(move || {
        started_tx.send(starting_coordinator.start()).unwrap();
    });
    let start_was_blocked = matches!(
        started_rx.recv_timeout(Duration::from_millis(100)),
        Err(mpsc::RecvTimeoutError::Timeout)
    );

    let (released, signal) = &*release_publication;
    *released.lock().unwrap() = true;
    signal.notify_one();
    publication.join().unwrap();
    let next_request = started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    start.join().unwrap();

    assert!(start_was_blocked);
    assert_ne!(request, next_request);
    assert!(coordinator.is_current(next_request));
}

#[test]
fn candidate_transition_and_publication_use_the_same_current_request_guard() {
    let coordinator = CaptureCoordinator::default();
    let stale = coordinator.start();
    let current = coordinator.start();
    let stale_published = AtomicBool::new(false);
    let current_published = AtomicBool::new(false);

    assert_eq!(
        coordinator.set_candidate_and_publish(stale, candidate(CaptureOrigin::Ocr), || {
            stale_published.store(true, Ordering::SeqCst);
        }),
        Err(CoordinatorError::StaleRequest)
    );
    coordinator
        .set_candidate_and_publish(current, candidate(CaptureOrigin::Ocr), || {
            current_published.store(true, Ordering::SeqCst);
        })
        .unwrap();

    assert!(!stale_published.load(Ordering::SeqCst));
    assert!(current_published.load(Ordering::SeqCst));
    coordinator.confirm_ocr(current).unwrap();
}

#[test]
fn persistence_failure_rolls_back_so_the_user_can_retry() {
    let coordinator = CaptureCoordinator::default();
    let request = coordinator.start();
    coordinator
        .set_candidate(request, candidate(CaptureOrigin::Accessibility))
        .unwrap();
    coordinator.begin_translation(request).unwrap();
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
