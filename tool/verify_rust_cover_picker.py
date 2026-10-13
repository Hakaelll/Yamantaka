"""Verify native cover-picker captures and their isolated SQLite results."""

import argparse
import hashlib
import json
import sqlite3
import struct
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def pixel_chunks(path):
    data = path.read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", path
    offset, result = 8, []
    while offset < len(data):
        length = struct.unpack_from(">I", data, offset)[0]
        kind = data[offset + 4:offset + 8]
        if kind in (b"IHDR", b"PLTE", b"IDAT", b"tRNS"):
            result.append(data[offset:offset + length + 12])
        offset += length + 12
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("executable", type=Path)
    parser.add_argument("--published", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    manifest = read_json(args.source / "capture-manifest.json")
    assert manifest["synthetic"] is True
    assert digest(args.executable) == manifest["executableSha256"]
    assert digest(root / "Cargo.lock") == manifest["cargoLockSha256"]
    inventory = {item["path"] for item in manifest["sources"]}
    assert inventory == {p.relative_to(root).as_posix() for p in (root / "rust/ui/src").rglob("*.rs")}
    for source in manifest["sources"]:
        assert digest(root / source["path"]) == source["sha256"], source["path"]
    expected = {"cover-picker-dark", "cover-picker-light-es", "cover-picker-small-200",
                "cover-picker-saved", "cover-picker-cancelled", "cover-picker-failed-save"}
    assert {item["name"] for item in manifest["captures"]} == expected
    published = read_json(args.published / "capture-manifest.json") if args.published else None
    for item in manifest["captures"]:
        name = item["name"]
        assert item["exitCode"] == 0, name
        assert digest(args.source / item["image"]) == item["imageSha256"], name
        assert digest(args.source / item["controls"]) == item["controlsSha256"], name
        record = read_json(args.source / item["controls"])
        assert record["synthetic"] and record["pagePresenter"] == "native-wgpu", name
        picker, demo = record["coverPicker"], record["coverPickerDemo"]
        assert demo["complete"] and demo["saved"] == (name == "cover-picker-saved"), name
        if name in ("cover-picker-saved", "cover-picker-cancelled"):
            assert picker == {"open": False}, name
            assert record["libraryCovers"]["sourcePages"]["demo-0"] == (1 if demo["saved"] else 0), name
        else:
            failed = name == "cover-picker-failed-save"
            assert picker == {"open": True, "document": "demo-0", "page": 1,
                              "previewPage": 1, "ready": not failed, "saving": False,
                              "error": failed}, name
            use_label = "Usar como portada" if name == "cover-picker-light-es" else "Use as cover"
            cancel_label = "Cancelar" if name == "cover-picker-light-es" else "Cancel"
            for label, enabled in ((use_label, not failed), (cancel_label, True)):
                control = next(c for c in record["controls"] if c["label"] == label)
                assert control["enabled"] == enabled, (name, label)
                rect, clip = control["rect"], control["clip"]
                assert rect[0] >= clip[0] - 1 and rect[1] >= clip[1] - 1, (name, label)
                assert rect[2] <= clip[2] + 1 and rect[3] <= clip[3] + 1, (name, label)
        if name in ("cover-picker-saved", "cover-picker-cancelled", "cover-picker-failed-save"):
            database = root / "build/demo-store/captures" / str(record["captureProcessId"]) / "export-preview.sqlite3"
            with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True) as connection:
                row = connection.execute("SELECT source_page_index,selection_mode,page_aspect_ratio,small_thumbnail_path,large_thumbnail_path FROM document_covers WHERE document_id='demo-0'").fetchone()
                assert row == ((1, "manual", 0.75, "", "") if demo["saved"] else None), (name, row)
                if demo["saved"]:
                    assert connection.execute("SELECT thumbnail_path FROM documents WHERE id='demo-0'").fetchone() == (None,)
        if published:
            published_item = next(c for c in published["captures"] if c["name"] == name)
            assert digest(args.published / item["image"]) == published_item["publishedImageSha256"], name
            assert digest(args.published / item["controls"]) == item["controlsSha256"], name
            assert pixel_chunks(args.published / item["image"]) == pixel_chunks(args.source / item["image"]), name
        print(f"PASS {name}")
    print(f"PASS {len(inventory)} UI source hashes; executable and lockfile; six native cases")


if __name__ == "__main__":
    main()
