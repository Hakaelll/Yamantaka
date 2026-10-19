use super::*;
use std::{
    io::ErrorKind,
    sync::{MutexGuard, TryLockError},
    time::Duration,
};
use yamantaka_domain::cover_cache::{CoverCacheInventory, CoverCacheOperation, CoverCacheReport};
fn maintenance_error(_: impl std::fmt::Display) -> Failure {
    Failure::Storage("Cover cache maintenance could not finish; check access and retry".into())
}
impl CoverCache {
    pub(super) fn try_writer(&self) -> Result<Option<MutexGuard<'_, ()>>> {
        match self.writer.try_lock() {
            Ok(guard) => Ok(Some(guard)),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Poisoned(_)) => Err(maintenance_error("writer stopped")),
        }
    }
    fn maintenance_root(&self) -> Result<Option<File>> {
        if !self.root.is_absolute() {
            return Err(maintenance_error("absolute cache root required"));
        }
        let metadata = match fs::symlink_metadata(&self.root) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(maintenance_error(error)),
        };
        if !metadata.is_dir() || is_reparse(&metadata) {
            return Err(maintenance_error("cache root is not an ordinary directory"));
        }
        let mut options = fs::OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // BACKUP_SEMANTICS opens a directory; OPEN_REPARSE_POINT rejects a swapped link.
            options.share_mode(3).custom_flags(0x02000000 | 0x00200000);
        }
        let guard = options.open(&self.root).map_err(maintenance_error)?;
        let actual = guard.metadata().map_err(maintenance_error)?;
        if !actual.is_dir() || is_reparse(&actual) {
            return Err(maintenance_error("cache root changed"));
        }
        Ok(Some(guard))
    }
    pub(super) fn maintain_images(
        &self,
        operation: CoverCacheOperation,
        cancel: &Cancellation,
    ) -> Result<CoverCacheReport> {
        let mut report = CoverCacheReport::default();
        if cancel.is_cancelled() {
            report.cancelled = true;
            return Ok(report);
        }
        let Some(_root) = self.maintenance_root()? else {
            report.inventory = Some(CoverCacheInventory::default());
            return Ok(report);
        };
        let _writer = if operation == CoverCacheOperation::Clear {
            loop {
                if cancel.is_cancelled() {
                    report.cancelled = true;
                    return Ok(report);
                }
                if let Some(guard) = self.try_writer()? {
                    break Some(guard);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        } else {
            None
        };
        let mut inventory = CoverCacheInventory::default();
        for entry in fs::read_dir(&self.root).map_err(maintenance_error)? {
            if cancel.is_cancelled() {
                report.cancelled = true;
                return Ok(report);
            }
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    report.failed_files += 1;
                    continue;
                }
            };
            let bytes = match owned_image(&entry) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => {
                    inventory.preserved += 1;
                    continue;
                }
                Err(_) => {
                    inventory.preserved += 1;
                    report.failed_files += 1;
                    continue;
                }
            };
            if operation == CoverCacheOperation::Clear {
                match fs::remove_file(entry.path()) {
                    Ok(()) => {
                        report.removed_files += 1;
                        report.removed_bytes += bytes;
                        continue;
                    }
                    Err(_) => report.failed_files += 1,
                }
            }
            inventory.files += 1;
            inventory.bytes += bytes;
        }
        report.inventory = Some(inventory);
        Ok(report)
    }
}
fn owned_image(entry: &fs::DirEntry) -> std::io::Result<Option<u64>> {
    if !entry.file_type()?.is_file() {
        return Ok(None);
    }
    let name = entry.file_name();
    let Some(name) = name.to_str() else {
        return Ok(None);
    };
    if name.len() != 69
        || !name.is_ascii()
        || !name.ends_with(".rgba")
        || !name[..64].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Ok(None);
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000);
    }
    let mut file = options.open(entry.path())?;
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || is_reparse(&metadata)
        || !(HEADER as u64..=(HEADER + MAX_PIXELS) as u64).contains(&metadata.len())
    {
        return Ok(None);
    }
    let mut header = [0u8; HEADER];
    file.read_exact(&mut header)?;
    if &header[..8] != MAGIC {
        return Ok(None);
    }
    let identity = header[8..40]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if identity != name[..64].to_ascii_lowercase() {
        return Ok(None);
    }
    let width = u32::from_le_bytes(header[40..44].try_into().unwrap());
    let height = u32::from_le_bytes(header[44..48].try_into().unwrap());
    if !(1..=512).contains(&width)
        || !(1..=512).contains(&height)
        || metadata.len() != HEADER as u64 + u64::from(width) * u64::from(height) * 4
    {
        return Ok(None);
    }
    Ok(Some(metadata.len()))
}

fn is_reparse(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}
