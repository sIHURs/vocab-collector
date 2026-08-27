use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use vocab_capture::{CaptureCoordinator, CoordinatorError};
use vocab_domain::CaptureCard;
use vocab_platform_api::{
    CaptureCandidate, CaptureOrigin, PlatformError, PlatformServices, SelectionProvider,
    TranslationProvider, TranslationResult,
};

use crate::{AppService, ApplicationError, CaptureRequest};

#[derive(Debug, PartialEq)]
pub struct PreparedCapture {
    pub request_id: Uuid,
    pub candidate: CaptureCandidate,
    pub translation: Option<TranslationResult>,
    pub translation_error: Option<PlatformError>,
}

#[derive(Debug, thiserror::Error)]
pub enum PlatformCaptureError {
    #[error("OCR-origin capture requires explicit confirmation")]
    OcrConfirmationRequired,
    #[error("{0}")]
    Platform(#[from] PlatformError),
    #[error("{0}")]
    Coordinator(#[from] CoordinatorError),
    #[error("{0}")]
    Application(#[from] ApplicationError),
}

pub struct PlatformCaptureWorkflow {
    application: Arc<AppService>,
    coordinator: CaptureCoordinator,
    selection: Arc<dyn SelectionProvider>,
    translation: Arc<dyn TranslationProvider>,
}

impl PlatformCaptureWorkflow {
    pub fn new(application: Arc<AppService>, services: PlatformServices) -> Self {
        Self {
            application,
            coordinator: CaptureCoordinator::default(),
            selection: services.selection,
            translation: services.translation,
        }
    }

    pub fn start_request(&self) -> Uuid {
        self.coordinator.start()
    }

    pub fn is_current(&self, request_id: Uuid) -> bool {
        self.coordinator.is_current(request_id)
    }

    pub fn set_candidate(
        &self,
        request_id: Uuid,
        candidate: CaptureCandidate,
    ) -> Result<(), PlatformCaptureError> {
        self.coordinator.set_candidate(request_id, candidate)?;
        Ok(())
    }

    pub fn confirm_ocr(&self, request_id: Uuid) -> Result<(), PlatformCaptureError> {
        self.coordinator.confirm_ocr(request_id)?;
        Ok(())
    }

    pub async fn translate(
        &self,
        request_id: Uuid,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformCaptureError> {
        if let Some(translation) = self.coordinator.translation(request_id)? {
            return Ok(translation);
        }
        self.coordinator.begin_translation(request_id)?;
        match self.translation.translate(text, source, target).await {
            Ok(translation) => {
                self.coordinator
                    .set_translation(request_id, translation.clone())?;
                Ok(translation)
            }
            Err(error) => {
                self.coordinator.translation_failed(request_id)?;
                Err(error.into())
            }
        }
    }

    pub async fn prepare_selection(&self) -> Result<PreparedCapture, PlatformCaptureError> {
        let request_id = self.start_request();
        self.prepare_selection_for(request_id).await
    }

    pub async fn prepare_selection_for(
        &self,
        request_id: Uuid,
    ) -> Result<PreparedCapture, PlatformCaptureError> {
        let candidate = self.selection.capture_selection().await?;
        if candidate.origin == CaptureOrigin::Ocr {
            return Err(PlatformCaptureError::OcrConfirmationRequired);
        }

        self.coordinator
            .set_candidate(request_id, candidate.clone())?;

        let settings = self.application.get_settings()?;
        let (translation, translation_error) = match self
            .translate(
                request_id,
                &candidate.selected_text,
                &settings.source_language,
                &settings.target_language,
            )
            .await
        {
            Ok(translation) => (Some(translation), None),
            Err(PlatformCaptureError::Platform(error)) => (None, Some(error)),
            Err(error) => return Err(error),
        };

        Ok(PreparedCapture {
            request_id,
            candidate,
            translation,
            translation_error,
        })
    }

    pub fn save(
        &self,
        request_id: Uuid,
        without_translation: bool,
        captured_at: DateTime<Utc>,
    ) -> Result<CaptureCard, PlatformCaptureError> {
        let settings = self.application.get_settings()?;
        Ok(self
            .coordinator
            .save_with(request_id, without_translation, |snapshot| {
                let candidate = &snapshot.candidate;
                let translation = snapshot.translation.as_ref();
                self.application.capture(CaptureRequest {
                    selected_text: candidate.selected_text.clone(),
                    lemma: None,
                    sentence: candidate.sentence.clone(),
                    source_language: translation.map_or_else(
                        || settings.source_language.clone(),
                        |value| value.source_language.clone(),
                    ),
                    target_language: translation.map_or_else(
                        || settings.target_language.clone(),
                        |value| value.target_language.clone(),
                    ),
                    translation: translation.map(|value| value.translated_text.clone()),
                    part_of_speech: None,
                    source_app: candidate.source_app.clone(),
                    source_title: candidate.source_title.clone(),
                    source_url: candidate.source_url.clone(),
                    capture_origin: candidate.origin,
                    captured_at,
                })
            })?)
    }
}
