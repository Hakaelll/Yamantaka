//! Native Windows integration with a Rust-only application identity.
pub mod annotation_export;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use windows::{
    Win32::{
        Foundation::{
            CloseHandle, ERROR_ALREADY_EXISTS, ERROR_NO_MORE_FILES, GetLastError, HANDLE,
        },
        System::{
            Com::CoTaskMemFree,
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
                TH32CS_SNAPPROCESS,
            },
            Threading::CreateMutexW,
        },
        UI::{
            Shell::{
                FOLDERID_LocalAppData, FOLDERID_RoamingAppData, KF_FLAG_DEFAULT,
                SHGetKnownFolderPath, ShellExecuteW,
            },
            WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW, SW_SHOWNORMAL},
        },
    },
    core::{HSTRING, PCWSTR},
};
use yamantaka_domain::{Failure, Result};

/// Startup errors must be visible even for a GUI release with no console.
pub fn show_startup_error(message: &str) {
    let message = HSTRING::from(message);
    let title = HSTRING::from("Yamantaka");
    // Both UTF-16 strings outlive the synchronous native dialog call. It owns
    // neither pointer and receives no passwords, document text or logging state.
    unsafe {
        MessageBoxW(None, &message, &title, MB_OK | MB_ICONERROR);
    }
}

fn platform_error(_: impl std::fmt::Display) -> Failure {
    Failure::Storage("Windows operation failed".into())
}
fn known_folder(id: &windows::core::GUID) -> Result<PathBuf> {
    // The Shell allocates a UTF-16 path; free it once the owned Rust path exists.
    unsafe {
        let pointer = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).map_err(platform_error)?;
        let result = pointer
            .to_string()
            .map(PathBuf::from)
            .map_err(platform_error);
        CoTaskMemFree(Some(pointer.0.cast()));
        result
    }
}
pub struct AppPaths {
    pub store: PathBuf,
    pub cache: PathBuf,
    pub activations: PathBuf,
    pub legacy_installed: PathBuf,
    pub portable: bool,
}
impl AppPaths {
    pub fn discover(executable: &Path) -> Result<Self> {
        let parent = executable
            .parent()
            .ok_or_else(|| Failure::Invalid("Executable directory missing".into()))?;
        let portable = parent.join(".yamantaka-rust-portable").is_file();
        let roaming = known_folder(&FOLDERID_RoamingAppData)?;
        let local = known_folder(&FOLDERID_LocalAppData)?;
        let store = if portable {
            parent.join("rust-data")
        } else {
            roaming.join("io.yamantaka.rust").join("Yamantaka")
        };
        let cache = if portable {
            parent.join("rust-cache")
        } else {
            local.join("io.yamantaka.rust").join("Yamantaka")
        };
        let activations = cache.join("activations");
        Ok(Self {
            store,
            cache,
            activations,
            legacy_installed: roaming.join("io.yamantaka").join("Yamantaka"),
            portable,
        })
    }
    /// Flutter's portable marker redirects SQLite, not shared_preferences.
    pub fn legacy_portable_database(executable: &Path) -> Option<PathBuf> {
        let parent = executable.parent()?;
        parent
            .join(".yamantaka-portable")
            .exists()
            .then(|| parent.join("data/yamantaka.sqlite3"))
    }
}

pub fn flutter_closed() -> Result<bool> {
    unsafe {
        let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(platform_error)?;
        let guard = NativeHandle(handle);
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut next = Process32FirstW(guard.0, &mut entry);
        loop {
            if let Err(error) = next {
                if error.code() == windows::core::HRESULT::from_win32(ERROR_NO_MORE_FILES.0) {
                    return Ok(true);
                }
                // A failed enumeration is not evidence that Flutter is closed.
                return Err(platform_error(error));
            }
            let length = entry
                .szExeFile
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..length])
                .eq_ignore_ascii_case("yamantaka.exe")
            {
                return Ok(false);
            }
            next = Process32NextW(guard.0, &mut entry);
        }
    }
}

pub fn pick_flutter_database() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("SQLite", &["sqlite3", "sqlite", "db"])
        .pick_file()
}
pub fn pick_flutter_preferences() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .pick_file()
}
pub fn save_library_backup() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_file_name("yamantaka-backup.json")
        .save_file()
}
pub fn save_diagnostics_report() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_file_name("yamantaka-diagnostics.json")
        .save_file()
}
pub fn pick_library_backup() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .pick_file()
}
pub fn show_restore_complete(safety: &Path, english: bool) {
    let message = if english {
        format!(
            "Backup restored. Reopen Yamantaka to use your library.\n\nPrevious library: {}",
            safety.display()
        )
    } else {
        format!(
            "Copia restaurada. Vuelve a abrir Yamantaka para usar tu biblioteca.\n\nBiblioteca anterior: {}",
            safety.display()
        )
    };
    rfd::MessageDialog::new()
        .set_title("Yamantaka")
        .set_description(message)
        .set_level(rfd::MessageLevel::Info)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

pub fn pick_pdf_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

pub fn pick_pdf() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("PDF", &["pdf"])
        .pick_file()
}

pub fn pick_pdfs() -> Option<Vec<PathBuf>> {
    rfd::FileDialog::new()
        .add_filter("PDF", &["pdf"])
        .pick_files()
}
pub fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}
pub fn open_external_uri(uri: &yamantaka_domain::external_uri::ExternalUri) -> Result<()> {
    let operation = HSTRING::from("open");
    let target = HSTRING::from(uri.as_str());
    // The validated URI is the shell target, with no executable or parameters.
    // Owned UTF-16 strings remain alive throughout the synchronous call.
    let result = unsafe {
        ShellExecuteW(
            None,
            &operation,
            &target,
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        return Err(Failure::Storage(
            "Windows could not open the external link".into(),
        ));
    }
    Ok(())
}
pub fn reveal(path: &Path) -> Result<()> {
    let executable = HSTRING::from("explorer.exe");
    let parameters = HSTRING::from(format!("/select,\"{}\"", path.display()));
    let operation = HSTRING::from("open");
    let result = unsafe {
        ShellExecuteW(
            None,
            &operation,
            &executable,
            &parameters,
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        return Err(Failure::Storage("Explorer could not open this path".into()));
    }
    Ok(())
}

struct NativeHandle(HANDLE);
impl Drop for NativeHandle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}
pub struct Instance {
    _mutex: NativeHandle,
    directory: PathBuf,
}
pub enum Activation {
    Primary(Instance),
    Forwarded,
}
impl Instance {
    /// Rust-only per-store mutex. Local files carry Unicode activations without
    /// sockets, URLs or command strings; only the primary opens these paths.
    pub fn acquire(directory: &Path, arguments: &[PathBuf]) -> Result<Activation> {
        fs::create_dir_all(directory).map_err(platform_error)?;
        let canonical = fs::canonicalize(directory).map_err(platform_error)?;
        let digest = format!(
            "{:x}",
            Sha256::digest(canonical.to_string_lossy().to_lowercase().as_bytes())
        );
        let name = HSTRING::from(format!("Local\\YamantakaRust-{digest}"));
        let handle = unsafe { CreateMutexW(None, false, &name) }.map_err(platform_error)?;
        let exists = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        let guard = NativeHandle(handle);
        if exists {
            let name = directory.join(format!(
                "{}-{}.activation",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ));
            let temp = name.with_extension("part");
            fs::write(
                &temp,
                serde_json::to_vec(arguments).map_err(platform_error)?,
            )
            .map_err(platform_error)?;
            fs::rename(temp, name).map_err(platform_error)?;
            drop(guard);
            return Ok(Activation::Forwarded);
        }
        Ok(Activation::Primary(Self {
            _mutex: guard,
            directory: directory.to_owned(),
        }))
    }
    pub fn watch(&self, notify: impl Fn(Vec<PathBuf>) + Send + 'static) -> Result<ActivationWatch> {
        let directory = self.directory.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let handle = thread::Builder::new()
            .name("yamantaka-activation".into())
            .spawn(move || {
                while !flag.load(Ordering::Acquire) {
                    if let Ok(entries) = fs::read_dir(&directory) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.extension().is_none_or(|value| value != "activation") {
                                continue;
                            }
                            if let Ok(metadata) = fs::metadata(&path)
                                && metadata.len() > 1024 * 1024
                            {
                                continue;
                            }
                            if let Ok(data) = fs::read(&path)
                                && let Ok(paths) = serde_json::from_slice::<Vec<PathBuf>>(&data)
                            {
                                notify(paths);
                            }
                            let _ = fs::remove_file(path);
                        }
                    }
                    thread::sleep(Duration::from_millis(200));
                }
            })
            .map_err(platform_error)?;
        Ok(ActivationWatch {
            stop,
            handle: Some(handle),
        })
    }
}
pub struct ActivationWatch {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}
impl Drop for ActivationWatch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
