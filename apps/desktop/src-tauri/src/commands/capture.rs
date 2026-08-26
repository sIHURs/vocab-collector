use tauri::{Emitter, LogicalPosition, Manager, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use uuid::Uuid;
use vocab_domain::{CaptureCard, CaptureOrigin, UserSettings};
use vocab_platform_api::{
    CaptureCandidate, MonitorWorkArea, PermissionKind, PermissionStatus, PlatformCapabilities,
    ScreenPoint, ScreenSize, TranslationResult,
};

use crate::{
    bootstrap::AppState,
    events::{NativeCaptureError, NativeCaptureEvent},
};

#[tauri::command]
pub fn replace_shortcut(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    candidate: String,
) -> Result<UserSettings, String> {
    let parsed = vocab_capture::parse_shortcut(&candidate).map_err(|error| error.to_string())?;
    let mut settings = state
        .application()
        .get_settings()
        .map_err(|error| error.to_string())?;
    let previous = settings.capture_shortcut.clone();
    if previous == parsed.canonical() {
        return Ok(settings);
    }
    app.global_shortcut()
        .register(parsed.canonical())
        .map_err(|_| "Shortcut unavailable. Try another combination.".to_string())?;
    settings.capture_shortcut = parsed.canonical().to_string();
    if let Err(error) = state.application().update_settings(settings.clone()) {
        let _ = app.global_shortcut().unregister(parsed.canonical());
        return Err(error.to_string());
    }
    if let Err(error) = app.global_shortcut().unregister(previous.as_str()) {
        let _ = app.global_shortcut().unregister(parsed.canonical());
        settings.capture_shortcut = previous;
        let _ = state.application().update_settings(settings.clone());
        return Err(error.to_string());
    }
    Ok(settings)
}

#[tauri::command]
pub async fn get_permission_status(
    state: State<'_, AppState>,
    kind: PermissionKind,
) -> Result<PermissionStatus, String> {
    state
        .permission_status(kind)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn request_accessibility_permission(
    state: State<'_, AppState>,
) -> Result<PermissionStatus, String> {
    state
        .request_permission(PermissionKind::Accessibility)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn capture_selected_text(state: State<'_, AppState>) -> Result<CaptureCandidate, String> {
    state
        .capture_selection()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn request_screen_recording_permission(
    state: State<'_, AppState>,
) -> Result<PermissionStatus, String> {
    state
        .request_permission(PermissionKind::ScreenRecording)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn capture_with_ocr(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<(), String> {
    if !state.is_ocr_request(request_id) {
        return Err(vocab_capture::CoordinatorError::StaleRequest.to_string());
    }
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| "capture window is unavailable".to_string())?;
    let (portable_pointer, primary_bounds) = pointer_and_primary_bounds(&app)?;
    // Tauri reports a logical top-left desktop point. AppKit's OCR entry point consumes
    // Cocoa global coordinates, whose Y axis starts at the primary display's bottom edge.
    let cocoa_pointer = portable_top_left_to_cocoa(portable_pointer, primary_bounds);
    window.hide().map_err(|error| error.to_string())?;
    let candidates = state.recognize_near(cocoa_pointer).await;
    window.show().map_err(|error| error.to_string())?;
    let best = candidates
        .map_err(|error| error.to_string())?
        .into_iter()
        .max_by(|left, right| left.confidence.total_cmp(&right.confidence))
        .ok_or_else(|| "OCR did not find readable text".to_string())?;
    let candidate = CaptureCandidate {
        selected_text: best.text.clone(),
        sentence: best.text,
        source_app: None,
        source_title: None,
        source_url: None,
        selection_bounds: Some(best.bounds),
        origin: CaptureOrigin::Ocr,
    };
    state
        .set_ocr_candidate(request_id, candidate.clone())
        .map_err(|error| error.to_string())?;
    window
        .emit(
            "ocr-candidate",
            NativeCaptureEvent {
                request_id,
                candidate,
            },
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn translate_text(
    state: State<'_, AppState>,
    request_id: Uuid,
    text: String,
    source_language: String,
    target_language: String,
) -> Result<TranslationResult, String> {
    if let Some(prepared) = state.prepared_translation(request_id) {
        return Ok(prepared);
    }
    let result = state
        .translate(&text, &source_language, &target_language)
        .await;
    match result {
        Ok(translation) => {
            if state.is_ocr_request(request_id) {
                state
                    .set_ocr_translation(request_id, translation.clone())
                    .map_err(|error| error.to_string())?;
            }
            state.remember_translation(request_id, translation.clone());
            Ok(translation)
        }
        Err(error) => {
            if state.is_ocr_request(request_id) {
                state
                    .fail_ocr_translation(request_id)
                    .map_err(|coordinator_error| coordinator_error.to_string())?;
            }
            Err(error.to_string())
        }
    }
}

#[tauri::command]
pub fn confirm_ocr(state: State<'_, AppState>, request_id: Uuid) -> Result<(), String> {
    state
        .confirm_ocr(request_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_native_capture(
    state: State<'_, AppState>,
    request_id: Uuid,
    without_translation: bool,
) -> Result<CaptureCard, String> {
    state.save_capture(request_id, without_translation)
}

#[tauri::command]
pub fn hide_capture_window(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("capture")
        .ok_or_else(|| "capture window is unavailable".to_string())?
        .hide()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_platform_capabilities(state: State<'_, AppState>) -> PlatformCapabilities {
    state.platform_capabilities()
}

pub(crate) async fn present_native_capture(
    app: &tauri::AppHandle,
) -> Result<(), NativeCaptureError> {
    let state = app.state::<AppState>();
    let error_request_id = state.start_ocr_request();
    let prepared = state
        .prepare_selection()
        .await
        .map_err(|error| NativeCaptureError {
            request_id: error_request_id,
            message: error.to_string(),
        })?;
    if let Some(translation) = prepared.translation.clone() {
        state.remember_translation(prepared.request_id, translation);
    }
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| NativeCaptureError {
            request_id: prepared.request_id,
            message: "capture window is unavailable".into(),
        })?;
    let (pointer, work_areas) =
        pointer_and_work_areas(app).map_err(|message| NativeCaptureError {
            request_id: prepared.request_id,
            message,
        })?;
    let anchor = prepared.candidate.selection_bounds.unwrap_or_default();
    let position = vocab_capture::place_floating_window(
        anchor,
        pointer,
        &work_areas,
        ScreenSize::new(380.0, 280.0),
    );
    window
        .set_position(LogicalPosition::new(position.x, position.y))
        .map_err(|error| NativeCaptureError {
            request_id: prepared.request_id,
            message: error.to_string(),
        })?;
    window.show().map_err(|error| NativeCaptureError {
        request_id: prepared.request_id,
        message: error.to_string(),
    })?;
    window
        .emit(
            "capture-ready",
            NativeCaptureEvent {
                request_id: prepared.request_id,
                candidate: prepared.candidate,
            },
        )
        .map_err(|error| NativeCaptureError {
            request_id: prepared.request_id,
            message: error.to_string(),
        })
}

fn pointer_and_work_areas(
    app: &tauri::AppHandle,
) -> Result<(ScreenPoint, Vec<MonitorWorkArea>), String> {
    let pointer = app.cursor_position().map_err(|error| error.to_string())?;
    let monitors = app
        .available_monitors()
        .map_err(|error| error.to_string())?;
    let monitor_scale = monitors
        .iter()
        .find(|monitor| {
            let area = monitor.work_area();
            pointer.x >= f64::from(area.position.x)
                && pointer.x < f64::from(area.position.x) + f64::from(area.size.width)
                && pointer.y >= f64::from(area.position.y)
                && pointer.y < f64::from(area.position.y) + f64::from(area.size.height)
        })
        .map_or(1.0, |monitor| monitor.scale_factor());
    let work_areas = monitors
        .iter()
        .map(|monitor| {
            let area = monitor.work_area();
            let scale = monitor.scale_factor();
            MonitorWorkArea::new(
                f64::from(area.position.x) / scale,
                f64::from(area.position.y) / scale,
                f64::from(area.size.width) / scale,
                f64::from(area.size.height) / scale,
                scale,
            )
        })
        .collect();
    Ok((
        ScreenPoint::new(pointer.x / monitor_scale, pointer.y / monitor_scale),
        work_areas,
    ))
}

fn pointer_and_primary_bounds(
    app: &tauri::AppHandle,
) -> Result<(ScreenPoint, MonitorWorkArea), String> {
    let (pointer, _) = pointer_and_work_areas(app)?;
    let primary = app
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "primary monitor is unavailable".to_string())?;
    let scale = primary.scale_factor();
    let position = primary.position();
    let size = primary.size();
    Ok((
        pointer,
        MonitorWorkArea::new(
            f64::from(position.x) / scale,
            f64::from(position.y) / scale,
            f64::from(size.width) / scale,
            f64::from(size.height) / scale,
            scale,
        ),
    ))
}

/// Converts the portable logical top-left convention into AppKit's Cocoa global convention.
pub(crate) fn portable_top_left_to_cocoa(
    point: ScreenPoint,
    primary: MonitorWorkArea,
) -> ScreenPoint {
    ScreenPoint::new(point.x, primary.y + primary.height - point.y)
}

#[cfg(test)]
mod tests {
    use vocab_platform_api::{MonitorWorkArea, ScreenPoint};

    use super::portable_top_left_to_cocoa;

    #[test]
    fn converts_portable_top_left_points_to_cocoa_bottom_left_points() {
        let primary = MonitorWorkArea::new(0.0, 0.0, 1440.0, 900.0, 2.0);

        assert_eq!(
            portable_top_left_to_cocoa(ScreenPoint::new(120.0, 75.0), primary),
            ScreenPoint::new(120.0, 825.0)
        );
        assert_eq!(
            portable_top_left_to_cocoa(ScreenPoint::new(-400.0, -200.0), primary),
            ScreenPoint::new(-400.0, 1100.0)
        );
        assert_eq!(
            portable_top_left_to_cocoa(ScreenPoint::new(500.0, 1100.0), primary),
            ScreenPoint::new(500.0, -200.0)
        );
    }
}
