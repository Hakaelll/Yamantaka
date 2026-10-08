use std::{path::PathBuf, process::Command, sync::mpsc, time::Duration};
use yamantaka_windows::{Activation, AppPaths, Instance};

#[test]
fn native_second_process_forwards_unicode_paths_and_does_not_take_primary() {
    const CHILD_STORE: &str = "YAMANTAKA_ACTIVATION_TEST_STORE";
    let paths = vec![
        PathBuf::from("C:/synthetic/文献/مرحبا.pdf"),
        PathBuf::from("C:/synthetic/estudio con espacios.pdf"),
    ];
    if let Some(directory) = std::env::var_os(CHILD_STORE) {
        assert!(matches!(
            Instance::acquire(&PathBuf::from(directory), &paths).unwrap(),
            Activation::Forwarded
        ));
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let Activation::Primary(instance) = Instance::acquire(directory.path(), &[]).unwrap() else {
        panic!("first process must be primary")
    };
    let (sender, receiver) = mpsc::channel();
    let watch = instance
        .watch(move |paths| {
            sender.send(paths).unwrap();
        })
        .unwrap();
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "native_second_process_forwards_unicode_paths_and_does_not_take_primary",
            "--nocapture",
        ])
        .env(CHILD_STORE, directory.path())
        .output()
        .unwrap();
    assert!(child.status.success(), "secondary process failed");
    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(5)).unwrap(),
        paths
    );
    assert!(
        receiver.recv_timeout(Duration::from_millis(400)).is_err(),
        "one activation was delivered twice"
    );
    drop(watch);
    drop(instance);
    assert!(
        std::fs::read_dir(directory.path())
            .unwrap()
            .next()
            .is_none()
    );
    assert!(
        matches!(
            Instance::acquire(directory.path(), &[]).unwrap(),
            Activation::Primary(_)
        ),
        "primary mutex was not released"
    );
}

#[test]
fn portable_identity_uses_its_own_data_and_does_not_reuse_flutter_marker() {
    let directory = tempfile::tempdir().unwrap();
    let executable = directory.path().join("yamantaka-rust.exe");
    std::fs::write(directory.path().join(".yamantaka-portable"), []).unwrap();
    let installed = AppPaths::discover(&executable).unwrap();
    assert!(!installed.portable);
    assert_ne!(installed.store, directory.path().join("data"));
    std::fs::write(directory.path().join(".yamantaka-rust-portable"), []).unwrap();
    let portable = AppPaths::discover(&executable).unwrap();
    assert!(portable.portable);
    assert_eq!(portable.store, directory.path().join("rust-data"));
    assert_eq!(portable.cache, directory.path().join("rust-cache"));
    assert_eq!(
        portable.activations,
        directory.path().join("rust-cache/activations")
    );
    assert_eq!(
        AppPaths::legacy_portable_database(&executable).unwrap(),
        directory.path().join("data/yamantaka.sqlite3")
    );
    assert!(
        !directory.path().join("rust-data").exists(),
        "path discovery unexpectedly created a store"
    );
}
