use yamantaka_domain::{
    Cancellation, LibraryRepository, NewDocument, Result, source_revision::FullSourceHash,
};
use yamantaka_storage::{Database, identity};
// The native fixtures are quiescent here; production keeps an OS read guard.
pub fn save(database: &mut Database, document: &NewDocument) -> Result<String> {
    let cancel = Cancellation::default();
    let hash = identity::full_sha256(&document.path, &cancel)?;
    let digest = std::array::from_fn(|index| {
        u8::from_str_radix(&hash[index * 2..index * 2 + 2], 16).unwrap()
    });
    let reference = FullSourceHash {
        source: document.source.clone(),
        digest,
    };
    database.import(document, &reference, &cancel, None)
}
