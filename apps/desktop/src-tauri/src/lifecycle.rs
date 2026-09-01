pub const TRAY_OPEN_ID: &str = "open";
pub const TRAY_EXIT_ID: &str = "exit";

pub fn open_main_window(
    show: impl FnOnce() -> Result<(), String>,
    focus: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    show()?;
    focus()
}

pub fn terminate_session(
    cleanup: impl FnOnce() -> Result<(), String>,
    exit: impl FnOnce(),
) -> Result<(), String> {
    let cleanup_result = cleanup();
    exit();
    cleanup_result
}
