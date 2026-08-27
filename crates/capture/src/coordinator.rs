use std::{fmt::Display, sync::Mutex};

use uuid::Uuid;
use vocab_platform_api::{CaptureCandidate, CaptureOrigin, TranslationResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Capturing,
    AwaitingOcrConfirmation,
    TranslationPending,
    Translating,
    TranslationFailed,
    ReadyToSave,
    Saving,
    Saved,
}

#[derive(Clone, Debug)]
pub struct CaptureSnapshot {
    pub request_id: Uuid,
    pub candidate: CaptureCandidate,
    pub translation: Option<TranslationResult>,
}

#[derive(Debug)]
struct Session {
    request_id: Uuid,
    phase: Phase,
    candidate: Option<CaptureCandidate>,
    translation: Option<TranslationResult>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CoordinatorError {
    #[error("capture request is stale")]
    StaleRequest,
    #[error("capture request is in an invalid state")]
    InvalidTransition,
    #[error("capture request was already saved")]
    AlreadySaved,
    #[error("translation is required; use the explicit save-without-translation action")]
    TranslationRequired,
    #[error("capture persistence failed: {0}")]
    Persistence(String),
}

#[derive(Default)]
pub struct CaptureCoordinator {
    session: Mutex<Option<Session>>,
}

impl CaptureCoordinator {
    pub fn is_current(&self, request_id: Uuid) -> bool {
        self.session
            .lock()
            .expect("capture coordinator poisoned")
            .as_ref()
            .is_some_and(|session| session.request_id == request_id)
    }

    /// Runs synchronous observable publication only while `request_id` is current.
    ///
    /// The session lock remains held for the closure, so `start` cannot replace
    /// the request between currentness validation and the visible side effects.
    /// Callers must not perform asynchronous work or re-enter the coordinator
    /// inside this closure.
    pub fn publish_if_current<T>(
        &self,
        request_id: Uuid,
        publish: impl FnOnce() -> T,
    ) -> Result<T, CoordinatorError> {
        let guard = self.current(request_id)?;
        let result = publish();
        drop(guard);
        Ok(result)
    }

    pub fn translation(
        &self,
        request_id: Uuid,
    ) -> Result<Option<TranslationResult>, CoordinatorError> {
        let guard = self.current(request_id)?;
        let session = guard.as_ref().ok_or(CoordinatorError::StaleRequest)?;
        Ok(session.translation.clone())
    }

    pub fn start(&self) -> Uuid {
        let request_id = Uuid::now_v7();
        *self.session.lock().expect("capture coordinator poisoned") = Some(Session {
            request_id,
            phase: Phase::Capturing,
            candidate: None,
            translation: None,
        });
        request_id
    }

    pub fn set_candidate(
        &self,
        request_id: Uuid,
        candidate: CaptureCandidate,
    ) -> Result<(), CoordinatorError> {
        let mut guard = self.current(request_id)?;
        let session = guard.as_mut().ok_or(CoordinatorError::StaleRequest)?;
        Self::accept_candidate(session, candidate)
    }

    /// Applies a candidate and synchronously publishes it under one current-request guard.
    pub fn set_candidate_and_publish<T>(
        &self,
        request_id: Uuid,
        candidate: CaptureCandidate,
        publish: impl FnOnce() -> T,
    ) -> Result<T, CoordinatorError> {
        let mut guard = self.current(request_id)?;
        let session = guard.as_mut().ok_or(CoordinatorError::StaleRequest)?;
        Self::accept_candidate(session, candidate)?;
        let result = publish();
        drop(guard);
        Ok(result)
    }

    fn accept_candidate(
        session: &mut Session,
        candidate: CaptureCandidate,
    ) -> Result<(), CoordinatorError> {
        if session.phase != Phase::Capturing {
            return Err(CoordinatorError::InvalidTransition);
        }
        session.phase = if candidate.origin == CaptureOrigin::Ocr {
            Phase::AwaitingOcrConfirmation
        } else {
            Phase::TranslationPending
        };
        session.candidate = Some(candidate);
        Ok(())
    }

    pub fn confirm_ocr(&self, request_id: Uuid) -> Result<(), CoordinatorError> {
        let mut guard = self.current(request_id)?;
        let session = guard.as_mut().ok_or(CoordinatorError::StaleRequest)?;
        if session.phase != Phase::AwaitingOcrConfirmation {
            return Err(CoordinatorError::InvalidTransition);
        }
        session.phase = Phase::TranslationPending;
        Ok(())
    }

    /// Authorizes a translation provider call for the current request.
    ///
    /// This check must happen before invoking an external provider so OCR
    /// candidates cannot leave the confirmation boundary implicitly.
    pub fn begin_translation(&self, request_id: Uuid) -> Result<(), CoordinatorError> {
        let mut guard = self.current(request_id)?;
        let session = guard.as_mut().ok_or(CoordinatorError::StaleRequest)?;
        match session.phase {
            Phase::TranslationPending | Phase::TranslationFailed => {
                session.phase = Phase::Translating;
            }
            _ => return Err(CoordinatorError::InvalidTransition),
        }
        Ok(())
    }

    pub fn set_translation(
        &self,
        request_id: Uuid,
        translation: TranslationResult,
    ) -> Result<(), CoordinatorError> {
        let mut guard = self.current(request_id)?;
        let session = guard.as_mut().ok_or(CoordinatorError::StaleRequest)?;
        if session.phase != Phase::Translating {
            return Err(CoordinatorError::InvalidTransition);
        }
        session.translation = Some(translation);
        session.phase = Phase::ReadyToSave;
        Ok(())
    }

    pub fn translation_failed(&self, request_id: Uuid) -> Result<(), CoordinatorError> {
        let mut guard = self.current(request_id)?;
        let session = guard.as_mut().ok_or(CoordinatorError::StaleRequest)?;
        if session.phase != Phase::Translating {
            return Err(CoordinatorError::InvalidTransition);
        }
        session.phase = Phase::TranslationFailed;
        Ok(())
    }

    pub fn save_with<T, E: Display>(
        &self,
        request_id: Uuid,
        without_translation: bool,
        persist: impl FnOnce(&CaptureSnapshot) -> Result<T, E>,
    ) -> Result<T, CoordinatorError> {
        let mut guard = self.current(request_id)?;
        let session = guard.as_mut().ok_or(CoordinatorError::StaleRequest)?;
        let previous_phase = session.phase;
        match session.phase {
            Phase::Saved | Phase::Saving => return Err(CoordinatorError::AlreadySaved),
            Phase::ReadyToSave if !without_translation => {}
            Phase::TranslationFailed if without_translation => {}
            Phase::TranslationFailed => return Err(CoordinatorError::TranslationRequired),
            _ => return Err(CoordinatorError::InvalidTransition),
        }
        let snapshot = CaptureSnapshot {
            request_id,
            candidate: session
                .candidate
                .clone()
                .ok_or(CoordinatorError::InvalidTransition)?,
            translation: session.translation.clone(),
        };
        session.phase = Phase::Saving;
        match persist(&snapshot) {
            Ok(value) => {
                session.phase = Phase::Saved;
                Ok(value)
            }
            Err(error) => {
                session.phase = previous_phase;
                Err(CoordinatorError::Persistence(error.to_string()))
            }
        }
    }

    fn current(
        &self,
        request_id: Uuid,
    ) -> Result<std::sync::MutexGuard<'_, Option<Session>>, CoordinatorError> {
        let guard = self.session.lock().expect("capture coordinator poisoned");
        if guard
            .as_ref()
            .is_some_and(|session| session.request_id == request_id)
        {
            Ok(guard)
        } else {
            Err(CoordinatorError::StaleRequest)
        }
    }
}
