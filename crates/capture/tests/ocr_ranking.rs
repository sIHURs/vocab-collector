use vocab_capture::{OcrCandidateResolution, ScreenPoint, ScreenRect, rank_ocr_candidates};
use vocab_platform_api::OcrCandidate;

fn candidate(text: &str, bounds: ScreenRect, confidence: f32) -> OcrCandidate {
    OcrCandidate {
        text: text.into(),
        bounds,
        confidence,
    }
}

#[test]
fn pointer_containment_outranks_distance_and_confidence() {
    let candidates = vec![
        candidate("near", ScreenRect::new(101.0, 100.0, 20.0, 20.0), 0.99),
        candidate(
            "under pointer",
            ScreenRect::new(90.0, 90.0, 20.0, 20.0),
            0.70,
        ),
    ];

    let resolution = rank_ocr_candidates(&candidates, ScreenPoint::new(100.0, 100.0)).unwrap();

    assert_eq!(resolution.candidates()[0].text, "under pointer");
    assert!(matches!(resolution, OcrCandidateResolution::Dominant(_)));
}

#[test]
fn rectangle_distance_outranks_confidence_when_no_candidate_contains_pointer() {
    let candidates = vec![
        candidate(
            "far confident",
            ScreenRect::new(160.0, 100.0, 20.0, 20.0),
            0.99,
        ),
        candidate("near", ScreenRect::new(112.0, 100.0, 20.0, 20.0), 0.60),
    ];

    let resolution = rank_ocr_candidates(&candidates, ScreenPoint::new(100.0, 100.0)).unwrap();

    assert_eq!(resolution.candidates()[0].text, "near");
    assert!(matches!(resolution, OcrCandidateResolution::Dominant(_)));
}

#[test]
fn confidence_breaks_equal_geometry_ties() {
    let candidates = vec![
        candidate("lower", ScreenRect::new(110.0, 100.0, 20.0, 20.0), 0.70),
        candidate("higher", ScreenRect::new(90.0, 100.0, 20.0, 20.0), 0.92),
    ];

    let resolution = rank_ocr_candidates(&candidates, ScreenPoint::new(100.0, 100.0)).unwrap();

    assert_eq!(resolution.candidates()[0].text, "higher");
}

#[test]
fn spatially_close_candidates_remain_available_in_ranked_order() {
    let candidates = vec![
        candidate("third", ScreenRect::new(130.0, 100.0, 20.0, 20.0), 0.99),
        candidate("second", ScreenRect::new(114.0, 100.0, 20.0, 20.0), 0.80),
        candidate("first", ScreenRect::new(108.0, 100.0, 20.0, 20.0), 0.75),
    ];

    let resolution = rank_ocr_candidates(&candidates, ScreenPoint::new(100.0, 100.0)).unwrap();

    let OcrCandidateResolution::Ambiguous(close) = resolution else {
        panic!("expected close candidates to require a choice");
    };
    assert_eq!(
        close
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
}

#[test]
fn confidence_gap_makes_equal_geometry_candidate_dominant() {
    let candidates = vec![
        candidate("uncertain", ScreenRect::new(90.0, 90.0, 20.0, 20.0), 0.60),
        candidate("clear", ScreenRect::new(90.0, 90.0, 20.0, 20.0), 0.90),
    ];

    let resolution = rank_ocr_candidates(&candidates, ScreenPoint::new(100.0, 100.0)).unwrap();

    let OcrCandidateResolution::Dominant(best) = resolution else {
        panic!("expected confidence gap to produce a dominant candidate");
    };
    assert_eq!(best.text, "clear");
}

#[test]
fn empty_candidate_sets_have_no_resolution() {
    assert_eq!(rank_ocr_candidates(&[], ScreenPoint::new(0.0, 0.0)), None);
}
