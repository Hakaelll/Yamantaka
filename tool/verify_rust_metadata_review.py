"""Verify native editor captures against their isolated SQLite stores."""
import argparse
import hashlib
import json
import sqlite3
from pathlib import Path


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def verify(directory, source_hash_file):
    root = Path(__file__).resolve().parent.parent
    manifest = read_json(directory / "capture-manifest.json")
    expected_names = {"metadata-review-" + suffix for suffix in
                      ("dark", "light-es", "small-200", "saved", "dismissed", "reset", "failed")}
    assert manifest["synthetic"] and {case["name"] for case in manifest["captures"]} == expected_names
    for source in manifest["sources"]:
        assert digest(root / source["path"]) == source["sha256"].upper(), "Capture source changed"
    expected_source = source_hash_file.read_text(encoding="utf-8-sig").strip().upper()
    for case in manifest["captures"]:
        for key, hash_key in (("image", "imageSha256"), ("controls", "controlsSha256")):
            assert digest(directory / case[key]) == case[hash_key].upper()
        controls = read_json(directory / case["controls"])
        review = controls["metadataReview"]
        assert controls["synthetic"] and review["document"] == "demo-0" and not review["pending"]
        for control in controls["controls"]:
            left, _, right, _ = control["rect"]
            clip_left, _, clip_right, _ = control["clip"]
            assert left >= clip_left - 1 and right <= clip_right + 1, (case["name"], control["label"])
        assert any(c["enabled"] and c["label"] in ("Save changes", "Guardar cambios") for c in controls["controls"])
        database = root / "build/demo-store/captures" / str(controls["captureProcessId"]) / "export-preview.sqlite3"
        with sqlite3.connect(database.resolve().as_uri() + "?mode=ro", uri=True) as db:
            db.row_factory = sqlite3.Row
            row = dict(db.execute("SELECT path,title,author,source_title,source_author,title_override,author_override,fingerprint,reading_progress,last_page_index FROM documents WHERE id='demo-0'").fetchone())
            analysis = dict(db.execute("SELECT status,suggested_title,suggested_author FROM document_enrichment WHERE document_id='demo-0'").fetchone())
            assert db.execute("SELECT count(*) FROM library_search").fetchone()[0] == 0
        assert digest(Path(row.pop("path"))) == expected_source
        overrides = ("Geometry of Light", "Ada Lovelace") if case["name"].endswith("-saved") else (None, None)
        assert row == {"title": "On the Nature of Things", "author": "Lucretius", "source_title": "Untitled", "source_author": "Unknown", "title_override": overrides[0], "author_override": overrides[1], "fingerprint": "synthetic", "reading_progress": 0.5, "last_page_index": 0}
        assert review["snapshot"]["titleOverride"] == overrides[0]
        assert review["snapshot"]["authorOverride"] == overrides[1]
        completed = case["name"].endswith(("-saved", "-dismissed"))
        assert analysis == {"status": "completed" if completed else "review", "suggested_title": None if completed else "Geometry of Light", "suggested_author": None if completed else "Ada Lovelace"}
        failed = case["name"].endswith("-failed")
        assert bool(review["error"]) == failed
        assert review["saved"] == case["name"].endswith(("-saved", "-dismissed", "-reset"))
        if failed:
            assert review["title"] == "Geometry of Light" and review["author"] == "Ada Lovelace"
            assert review["snapshot"]["title"] == "On the Nature of Things"
    print("Verified seven native metadata states, horizontal control bounds, independent SQLite results, opt-in search, and unchanged PDF source.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--source-hash-file", type=Path, required=True)
    args = parser.parse_args()
    verify(args.directory, args.source_hash_file)
