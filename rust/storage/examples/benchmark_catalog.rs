//! Actual repository-query measurements; no Flutter or frame-speed claims.
use serde_json::json;
use std::{path::PathBuf, time::Instant};
use yamantaka_domain::{LibraryQuery, LibraryRepository};
use yamantaka_storage::Database;

fn percentile(sorted: &[u128], percentile: f64) -> u128 {
    sorted[((sorted.len() - 1) as f64 * percentile).ceil() as usize]
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("build/verification/rust-catalog-benchmark.json"));
    let mut results = Vec::new();
    for count in [5_000, 25_000, 100_000] {
        for run in 0..5 {
            let db = Database::in_memory_at(11)?;
            let insert = Instant::now();
            db.execute_batch(&format!("WITH RECURSIVE corpus(n) AS (SELECT 0 UNION ALL SELECT n+1 FROM corpus WHERE n+1<{count}) INSERT INTO documents(id,path,normalized_path,file_name,title,author,page_count,file_size,file_modified_at,added_at,last_opened_at,reading_progress,fingerprint,is_favorite) SELECT printf('fixture-%06d',n),printf('fixture-%d.pdf',n),printf('fixture-%d.pdf',n),printf('fixture-%d.pdf',n),printf('Synthetic document %d',n),printf('Synthetic author %d',n%200),100+n%900,1024+n,'2026-01-01T00:00:00Z',strftime('%Y-%m-%dT%H:%M:%fZ','2026-01-01',printf('-%d minutes',n)),strftime('%Y-%m-%dT%H:%M:%fZ','2026-01-01',printf('-%d seconds',n)),(n%101)/100.0,printf('fingerprint-%d',n),n%17=0 FROM corpus;"))?;
            let insert_us = insert.elapsed().as_micros();
            db.integrity()?;
            let first = Instant::now();
            let docs = db.query(&LibraryQuery::default())?;
            let total = db.count(&LibraryQuery::default())?;
            let first_us = first.elapsed().as_micros();
            assert_eq!(docs.len(), 80);
            assert_eq!(total, count as u64);
            let mut samples = Vec::with_capacity(200);
            let mut command_samples = Vec::with_capacity(200);
            for sample in 0..200 {
                // Repeat the incumbent benchmark's exact 20-query pattern ten
                // times per pass. A separate series includes the count query
                // that the application's command currently executes as well.
                let pass = sample % 20;
                let query = LibraryQuery {
                    offset: (pass * 80) % (count - 100),
                    page_size: 80,
                    text: if pass % 2 == 0 {
                        String::new()
                    } else {
                        "Synthetic document".into()
                    },
                    favorites_only: pass % 5 == 0,
                    ..Default::default()
                };
                let start = Instant::now();
                let docs = db.query(&query)?;
                samples.push(start.elapsed().as_micros());
                assert!(docs.len() <= 80);
                let start = Instant::now();
                db.query(&query)?;
                db.count(&query)?;
                command_samples.push(start.elapsed().as_micros());
            }
            samples.sort_unstable();
            command_samples.sort_unstable();
            let p95 = percentile(&command_samples, 0.95);
            results.push(json!({"documents":count,"run":run+1,"samples":200,"insertUs":insert_us,
                "firstQueryAndCountUs":first_us,"queryP50Us":percentile(&samples,0.5),
                "queryP95Us":percentile(&samples,0.95),"commandP50Us":percentile(&command_samples,0.5),
                "commandP95Us":p95,"within50msBudget":p95<=50_000,"materializedLimit":80}));
            println!(
                "{count} documents, run {}: query+count p95 {p95} us",
                run + 1
            );
        }
    }
    let passed = results.iter().all(|r| r["within50msBudget"] == true);
    let report = json!({"format":"yamantaka-rust-catalog-benchmark","version":1,
        "generatedAt":chrono::Utc::now().to_rfc3339(),"profile":if cfg!(debug_assertions){"debug"}else{"release"},
        "samplesPerSize":1000,"passes":5,"pageSize":80,"containsPersonalData":false,
        "scope":"repository query and query+count only; no renderer, UI, memory or Flutter comparison",
        "allCommandP95Within50ms":passed,"results":results});
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, serde_json::to_vec_pretty(&report)?)?;
    if !passed {
        return Err("catalog command p95 exceeded 50 ms".into());
    }
    Ok(())
}
