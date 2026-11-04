# Recover the Flutter checkpoint

The active product is native Rust for Windows. Flutter, Dart and the earlier
Android, iOS and web scaffolding were retired for 0.16.0 after the owner's final
acceptance on 2026-11-04. Existing installed libraries and applications were not
removed or synchronized with Rust.

The complete pre-cutover source and tools remain in the Git tag
[`archive/flutter-before-rust-0.16.0`](https://github.com/Hakaelll/Yamantaka/tree/archive/flutter-before-rust-0.16.0).
The earlier Flutter baseline is also retained at commit `c773acf`.

Recover a separate checkout without replacing a current Rust library:

```powershell
git clone --branch archive/flutter-before-rust-0.16.0 --single-branch https://github.com/Hakaelll/Yamantaka.git Yamantaka-flutter
```

Historical validation reports retain their original source paths, commands,
results and limitations. Those paths refer to the archived checkout. Runtime
migration adapters and the committed historical SQL remain in the Rust project.
