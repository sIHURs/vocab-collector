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
    events::{CaptureFailure, NativeCaptureError, NativeCaptureEvent},
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
) -> Result<PermissionStatus, CaptureFailure> {
    state
        .permission_status(kind)
        .await
        .map_err(CaptureFailure::from)
}

#[tauri::command]
pub async fn request_accessibility_permission(
    state: State<'_, AppState>,
) -> Result<PermissionStatus, CaptureFailure> {
    state
        .request_permission(PermissionKind::Accessibility)
        .await
        .map_err(CaptureFailure::from)
}

#[tauri::command]
pub async fn capture_selected_text(
    state: State<'_, AppState>,
) -> Result<CaptureCandidate, CaptureFailure> {
    state
        .capture_selection()
        .await
        .map_err(CaptureFailure::from)
}

#[tauri::command]
pub async fn request_screen_recording_permission(
    state: State<'_, AppState>,
) -> Result<PermissionStatus, CaptureFailure> {
    state
        .request_permission(PermissionKind::ScreenRecording)
        .await
        .map_err(CaptureFailure::from)
}

#[tauri::command]
pub async fn capture_with_ocr(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<(), CaptureFailure> {
    if !state.is_current_capture_request(request_id) {
        return Err(vocab_capture::CoordinatorError::StaleRequest.into());
    }
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| CaptureFailure::operation("capture window is unavailable"))?;
    let (portable_pointer, primary_bounds) =
        pointer_and_primary_bounds(&app).map_err(CaptureFailure::operation)?;
    // Tauri reports a logical top-left desktop point. AppKit's OCR entry point consumes
    // Cocoa global coordinates, whose Y axis starts at the primary display's bottom edge.
    let cocoa_pointer = portable_top_left_to_cocoa(portable_pointer, primary_bounds);
    state
        .publish_if_current(request_id, || {
            window.hide().map_err(|error| error.to_string())
        })
        .map_err(CaptureFailure::from)?
        .map_err(CaptureFailure::operation)?;
    let candidates = state.recognize_near(cocoa_pointer).await;
    let candidates = match candidates {
        Ok(candidates) => candidates,
        Err(error) => {
            state
                .publish_if_current(request_id, || {
                    window.show().map_err(|error| error.to_string())
                })
                .map_err(CaptureFailure::from)?
                .map_err(CaptureFailure::operation)?;
            return Err(error.into());
        }
    };
    let Some(best) = candidates
        .into_iter()
        .max_by(|left, right| left.confidence.total_cmp(&right.confidence))
    else {
        state
            .publish_if_current(request_id, || {
                window.show().map_err(|error| error.to_string())
            })
            .map_err(CaptureFailure::from)?
            .map_err(CaptureFailure::operation)?;
        return Err(CaptureFailure::operation("OCR did not find readable text"));
    };
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
        .set_capture_candidate_and_publish(request_id, candidate.clone(), || {
            window.show().map_err(|error| error.to_string())?;
            window
                .emit(
                    "ocr-candidate",
                    NativeCaptureEvent {
                        request_id,
                        candidate,
                    },
                )
                .map_err(|error| error.to_string())
        })
        .map_err(CaptureFailure::from)?
        .map_err(CaptureFailure::operation)
}

#[tauri::command]
pub async fn translate_text(
    state: State<'_, AppState>,
    request_id: Uuid,
    text: String,
    source_language: String,
    target_language: String,
) -> Result<TranslationResult, CaptureFailure> {
    state
        .translate_capture(request_id, &text, &source_language, &target_language)
        .await
        .map_err(CaptureFailure::from_translation)
}

#[tauri::command]
pub fn confirm_ocr(state: State<'_, AppState>, request_id: Uuid) -> Result<(), CaptureFailure> {
    state.confirm_ocr(request_id).map_err(CaptureFailure::from)
}

#[tauri::command]
pub fn save_native_capture(
    state: State<'_, AppState>,
    request_id: Uuid,
    without_translation: bool,
) -> Result<CaptureCard, CaptureFailure> {
    state
        .save_capture(request_id, without_translation)
        .map_err(CaptureFailure::from)
}

pub fn hide_capture_window_for(
    state: &AppState,
    request_id: Uuid,
    hide: impl FnOnce() -> Result<(), String>,
) -> Result<(), CaptureFailure> {
    state
        .publish_if_current(request_id, hide)
        .map_err(CaptureFailure::from)?
        .map_err(CaptureFailure::operation)
}

#[tauri::command]
pub fn hide_capture_window(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<(), CaptureFailure> {
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| CaptureFailure::operation("capture window is unavailable"))?;
    hide_capture_window_for(&state, request_id, || {
        window.hide().map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn get_platform_capabilities(state: State<'_, AppState>) -> PlatformCapabilities {
    state.platform_capabilities()
}

pub(crate) async fn present_native_capture(
    app: &tauri::AppHandle,
) -> Result<(), NativeCaptureError> {
    let state = app.state::<AppState>();
    let request_id = state.start_capture_request();
    let prepared = state
        .prepare_selection_for(request_id)
        .await
        .map_err(|error| NativeCaptureError {
            request_id,
            failure: error.into(),
        })?;
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| NativeCaptureError {
            request_id: prepared.request_id,
            failure: CaptureFailure::operation("capture window is unavailable"),
        })?;
    let (pointer, work_areas) =
        pointer_and_work_areas(app).map_err(|message| NativeCaptureError {
            request_id: prepared.request_id,
            failure: CaptureFailure::operation(message),
        })?;
    let anchor = prepared.candidate.selection_bounds.unwrap_or_default();
    let position = vocab_capture::place_floating_window(
        anchor,
        pointer,
        &work_areas,
        ScreenSize::new(380.0, 280.0),
    );
    state
        .publish_if_current(prepared.request_id, || {
            window
                .set_position(LogicalPosition::new(position.x, position.y))
                .map_err(|error| error.to_string())?;
            window.show().map_err(|error| error.to_string())?;
            window
                .emit(
                    "capture-ready",
                    NativeCaptureEvent {
                        request_id: prepared.request_id,
                        candidate: prepared.candidate,
                    },
                )
                .map_err(|error| error.to_string())
        })
        .map_err(|error| NativeCaptureError {
            request_id: prepared.request_id,
            failure: error.into(),
        })?
        .map_err(|message| NativeCaptureError {
            request_id: prepared.request_id,
            failure: CaptureFailure::operation(message),
        })
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PhysicalMonitorWorkArea {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    scale_factor: f64,
}

impl PhysicalMonitorWorkArea {
    const fn new(x: f64, y: f64, width: f64, height: f64, scale_factor: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            scale_factor,
        }
    }
}

fn normalize_virtual_desktop(
    pointer: ScreenPoint,
    primary_scale: f64,
    monitors: &[PhysicalMonitorWorkArea],
) -> (ScreenPoint, Vec<MonitorWorkArea>) {
    let pointer = ScreenPoint::new(pointer.x / primary_scale, pointer.y / primary_scale);
    let monitors = monitors
        .iter()
        .map(|monitor| {
            MonitorWorkArea::new(
                monitor.x / monitor.scale_factor,
                monitor.y / monitor.scale_factor,
                monitor.width / monitor.scale_factor,
                monitor.height / monitor.scale_factor,
                monitor.scale_factor,
            )
        })
        .collect();
    (pointer, monitors)
}

fn pointer_and_work_areas(
    app: &tauri::AppHandle,
) -> Result<(ScreenPoint, Vec<MonitorWorkArea>), String> {
    let pointer = app.cursor_position().map_err(|error| error.to_string())?;
    let primary_scale = app
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "primary monitor is unavailable".to_string())?
        .scale_factor();
    let monitors = app
        .available_monitors()
        .map_err(|error| error.to_string())?;
    let physical_work_areas = monitors
        .iter()
        .map(|monitor| {
            let area = monitor.work_area();
            PhysicalMonitorWorkArea::new(
                f64::from(area.position.x),
                f64::from(area.position.y),
                f64::from(area.size.width),
                f64::from(area.size.height),
                monitor.scale_factor(),
            )
        })
        .collect::<Vec<_>>();
    Ok(normalize_virtual_desktop(
        ScreenPoint::new(pointer.x, pointer.y),
        primary_scale,
        &physical_work_areas,
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

    use super::{PhysicalMonitorWorkArea, normalize_virtual_desktop, portable_top_left_to_cocoa};

    #[test]
    fn normalizes_mixed_scale_positive_and_negative_monitor_origins_into_one_logical_space() {
        let positive = normalize_virtual_desktop(
            ScreenPoint::new(3_600.0, 500.0),
            2.0,
            &[
                PhysicalMonitorWorkArea::new(0.0, 0.0, 2_880.0, 1_800.0, 2.0),
                PhysicalMonitorWorkArea::new(1_440.0, 100.0, 1_920.0, 1_080.0, 1.0),
            ],
        );
        assert_eq!(positive.0, ScreenPoint::new(1_800.0, 250.0));
        assert_eq!(
            positive.1,
            vec![
                MonitorWorkArea::new(0.0, 0.0, 1_440.0, 900.0, 2.0),
                MonitorWorkArea::new(1_440.0, 100.0, 1_920.0, 1_080.0, 1.0),
            ]
        );
        assert!(positive.1[1].contains(positive.0));

        let negative = normalize_virtual_desktop(
            ScreenPoint::new(-1_280.0, 200.0),
            2.0,
            &[
                PhysicalMonitorWorkArea::new(0.0, 0.0, 2_880.0, 1_800.0, 2.0),
                PhysicalMonitorWorkArea::new(-1_920.0, -300.0, 1_920.0, 1_200.0, 1.5),
            ],
        );
        assert_eq!(negative.0, ScreenPoint::new(-640.0, 100.0));
        assert_eq!(
            negative.1[1],
            MonitorWorkArea::new(-1_280.0, -200.0, 1_280.0, 800.0, 1.5)
        );
        assert!(negative.1[1].contains(negative.0));
    }

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
