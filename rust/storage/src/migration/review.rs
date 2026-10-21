use super::*;

/// The source and fingerprints stay frozen between review and confirmation.
#[derive(Debug, Clone)]
pub struct MigrationReview {
    source: LegacySource,
    schema: u32,
    rows: BTreeMap<String, u64>,
    identities: BTreeMap<String, String>,
    preferences_sha256: String,
    schema_sha256: String,
    preferences_count: usize,
}
impl MigrationReview {
    pub fn source(&self) -> &LegacySource {
        &self.source
    }
    pub fn source_schema(&self) -> u32 {
        self.schema
    }
    pub fn imported_rows(&self) -> &BTreeMap<String, u64> {
        &self.rows
    }
    pub fn preferences_count(&self) -> usize {
        self.preferences_count
    }
    pub(super) fn check(
        &self,
        schema: u32,
        rows: &BTreeMap<String, u64>,
        identities: &BTreeMap<String, String>,
        preferences_sha256: &str,
        schema_sha256: &str,
    ) -> Result<()> {
        if self.schema != schema
            || &self.rows != rows
            || &self.identities != identities
            || self.preferences_sha256 != preferences_sha256
            || self.schema_sha256 != schema_sha256
        {
            return Err(Failure::MigrationSourceChanged);
        }
        Ok(())
    }
}

pub fn review(
    source: &LegacySource,
    flutter_closed: impl Fn() -> Result<bool>,
    cancel: &Cancellation,
) -> Result<MigrationReview> {
    cancel.check()?;
    if !flutter_closed()? {
        return Err(Failure::LegacyRunning);
    }
    let source = LegacySource {
        database: source.database.canonicalize().map_err(storage)?,
        preferences: source.preferences.canonicalize().map_err(storage)?,
    };
    let raw = fs::read(&source.preferences).map_err(storage)?;
    let preferences = Preferences::from_flutter_bytes(&raw)?;
    let database = readonly(&source.database)?;
    database.execute_batch("BEGIN").map_err(storage)?;
    let schema = version(&database)?;
    if schema == 0 || schema > SCHEMA_VERSION {
        return Err(Failure::Invalid("Unsupported Flutter database".into()));
    }
    validate(&database)?;
    let rows = table_counts(&database)?;
    let identities = fingerprints(&table_identities(&database, cancel)?);
    let schema_sha256 = schema_hash(&database, cancel)?;
    database.execute_batch("COMMIT").map_err(storage)?;
    cancel.check()?;
    confirm_source_preferences(&source, &raw, &flutter_closed)?;
    cancel.check()?;
    Ok(MigrationReview {
        source,
        schema,
        rows,
        identities,
        schema_sha256,
        preferences_sha256: format!("{:x}", Sha256::digest(&raw)),
        preferences_count: preferences.legacy.len(),
    })
}
