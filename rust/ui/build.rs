use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn compiler() -> PathBuf {
    if let Some(configured) = env::var_os("RC") {
        return configured.into();
    }
    let program_files =
        env::var_os("ProgramFiles(x86)").expect("Windows SDK location is unavailable");
    let root = PathBuf::from(program_files).join("Windows Kits/10/bin");
    let mut versions = fs::read_dir(&root)
        .expect("Install the Windows SDK resource compiler")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    versions.sort();
    versions
        .into_iter()
        .rev()
        .map(|path| path.join("x64/rc.exe"))
        .find(|path| path.is_file())
        .expect("Windows SDK x64 rc.exe was not found; set RC explicitly")
}
fn main() {
    println!("cargo:rerun-if-env-changed=RC");
    println!("cargo:rerun-if-changed=../../assets/branding/yamantaka.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    assert_eq!(
        env::var("CARGO_CFG_TARGET_ENV").as_deref(),
        Ok("msvc"),
        "Windows branding requires the supported MSVC target"
    );
    let icon = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../assets/branding/yamantaka.ico")
        .canonicalize()
        .expect("Generate application icons before building");
    let icon = icon.to_string_lossy().replace(char::from(92), "/");
    let icon = icon.strip_prefix("//?/").unwrap_or(&icon);
    let icon = if let Some(unc) = icon.strip_prefix("UNC/") {
        format!("//{unc}")
    } else {
        icon.to_owned()
    };
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let numbers = version
        .split('-')
        .next()
        .unwrap()
        .split('.')
        .map(|part| part.parse::<u16>().expect("Invalid package version"))
        .collect::<Vec<_>>();
    let source = format!(
        r#"1 ICON "{icon}"
1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
FILEOS 0x40004
FILETYPE 0x1
BEGIN
 BLOCK "StringFileInfo"
 BEGIN
  BLOCK "040904b0"
  BEGIN
   VALUE "FileDescription", "Yamantaka\0"
   VALUE "ProductName", "Yamantaka\0"
   VALUE "FileVersion", "{version}\0"
   VALUE "ProductVersion", "{version}\0"
   VALUE "InternalName", "yamantaka-rust\0"
   VALUE "OriginalFilename", "yamantaka-rust.exe\0"
  END
 END
 BLOCK "VarFileInfo"
 BEGIN
  VALUE "Translation", 0x409, 1200
 END
END
"#,
        major = numbers[0],
        minor = numbers[1],
        patch = numbers[2]
    );
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let rc = output.join("yamantaka.rc");
    let resource = output.join("yamantaka.res");
    let bytes = [
        vec![0xff, 0xfe],
        source.encode_utf16().flat_map(u16::to_le_bytes).collect(),
    ]
    .concat();
    fs::write(&rc, bytes).expect("Could not write Windows resource source");
    let result = Command::new(compiler())
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg(&rc)
        .output()
        .expect("Could not run the Windows resource compiler");
    assert!(
        result.status.success(),
        "Windows icon embedding failed: {} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    println!(
        "cargo:rustc-link-arg-bin=yamantaka-rust={}",
        resource.display()
    );
}
