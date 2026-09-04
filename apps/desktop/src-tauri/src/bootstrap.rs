use std::sync::{Arc, Mutex};

use uuid::Uuid;
use vocab_application::{AppService, PlatformCaptureWorkflow, PreparedCapture};
use vocab_platform_api::{
    CaptureCandidate, OcrCandidate, OcrProvider, PermissionKind, PermissionProvider,
    PermissionStatus, PlatformCapabilities, PlatformError, PlatformServices, SelectionProvider,
    TranslationProvider, TranslationResult, WindowProvider,
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
    pending_ocr: Mutex<Option<(Uuid, Vec<OcrCandidate>)>>,
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
        pending_ocr: Mutex::new(None),
    }
}

/// Selects the build target's adapter. Target conditionals are confined to bootstrap.
#[cfg(target_os = "macos")]
pub fn selected_platform() -> Result<PlatformServices, PlatformError> {
    vocab_platform_macos::MacPlatform::new()
}

#[cfg(target_os = "linux")]
pub fn selected_platform() -> Result<PlatformServices, PlatformError> {
    Ok(vocab_platform_linux::LinuxPlatform::new())
}

#[cfg(target_os = "windows")]
pub fn selected_platform() -> Result<PlatformServices, PlatformError> {
    let Some(config) = crate::config::translation_provider_config()? else {
        return Ok(vocab_platform_windows::WindowsPlatform::new());
    };
    let provider: Arc<dyn TranslationProvider> = match config {
        crate::config::TranslationProviderConfig::Azure(config) => Arc::new(
            vocab_translation_azure::AzureTranslationProvider::new(config)?,
        ),
        crate::config::TranslationProviderConfig::DeepL(config) => Arc::new(
            vocab_translation_deepl::DeepLTranslationProvider::new(config)?,
        ),
    };
    Ok(vocab_platform_windows::WindowsPlatform::with_translation(
        provider,
    ))
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
compile_error!(
    "Vocab Collector desktop supports only macOS, Linux, and Windows composition targets"
);

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

    pub async fn recognize_region(
        &self,
        region: vocab_platform_api::ScreenRect,
    ) -> Result<Vec<OcrCandidate>, PlatformError> {
        self.ocr.recognize_region(region).await
    }

    pub async fn translate(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        self.translation.translate(text, source, target).await
    }

    pub fn start_capture_request(&self) -> Uuid {
        let request_id = self.workflow.start_request();
        *self.pending_ocr.lock().unwrap() = None;
        request_id
    }

    pub fn is_current_capture_request(&self, request_id: Uuid) -> bool {
        self.workflow.is_current(request_id)
    }

    pub fn publish_if_current<T>(
        &self,
        request_id: Uuid,
        publish: impl FnOnce() -> T,
    ) -> Result<T, vocab_application::PlatformCaptureError> {
        self.workflow.publish_if_current(request_id, publish)
    }

    pub fn dismiss_and_publish<T>(
        &self,
        request_id: Uuid,
        publish: impl FnOnce() -> T,
    ) -> Result<T, vocab_application::PlatformCaptureError> {
        let result = self.workflow.dismiss_and_publish(request_id, publish)?;
        *self.pending_ocr.lock().unwrap() = None;
        Ok(result)
    }

    pub async fn prepare_selection_for(
        &self,
        request_id: Uuid,
    ) -> Result<PreparedCapture, vocab_application::PlatformCaptureError> {
        self.workflow.prepare_selection_for(request_id).await
    }

    pub fn configure_capture_window(&self) -> Result<(), PlatformError> {
        if !self.capabilities.non_activating_window {
            return Ok(());
        }
        self.window.configure_capture_window()
    }

    pub(crate) fn publish_ocr_candidates<T>(
        &self,
        request_id: Uuid,
        candidates: Vec<OcrCandidate>,
        publish: impl FnOnce() -> T,
    ) -> Result<T, vocab_application::PlatformCaptureError> {
        self.workflow.publish_if_current(request_id, || {
            *self.pending_ocr.lock().unwrap() = Some((request_id, candidates));
            publish()
        })
    }

    pub(crate) fn confirm_ocr_draft(
        &self,
        request_id: Uuid,
        selected_text: String,
        sentence: String,
    ) -> Result<(), vocab_application::PlatformCaptureError> {
        if selected_text.trim().is_empty() {
            return Err(vocab_capture::CoordinatorError::InvalidTransition.into());
        }
        let bounds = self
            .pending_ocr
            .lock()
            .unwrap()
            .as_ref()
            .filter(|(pending_request, _)| *pending_request == request_id)
            .and_then(|(_, candidates)| candidates.first())
            .map(|candidate| candidate.bounds)
            .ok_or(vocab_capture::CoordinatorError::StaleRequest)?;
        let candidate = CaptureCandidate {
            selected_text: selected_text.trim().into(),
            sentence: sentence.trim().into(),
            source_app: None,
            source_title: None,
            source_url: None,
            selection_bounds: Some(bounds),
            origin: vocab_domain::CaptureOrigin::Ocr,
        };
        self.workflow.set_candidate(request_id, candidate)?;
        self.workflow.confirm_ocr(request_id)
    }

    pub async fn translate_capture(
        &self,
        request_id: Uuid,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, vocab_application::PlatformCaptureError> {
        self.workflow
            .translate(request_id, text, source, target)
            .await
    }

    pub fn save_capture(
        &self,
        request_id: Uuid,
        without_translation: bool,
    ) -> Result<vocab_domain::CaptureCard, vocab_application::PlatformCaptureError> {
        self.workflow
            .save(request_id, without_translation, chrono::Utc::now())
    }

    pub fn correct_capture(
        &self,
        request_id: Uuid,
        selected_text: String,
        sentence: String,
        manual_translation: Option<String>,
    ) -> Result<(), vocab_application::PlatformCaptureError> {
        self.workflow
            .correct(request_id, selected_text, sentence, manual_translation)
    }

    pub fn undo_native_capture(
        &self,
        request_id: Uuid,
        encounter_id: Uuid,
    ) -> Result<(), vocab_application::PlatformCaptureError> {
        self.workflow.undo(request_id, encounter_id)
    }
}
