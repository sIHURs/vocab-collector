//! Deterministic synthetic benchmark. Never opens an existing user database.
use std::{sync::Arc, time::Instant};

use chrono::Utc;
use rusqlite::{Connection, params};
use serde_json::json;
use uuid::Uuid;
use vocab_application::AppService;
use vocab_storage::{CaptureRecord, SqliteStore};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count: usize = std::env::args().nth(1).unwrap_or("10000".into()).parse()?;
    let repeats: usize = std::env::args().nth(2).unwrap_or("5".into()).parse()?;
    assert!(count > 0 && repeats > 0);
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("synthetic.db");
    drop(SqliteStore::open(&path)?);
    let mut db = Connection::open(&path)?;
    let tx = db.transaction()?;
    let date = "2026-09-01T12:00:00+00:00";
    for i in 0..count {
        let id = Uuid::from_u128(i as u128 + 1).to_string();
        let lemma = format!("synthetic{i:06}");
        let status = ["\"learning\"", "\"mastered\"", "\"paused\""][i % 3];
        let achieved = (i % 30 == 1).then_some(date);
        tx.execute("INSERT INTO words(id,dedupe_key,owner_scope,lemma,display_form,source_language,target_language,status,created_at,updated_at,review_state_json,mastered_at,achieved_at,delete_after) VALUES(?1,?2,'\"guest\"',?3,?3,'en','de',?4,?5,?5,'null',?6,?7,?7)", params![id,format!("\"guest\"|{}",vocab_domain::dedupe_key(&lemma,"en")),lemma,status,date,(i%3==1).then_some(date),achieved])?;
        for language in ["de", "zh"] {
            tx.execute("INSERT INTO translation_history(event_id,word_id,target_language,text,saved_at) VALUES(?1,?2,?3,?4,?5)",params![format!("{id}:{language}"),id,language,format!("meaning {i} {language}"),date])?;
        }
        for j in 0..10 {
            tx.execute("INSERT INTO encounters(id,word_id,selected_text,sentence,captured_at,updated_at) VALUES(?1,?2,?3,'synthetic context',?4,?4)",params![Uuid::from_u128((count+i*10+j+1) as u128).to_string(),id,lemma,date])?;
        }
        tx.execute("INSERT INTO review_logs(id,word_id,rating,reviewed_at,received_at,device_id) VALUES(?1,?1,'\"remembered\"',?2,?2,?1)",params![id,date])?;
    }
    tx.commit()?;
    let sqlite: String = db.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
    let store = Arc::new(SqliteStore::open(&path)?);
    let app = AppService::new(store.clone(), Uuid::from_u128(1));
    let mut samples = Vec::new();
    let candidate_plan = db.prepare("EXPLAIN QUERY PLAN SELECT id FROM words WHERE lemma='synthetic000000' AND owner_scope='\"guest\"' AND deleted_at IS NULL ORDER BY id")?
        .query_map([], |r|r.get::<_,String>(3))?.collect::<Result<Vec<_>,_>>()?;
    for run in 0..repeats {
        let start = Instant::now();
        let words = app.list_words()?;
        let list_ms = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(
            words.len(),
            count - (0..count).filter(|i| i % 30 == 1).count()
        );
        let start = Instant::now();
        let visible: Vec<_> = words
            .iter()
            .filter(|w| w.display_form.contains("000"))
            .take(50)
            .collect();
        let search_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let page = words.iter().skip(50).take(50).count();
        let page_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let check: String = db.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        assert_eq!(check, "ok");
        let check_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let capture = CaptureRecord {
            selected_text: "synthetic000000".into(),
            lemma: "synthetic000000".into(),
            sentence: "synthetic context".into(),
            source_language: "en".into(),
            target_language: "de".into(),
            translation: Some("meaning".into()),
            part_of_speech: None,
            source_app: None,
            source_title: None,
            source_url: None,
            capture_origin: Default::default(),
            captured_at: Utc::now(),
        };
        store.capture(&capture)?;
        let capture_ms = start.elapsed().as_secs_f64() * 1000.0;
        let concurrent = std::cell::RefCell::new(None);
        let health = vocab_storage::diagnostics::check(
            &path,
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
            |stage| {
                if stage == "sqlite" {
                    let writer = store.clone();
                    let capture = capture.clone();
                    *concurrent.borrow_mut() = Some(std::thread::spawn(move || {
                        let start = Instant::now();
                        writer
                            .capture(&capture)
                            .expect("save during read-only check must succeed");
                        start.elapsed().as_secs_f64() * 1000.0
                    }));
                }
            },
        );
        assert_eq!(health.status, "passed");
        let concurrent_save_ms = concurrent.into_inner().unwrap().join().unwrap();
        samples.push(json!({"run":run,"list_application_ms":list_ms,"search_simulation_ms":search_ms,"page_simulation_ms":page_ms,"quick_check_ms":check_ms,"capture_storage_ms":capture_ms,"search_count":visible.len(),"page_count":page,"health_check_ms":health.elapsed_ms,"save_during_check_ms":concurrent_save_ms}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"words":count,"encounters_initial":count*10,"sqlite":sqlite,"os":std::env::consts::OS,"processor":std::env::var("PROCESSOR_IDENTIFIER").ok(),"candidate_plan":candidate_plan,"samples":samples,"scope":"Rust application/storage; search/page simulations exclude WebView rendering and IPC"})
        )?
    );
    drop(app);
    drop(store);
    drop(db);
    directory.close()?;
    Ok(())
}
