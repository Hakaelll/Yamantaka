"""Verify native restoration captures, exact stored decisions and cache records."""
import argparse
import sqlite3
from pathlib import Path
from verify_rust_cover_picker import digest, pixel_chunks, read_json
from verify_rust_cover_cache import verify_cache_records


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("executable", type=Path)
    parser.add_argument("--published", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    manifest = read_json(args.source / "capture-manifest.json")
    assert manifest["synthetic"] and digest(args.executable) == manifest["executableSha256"]
    assert digest(root / "Cargo.lock") == manifest["cargoLockSha256"]
    assert {source["path"] for source in manifest["sources"]} == {p.relative_to(root).as_posix() for p in (root / "rust/ui/src").rglob("*.rs")}
    for source in manifest["sources"]:
        assert digest(root / source["path"]) == source["sha256"]
    assert {item["name"] for item in manifest["captures"]} == {"cover-restore-controls", "cover-restored-dark", "cover-restored-light-es", "cover-restore-failed", "cover-restore-small-200"}
    assert digest(root / "build/rust-fixtures/automatic-cover.pdf") == "996C82E1DEB83AFD8C13DE7834FAE7603E4F9FE23109DC08E4A5B781C1047609"
    published = read_json(args.published / "capture-manifest.json") if args.published else None
    for item in manifest["captures"]:
        name = item["name"]
        restored = name.startswith("cover-restored-")
        failed = name == "cover-restore-failed"
        assert item["exitCode"] == 0
        assert digest(args.source / item["image"]) == item["imageSha256"]
        assert digest(args.source / item["controls"]) == item["controlsSha256"]
        record = read_json(args.source / item["controls"])
        assert record["synthetic"] and record["pagePresenter"] == "native-wgpu"
        assert record["coverPickerDemo"] == {"action": "restore" if restored else "failed-restore" if failed else "open", "complete": True, "saved": restored}
        picker = record["coverPicker"]
        if restored:
            assert picker == {"open": False}
            assert record["libraryCovers"]["sourcePages"] == {f"demo-{index}": 1 if index == 0 else 0 for index in range(5)}
            assert (record["libraryCovers"]["resident"], record["libraryCovers"]["ready"], record["libraryCovers"]["pending"]) == (5, 5, 0)
        else:
            assert picker == {"open": True, "document": "demo-0", "page": 1, "previewPage": 1, "ready": not failed, "saving": False, "error": failed}
            for label, enabled in (("Use as cover", not failed), ("Cancel", True)):
                control = next(c for c in record["controls"] if c["label"] == label)
                assert control["enabled"] == enabled
                rect, clip = control["rect"], control["clip"]
                assert rect[0] >= clip[0] - 1 and rect[1] >= clip[1] - 1 and rect[2] <= clip[2] + 1 and rect[3] <= clip[3] + 1
            automatic = next(c for c in record["controls"] if c["label"] == "Restore automatic cover")
            assert automatic["enabled"] == (not failed)
            if name != "cover-restore-small-200":
                assert automatic["rect"][3] <= automatic["clip"][3] + 1
            if failed:
                assert next(c for c in record["controls"] if c["label"] == "Retry")["enabled"]
        store = root / "build/demo-store/captures" / str(record["captureProcessId"])
        with sqlite3.connect((store / "export-preview.sqlite3").as_uri() + "?mode=ro", uri=True) as connection:
            row = connection.execute("SELECT source_page_index,selection_mode,algorithm_version,confidence,crop_mode,small_thumbnail_path,large_thumbnail_path,generated_at,source_modified_at,page_aspect_ratio FROM document_covers WHERE document_id='demo-0'").fetchone()
            if restored:
                assert row[:3] == (1, "automatic", 3) and 0.45 <= row[3] <= 1
                assert row[4:7] == ("contain", "", "") and row[8] == "2026-10-08"
            else:
                assert row == (0, "manual", 4, 1.0, "contain", "", "", "2026-10-13", "2026-10-08", 0.7142857142857143)
            assert connection.execute("SELECT page_count,last_page_index,reading_progress FROM documents WHERE id='demo-0'").fetchone() == (7, 3, 4 / 7)
        verify_cache_records(store / "cover-cache-v1", 5 if restored else 1)
        if published:
            published_item = next(c for c in published["captures"] if c["name"] == name)
            assert digest(args.published / item["image"]) == published_item["publishedImageSha256"]
            assert digest(args.published / item["controls"]) == item["controlsSha256"]
            assert pixel_chunks(args.source / item["image"]) == pixel_chunks(args.published / item["image"])
        print(f"PASS {name}: stored decision, progress, control state and cache records")
    print(f"PASS {len(manifest['sources'])} UI source hashes; executable, lockfile, fixture and capture hashes")


if __name__ == "__main__":
    main()
