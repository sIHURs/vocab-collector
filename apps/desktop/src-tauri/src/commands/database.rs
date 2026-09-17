use crate::bootstrap::AppState;
use serde::Serialize;
use std::{
    io::Write,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use vocab_storage::diagnostics::CheckReport;

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CheckSnapshot {
    id: String,
    running: bool,
    stage: String,
    report: Option<CheckReport>,
}
pub struct DatabaseTools {
    path: PathBuf,
    snapshot: Arc<Mutex<CheckSnapshot>>,
    cancel: Arc<AtomicBool>,
    saving: Arc<AtomicBool>,
}
impl DatabaseTools {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            snapshot: Default::default(),
            cancel: Default::default(),
            saving: Default::default(),
        }
    }
}
impl Drop for DatabaseTools {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
#[tauri::command]
pub fn start_database_check(tools: State<'_, DatabaseTools>) -> Result<CheckSnapshot, String> {
    let mut snapshot = tools.snapshot.lock().map_err(|_| "Check unavailable")?;
    if snapshot.running {
        return Ok(snapshot.clone());
    }
    tools.cancel.store(false, Ordering::Relaxed);
    *snapshot = CheckSnapshot {
        id: uuid::Uuid::now_v7().to_string(),
        running: true,
        stage: "structure".into(),
        report: None,
    };
    let initial = snapshot.clone();
    let shared = tools.snapshot.clone();
    let cancel = tools.cancel.clone();
    let path = tools.path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            vocab_storage::diagnostics::check(&path, cancel, |stage| {
                if let Ok(mut state) = shared.lock() {
                    state.stage = stage.into();
                }
            })
        }));
        if let Ok(mut state) = shared.lock() {
            state.running = false;
            match result {
                Ok(report) => state.report = Some(report),
                Err(_) => state.stage = "incomplete".into(),
            }
        }
    });
    Ok(initial)
}
#[tauri::command]
pub fn get_database_check(tools: State<'_, DatabaseTools>) -> Result<CheckSnapshot, String> {
    tools
        .snapshot
        .lock()
        .map(|s| s.clone())
        .map_err(|_| "Check unavailable".into())
}
#[tauri::command]
pub fn cancel_database_check(tools: State<'_, DatabaseTools>, id: String) -> Result<(), String> {
    let state = tools.snapshot.lock().map_err(|_| "Check unavailable")?;
    if state.id == id && state.running {
        tools.cancel.store(true, Ordering::Relaxed);
    }
    Ok(())
}

fn save_bytes(
    app: &tauri::AppHandle,
    name: &str,
    extension: &str,
    bytes: &[u8],
) -> Result<bool, String> {
    let Some(file) = app
        .dialog()
        .file()
        .set_file_name(name)
        .add_filter(extension, &[extension])
        .blocking_save_file()
    else {
        return Ok(false);
    };
    let path = file.into_path().map_err(|_| "Choose a local file")?;
    if path
        .extension()
        .and_then(|s| s.to_str())
        .is_none_or(|s| !s.eq_ignore_ascii_case(extension))
    {
        return Err(format!("Choose a .{extension} file"));
    }
    persist_bytes(&path, bytes)?;
    Ok(true)
}

fn persist_bytes(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("Invalid destination")?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|_| "Cannot write to that folder")?;
    temporary
        .write_all(bytes)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|_| "Could not save file; existing file was kept")?;
    temporary
        .persist(path)
        .map_err(|_| "Could not replace file; existing file was kept")?;
    Ok(())
}
struct Saving(Arc<AtomicBool>);
impl Drop for Saving {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Relaxed);
    }
}

#[tauri::command]
pub async fn export_vocabulary_csv(app: tauri::AppHandle) -> Result<Option<usize>, String> {
    let tools = app.state::<DatabaseTools>();
    if tools.saving.swap(true, Ordering::Relaxed) {
        return Err("A save is already in progress".into());
    }
    let guard = Saving(tools.saving.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        let (bytes, count) = app
            .state::<AppState>()
            .application()
            .export_vocabulary_csv()
            .map_err(|_| "Could not read vocabulary")?;
        save_bytes(&app, "vocabulary.csv", "csv", &bytes).map(|saved| saved.then_some(count))
    })
    .await
    .map_err(|_| "Export interrupted".to_string())?
}
#[tauri::command]
pub async fn save_database_check_report(app: tauri::AppHandle, id: String) -> Result<bool, String> {
    let tools = app.state::<DatabaseTools>();
    let snapshot = tools
        .snapshot
        .lock()
        .map_err(|_| "Check unavailable")?
        .clone();
    if snapshot.running || snapshot.id != id {
        return Err("No completed report for this check".into());
    }
    let report = snapshot.report.ok_or("No completed report")?;
    let bytes = serde_json::to_vec_pretty(
        &serde_json::json!({"appVersion":app.package_info().version.to_string(),"check":report}),
    )
    .map_err(|_| "Report unavailable")?;
    if tools.saving.swap(true, Ordering::Relaxed) {
        return Err("A save is already in progress".into());
    }
    let guard = Saving(tools.saving.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        save_bytes(&app, "database-check.json", "json", &bytes)
    })
    .await
    .map_err(|_| "Save interrupted".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn publication_replaces_complete_file_and_preserves_failed_destination() {
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("vocabulary.csv");
        std::fs::write(&destination, b"old complete file").unwrap();
        persist_bytes(&destination, b"new complete file").unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), b"new complete file");
        let invalid = directory.path().join("occupied.csv");
        std::fs::create_dir(&invalid).unwrap();
        std::fs::write(invalid.join("sentinel"), b"keep").unwrap();
        assert!(persist_bytes(&invalid, b"new bytes").is_err());
        assert_eq!(std::fs::read(invalid.join("sentinel")).unwrap(), b"keep");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
    }
}
