use std::sync::Mutex;

use vocab_platform_api::PlatformError;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetForegroundWindow, GetWindowLongPtrW, HWND_TOPMOST, SW_SHOWNOACTIVATE,
        SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetForegroundWindow,
        SetWindowLongPtrW, SetWindowPos, ShowWindow, WS_EX_APPWINDOW, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW,
    },
};

static SOURCE_WINDOW: Mutex<Option<NativeWindowHandle>> = Mutex::new(None);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeWindowHandle(isize);

impl NativeWindowHandle {
    pub const fn new(value: isize) -> Self {
        Self(value)
    }
}

pub const fn capture_window_extended_style(current: u32) -> u32 {
    (current | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0) & !WS_EX_APPWINDOW.0
}

pub fn configure_capture_window(hwnd: NativeWindowHandle) -> Result<(), PlatformError> {
    let hwnd = native_handle(hwnd);
    // SAFETY: Tauri supplies the live capture-window HWND and all calls stay on the desktop side.
    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            capture_window_extended_style(current) as isize,
        );
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        )
        .map_err(|_| operation("Windows could not configure the capture window"))?;
    }
    Ok(())
}

pub fn show_without_activation(hwnd: NativeWindowHandle) -> Result<(), PlatformError> {
    let hwnd = native_handle(hwnd);
    // SAFETY: both handles are process-external opaque values and are never dereferenced.
    unsafe {
        let source = GetForegroundWindow();
        if source != hwnd {
            *SOURCE_WINDOW
                .lock()
                .map_err(|_| operation("Capture focus state is unavailable"))? =
                Some(NativeWindowHandle::new(source.0 as isize));
        }
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
        .map_err(|_| operation("Windows could not present the capture window"))?;
    }
    Ok(())
}

pub fn activate_for_editing(hwnd: NativeWindowHandle) -> Result<(), PlatformError> {
    // SAFETY: Tauri supplies a live capture-window HWND.
    if unsafe { SetForegroundWindow(native_handle(hwnd)).as_bool() } {
        Ok(())
    } else {
        Err(operation("Windows could not focus the capture window"))
    }
}

pub fn restore_source_focus() -> Result<(), PlatformError> {
    let source = *SOURCE_WINDOW
        .lock()
        .map_err(|_| operation("Capture focus state is unavailable"))?;
    let Some(source) = source else {
        return Ok(());
    };
    // SAFETY: the stored value came from GetForegroundWindow; Windows validates stale handles.
    if unsafe { SetForegroundWindow(native_handle(source)).as_bool() } {
        Ok(())
    } else {
        Err(operation("Windows could not restore source focus"))
    }
}

const fn native_handle(value: NativeWindowHandle) -> HWND {
    HWND(value.0 as *mut core::ffi::c_void)
}

fn operation(message: &str) -> PlatformError {
    PlatformError::Operation(message.to_string())
}
