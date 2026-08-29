use std::io::Cursor;

use uuid::Uuid;
use vocab_application::debug::CoreDebugSession;

#[path = "../examples/core_debug.rs"]
#[allow(dead_code)]
mod core_debug;

use core_debug::{Command, parse_command, run};

const CAPTURE_JSON: &str = concat!(
    "{\"selectedText\":\"Serendipity\",\"lemma\":null,",
    "\"sentence\":\"A lucky moment.\",\"sourceLanguage\":\"en\",",
    "\"targetLanguage\":\"de\",\"translation\":\"glücklicher Zufall\",",
    "\"partOfSpeech\":\"noun\",\"sourceApp\":\"core-debug\",",
    "\"sourceTitle\":null,\"sourceUrl\":null,\"captureOrigin\":\"manual\",",
    "\"capturedAt\":\"2026-08-29T12:00:00Z\"}"
);

#[test]
fn parser_accepts_supported_commands_and_rejects_unsafe_input() {
    let word_id = "018f47d2-58d8-7a71-8b72-4b4f5d8f47a1";
    let encounter_id = "018f47d2-58d8-7a71-8b72-4b4f5d8f47a2";
    assert!(matches!(parse_command("help").unwrap(), Command::Help));
    assert!(matches!(parse_command("list").unwrap(), Command::List));
    assert!(matches!(
        parse_command(&format!("show {word_id}")).unwrap(),
        Command::Show(_)
    ));
    assert!(matches!(parse_command("today").unwrap(), Command::Today(_)));
    assert!(matches!(
        parse_command("today 2026-08-29T12:00:00Z").unwrap(),
        Command::Today(_)
    ));
    assert!(matches!(
        parse_command(&format!("review {word_id} remembered 2026-08-29T12:00:00Z")).unwrap(),
        Command::Review { .. }
    ));
    assert!(matches!(
        parse_command(&format!("undo {encounter_id}")).unwrap(),
        Command::Undo(_)
    ));
    assert!(matches!(
        parse_command("settings").unwrap(),
        Command::Settings
    ));
    assert!(matches!(
        parse_command(concat!(
            "settings set {\"sourceLanguage\":\"en\",\"targetLanguage\":\"zh\",",
            "\"captureShortcut\":\"Alt+Shift+V\",\"reviewTime\":\"20:00\",",
            "\"dailyLimit\":9,\"launchAtLogin\":false,\"appearance\":\"system\",",
            "\"reducedMotion\":false}"
        ))
        .unwrap(),
        Command::UpdateSettings(_)
    ));
    assert!(matches!(
        parse_command("db summary").unwrap(),
        Command::DatabaseSummary
    ));
    assert!(matches!(
        parse_command("db outbox").unwrap(),
        Command::Outbox
    ));
    assert!(matches!(parse_command("quit").unwrap(), Command::Quit));
    assert!(matches!(
        parse_command(&format!("capture {CAPTURE_JSON}")).unwrap(),
        Command::Capture(_)
    ));
    assert!(parse_command("show not-a-uuid").is_err());
    assert!(parse_command("today tomorrow").is_err());
    assert!(parse_command(&format!("review {word_id} maybe")).is_err());
    assert!(parse_command("capture {\"selectedText\":").is_err());
    assert!(parse_command("settings set {\"sourceLanguage\":").is_err());
    assert!(parse_command("db delete").is_err());
    assert!(parse_command("select * from words").is_err());
    assert!(parse_command("").is_err());
}

#[test]
fn scripted_repl_walks_through_core_and_recovers_from_errors() {
    let input = Cursor::new(
        format!("help\ncapture {CAPTURE_JSON}\nshow not-a-uuid\ndb summary\ndb outbox\nquit\n")
            .into_bytes(),
    );
    let mut output = Vec::new();

    run(
        input,
        &mut output,
        CoreDebugSession::in_memory(Uuid::nil()).unwrap(),
    )
    .unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("1. Input accepted"));
    assert!(output.contains("2. Domain normalization"));
    assert!(output.contains("3. Application result"));
    assert!(output.contains("4. Persistence readback and database delta"));
    assert!(output.contains("error: invalid word UUID"));
    assert!(output.contains("\"wordCount\": 1"));
    assert!(output.contains("\"entityType\": \"word\""));
}
