//! Read-only, bounded diagnostics. Never initializes or migrates a database.
use chrono::Utc;
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub checked_at: String,
    pub status: String,
    pub stage: String,
    pub sqlite_version: String,
    pub schema_version: Option<u32>,
    pub issues: Vec<String>,
    pub coverage: Vec<String>,
    pub elapsed_ms: u64,
    pub counts: std::collections::BTreeMap<String, i64>,
    pub database_bytes: Option<u64>,
}

/// The callback contains only public stage names, never SQL errors or row data.
pub fn check(path: &Path, cancelled: Arc<AtomicBool>, progress: impl Fn(&str)) -> CheckReport {
    let start = Instant::now();
    let mut report = CheckReport {
        checked_at: Utc::now().to_rfc3339(),
        status: "incomplete".into(),
        stage: "structure".into(),
        sqlite_version: rusqlite::version().into(),
        schema_version: None,
        issues: vec![],
        coverage: vec![],
        elapsed_ms: 0,
        counts: Default::default(),
        database_bytes: std::fs::metadata(path).ok().map(|m| m.len()),
    };
    let run = (|| -> rusqlite::Result<()> {
        if cancelled.load(Ordering::Relaxed) {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let db = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        db.busy_timeout(Duration::from_millis(250))?;
        let stop = cancelled.clone();
        db.progress_handler(
            1000,
            Some(move || stop.load(Ordering::Relaxed) || start.elapsed() > Duration::from_secs(30)),
        )?;
        db.execute_batch("BEGIN")?;
        progress("structure");
        let version: u32 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        report.schema_version = Some(version);
        if version != 9 {
            report.issues.push("unsupported_schema_version".into());
            return Err(rusqlite::Error::InvalidQuery);
        }
        for query in [
            "SELECT id,owner_scope,lemma,display_form,source_language,target_language,translation,part_of_speech,status,created_at,updated_at,deleted_at,mastered_at,achieved_at,delete_after,review_state_json FROM words LIMIT 0",
            "SELECT id,word_id,selected_text,sentence,source_app,source_title,source_url,capture_origin,captured_at,updated_at,deleted_at,saved_translation_json FROM encounters LIMIT 0",
            "SELECT id,word_id,rating,reviewed_at,received_at,device_id FROM review_logs LIMIT 0",
            "SELECT word_id,target_language,text,saved_at FROM translation_history LIMIT 0",
            "SELECT payload FROM user_settings LIMIT 0",
            "SELECT date,count FROM vocabulary_daily_counts LIMIT 0",
            "SELECT mutation_id,entity_type,entity_id,operation,payload,created_at,attempt_count FROM outbox LIMIT 0",
            "SELECT singleton,vocabulary_count,encounter_count,review_count,remembered_count,forgotten_count FROM lifetime_archive LIMIT 0",
            "SELECT singleton,started_on FROM vocabulary_log_coverage LIMIT 0",
            "SELECT word_id,revision FROM word_revisions LIMIT 0",
            "SELECT encounter_id,word_id,revision,before_json FROM capture_undo LIMIT 0",
            "SELECT token,word_id,revision,before_json FROM learning_status_undo LIMIT 0",
        ] {
            db.prepare(query)?;
        }
        report.coverage.push("structure".into());
        report.stage = "sqlite".into();
        progress("sqlite");
        let mut statement = db.prepare("PRAGMA quick_check(20)")?;
        let errors = statement
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if errors.iter().any(|e| e != "ok") {
            report.issues.push("sqlite_structure_error".into());
        }
        report.coverage.push("quick_check".into());
        report.stage = "relationships".into();
        progress("relationships");
        if db
            .prepare("PRAGMA foreign_key_check")?
            .query([])?
            .next()?
            .is_some()
        {
            report.issues.push("foreign_key_violation".into());
        }
        report.coverage.push("foreign_keys".into());
        report.stage = "formats".into();
        progress("formats");
        for (query, code) in [(
            "SELECT id,owner_scope,lemma,display_form,source_language,target_language,translation,part_of_speech,status,created_at,updated_at,deleted_at,mastered_at,achieved_at,delete_after,review_state_json FROM words",
            "invalid_vocabulary_format",
        )] {
            let mut stmt = db.prepare(query)?;
            for word in stmt.query_map([], super::map_word)? {
                if cancelled.load(Ordering::Relaxed) || start.elapsed() > Duration::from_secs(30) {
                    return Err(rusqlite::Error::InvalidQuery);
                }
                if word.is_err() && !report.issues.iter().any(|s| s == code) {
                    report.issues.push(code.into());
                }
            }
        }
        for (query, code, kind) in [
            (
                "SELECT captured_at FROM encounters UNION ALL SELECT updated_at FROM encounters UNION ALL SELECT reviewed_at FROM review_logs UNION ALL SELECT received_at FROM review_logs UNION ALL SELECT saved_at FROM translation_history",
                "invalid_timestamp",
                0,
            ),
            (
                "SELECT saved_translation_json FROM encounters WHERE saved_translation_json IS NOT NULL",
                "invalid_translation_snapshot",
                1,
            ),
            ("SELECT payload FROM user_settings", "invalid_settings", 2),
            ("SELECT rating FROM review_logs", "invalid_review_rating", 3),
        ] {
            let mut stmt = db.prepare(query)?;
            for value in stmt.query_map([], |r| r.get::<_, String>(0))? {
                if cancelled.load(Ordering::Relaxed) || start.elapsed() > Duration::from_secs(30) {
                    return Err(rusqlite::Error::InvalidQuery);
                }
                let valid = value.is_ok_and(|value| match kind {
                    0 => chrono::DateTime::parse_from_rfc3339(&value).is_ok(),
                    1 => serde_json::from_str::<vocab_domain::SavedTranslation>(&value).is_ok(),
                    2 => serde_json::from_str::<vocab_domain::UserSettings>(&value).is_ok(),
                    _ => serde_json::from_str::<vocab_domain::ReviewRating>(&value).is_ok(),
                });
                if !valid && !report.issues.iter().any(|s| s == code) {
                    report.issues.push(code.into());
                }
            }
        }
        report.coverage.push("application_formats".into());
        for (name, query) in [
            (
                "vocabulary",
                "SELECT COUNT(*) FROM words WHERE deleted_at IS NULL",
            ),
            (
                "encounters",
                "SELECT COUNT(*) FROM encounters WHERE deleted_at IS NULL",
            ),
            ("reviews", "SELECT COUNT(*) FROM review_logs"),
            ("pending_changes", "SELECT COUNT(*) FROM outbox"),
        ] {
            report
                .counts
                .insert(name.into(), db.query_row(query, [], |r| r.get(0))?);
        }
        db.execute_batch("ROLLBACK")?;
        Ok(())
    })();
    report.status = if cancelled.load(Ordering::Relaxed) {
        "cancelled"
    } else if run.is_err() {
        if report.issues.is_empty() {
            let code = if start.elapsed() > Duration::from_secs(30) {
                "check_time_limit"
            } else if let Err(rusqlite::Error::SqliteFailure(error, _)) = &run {
                match error.code {
                    rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked => {
                        "database_busy"
                    }
                    rusqlite::ErrorCode::CannotOpen => "database_unavailable",
                    rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase => {
                        "database_unreadable"
                    }
                    _ => "check_could_not_complete",
                }
            } else {
                "check_could_not_complete"
            };
            report.issues.push(code.into());
        }
        "incomplete"
    } else if report.issues.is_empty() {
        "passed"
    } else {
        "issues"
    }
    .into();
    report.elapsed_ms = start.elapsed().as_millis().min(u64::MAX as u128) as u64;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("vocab-check-{}.db", uuid::Uuid::now_v7()));
        let store = crate::SqliteStore::open(&path).unwrap();
        store
            .capture(&crate::CaptureRecord {
                selected_text: "private vocabulary".into(),
                lemma: "private vocabulary".into(),
                sentence: "private context".into(),
                source_language: "en".into(),
                target_language: "zh".into(),
                translation: Some("私人数据".into()),
                part_of_speech: None,
                source_app: None,
                source_title: None,
                source_url: Some("https://private.example".into()),
                capture_origin: Default::default(),
                captured_at: Utc::now(),
            })
            .unwrap();
        path
    }

    #[test]
    fn finds_format_and_relationship_errors_without_disclosing_values() {
        let path = fixture();
        let db = Connection::open(&path).unwrap();
        db.execute_batch("PRAGMA foreign_keys=OFF; UPDATE words SET status='private-invalid-value'; UPDATE encounters SET word_id='missing-private-id';").unwrap();
        drop(db);
        let before = std::fs::read(&path).unwrap();
        let report = check(&path, Arc::new(AtomicBool::new(false)), |_| {});
        assert_eq!(report.status, "issues");
        assert!(report.issues.contains(&"foreign_key_violation".into()));
        assert!(report.issues.contains(&"invalid_vocabulary_format".into()));
        let wire = serde_json::to_string(&report).unwrap();
        assert!(!wire.contains("private"));
        assert!(!wire.contains("私人"));
        assert_eq!(std::fs::read(&path).unwrap(), before);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn cancellation_and_lock_failure_release_the_connection() {
        let path = fixture();
        let stop = Arc::new(AtomicBool::new(false));
        let trigger = stop.clone();
        let result = check(&path, stop, |stage| {
            if stage == "formats" {
                trigger.store(true, Ordering::Relaxed);
            }
        });
        assert_eq!(result.status, "cancelled");
        let db = Connection::open(&path).unwrap();
        db.execute_batch("BEGIN EXCLUSIVE").unwrap();
        let start = Instant::now();
        assert_eq!(
            check(&path, Arc::new(AtomicBool::new(false)), |_| {}).status,
            "incomplete"
        );
        assert!(start.elapsed() < Duration::from_secs(3));
        db.execute_batch("ROLLBACK").unwrap();
        drop(db);
        assert_eq!(
            check(&path, Arc::new(AtomicBool::new(false)), |_| {}).status,
            "passed"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn corrupt_or_newer_database_is_never_initialized() {
        let path = fixture();
        let db = Connection::open(&path).unwrap();
        db.execute_batch("PRAGMA user_version=100").unwrap();
        drop(db);
        assert_eq!(
            check(&path, Arc::new(AtomicBool::new(false)), |_| {}).issues,
            vec!["unsupported_schema_version"]
        );
        std::fs::write(&path, b"private broken database").unwrap();
        assert_eq!(
            check(&path, Arc::new(AtomicBool::new(false)), |_| {}).status,
            "incomplete"
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"private broken database");
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn diagnostic_is_read_only_and_redacts_errors() {
        let path = std::env::temp_dir().join(format!("vocab-check-{}.db", uuid::Uuid::now_v7()));
        drop(crate::SqliteStore::open(&path).unwrap());
        let before = std::fs::read(&path).unwrap();
        assert_eq!(
            check(&path, Arc::new(AtomicBool::new(false)), |_| {}).status,
            "passed"
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let db = Connection::open(&path).unwrap();
        db.execute_batch("DROP TABLE review_logs").unwrap();
        drop(db);
        let report = check(&path, Arc::new(AtomicBool::new(false)), |_| {});
        assert_eq!(report.status, "incomplete");
        assert!(
            !serde_json::to_string(&report)
                .unwrap()
                .contains(path.to_str().unwrap())
        );
        assert_eq!(
            check(&path, Arc::new(AtomicBool::new(true)), |_| {}).status,
            "cancelled"
        );
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            check(&path, Arc::new(AtomicBool::new(false)), |_| {}).status,
            "incomplete"
        );
        assert!(!path.exists());
    }
}
