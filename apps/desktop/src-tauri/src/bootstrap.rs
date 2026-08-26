use std::sync::{Arc, Mutex};

use uuid::Uuid;
use vocab_application::{AppService, PlatformCaptureWorkflow, PreparedCapture};
use vocab_capture::CaptureCoordinator;
use vocab_platform_api::{
    CaptureCandidate, OcrCandidate, OcrProvider, PermissionKind, PermissionProvider,
    PermissionStatus, PlatformCapabilities, PlatformError, PlatformServices, ScreenPoint,
    SelectionProvider, TranslationProvider, TranslationResult, WindowProvider,
};
use vocab_storage::SqliteStore;

/// Shared application and portable platform providers managed by Tauri.
pub struct AppState {
    application: Arc<AppService>,
    workflow: PlatformCaptureWorkflow,
    capabilities: PlatformCapabilities,
    selection: Arc<dyn SelectionProvider>,
    ocr: Arc<dyn OcrProvider>,
    translation: Arc<dyn TranslationProvider>,
    permissions: Arc<dyn PermissionProvider>,
    window: Arc<dyn WindowProvider>,
    ocr_coordinator: CaptureCoordinator,
    prepared_translation: Mutex<Option<(Uuid, TranslationResult)>>,
}

/// Composes storage, shared application behavior, and the selected platform adapter.
pub fn build_app_state(repository: Arc<SqliteStore>, platform: PlatformServices) -> AppState {
    let application = Arc::new(AppService::new(repository, Uuid::now_v7()));
    let capabilities = platform.capabilities;
    let selection = Arc::clone(&platform.selection);
    let ocr = Arc::clone(&platform.ocr);
    let translation = Arc::clone(&platform.translation);
    let permissions = Arc::clone(&platform.permissions);
    let window = Arc::clone(&platform.window);
    let workflow = PlatformCaptureWorkflow::new(Arc::clone(&application), platform);

    AppState {
        application,
        workflow,
        capabilities,
        selection,
        ocr,
        translation,
        permissions,
        window,
        ocr_coordinator: CaptureCoordinator::default(),
        prepared_translation: Mutex::new(None),
    }
}

/// Selects the build target's adapter. Target conditionals are confined to bootstrap.
#[cfg(target_os = "macos")]
pub fn selected_platform() -> Result<PlatformServices, PlatformError> {
    vocab_platform_macos::MacPlatform::new()
}

impl AppState {
    pub fn application(&self) -> &AppService {
        &self.application
    }

    pub fn platform_capabilities(&self) -> PlatformCapabilities {
        self.capabilities
    }

    pub async fn permission_status(
        &self,
        kind: PermissionKind,
    ) -> Result<PermissionStatus, PlatformError> {
        self.permissions.status(kind).await
    }

    pub async fn request_permission(
        &self,
        kind: PermissionKind,
    ) -> Result<PermissionStatus, PlatformError> {
        self.permissions.request(kind).await
    }

    pub async fn capture_selection(&self) -> Result<CaptureCandidate, PlatformError> {
        self.selection.capture_selection().await
    }

    pub async fn recognize_near(
        &self,
        pointer: ScreenPoint,
    ) -> Result<Vec<OcrCandidate>, PlatformError> {
        self.ocr.recognize_near(pointer).await
    }

    pub async fn translate(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        self.translation.translate(text, source, target).await
    }

    pub(crate) async fn prepare_selection(
        &self,
    ) -> Result<PreparedCapture, vocab_application::PlatformCaptureError> {
        self.workflow.prepare_selection().await
    }

    pub(crate) fn configure_capture_window(&self) -> Result<(), PlatformError> {
        self.window.configure_capture_window()
    }

    pub(crate) fn remember_translation(&self, request_id: Uuid, value: TranslationResult) {
        *self
            .prepared_translation
            .lock()
            .expect("prepared translation cache poisoned") = Some((request_id, value));
    }

    pub(crate) fn prepared_translation(&self, request_id: Uuid) -> Option<TranslationResult> {
        self.prepared_translation
            .lock()
            .expect("prepared translation cache poisoned")
            .as_ref()
            .filter(|(prepared_id, _)| *prepared_id == request_id)
            .map(|(_, value)| value.clone())
    }

    pub(crate) fn start_ocr_request(&self) -> Uuid {
        self.ocr_coordinator.start()
    }

    pub(crate) fn is_ocr_request(&self, request_id: Uuid) -> bool {
        self.ocr_coordinator.is_current(request_id)
    }

    pub(crate) fn set_ocr_candidate(
        &self,
        request_id: Uuid,
        candidate: CaptureCandidate,
    ) -> Result<(), vocab_capture::CoordinatorError> {
        self.ocr_coordinator.set_candidate(request_id, candidate)
    }

    pub(crate) fn confirm_ocr(
        &self,
        request_id: Uuid,
    ) -> Result<(), vocab_capture::CoordinatorError> {
        self.ocr_coordinator.confirm_ocr(request_id)
    }

    pub(crate) fn set_ocr_translation(
        &self,
        request_id: Uuid,
        translation: TranslationResult,
    ) -> Result<(), vocab_capture::CoordinatorError> {
        self.ocr_coordinator
            .set_translation(request_id, translation)
    }

    pub(crate) fn fail_ocr_translation(
        &self,
        request_id: Uuid,
    ) -> Result<(), vocab_capture::CoordinatorError> {
        self.ocr_coordinator.translation_failed(request_id)
    }

    pub(crate) fn save_capture(
        &self,
        request_id: Uuid,
        without_translation: bool,
    ) -> Result<vocab_domain::CaptureCard, String> {
        if self.is_ocr_request(request_id) {
            let settings = self
                .application
                .get_settings()
                .map_err(|error| error.to_string())?;
            return self
                .ocr_coordinator
                .save_with(request_id, without_translation, |snapshot| {
                    let candidate = &snapshot.candidate;
                    let translation = snapshot.translation.as_ref();
                    self.application.capture(vocab_application::CaptureRequest {
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
                        captured_at: chrono::Utc::now(),
                    })
                })
                .map_err(|error| error.to_string());
        }

        self.workflow
            .save(request_id, without_translation, chrono::Utc::now())
            .map_err(|error| error.to_string())
    }
}
