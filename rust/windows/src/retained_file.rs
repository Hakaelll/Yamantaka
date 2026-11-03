use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    io,
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::AsRawHandle},
    path::Path,
};
use windows::{
    Wdk::Storage::FileSystem::{
        FILE_RENAME_INFORMATION, FileRenameInformation, NtSetInformationFile,
    },
    Win32::{
        Foundation::HANDLE,
        Storage::FileSystem::{
            DELETE, FILE_ATTRIBUTE_REPARSE_POINT, FILE_DISPOSITION_INFO,
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_GENERIC_READ,
            FILE_SHARE_READ, FILE_SHARE_WRITE, FileDispositionInfo, SetFileInformationByHandle,
        },
        System::IO::IO_STATUS_BLOCK,
    },
};

/// Holds the exact file and parent through a rename without replacing another file.
pub struct RetainedFile {
    file: File,
    _parent: File,
}
impl RetainedFile {
    pub fn open(path: &Path) -> io::Result<Self> {
        use std::os::windows::fs::MetadataExt;
        let parent = OpenOptions::new()
            .read(true)
            .share_mode((FILE_SHARE_READ | FILE_SHARE_WRITE).0)
            .custom_flags((FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT).0)
            .open(
                path.parent()
                    .ok_or_else(|| io::Error::other("Missing record parent"))?,
            )?;
        let file = OpenOptions::new()
            .access_mode(FILE_GENERIC_READ.0 | DELETE.0)
            .share_mode(FILE_SHARE_READ.0)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0)
            .open(path)?;
        let metadata = file.metadata()?;
        if parent.metadata()?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0
            || !parent.metadata()?.is_dir()
            || !metadata.is_file()
            || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0
        {
            return Err(io::Error::other("The record must be an ordinary file"));
        }
        Ok(Self {
            file,
            _parent: parent,
        })
    }
    pub fn file(&self) -> &File {
        &self.file
    }
    pub fn remove(self) -> io::Result<()> {
        let info = FILE_DISPOSITION_INFO { DeleteFile: true };
        // Mark only the held file for deletion; dropping its handle completes removal.
        unsafe {
            SetFileInformationByHandle(
                HANDLE(self.file.as_raw_handle()),
                FileDispositionInfo,
                (&info as *const FILE_DISPOSITION_INFO).cast(),
                std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
            .map_err(io::Error::other)
        }
    }
    pub fn retain_as(&self, name: &OsStr) -> io::Result<()> {
        let wide: Vec<_> = name.encode_wide().collect();
        if wide.is_empty()
            || wide.len() > 255
            || wide
                .iter()
                .any(|&c| [0, b'/' as u16, b'\\' as u16, b':' as u16].contains(&c))
            || name == "."
            || name == ".."
        {
            return Err(io::Error::other("Choose a single record filename"));
        }
        let offset = std::mem::offset_of!(FILE_RENAME_INFORMATION, FileName);
        let size = (offset + wide.len() * 2).max(std::mem::size_of::<FILE_RENAME_INFORMATION>());
        let mut buffer = vec![0usize; size.div_ceil(std::mem::size_of::<usize>())];
        // The aligned buffer holds the fixed header and its variable UTF-16 tail.
        unsafe {
            let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
            info.write(FILE_RENAME_INFORMATION::default());
            (*info).FileNameLength = (wide.len() * 2) as u32;
            std::ptr::copy_nonoverlapping(
                wide.as_ptr(),
                std::ptr::addr_of_mut!((*info).FileName).cast::<u16>(),
                wide.len(),
            );
            // A simple name and null root rename within the source's actual parent.
            NtSetInformationFile(
                HANDLE(self.file.as_raw_handle()),
                &mut IO_STATUS_BLOCK::default(),
                info.cast(),
                size as u32,
                FileRenameInformation,
            )
            .ok()
            .map_err(io::Error::other)
        }
    }
}
