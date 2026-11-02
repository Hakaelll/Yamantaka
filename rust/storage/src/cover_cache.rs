use crate::database::storage;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use yamantaka_domain::{
    Cancellation, Failure, RenderKey, RenderedPage, Result,
    cover_cache::{CoverCacheRequest, CoverImageCache},
    memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
};

mod maintenance;
#[cfg(test)]
mod maintenance_test;

const MAGIC: &[u8; 8] = b"YMCOVER1";
const HEADER: usize = 80;
const MAX_PIXELS: usize = 512 * 512 * 4;
const MAX_FILES: usize = 4096;

fn pixel_digest(width: u32, height: u32, pixels: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(width.to_le_bytes());
    hash.update(height.to_le_bytes());
    hash.update(pixels);
    hash.finalize().into()
}

pub struct CoverCache {
    root: PathBuf,
    limit: u64,
    writer: std::sync::Mutex<()>,
    checked_quota: std::sync::atomic::AtomicBool,
}
impl CoverCache {
    pub fn new(root: PathBuf, limit: u64) -> Self {
        Self {
            root,
            limit: limit.min(1024 * 1024 * 1024),
            writer: Default::default(),
            checked_quota: Default::default(),
        }
    }
    fn path(&self, identity: [u8; 32]) -> PathBuf {
        let name: String = identity.iter().map(|byte| format!("{byte:02x}")).collect();
        self.root.join(format!("{name}.rgba"))
    }
    fn trim(&self, incoming: u64, cancel: &Cancellation) -> Result<()> {
        loop {
            cancel.check()?;
            let mut total = incoming;
            let mut count = 0usize;
            let mut oldest = None;
            for entry in fs::read_dir(&self.root).map_err(storage)? {
                cancel.check()?;
                let entry = entry.map_err(storage)?;
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.len() != 69
                    || !name.is_ascii()
                    || !name.ends_with(".rgba")
                    || !name[..64].bytes().all(|b| b.is_ascii_hexdigit())
                {
                    continue;
                }
                if !entry.file_type().map_err(storage)?.is_file() {
                    continue;
                }
                let metadata = entry.metadata().map_err(storage)?;
                count += 1;
                total = total.saturating_add(metadata.len());
                let modified = metadata.modified().map_err(storage)?;
                if oldest.as_ref().is_none_or(|(time, _)| modified < *time) {
                    oldest = Some((modified, entry.path()));
                }
            }
            if total <= self.limit && count.saturating_add(usize::from(incoming > 0)) <= MAX_FILES {
                return Ok(());
            }
            let Some((_, path)) = oldest else {
                return Err(Failure::Invalid("Cover exceeds cache limit".into()));
            };
            fs::remove_file(path).map_err(storage)?;
        }
    }
}
impl CoverImageCache for CoverCache {
    fn maintain(
        &self,
        operation: yamantaka_domain::cover_cache::CoverCacheOperation,
        cancel: &Cancellation,
    ) -> Result<yamantaka_domain::cover_cache::CoverCacheReport> {
        self.maintain_images(operation, cancel)
    }

    fn identity(&self, path: &Path, page: u32, cancel: &Cancellation) -> Result<[u8; 32]> {
        cancel.check()?;
        let canonical = path.canonicalize().map_err(storage)?;
        let mut file = File::open(&canonical).map_err(storage)?;
        let before = file.metadata().map_err(storage)?;
        let physical = yamantaka_windows::source_stamp::Stamp::from_file(&file)
            .ok_or_else(|| Failure::Storage("Cover source attributes could not be read".into()))?;
        let modified = before
            .modified()
            .map_err(storage)?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(storage)?;
        let mut hash = Sha256::new();
        hash.update(MAGIC);
        hash.update(canonical.as_os_str().as_encoded_bytes());
        hash.update(before.len().to_le_bytes());
        hash.update(modified.as_nanos().to_le_bytes());
        hash.update(page.to_le_bytes());
        hash.update(physical.token());
        // Match the bounded source fingerprint convention without allocating a PDF copy.
        let mut remaining = 64 * 1024;
        let mut buffer = [0u8; 4096];
        while remaining > 0 {
            cancel.check()?;
            let amount = remaining.min(buffer.len());
            let read = file.read(&mut buffer[..amount]).map_err(storage)?;
            if read == 0 {
                break;
            }
            hash.update(&buffer[..read]);
            remaining -= read;
        }
        let after = file.metadata().map_err(storage)?;
        if before.len() != after.len()
            || before.modified().ok() != after.modified().ok()
            || yamantaka_windows::source_stamp::Stamp::from_file(&file).as_ref() != Some(&physical)
            || yamantaka_windows::source_stamp::Stamp::read(&canonical).as_ref() != Some(&physical)
        {
            return Err(Failure::Invalid(
                "PDF changed during cache validation".into(),
            ));
        }
        cancel.check()?;
        Ok(hash.finalize().into())
    }
    fn load(
        &self,
        request: CoverCacheRequest<'_>,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Option<Arc<RenderedPage>>> {
        cancel.check()?;
        if !self
            .checked_quota
            .load(std::sync::atomic::Ordering::Acquire)
            && self.root.is_dir()
        {
            let Some(_guard) = self.try_writer()? else {
                return Ok(None);
            };
            if !self
                .checked_quota
                .load(std::sync::atomic::Ordering::Acquire)
            {
                self.trim(0, cancel)?;
                self.checked_quota
                    .store(true, std::sync::atomic::Ordering::Release);
            }
        }
        let path = self.path(request.identity);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(storage(error)),
        };
        if fs::symlink_metadata(&path)
            .map_err(storage)?
            .file_type()
            .is_symlink()
        {
            return Ok(None);
        }
        let mut header = [0u8; HEADER];
        if file.read_exact(&mut header).is_err()
            || &header[..8] != MAGIC
            || header[8..40] != request.identity
        {
            return Ok(None);
        }
        let width = u32::from_le_bytes(header[40..44].try_into().unwrap());
        let height = u32::from_le_bytes(header[44..48].try_into().unwrap());
        if !(1..=512).contains(&width) || !(1..=512).contains(&height) {
            return Ok(None);
        }
        let size = width as usize * height as usize * 4;
        if file.metadata().map_err(storage)?.len() != (HEADER + size) as u64 {
            return Ok(None);
        }
        let mut pixels = TrackedBytesBuilder::try_zeroed(governor, MemoryCategory::Pages, size)
            .map_err(|_| Failure::MemoryBudget)?;
        for chunk in pixels.as_mut_slice().chunks_mut(64 * 1024) {
            cancel.check()?;
            if file.read_exact(chunk).is_err() {
                return Ok(None);
            }
        }
        if pixel_digest(width, height, pixels.as_mut_slice())[..] != header[48..80] {
            return Ok(None);
        }
        cancel.check()?;
        let key = RenderKey {
            document_id: request.document.into(),
            page: request.page,
            region: None,
            physical_width: width,
            physical_height: height,
            rotation: 0,
            generation: request.generation,
        };
        Ok(Some(Arc::new(RenderedPage::from_pixels(
            key,
            pixels.finish(),
        )?)))
    }
    fn save(&self, identity: [u8; 32], image: &RenderedPage, cancel: &Cancellation) -> Result<()> {
        cancel.check()?;
        if image.width == 0
            || image.height == 0
            || image.width > 512
            || image.height > 512
            || image.key.region.is_some()
            || image.key.rotation != 0
            || image.pixels().len() > MAX_PIXELS
            || image.pixels().len() != image.width as usize * image.height as usize * 4
            || image.key.output_dimensions()? != [image.width, image.height]
        {
            return Err(Failure::Invalid("Invalid cached cover dimensions".into()));
        }
        if self.limit < (HEADER + image.pixels().len()) as u64 {
            return Ok(());
        }
        let Some(_guard) = self.try_writer()? else {
            return Ok(());
        };
        fs::create_dir_all(&self.root).map_err(storage)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.root).map_err(storage)?;
        temporary.write_all(MAGIC).map_err(storage)?;
        temporary.write_all(&identity).map_err(storage)?;
        temporary
            .write_all(&image.width.to_le_bytes())
            .map_err(storage)?;
        temporary
            .write_all(&image.height.to_le_bytes())
            .map_err(storage)?;
        temporary
            .write_all(&pixel_digest(image.width, image.height, image.pixels()))
            .map_err(storage)?;
        for chunk in image.pixels().chunks(64 * 1024) {
            cancel.check()?;
            temporary.write_all(chunk).map_err(storage)?;
        }
        temporary.as_file().sync_all().map_err(storage)?;
        self.trim((HEADER + image.pixels().len()) as u64, cancel)?;
        cancel.check()?;
        temporary.persist(self.path(identity)).map_err(storage)?;
        Ok(())
    }
}
