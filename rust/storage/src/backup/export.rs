use super::{MAX_BACKUP_BYTES, TABLE_ORDER, exported_tables, hash_json};
use crate::{Database, database::storage, preferences::Preferences};
use rusqlite::types::ValueRef;
use serde::Serialize;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{self, BufWriter, Write},
    path::Path,
};
use yamantaka_domain::{Cancellation, Failure, Result};

#[derive(Clone, Copy, Debug)]
pub struct BackupProgress {
    pub table: &'static str,
    pub rows: u64,
}

struct Output<W> {
    writer: W,
    cancel: Cancellation,
    bytes: u64,
    limit_hit: bool,
    hash: Option<Sha256>,
}
impl<W: Write> Output<W> {
    fn result<T>(&self, result: std::result::Result<T, impl std::fmt::Display>) -> Result<T> {
        self.cancel.check()?;
        if self.limit_hit {
            return Err(Failure::Invalid("Backup exceeds 128 MiB".into()));
        }
        result.map_err(storage)
    }
    fn raw(&mut self, bytes: &[u8]) -> Result<()> {
        let result = self.write_all(bytes);
        self.result(result)
    }
    fn json(&mut self, value: &impl Serialize) -> Result<()> {
        let result = serde_json::to_writer(&mut *self, value);
        self.result(result)
    }
}
impl<W: Write> Write for Output<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.cancel.is_cancelled() {
            return Err(io::Error::other("Backup cancelled"));
        }
        if bytes.len() as u64 > MAX_BACKUP_BYTES - self.bytes {
            self.limit_hit = true;
            return Err(io::Error::other("Backup size limit"));
        }
        let count = self.writer.write(bytes)?;
        self.bytes += count as u64;
        if let Some(hash) = &mut self.hash {
            hash.update(&bytes[..count]);
        }
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.cancel.is_cancelled() {
            return Err(io::Error::other("Backup cancelled"));
        }
        self.writer.flush()
    }
}
impl Database {
    pub fn export_backup(
        &self,
        path: &Path,
        preferences: &Preferences,
        cancel: &Cancellation,
    ) -> Result<()> {
        self.export_backup_progress(path, preferences, cancel, |_| {})
    }
    /// One read transaction, one row at a time; failure never installs a partial file.
    pub fn export_backup_progress(
        &self,
        path: &Path,
        preferences: &Preferences,
        cancel: &Cancellation,
        mut progress: impl FnMut(BackupProgress),
    ) -> Result<()> {
        cancel.check()?;
        if preferences.version != 1 {
            return Err(Failure::Invalid(
                "Unsupported backup preferences version".into(),
            ));
        }
        let parent = path
            .parent()
            .ok_or_else(|| Failure::Invalid("A parent directory is required".into()))?;
        std::fs::create_dir_all(parent).map_err(storage)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(storage)?;
        {
            let mut output = Output {
                writer: BufWriter::with_capacity(64 * 1024, temporary.as_file_mut()),
                cancel: cancel.clone(),
                bytes: 0,
                limit_hit: false,
                hash: None,
            };
            let tx = self.connection.unchecked_transaction().map_err(storage)?;
            let exported = chrono::Utc::now().to_rfc3339();
            output.raw(b"{\"format\":\"yamantaka-local-backup\",\"version\":9,\"exportedAt\":")?;
            output.json(&exported)?;
            output.raw(b",\"containsPdfFiles\":false,\"tables\":{")?;
            let mut categories = Vec::with_capacity(TABLE_ORDER.len() + 3);
            for (index, table) in exported_tables().enumerate() {
                cancel.check()?;
                if index > 0 {
                    output.raw(b",")?;
                }
                output.json(&table)?;
                output.raw(b":")?;
                output.hash = Some(Sha256::new());
                output.raw(b"[")?;
                let mut statement = tx
                    .prepare(&format!(
                        "SELECT * FROM {table}{}",
                        if table == "document_covers" {
                            " WHERE selection_mode='manual'"
                        } else {
                            ""
                        }
                    ))
                    .map_err(storage)?;
                let names: Vec<_> = statement
                    .column_names()
                    .into_iter()
                    .map(str::to_owned)
                    .collect();
                let mut cursor = statement.query([]).map_err(storage)?;
                let mut count = 0_u64;
                while let Some(row) = cursor.next().map_err(storage)? {
                    cancel.check()?;
                    if count == 100_000 {
                        return Err(Failure::Invalid("Backup table exceeds 100,000 rows".into()));
                    }
                    let mut values = Map::new();
                    for (index, name) in names.iter().enumerate() {
                        let value = if table == "document_covers"
                            && matches!(
                                name.as_str(),
                                "small_thumbnail_path" | "large_thumbnail_path"
                            ) {
                            Value::String(String::new())
                        } else {
                            match row.get_ref(index).map_err(storage)? {
                                ValueRef::Null => Value::Null,
                                ValueRef::Integer(value) => json!(value),
                                ValueRef::Real(value) => json!(value),
                                ValueRef::Text(value) => {
                                    json!(std::str::from_utf8(value).map_err(storage)?)
                                }
                                ValueRef::Blob(_) => {
                                    return Err(Failure::Invalid(
                                        "Unexpected blob in legacy backup table".into(),
                                    ));
                                }
                            }
                        };
                        values.insert(name.clone(), value);
                    }
                    if count > 0 {
                        output.raw(b",")?;
                    }
                    output.json(&values)?;
                    count += 1;
                    progress(BackupProgress { table, rows: count });
                }
                output.raw(b"]")?;
                let digest = format!("{:x}", output.hash.take().unwrap().finalize());
                categories.push(json!({"name":table,"rows":count,"sha256":digest}));
            }
            tx.commit().map_err(storage)?;
            output.raw(b"},\"settings\":")?;
            let mut settings = preferences.legacy.clone();
            settings.extend(preferences.rust.clone());
            output.json(&settings)?;
            categories
                .push(json!({"name":"settings","rows":1,"sha256":hash_json(&json!([settings]))?}));
            output.raw(b",\"rustPreferences\":")?;
            output.json(preferences)?;
            categories.push(json!({"name":"rustPreferences","rows":1,"sha256":hash_json(&json!([preferences]))?}));
            output.raw(b",\"manifest\":")?;
            output.json(&json!({"format":"yamantaka-local-backup","version":9,"schemaVersion":11,"exportedAt":exported,"containsPdfFiles":false,"categories":categories}))?;
            output.raw(b"}\n")?;
            let result = output.flush();
            output.result(result)?;
        }
        temporary.as_file().sync_all().map_err(storage)?;
        cancel.check()?;
        temporary.persist(path).map_err(storage)?;
        Ok(())
    }
}
