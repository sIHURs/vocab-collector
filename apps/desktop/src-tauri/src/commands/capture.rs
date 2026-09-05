#[cfg(not(target_os = "windows"))]
use tauri::LogicalPosition;
#[cfg(target_os = "windows")]
use tauri::PhysicalPosition;
use tauri::{Emitter, Manager, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use uuid::Uuid;
use vocab_domain::{CaptureCard, UserSettings};
use vocab_platform_api::{
    CaptureCandidate, MonitorWorkArea, PermissionKind, PermissionStatus, PlatformCapabilities,
    ScreenPoint, ScreenSize, TranslationResult,
};

use crate::{
    bootstrap::AppState,
    events::{
        CaptureFailure, CaptureFailureCode, LIBRARY_CHANGED_EVENT, NativeCaptureError,
        NativeCaptureErrorEvent, NativeCaptureEvent, OcrCandidatesEvent, RegionOcrStartEvent,
    },
};

pub fn present_capture_window(
    position: impl FnOnce() -> Result<(), String>,
    show_without_activation: impl FnOnce() -> Result<(), String>,
    emit: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    position()?;
    show_without_activation()?;
    emit()
}

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
    let previous = settings.selection_capture_shortcut.clone();
    if previous == parsed.canonical() {
        return Ok(settings);
    }
    app.global_shortcut()
        .register(parsed.canonical())
        .map_err(|_| "Shortcut unavailable. Try another combination.".to_string())?;
    settings.selection_capture_shortcut = parsed.canonical().to_string();
    if let Err(error) = state.application().update_settings(settings.clone()) {
        let _ = app.global_shortcut().unregister(parsed.canonical());
        return Err(error.to_string());
    }
    if let Err(error) = app.global_shortcut().unregister(previous.as_str()) {
        let _ = app.global_shortcut().unregister(parsed.canonical());
        settings.selection_capture_shortcut = previous;
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
pub async fn capture_ocr_region(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
    region: vocab_platform_api::ScreenRect,
) -> Result<(), CaptureFailure> {
    if !state.is_current_capture_request(request_id) {
        return Err(vocab_capture::CoordinatorError::StaleRequest.into());
    }
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| CaptureFailure::operation("capture window is unavailable"))?;
    if let Some(overlay) = app.get_webview_window("ocr-overlay") {
        overlay
            .hide()
            .map_err(|error| CaptureFailure::operation(error.to_string()))?;
    }
    if region.width < 4.0 || region.height < 4.0 {
        return Err(CaptureFailure::operation("OCR region is too small"));
    }
    state
        .publish_if_current(request_id, || {
            window.hide().map_err(|error| error.to_string())
        })
        .map_err(CaptureFailure::from)?
        .map_err(CaptureFailure::operation)?;
    let (_, primary_bounds) =
        pointer_and_primary_bounds(&app).map_err(CaptureFailure::operation)?;
    let region = ocr_region_for_target(region, primary_bounds);
    let candidates = state.recognize_region(region).await;
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
    if candidates.is_empty() {
        state
            .publish_if_current(request_id, || {
                window.show().map_err(|error| error.to_string())
            })
            .map_err(CaptureFailure::from)?
            .map_err(CaptureFailure::operation)?;
        return Err(CaptureFailure::operation("OCR did not find readable text"));
    }
    let ambiguous = candidates.len() > 1;
    state
        .publish_ocr_candidates(request_id, candidates.clone(), || {
            window.show().map_err(|error| error.to_string())?;
            window
                .emit(
                    "ocr-candidate",
                    OcrCandidatesEvent {
                        request_id,
                        candidates,
                        ambiguous,
                    },
                )
                .map_err(|error| error.to_string())
        })
        .map_err(CaptureFailure::from)?
        .map_err(CaptureFailure::operation)
}

#[tauri::command]
pub fn start_region_ocr_capture(app: tauri::AppHandle) -> Result<Uuid, CaptureFailure> {
    present_region_ocr_capture(&app)
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
pub fn confirm_ocr(
    state: State<'_, AppState>,
    request_id: Uuid,
    selected_text: String,
    sentence: String,
) -> Result<(), CaptureFailure> {
    state
        .confirm_ocr_draft(request_id, selected_text, sentence)
        .map_err(CaptureFailure::from)
}

#[tauri::command]
pub fn save_native_capture(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
    without_translation: bool,
) -> Result<CaptureCard, CaptureFailure> {
    let saved = state
        .save_capture(request_id, without_translation)
        .map_err(CaptureFailure::from)?;
    let _ = app.emit_to("main", LIBRARY_CHANGED_EVENT, ());
    Ok(saved)
}

#[tauri::command]
pub fn find_achieved_native_capture(
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<Option<vocab_domain::AchievedCaptureConflict>, CaptureFailure> {
    state
        .find_achieved_capture(request_id)
        .map_err(CaptureFailure::from)
}

#[tauri::command]
pub fn restore_achieved_and_save_native_capture(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
    word_id: Uuid,
    without_translation: bool,
) -> Result<CaptureCard, CaptureFailure> {
    let saved = state
        .restore_achieved_and_save_capture(request_id, word_id, without_translation)
        .map_err(CaptureFailure::from)?;
    let _ = app.emit_to("main", LIBRARY_CHANGED_EVENT, ());
    Ok(saved)
}

#[tauri::command]
pub fn correct_native_capture(
    state: State<'_, AppState>,
    request_id: Uuid,
    selected_text: String,
    sentence: String,
    manual_translation: Option<String>,
) -> Result<(), CaptureFailure> {
    state
        .correct_capture(request_id, selected_text, sentence, manual_translation)
        .map_err(CaptureFailure::from)
}

#[tauri::command]
pub fn undo_native_capture(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
    encounter_id: Uuid,
) -> Result<(), CaptureFailure> {
    state
        .undo_native_capture(request_id, encounter_id)
        .map_err(CaptureFailure::from)?;
    let _ = app.emit_to("main", LIBRARY_CHANGED_EVENT, ());
    Ok(())
}

pub fn hide_capture_window_for(
    state: &AppState,
    request_id: Uuid,
    hide: impl FnOnce() -> Result<(), String>,
) -> Result<(), CaptureFailure> {
    state
        .dismiss_and_publish(request_id, hide)
        .map_err(CaptureFailure::from)?
        .map_err(CaptureFailure::operation)
}

pub fn close_capture_window_for(
    state: &AppState,
    request_id: Uuid,
    hide: impl FnOnce() -> Result<(), String>,
) -> Result<(), CaptureFailure> {
    hide().map_err(CaptureFailure::operation)?;
    // Explicit close is an escape hatch: stale session bookkeeping must never
    // prevent the native window from being hidden.
    let _ = state.dismiss_and_publish(request_id, || ());
    Ok(())
}

pub fn hide_then_restore_focus(
    hide: impl FnOnce() -> Result<(), String>,
    restore_focus: impl FnOnce(),
) -> Result<(), String> {
    hide()?;
    restore_focus();
    Ok(())
}

#[tauri::command]
pub fn close_capture_window(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<(), CaptureFailure> {
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| CaptureFailure::operation("capture window is unavailable"))?;
    close_capture_window_for(&state, request_id, || {
        hide_then_restore_focus(
            || {
                #[cfg(target_os = "windows")]
                return vocab_platform_windows::window::hide_capture_window(
                    vocab_platform_windows::window::NativeWindowHandle::new(
                        window.hwnd().map_err(|error| error.to_string())?.0 as isize,
                    ),
                )
                .map_err(|error| error.to_string());
                #[cfg(not(target_os = "windows"))]
                window.hide().map_err(|error| error.to_string())
            },
            || {
                #[cfg(target_os = "windows")]
                let _ = vocab_platform_windows::window::restore_source_focus();
            },
        )
    })
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
        hide_then_restore_focus(
            || {
                #[cfg(target_os = "windows")]
                return vocab_platform_windows::window::hide_capture_window(
                    vocab_platform_windows::window::NativeWindowHandle::new(
                        window.hwnd().map_err(|error| error.to_string())?.0 as isize,
                    ),
                )
                .map_err(|error| error.to_string());
                #[cfg(not(target_os = "windows"))]
                window.hide().map_err(|error| error.to_string())
            },
            || {
                #[cfg(target_os = "windows")]
                let _ = vocab_platform_windows::window::restore_source_focus();
            },
        )
    })
}

#[tauri::command]
pub fn focus_capture_window(app: tauri::AppHandle) -> Result<(), CaptureFailure> {
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| CaptureFailure::operation("capture window is unavailable"))?;
    #[cfg(target_os = "windows")]
    return vocab_platform_windows::window::activate_for_editing(
        vocab_platform_windows::window::NativeWindowHandle::new(
            window
                .hwnd()
                .map_err(|error| CaptureFailure::operation(error.to_string()))?
                .0 as isize,
        ),
    )
    .map_err(CaptureFailure::from);
    #[cfg(not(target_os = "windows"))]
    window
        .set_focus()
        .map_err(|error| CaptureFailure::operation(error.to_string()))
}

#[tauri::command]
pub fn release_capture_window_focus() -> Result<(), CaptureFailure> {
    #[cfg(target_os = "windows")]
    return vocab_platform_windows::window::restore_source_focus().map_err(CaptureFailure::from);
    #[cfg(not(target_os = "windows"))]
    Ok(())
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
    let (pointer, work_areas, physical_work_areas) =
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
    #[cfg(target_os = "windows")]
    let physical_position =
        physical_position_for_logical(position, &work_areas, &physical_work_areas);
    state
        .publish_if_current(prepared.request_id, || {
            present_capture_window(
                || {
                    #[cfg(target_os = "windows")]
                    return window
                        .set_position(physical_position)
                        .map_err(|error| error.to_string());
                    #[cfg(not(target_os = "windows"))]
                    window
                        .set_position(LogicalPosition::new(position.x, position.y))
                        .map_err(|error| error.to_string())
                },
                || {
                    #[cfg(target_os = "windows")]
                    return vocab_platform_windows::window::show_without_activation(
                        vocab_platform_windows::window::NativeWindowHandle::new(
                            window.hwnd().map_err(|error| error.to_string())?.0 as isize,
                        ),
                    )
                    .map_err(|error| error.to_string());
                    #[cfg(not(target_os = "windows"))]
                    window.show().map_err(|error| error.to_string())
                },
                || {
                    window
                        .emit(
                            "capture-ready",
                            NativeCaptureEvent {
                                request_id: prepared.request_id,
                                candidate: prepared.candidate,
                            },
                        )
                        .map_err(|error| error.to_string())
                },
            )
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

pub(crate) fn present_region_ocr_capture(app: &tauri::AppHandle) -> Result<Uuid, CaptureFailure> {
    let state = app.state::<AppState>();
    if !state.platform_capabilities().screenshot_ocr {
        return Err(vocab_platform_api::PlatformError::Unsupported(
            vocab_platform_api::Capability::ScreenshotOcr,
        )
        .into());
    }
    let request_id = state.start_capture_request();
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| CaptureFailure::operation("capture window is unavailable"))?;
    let overlay = app
        .get_webview_window("ocr-overlay")
        .ok_or_else(|| CaptureFailure::operation("OCR overlay is unavailable"))?;
    window
        .hide()
        .map_err(|error| CaptureFailure::operation(error.to_string()))?;
    overlay
        .show()
        .and_then(|()| overlay.set_focus())
        .map_err(|error| CaptureFailure::operation(error.to_string()))?;
    state
        .publish_if_current(request_id, || {
            app.emit_to(
                "capture",
                "region-ocr-start",
                RegionOcrStartEvent { request_id },
            )
            .map_err(|error| error.to_string())?;
            overlay
                .emit("region-ocr-start", RegionOcrStartEvent { request_id })
                .map_err(|error| error.to_string())
        })
        .map_err(CaptureFailure::from)?
        .map_err(CaptureFailure::operation)?;
    Ok(request_id)
}

#[tauri::command]
pub fn show_region_ocr_failure(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<(), CaptureFailure> {
    state
        .publish_if_current(request_id, || {
            let window = app
                .get_webview_window("capture")
                .ok_or_else(|| "capture window is unavailable".to_string())?;
            window.show().map_err(|error| error.to_string())?;
            window
                .emit(
                    "capture-error",
                    NativeCaptureErrorEvent {
                        request_id,
                        code: CaptureFailureCode::Operation,
                        message: "OCR did not find readable text".into(),
                    },
                )
                .map_err(|error| error.to_string())
        })
        .map_err(CaptureFailure::from)?
        .map_err(CaptureFailure::operation)
}

#[tauri::command]
pub fn open_manual_capture(app: tauri::AppHandle) -> Result<(), CaptureFailure> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| CaptureFailure::operation("main window is unavailable"))?;
    main.show()
        .and_then(|()| main.set_focus())
        .map_err(|error| CaptureFailure::operation(error.to_string()))?;
    main.emit("open-manual-capture", ())
        .map_err(|error| CaptureFailure::operation(error.to_string()))
}

#[tauri::command]
pub fn cancel_region_ocr_capture(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<(), CaptureFailure> {
    state
        .dismiss_and_publish(request_id, || {
            if let Some(overlay) = app.get_webview_window("ocr-overlay") {
                overlay.hide().map_err(|error| error.to_string())?;
            }
            Ok::<_, String>(())
        })
        .map_err(CaptureFailure::from)?
        .map_err(CaptureFailure::operation)
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
    let primary = monitors
        .iter()
        .find(|monitor| {
            monitor.x <= 0.0
                && monitor.x + monitor.width > 0.0
                && monitor.y <= 0.0
                && monitor.y + monitor.height > 0.0
        })
        .or_else(|| monitors.first());
    let normalized_monitors = monitors
        .iter()
        .map(|monitor| {
            let (x, y) = primary.map_or(
                (
                    monitor.x / monitor.scale_factor,
                    monitor.y / monitor.scale_factor,
                ),
                |primary| {
                    (
                        normalize_axis(
                            monitor.x,
                            monitor.width,
                            monitor.scale_factor,
                            primary.x,
                            primary.width,
                            primary_scale,
                        ),
                        normalize_axis(
                            monitor.y,
                            monitor.height,
                            monitor.scale_factor,
                            primary.y,
                            primary.height,
                            primary_scale,
                        ),
                    )
                },
            );
            MonitorWorkArea::new(
                x,
                y,
                monitor.width / monitor.scale_factor,
                monitor.height / monitor.scale_factor,
                monitor.scale_factor,
            )
        })
        .collect::<Vec<_>>();
    let pointer_monitor_index = monitors
        .iter()
        .position(|monitor| {
            pointer.x >= monitor.x
                && pointer.x <= monitor.x + monitor.width
                && pointer.y >= monitor.y
                && pointer.y <= monitor.y + monitor.height
        })
        .unwrap_or(0);
    let pointer_monitor = monitors.get(pointer_monitor_index);
    let logical_monitor = normalized_monitors.get(pointer_monitor_index);
    let pointer = match (pointer_monitor, logical_monitor) {
        (Some(physical), Some(logical)) => ScreenPoint::new(
            logical.x + (pointer.x - physical.x) / physical.scale_factor,
            logical.y + (pointer.y - physical.y) / physical.scale_factor,
        ),
        _ => ScreenPoint::new(pointer.x / primary_scale, pointer.y / primary_scale),
    };
    (pointer, normalized_monitors)
}

fn normalize_axis(
    origin: f64,
    length: f64,
    scale: f64,
    primary_origin: f64,
    primary_length: f64,
    primary_scale: f64,
) -> f64 {
    let primary_end = primary_origin + primary_length;
    if origin >= primary_end {
        primary_origin / primary_scale
            + primary_length / primary_scale
            + (origin - primary_end) / primary_scale
    } else if origin + length <= primary_origin {
        primary_origin / primary_scale
            - length / scale
            - (primary_origin - origin - length) / primary_scale
    } else {
        (origin - primary_origin) / primary_scale + primary_origin / primary_scale
    }
}

#[cfg(target_os = "windows")]
fn physical_position_for_logical(
    position: ScreenPoint,
    monitors: &[MonitorWorkArea],
    physical_monitors: &[PhysicalMonitorWorkArea],
) -> PhysicalPosition<i32> {
    let monitor_index = monitors
        .iter()
        .position(|monitor| monitor.contains(position))
        .unwrap_or(0);
    match (
        monitors.get(monitor_index),
        physical_monitors.get(monitor_index),
    ) {
        (Some(logical), Some(physical)) => PhysicalPosition::new(
            (physical.x + (position.x - logical.x) * physical.scale_factor).round() as i32,
            (physical.y + (position.y - logical.y) * physical.scale_factor).round() as i32,
        ),
        _ => PhysicalPosition::new(position.x.round() as i32, position.y.round() as i32),
    }
}

fn pointer_and_work_areas(
    app: &tauri::AppHandle,
) -> Result<
    (
        ScreenPoint,
        Vec<MonitorWorkArea>,
        Vec<PhysicalMonitorWorkArea>,
    ),
    String,
> {
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
    let (pointer, work_areas) = normalize_virtual_desktop(
        ScreenPoint::new(pointer.x, pointer.y),
        primary_scale,
        &physical_work_areas,
    );
    Ok((pointer, work_areas, physical_work_areas))
}

fn pointer_and_primary_bounds(
    app: &tauri::AppHandle,
) -> Result<(ScreenPoint, MonitorWorkArea), String> {
    let (pointer, _, _) = pointer_and_work_areas(app)?;
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
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub(crate) fn portable_top_left_to_cocoa(
    point: ScreenPoint,
    primary: MonitorWorkArea,
) -> ScreenPoint {
    ScreenPoint::new(point.x, primary.y + primary.height - point.y)
}

#[cfg(target_os = "windows")]
fn ocr_region_for_target(
    region: vocab_platform_api::ScreenRect,
    _primary: MonitorWorkArea,
) -> vocab_platform_api::ScreenRect {
    region
}

#[cfg(not(target_os = "windows"))]
fn ocr_region_for_target(
    region: vocab_platform_api::ScreenRect,
    primary: MonitorWorkArea,
) -> vocab_platform_api::ScreenRect {
    vocab_platform_api::ScreenRect::new(
        region.x,
        primary.y + primary.height - region.y - region.height,
        region.width,
        region.height,
    )
}

#[cfg(test)]
mod tests {
    use vocab_platform_api::{MonitorWorkArea, ScreenPoint};

    #[cfg(target_os = "windows")]
    use super::physical_position_for_logical;
    use super::{PhysicalMonitorWorkArea, normalize_virtual_desktop, portable_top_left_to_cocoa};

    #[test]
    fn normalizes_mixed_scale_positive_and_negative_monitor_origins_into_one_logical_space() {
        let positive = normalize_virtual_desktop(
            ScreenPoint::new(3_600.0, 500.0),
            2.0,
            &[
                PhysicalMonitorWorkArea::new(0.0, 0.0, 2_880.0, 1_800.0, 2.0),
                PhysicalMonitorWorkArea::new(2_880.0, 100.0, 1_920.0, 1_080.0, 1.0),
            ],
        );
        assert_eq!(positive.0, ScreenPoint::new(2_160.0, 450.0));
        assert_eq!(
            positive.1,
            vec![
                MonitorWorkArea::new(0.0, 0.0, 1_440.0, 900.0, 2.0),
                MonitorWorkArea::new(1_440.0, 50.0, 1_920.0, 1_080.0, 1.0),
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
        assert_eq!(
            negative.0,
            ScreenPoint::new(-853.3333333333333, 183.33333333333331)
        );
        assert_eq!(
            negative.1[1],
            MonitorWorkArea::new(-1_280.0, -150.0, 1_280.0, 800.0, 1.5)
        );
        assert!(negative.1[1].contains(negative.0));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn maps_logical_placement_back_through_the_selected_monitors_scale() {
        let monitors = [
            MonitorWorkArea::new(0.0, 0.0, 1_440.0, 900.0, 2.0),
            MonitorWorkArea::new(-1_280.0, -200.0, 1_280.0, 800.0, 1.5),
        ];
        let physical = [
            PhysicalMonitorWorkArea::new(0.0, 0.0, 2_880.0, 1_800.0, 2.0),
            PhysicalMonitorWorkArea::new(-1_920.0, -300.0, 1_920.0, 1_200.0, 1.5),
        ];

        assert_eq!(
            physical_position_for_logical(ScreenPoint::new(-1_000.0, 100.0), &monitors, &physical,),
            tauri::PhysicalPosition::new(-1_500, 150),
        );
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

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_ocr_keeps_the_portable_top_left_region() {
        let region = vocab_platform_api::ScreenRect::new(-400.0, 75.0, 80.0, 24.0);
        let primary = MonitorWorkArea::new(0.0, 0.0, 1440.0, 900.0, 2.0);

        assert_eq!(super::ocr_region_for_target(region, primary), region);
    }
}
