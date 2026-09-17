use chrono::{DateTime, Duration, Utc};

use crate::{ReviewRating, ReviewResult, ReviewSessionInsight, ReviewState, Word, WordStatus};

pub fn summarize_review_session(
    results: &[ReviewResult],
    next_day_due_count: usize,
) -> ReviewSessionInsight {
    ReviewSessionInsight {
        reviewed_count: results.len(),
        remembered_count: results
            .iter()
            .filter(|result| result.rating == ReviewRating::Remembered)
            .count(),
        forgotten_count: results
            .iter()
            .filter(|result| result.rating == ReviewRating::Forgot)
            .count(),
        attention_word_ids: results
            .iter()
            .filter(|result| result.repeated_forgetting)
            .map(|result| result.word_id)
            .collect(),
        next_day_due_count,
    }
}

pub fn apply_review(
    prior: Option<&ReviewState>,
    rating: ReviewRating,
    reviewed_at: DateTime<Utc>,
) -> ReviewState {
    let difficulty = prior.map_or(5.0, |state| state.difficulty);
    let stability = prior.map_or(1.0, |state| state.stability);
    let lapse_count = prior.map_or(0, |state| state.lapse_count);

    match rating {
        ReviewRating::Forgot => ReviewState {
            difficulty: (difficulty + 0.5).min(10.0),
            stability: (stability * 0.4).max(0.5),
            due_at: reviewed_at + Duration::days(1),
            last_reviewed_at: Some(reviewed_at),
            lapse_count: lapse_count + 1,
        },
        ReviewRating::Remembered => {
            let new_stability = (stability * 2.5).max(3.0);
            ReviewState {
                difficulty: (difficulty - 0.15).max(1.0),
                stability: new_stability,
                due_at: reviewed_at + Duration::days(new_stability.round() as i64),
                last_reviewed_at: Some(reviewed_at),
                lapse_count,
            }
        }
    }
}

pub fn build_review_queue<'a>(
    words: impl IntoIterator<Item = &'a Word>,
    now: DateTime<Utc>,
    limit: usize,
) -> Vec<&'a Word> {
    let mut due: Vec<_> = words
        .into_iter()
        .filter(|word| {
            word.deleted_at.is_none()
                && word.status == WordStatus::Learning
                && word
                    .review_state
                    .as_ref()
                    .is_some_and(|state| state.due_at <= now)
        })
        .collect();
    due.sort_by_key(|word| word.review_state.as_ref().map(|state| state.due_at));
    due.truncate(limit);
    due
}
