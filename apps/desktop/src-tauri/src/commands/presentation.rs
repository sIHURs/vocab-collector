/// Selects the page-level presentation at the desktop composition boundary.
#[tauri::command]
pub const fn get_presentation_family() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else {
        "shared"
    }
}
