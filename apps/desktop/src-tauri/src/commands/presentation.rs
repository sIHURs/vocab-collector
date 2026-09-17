/// Selects the page-level presentation at the desktop composition boundary.
#[tauri::command]
pub const fn get_presentation_family() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else {
        "shared"
    }
}
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct StartupReady(AtomicBool);

impl StartupReady {
    pub fn complete(&self) {
        self.0.store(true, Ordering::Release);
    }
}

// Registered by Builder before any WebView exists; safe during setup.
#[tauri::command]
pub fn get_startup_ready(state: tauri::State<'_, StartupReady>) -> bool {
    state.0.load(Ordering::Acquire)
}
