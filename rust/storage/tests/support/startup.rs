use yamantaka_domain::startup::StartupMode;
use yamantaka_storage::startup::StartupRun;

pub fn verify() {
    let root = tempfile::tempdir().unwrap();
    let marker = root.path().join("rust-startup-v1.json");
    assert!(!StartupRun::begin(root.path(), false).mode().is_safe());
    assert!(!StartupRun::begin(root.path(), false).mode().is_safe());
    assert!(!StartupRun::begin(root.path(), false).mode().is_safe());
    let safe = StartupRun::begin(root.path(), false);
    assert!(matches!(
        safe.mode(),
        StartupMode::RepeatedUnclean { failures: 3 }
    ));
    safe.finish().unwrap();
    let normal = StartupRun::begin(root.path(), false);
    assert!(matches!(normal.mode(), StartupMode::Normal));
    normal.finish().unwrap();
    let manual = StartupRun::begin(root.path(), true);
    assert!(matches!(manual.mode(), StartupMode::Requested));
    manual.finish().unwrap();
    let old = StartupRun::begin(root.path(), false);
    let current = StartupRun::begin(root.path(), false);
    let bytes = std::fs::read(&marker).unwrap();
    assert!(old.finish().is_err());
    assert_eq!(std::fs::read(&marker).unwrap(), bytes);
    current.finish().unwrap();
    for payload in [b"{\"version\":99}".as_slice(), b"malformed".as_slice()] {
        std::fs::write(&marker, payload).unwrap();
        let unknown = StartupRun::begin(root.path(), false);
        assert!(matches!(unknown.mode(), StartupMode::StateUnavailable));
        unknown.finish().unwrap();
        assert_eq!(std::fs::read(&marker).unwrap(), payload);
    }
    std::fs::write(&marker, vec![b'X'; 4097]).unwrap();
    assert!(matches!(
        StartupRun::begin(root.path(), true).mode(),
        StartupMode::StateUnavailable
    ));
    assert_eq!(std::fs::metadata(&marker).unwrap().len(), 4097);
}
