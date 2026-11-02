//! A file-version hint from one open Windows file; not a full content digest.
use std::{
    fs::OpenOptions,
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::Path,
};
use windows::Win32::{
    Foundation::HANDLE,
    Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_BASIC_INFO,
        FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, FileBasicInfo,
        GetFileInformationByHandle, GetFileInformationByHandleEx,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp {
    identity: (u32, u64),
    size: u64,
    times: [i64; 3],
}
impl Stamp {
    pub fn version(&self) -> yamantaka_domain::source_revision::PhysicalSourceVersion {
        yamantaka_domain::source_revision::PhysicalSourceVersion::from_digest(self.token())
    }
    pub fn read(path: &Path) -> Option<Self> {
        // Attribute-only access does not lock the PDF against editing or deletion.
        let file = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES.0)
            .share_mode((FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE).0)
            .open(path)
            .ok()?;
        Self::from_file(&file)
    }
    pub fn from_file(file: &std::fs::File) -> Option<Self> {
        let handle = HANDLE(file.as_raw_handle());
        let basic = || {
            let mut info = FILE_BASIC_INFO::default();
            unsafe {
                GetFileInformationByHandleEx(
                    handle,
                    FileBasicInfo,
                    (&mut info as *mut FILE_BASIC_INFO).cast(),
                    size_of::<FILE_BASIC_INFO>() as u32,
                )
                .ok()?;
            }
            Some([info.CreationTime, info.LastWriteTime, info.ChangeTime])
        };
        let before = basic()?;
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        unsafe {
            GetFileInformationByHandle(handle, &mut info).ok()?;
        }
        if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0 || basic()? != before {
            return None;
        }
        Some(Self {
            identity: (
                info.dwVolumeSerialNumber,
                (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
            ),
            size: (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow),
            times: before,
        })
    }
    pub fn token(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        hash.update(self.identity.0.to_le_bytes());
        hash.update(self.identity.1.to_le_bytes());
        hash.update(self.size.to_le_bytes());
        for time in self.times {
            hash.update(time.to_le_bytes());
        }
        hash.finalize().into()
    }
}

/// A short read lease held through a source-bound database write.
pub struct Lease {
    file: std::fs::File,
    path: std::path::PathBuf,
    version: yamantaka_domain::source_revision::PhysicalSourceVersion,
}
impl Lease {
    pub fn acquire(
        path: &Path,
        version: yamantaka_domain::source_revision::PhysicalSourceVersion,
    ) -> yamantaka_domain::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ.0)
            .open(path)
            .map_err(|_| yamantaka_domain::Failure::CoverSourceChanged)?;
        let lease = Self {
            file,
            path: path.to_owned(),
            version,
        };
        lease.verify()?;
        Ok(lease)
    }
    pub fn verify(&self) -> yamantaka_domain::Result<()> {
        if Stamp::from_file(&self.file).map(|stamp| stamp.version()) != Some(self.version)
            || Stamp::read(&self.path).map(|stamp| stamp.version()) != Some(self.version)
        {
            return Err(yamantaka_domain::Failure::CoverSourceChanged);
        }
        Ok(())
    }
}
