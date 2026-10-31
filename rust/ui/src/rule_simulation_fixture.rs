use std::{ffi::OsString, path::Path};
use yamantaka_storage::{Database, identity};

pub fn seed(
    db: &Database,
    store: &Path,
    fixture: &Path,
    args: &[OsString],
) -> Result<(), Box<dyn std::error::Error>> {
    if args.iter().any(|arg| arg == "--rule-simulation-duplicates") {
        db.execute_batch("DELETE FROM documents WHERE id NOT IN ('demo-0','demo-1','demo-2'); UPDATE automation_rules SET trigger_json='{\"kind\":\"exactDuplicate\",\"value\":null}' WHERE id='legacy-rule';")?;
        let mut bytes = std::fs::read(fixture)?;
        bytes.resize(70000, b' ');
        for index in 0..3 {
            *bytes.last_mut().unwrap() = if index == 2 { b'B' } else { b'A' };
            let path = store.join(format!("duplicate-{index}.pdf"));
            std::fs::write(&path, &bytes)?;
            let path = path.canonicalize()?;
            let fingerprint = identity::legacy_fingerprint(&path)?;
            let modified =
                chrono::DateTime::<chrono::Utc>::from(path.metadata()?.modified()?).to_rfc3339();
            let escaped = path.to_string_lossy().replace('\'', "''");
            db.execute_batch(&format!("UPDATE documents SET path='{escaped}',normalized_path='{}',file_size=70000,fingerprint='{fingerprint}',file_modified_at='{modified}',is_missing=0 WHERE id='demo-{index}'", escaped.to_lowercase()))?;
        }
    } else if args.iter().any(|arg| arg == "--rule-simulation-cancel") {
        db.execute_batch("WITH RECURSIVE n(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM n WHERE x<99999) INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) SELECT printf('scan-%06d',x),printf('scan-%06d.pdf',x),printf('scan-%06d.pdf',x),'scan.pdf',2,70000,'2026-10-31','2026-10-31','2026-10-31','synthetic' FROM n;")?;
    }
    Ok(())
}
