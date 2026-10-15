"""Verify native reader captures and actual progressive-render submission traces."""

import argparse
from pathlib import Path
from verify_rust_cover_picker import digest, pixel_chunks, read_json


CASES = {
    "reader-dark", "reader-text-light-es", "reader-facing", "reader-continuous",
    "reader-sepia-rotated", "reader-small-200",
}


def verify_render_events(control):
    assert control["synthetic"] is True
    assert control["pagePresenter"] == "native-wgpu"
    assert control["readerOpening"]["pending"] is None
    events = control["readerRenderEvents"]
    assert 0 < len(events) < 128, "Missing or truncated render trace"
    pages = {}
    for event in events:
        identity = tuple(event[key] for key in ("document", "page", "generation", "rotation"))
        page = pages.setdefault(identity, {"submitted": [], "installed": []})
        assert event["event"] in page
        assert all(0 < side <= 16384 for side in event["pixels"])
        page[event["event"]].append(event)
        if event["event"] == "installed":
            index = len(page["installed"]) - 1
            assert index < len(page["submitted"]), "Installed an unrequested raster"
            assert event["pixels"] == page["submitted"][index]["pixels"]
            assert event["replacedImage"] == (index > 0)
    for page in pages.values():
        requests, installed = page["submitted"], page["installed"]
        assert len(requests) == len(installed) and 1 <= len(requests) <= 2
        first = requests[0]
        assert max(first["pixels"]) <= 768, "First visible raster exceeds preview cap"
        assert not first["retainedImage"]
        assert requests[-1]["pixels"] == requests[-1]["target"], "Final refinement is missing"
        assert all(item["viewport"] == first["viewport"] for item in requests), "PDF viewport changed during asynchronous loading"
        if len(requests) == 2:
            assert requests[1]["retainedImage"], "Refinement discarded the visible preview"
            assert requests[1]["pixels"] != first["pixels"], "Duplicated the same raster"
    assert control["trackedMemory"]["total"] <= 104 * 1024 * 1024
    assert control["trackedMemory"]["transient"] == 0
    return len(pages), sum(len(page["submitted"]) == 2 for page in pages.values())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("executable", type=Path)
    parser.add_argument("--published", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    manifest = read_json(args.source / "capture-manifest.json")
    assert manifest["synthetic"] is True
    assert {item["name"] for item in manifest["captures"]} == CASES
    assert digest(args.executable) == manifest["executableSha256"]
    assert digest(root / "Cargo.lock") == manifest["cargoLockSha256"]
    assert {item["path"] for item in manifest["sources"]} == {
        path.relative_to(root).as_posix() for path in (root / "rust/ui/src").rglob("*.rs")
    }
    for item in manifest["sources"]:
        assert digest(root / item["path"]) == item["sha256"], item["path"]
    published = read_json(args.published / "capture-manifest.json") if args.published else None
    for item in manifest["captures"]:
        assert item["exitCode"] == 0
        assert digest(args.source / item["image"]) == item["imageSha256"]
        assert digest(args.source / item["controls"]) == item["controlsSha256"]
        pages, refined = verify_render_events(read_json(args.source / item["controls"]))
        if published:
            public = next(record for record in published["captures"] if record["name"] == item["name"])
            assert digest(args.published / item["image"]) == public["publishedImageSha256"]
            assert digest(args.published / item["controls"]) == item["controlsSha256"]
            assert pixel_chunks(args.source / item["image"]) == pixel_chunks(args.published / item["image"])
        print(f"PASS {item['name']}: {pages} visible page(s), {refined} retained-preview refinement(s), stable viewport")
    print(f"PASS {len(manifest['sources'])} UI source hashes; executable, lockfile and native capture hashes")


if __name__ == "__main__":
    main()
