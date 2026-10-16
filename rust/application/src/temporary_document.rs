use yamantaka_domain::{Failure, LibraryRepository, Result};

pub(crate) fn reserve(repository: &impl LibraryRepository, prefix: &str) -> Result<String> {
    for _ in 0..4 {
        let id = format!("{prefix}-{}", uuid::Uuid::new_v4());
        if repository.get(&id)?.is_none() {
            return Ok(id);
        }
    }
    Err(Failure::Invalid(
        "Could not reserve a temporary document identity".into(),
    ))
}
