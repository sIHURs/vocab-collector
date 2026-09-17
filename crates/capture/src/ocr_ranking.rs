use std::cmp::Ordering;

use vocab_platform_api::{OcrCandidate, ScreenPoint, ScreenRect};

const CLOSE_DISTANCE: f64 = 12.0;
const DOMINANT_CONFIDENCE_GAP: f32 = 0.20;

#[derive(Clone, Debug, PartialEq)]
pub enum OcrCandidateResolution {
    Dominant(OcrCandidate),
    Ambiguous(Vec<OcrCandidate>),
}

impl OcrCandidateResolution {
    pub fn candidates(&self) -> &[OcrCandidate] {
        match self {
            Self::Dominant(candidate) => std::slice::from_ref(candidate),
            Self::Ambiguous(candidates) => candidates,
        }
    }
}

pub fn rank_ocr_candidates(
    candidates: &[OcrCandidate],
    pointer: ScreenPoint,
) -> Option<OcrCandidateResolution> {
    let mut ranked = candidates
        .iter()
        .cloned()
        .map(|candidate| {
            let contains = contains(candidate.bounds, pointer);
            let distance = distance_to_rect(candidate.bounds, pointer);
            (candidate, contains, distance)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.2.total_cmp(&right.2))
            .then_with(|| {
                right
                    .0
                    .confidence
                    .partial_cmp(&left.0.confidence)
                    .unwrap_or(Ordering::Equal)
            })
    });

    let (_, best_contains, best_distance) = ranked.first()?;
    let close = ranked
        .iter()
        .take_while(|(_, contains, distance)| {
            contains == best_contains && distance - best_distance <= CLOSE_DISTANCE
        })
        .map(|(candidate, _, _)| candidate.clone())
        .collect::<Vec<_>>();

    let confidence_is_dominant = close
        .get(1)
        .is_some_and(|second| close[0].confidence - second.confidence >= DOMINANT_CONFIDENCE_GAP);
    if close.len() == 1 || confidence_is_dominant {
        Some(OcrCandidateResolution::Dominant(close.into_iter().next()?))
    } else {
        Some(OcrCandidateResolution::Ambiguous(close))
    }
}

fn contains(rect: ScreenRect, point: ScreenPoint) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.width
        && point.y >= rect.y
        && point.y <= rect.y + rect.height
}

fn distance_to_rect(rect: ScreenRect, point: ScreenPoint) -> f64 {
    let dx = if point.x < rect.x {
        rect.x - point.x
    } else if point.x > rect.x + rect.width {
        point.x - (rect.x + rect.width)
    } else {
        0.0
    };
    let dy = if point.y < rect.y {
        rect.y - point.y
    } else if point.y > rect.y + rect.height {
        point.y - (rect.y + rect.height)
    } else {
        0.0
    };
    dx.hypot(dy)
}
