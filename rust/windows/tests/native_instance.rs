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
    verify_visible_source_monitor(directory.path());
}

fn verify_visible_source_monitor(root: &std::path::Path) {
    use std::{fs::FileTimes, io::Write};
    use yamantaka_windows::source_monitor::{FileState, Monitor, Target};
    let path = root.join("文献 con espacios.pdf");
    std::fs::write(&path, b"before").unwrap();
    let modified = path.metadata().unwrap().modified().unwrap();
    let (send, notifications) = mpsc::sync_channel(16);
    let monitor = Monitor::start(move || {
        let _ = send.try_send(());
    })
    .unwrap();
    let target = Target {
        document: "registered".into(),
        path: path.clone(),
        generation: 1,
    };
    monitor.set_targets(vec![target.clone()]).unwrap();
    let next = || {
        notifications.recv_timeout(Duration::from_secs(5)).unwrap();
        monitor.take().unwrap()
    };
    let first = next();
    assert!(matches!(first[0].state, FileState::Present(_)));
    assert!(monitor.set_targets(vec![target.clone(); 17]).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"before");
    {
        let mut file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.write_all(b"after!").unwrap();
        file.set_times(FileTimes::new().set_modified(modified))
            .unwrap();
    }
    assert_eq!(path.metadata().unwrap().modified().unwrap(), modified);
    assert_eq!(path.metadata().unwrap().len(), 6);
    let changed = next();
    assert!(matches!(changed[0].state, FileState::Present(_)));
    assert_ne!(first[0].state, changed[0].state);
    assert!(
        notifications
            .recv_timeout(Duration::from_millis(1200))
            .is_err(),
        "Unchanged source kept waking the UI"
    );
    std::fs::remove_file(&path).unwrap();
    assert_eq!(next()[0].state, FileState::Unavailable);
    std::fs::write(&path, b"restored").unwrap();
    assert!(matches!(next()[0].state, FileState::Present(_)));
    let mut newer = target;
    newer.generation = 2;
    monitor.set_targets(vec![newer]).unwrap();
    assert_eq!(next()[0].target.generation, 2);
    monitor.set_targets(Vec::new()).unwrap();
    assert!(next().is_empty());
    std::fs::write(&path, b"unobserved").unwrap();
    assert!(
        notifications
            .recv_timeout(Duration::from_millis(1200))
            .is_err(),
        "Inactive observer did not sleep"
    );
    drop(monitor);
}
