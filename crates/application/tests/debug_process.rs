use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

const CAPTURE: &str = concat!(
    "capture {\"selectedText\":\"Persistent\",\"lemma\":null,",
    "\"sentence\":\"Stored across sessions.\",\"sourceLanguage\":\"en\",",
    "\"targetLanguage\":\"de\",\"translation\":\"dauerhaft\",",
    "\"partOfSpeech\":\"adjective\",\"sourceApp\":\"core-debug\",",
    "\"sourceTitle\":null,\"sourceUrl\":null,\"captureOrigin\":\"manual\",",
    "\"capturedAt\":\"2026-08-29T14:00:00Z\"}"
);

fn run_example(arguments: &[&str], input: &str) -> Output {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["run", "--quiet", "--example", "core_debug", "--"])
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn process_reports_safe_default_banner_and_usage_exit_code() {
    let output = run_example(&[], "help\nquit\n");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("database: in-memory (discarded on exit)"));
    assert!(stdout.contains("capture <DebugCaptureInput JSON>"));

    let invalid = run_example(&["--unknown"], "");
    assert_eq!(invalid.status.code(), Some(2));
    assert!(
        String::from_utf8(invalid.stderr)
            .unwrap()
            .contains("usage: cargo run -p vocab-application --example core_debug")
    );
}

#[test]
fn explicit_database_path_persists_across_processes() {
    let database = TempDatabase::new();
    let path = database.path();
    let path_text = path.to_string_lossy().into_owned();

    let first = run_example(&["--db", &path_text], &format!("{CAPTURE}\nquit\n"));
    assert!(first.status.success());
    let second = run_example(&["--db", &path_text], "list\ndb summary\nquit\n");
    assert!(second.status.success());
    let stdout = String::from_utf8(second.stdout).unwrap();
    assert!(stdout.contains("(persistent)"));
    assert!(stdout.contains("\"displayForm\": \"Persistent\""));
    assert!(stdout.contains("\"wordCount\": 1"));
}

struct TempDatabase {
    path: std::path::PathBuf,
}

impl TempDatabase {
    fn new() -> Self {
        Self {
            path: std::env::temp_dir().join(format!(
                "vocab-core-debug-process-{}.db",
                uuid::Uuid::now_v7()
            )),
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDatabase {
    fn drop(&mut self) {
        if !self.path.exists() {
            return;
        }
        let resolved = self.path.canonicalize().expect("resolve debug database");
        let temp = std::env::temp_dir()
            .canonicalize()
            .expect("resolve temp dir");
        assert!(resolved.starts_with(temp));
        std::fs::remove_file(resolved).expect("remove debug database");
    }
}
