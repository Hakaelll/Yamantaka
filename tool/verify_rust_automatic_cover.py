"""Verify connected automatic cover captures, persisted choices and cache records."""
import argparse
import sqlite3
from pathlib import Path
from verify_rust_cover_picker import digest, pixel_chunks, read_json
from verify_rust_cover_cache import verify_cache_records
from PIL import Image


def verify_selected_artwork(path):
    with Image.open(path) as image:
        image = image.convert("RGB")
        width, height = image.size
        patch = image.crop((int(width * 0.08), int(height * 0.45), int(width * 0.15), int(height * 0.60)))
        pixels = patch.tobytes()
        purple = sum(20 < red < 120 and green < red * 0.65 and blue > green * 1.3
                     for red, green, blue in zip(pixels[0::3], pixels[1::3], pixels[2::3]))
        assert purple / (patch.width * patch.height) > 0.65, "Selected synthetic cover artwork is missing from the rendered book"


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
    names = {item["name"] for item in manifest["captures"]}
    assert names in (
        {"automatic-cover-dark", "automatic-cover-light-es", "automatic-cover-manual"},
        {"cover-refresh-version", "cover-refresh-source", "cover-refresh-manual"},
    )
    assert digest(root / "build/rust-fixtures/automatic-cover.pdf") == "996C82E1DEB83AFD8C13DE7834FAE7603E4F9FE23109DC08E4A5B781C1047609"
    published = read_json(args.published / "capture-manifest.json") if args.published else None
    for item in manifest["captures"]:
        assert item["exitCode"] == 0
        assert digest(args.source / item["image"]) == item["imageSha256"]
        assert digest(args.source / item["controls"]) == item["controlsSha256"]
        record = read_json(args.source / item["controls"])
        manual = item["name"] in {"automatic-cover-manual", "cover-refresh-manual"}
        if not manual:
            verify_selected_artwork(args.source / item["image"])
        assert record["synthetic"] and record["pagePresenter"] == "native-wgpu"
        covers = record["libraryCovers"]
        assert (covers["resident"], covers["ready"], covers["pending"]) == (5, 5, 0)
        assert covers["sourcePages"] == {f"demo-{index}": (0 if manual or index else 1) for index in range(5)}
        store = root / "build/demo-store/captures" / str(record["captureProcessId"])
        with sqlite3.connect((store / "export-preview.sqlite3").as_uri() + "?mode=ro", uri=True) as connection:
            rows = connection.execute("SELECT document_id,source_page_index,selection_mode,algorithm_version,confidence,source_modified_at,generated_at FROM document_covers ORDER BY document_id").fetchall()
            assert len(rows) == 5
            for index, (document, page, mode, version, confidence, source_modified, generated) in enumerate(rows):
                assert document == f"demo-{index}"
                assert source_modified == "2026-10-08"
                if index == 0 and manual:
                    assert (page, mode, version, confidence) == (0, "manual", 4, 1.0)
                    assert generated == "2026-10-13"
                else:
                    assert (page, mode, version) == (1 if index == 0 else 0, "automatic", 3)
                    assert 0 <= confidence <= 1 and (index != 0 or confidence >= 0.45)
                    assert generated != "2026-10-13"
            count, last, progress = connection.execute("SELECT page_count,last_page_index,reading_progress FROM documents WHERE id='demo-0'").fetchone()
            assert (count, last, progress) == (7, 3, 4 / 7)
            if manual:
                assert connection.execute("SELECT * FROM document_covers WHERE document_id='demo-0'").fetchone() == (
                    "demo-0", 0, "", "", "2026-10-13", "2026-10-08", 1.0,
                    "manual", 4, "contain", 0.7142857142857143,
                )
        verify_cache_records(store / "cover-cache-v1", 5)
        if published:
            published_item = next(c for c in published["captures"] if c["name"] == item["name"])
            assert digest(args.published / item["image"]) == published_item["publishedImageSha256"]
            assert digest(args.published / item["controls"]) == item["controlsSha256"]
            assert pixel_chunks(args.source / item["image"]) == pixel_chunks(args.published / item["image"])
        print(f"PASS {item['name']}: actual stored choices, source pages and five cache records")
    print(f"PASS {len(manifest['sources'])} UI source hashes; binary, lockfile, fixture and capture hashes")


if __name__ == "__main__":
    main()
