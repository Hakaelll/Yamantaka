use super::*;
use yamantaka_domain::duplicates::merge::{Guard, Selection};
struct Sources {
    files: [HashedFile; 2],
    _lease: yamantaka_domain::memory::ByteLease,
}
impl Guard for Sources {
    fn verify(&self) -> Result<()> {
        for file in &self.files {
            file.verify()?;
        }
        Ok(())
    }
}
pub fn freeze(
    selection: &Selection,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<Box<dyn Guard>> {
    cancel.check()?;
    let bytes = [&selection.kept.file.path, &selection.incoming.file.path]
        .iter()
        .try_fold(1024usize, |bytes, path| {
            bytes.checked_add(path.as_os_str().as_encoded_bytes().len().checked_mul(8)?)
        })
        .ok_or(Failure::MemoryBudget)?;
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Other,
            MemoryRepresentation::Cpu,
            bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut buffer = TrackedBytesBuilder::try_zeroed(governor, MemoryCategory::Other, 65536)
        .map_err(|_| Failure::MemoryBudget)?;
    let read = |source: &Source, proof: &FullSourceHash, buffer: &mut [u8]| -> Result<HashedFile> {
        let file = HashedFile::read(&source.file.path, buffer, cancel)?;
        if file.reference(&proof.source)?.digest != proof.digest
            || file.fingerprint() != source.file.fingerprint
            || file.modified_matches(&source.file.modified) != Some(true)
        {
            return Err(Failure::Invalid(
                "Duplicate file changed; search and review again".into(),
            ));
        }
        Ok(file)
    };
    let left = read(
        &selection.kept,
        &selection.kept_proof,
        buffer.as_mut_slice(),
    )?;
    let right = read(
        &selection.incoming,
        &selection.incoming_proof,
        buffer.as_mut_slice(),
    )?;
    let guard = Sources {
        files: [left, right],
        _lease: lease,
    };
    guard.verify()?;
    Ok(Box::new(guard))
}
