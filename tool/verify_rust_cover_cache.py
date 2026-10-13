"""Verify synthetic library captures and the actual bounded cover-cache records."""
import argparse
import hashlib
import struct
from pathlib import Path
from verify_rust_cover_picker import digest, pixel_chunks, read_json


def verify_cache_records(cache, expected_count):
    records = list(cache.glob("*.rgba"))
    assert len(records) == expected_count, len(records)
    for path in records:
        data = path.read_bytes()
        assert data[:8] == b"YMCOVER1" and len(data) >= 80
        assert path.stem == data[8:40].hex()
        width, height = struct.unpack_from("<II", data, 40)
        assert 1 <= width <= 512 and 1 <= height <= 512
        assert len(data) == 80 + width * height * 4
        assert hashlib.sha256(data[40:48] + data[80:]).digest() == data[48:80]


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
        assert digest(root / source["path"]) == source["sha256"], source["path"]
    assert {item["name"] for item in manifest["captures"]} == {"pdf-covers-dark", "pdf-saved-cover-page"}
    published = read_json(args.published / "capture-manifest.json") if args.published else None
    for item in manifest["captures"]:
        assert item["exitCode"] == 0
        assert digest(args.source / item["image"]) == item["imageSha256"]
        assert digest(args.source / item["controls"]) == item["controlsSha256"]
        record = read_json(args.source / item["controls"])
        assert record["synthetic"] and record["pagePresenter"] == "native-wgpu"
        covers = record["libraryCovers"]
        expected_count = 5 if item["name"] == "pdf-saved-cover-page" else 10
        assert covers["ready"] == expected_count and covers["pending"] == 0 and covers["resident"] == expected_count
        assert set(covers["sourcePages"]) == {f"demo-{index}" for index in range(expected_count)}
        assert covers["sourcePages"]["demo-0"] == (1 if item["name"] == "pdf-saved-cover-page" else 0)
        cache = root / "build/demo-store/captures" / str(record["captureProcessId"]) / "cover-cache-v1"
        verify_cache_records(cache, expected_count)
        if published:
            published_item = next(c for c in published["captures"] if c["name"] == item["name"])
            assert digest(args.published / item["image"]) == published_item["publishedImageSha256"]
            assert digest(args.published / item["controls"]) == item["controlsSha256"]
            assert pixel_chunks(args.source / item["image"]) == pixel_chunks(args.published / item["image"])
        print(f"PASS {item['name']}: {expected_count} native covers; {expected_count} valid persistent records")
    print(f"PASS {len(manifest['sources'])} UI source hashes; executable, lockfile and capture hashes")


if __name__ == "__main__":
    main()
