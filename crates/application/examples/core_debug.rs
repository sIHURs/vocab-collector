use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;
use vocab_application::debug::{CaptureTrace, CoreDebugSession, DebugCaptureInput, DebugStage};
use vocab_domain::{ReviewRating, UserSettings};

const HELP: &str = concat!(
    "Commands:\n",
    "  help\n",
    "  capture <DebugCaptureInput JSON>\n",
    "  list\n",
    "  show <word-uuid>\n",
    "  today [RFC3339 timestamp]\n",
    "  review <word-uuid> forgot|remembered [RFC3339 timestamp]\n",
    "  undo <encounter-uuid>\n",
    "  settings\n",
    "  settings set <UserSettings JSON>\n",
    "  db summary\n",
    "  db outbox\n",
    "  quit\n\n",
    "Example:\n",
    "capture {\"selectedText\":\"Serendipity\",\"lemma\":null,",
    "\"sentence\":\"A lucky moment.\",\"sourceLanguage\":\"en\",",
    "\"targetLanguage\":\"de\",\"translation\":\"glücklicher Zufall\",",
    "\"partOfSpeech\":\"noun\",\"sourceApp\":\"core-debug\",",
    "\"sourceTitle\":null,\"sourceUrl\":null,\"captureOrigin\":\"manual\",",
    "\"capturedAt\":\"2026-08-29T12:00:00Z\"}"
);

#[derive(Debug)]
pub enum Command {
    Help,
    Capture(DebugCaptureInput),
    List,
    Show(Uuid),
    Today(DateTime<Utc>),
    Review {
        word_id: Uuid,
        rating: ReviewRating,
        at: DateTime<Utc>,
    },
    Undo(Uuid),
    Settings,
    UpdateSettings(UserSettings),
    DatabaseSummary,
    Outbox,
    Quit,
}

pub fn parse_command(line: &str) -> Result<Command, String> {
    let line = line.trim();
    match line {
        "help" => return Ok(Command::Help),
        "list" => return Ok(Command::List),
        "settings" => return Ok(Command::Settings),
        "db summary" => return Ok(Command::DatabaseSummary),
        "db outbox" => return Ok(Command::Outbox),
        "quit" | "exit" => return Ok(Command::Quit),
        _ => {}
    }

    if let Some(json) = line.strip_prefix("capture ") {
        return serde_json::from_str(json)
            .map(Command::Capture)
            .map_err(|error| format!("invalid capture JSON: {error}"));
    }
    if let Some(json) = line.strip_prefix("settings set ") {
        return serde_json::from_str(json)
            .map(Command::UpdateSettings)
            .map_err(|error| format!("invalid settings JSON: {error}"));
    }
    if let Some(value) = line.strip_prefix("show ") {
        return parse_uuid(value, "word").map(Command::Show);
    }
    if let Some(value) = line.strip_prefix("undo ") {
        return parse_uuid(value, "encounter").map(Command::Undo);
    }
    if line == "today" {
        return Ok(Command::Today(Utc::now()));
    }
    if let Some(value) = line.strip_prefix("today ") {
        return parse_time(value).map(Command::Today);
    }
    if let Some(value) = line.strip_prefix("review ") {
        let parts: Vec<_> = value.split_whitespace().collect();
        if !(parts.len() == 2 || parts.len() == 3) {
            return Err("usage: review <word-uuid> forgot|remembered [RFC3339 timestamp]".into());
        }
        let word_id = parse_uuid(parts[0], "word")?;
        let rating = match parts[1] {
            "forgot" => ReviewRating::Forgot,
            "remembered" => ReviewRating::Remembered,
            _ => return Err("rating must be forgot or remembered".into()),
        };
        let at = parts
            .get(2)
            .map_or_else(|| Ok(Utc::now()), |value| parse_time(value))?;
        return Ok(Command::Review {
            word_id,
            rating,
            at,
        });
    }

    Err("unknown command; run help for the supported command list".into())
}

fn parse_uuid(value: &str, kind: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value.trim()).map_err(|_| format!("invalid {kind} UUID"))
}

fn parse_time(value: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(value.trim())
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|_| "invalid RFC3339 timestamp".into())
}

pub fn run<R: BufRead, W: Write>(
    mut input: R,
    mut output: W,
    session: CoreDebugSession,
) -> io::Result<()> {
    let mut line = String::new();
    loop {
        write!(output, "core-debug> ")?;
        output.flush()?;
        line.clear();
        if input.read_line(&mut line)? == 0 {
            writeln!(output)?;
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let command = match parse_command(&line) {
            Ok(command) => command,
            Err(error) => {
                writeln!(output, "error: {error}")?;
                continue;
            }
        };

        match command {
            Command::Help => writeln!(output, "{HELP}")?,
            Command::Capture(value) => match session.capture(value) {
                Ok(trace) => print_trace(&mut output, &trace)?,
                Err(error) => writeln!(output, "error: {error}")?,
            },
            Command::List => emit(&mut output, session.list_words())?,
            Command::Show(word_id) => emit(&mut output, session.inspect_word(word_id))?,
            Command::Today(now) => emit(&mut output, session.today(now))?,
            Command::Review {
                word_id,
                rating,
                at,
            } => emit(&mut output, session.review(word_id, rating, at))?,
            Command::Undo(encounter_id) => emit(&mut output, session.undo(encounter_id))?,
            Command::Settings => emit(&mut output, session.settings())?,
            Command::UpdateSettings(settings) => {
                emit(&mut output, session.update_settings(settings))?;
            }
            Command::DatabaseSummary => emit(&mut output, session.database_summary())?,
            Command::Outbox => emit(&mut output, session.outbox())?,
            Command::Quit => break,
        }
    }
    Ok(())
}

fn print_trace(output: &mut impl Write, trace: &CaptureTrace) -> io::Result<()> {
    for observation in &trace.observations {
        let label = match observation.stage {
            DebugStage::Input => "1. Input accepted",
            DebugStage::Domain => "2. Domain normalization",
            DebugStage::Application => "3. Application result",
            DebugStage::Persistence => "4. Persistence readback and database delta",
        };
        writeln!(output, "{label}")?;
        write_json(output, &observation.data)?;
    }
    Ok(())
}

fn emit<W: Write, T: Serialize, E: std::fmt::Display>(
    output: &mut W,
    result: Result<T, E>,
) -> io::Result<()> {
    match result {
        Ok(value) => write_json(output, &value),
        Err(error) => writeln!(output, "error: {error}"),
    }
}

fn write_json(output: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *output, value).map_err(io::Error::other)?;
    writeln!(output)
}

fn usage() -> &'static str {
    "usage: cargo run -p vocab-application --example core_debug -- [--db <path>]"
}

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let device_id = Uuid::now_v7();
    let session = match arguments.as_slice() {
        [] => {
            println!("database: in-memory (discarded on exit)");
            CoreDebugSession::in_memory(device_id)
        }
        [flag, path] if flag == "--db" => {
            let path = match std::path::absolute(PathBuf::from(path)) {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("error: could not resolve database path: {error}");
                    std::process::exit(1);
                }
            };
            println!("database: {} (persistent)", path.display());
            CoreDebugSession::open(path, device_id)
        }
        _ => {
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    };
    let session = match session {
        Ok(session) => session,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    };

    let stdin = io::stdin();
    let stdout = io::stdout();
    if let Err(error) = run(stdin.lock(), stdout.lock(), session) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
