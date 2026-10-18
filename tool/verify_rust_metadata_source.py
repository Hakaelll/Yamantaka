"""Verify native metadata evidence navigation against isolated SQLite and PDFs."""

import argparse
import hashlib
import json
import sqlite3
from pathlib import Path

from verify_rust_metadata_review import digest, read_json
from verify_rust_reader_fluidity import verify_render_events


def verify(directory, executable, source_hash_file):
    root = Path(__file__).resolve().parent.parent
    manifest = read_json(directory / "capture-manifest.json")
    names = {"metadata-source-" + suffix for suffix in
             ("dark", "light-es", "small-200", "small-reference-200", "open", "return", "draft", "stale")}
    assert manifest["synthetic"] is True
    assert {case["name"] for case in manifest["captures"]} == names
    assert digest(executable) == manifest["executableSha256"]
    assert digest(root / "Cargo.lock") == manifest["cargoLockSha256"]
    for source in manifest["sources"]:
        assert digest(root / source["path"]) == source["sha256"].upper()
    hashes = {item["File"]: item["Sha256"] for item in read_json(source_hash_file)}
    for case in manifest["captures"]:
        assert case["exitCode"] == 0
        for key, hash_key in (("image", "imageSha256"), ("controls", "controlsSha256")):
            assert digest(directory / case[key]) == case[hash_key].upper()
        controls = read_json(directory / case["controls"])
        review = controls["metadataReview"]
        assert controls["synthetic"] and controls["pagePresenter"] == "native-wgpu"
        assert review["document"] == "demo-0"
        assert review["analyzed"] and not review["pending"] and not review["sourcePending"]
        assert not review["saved"] and not review["reference"]["pending"]
        stale = case["name"].endswith("-stale")
        assert review["reference"]["evidencePage"] == 1
        if stale:
            assert review["reference"]["error"] and review["reference"]["page"] == 0
        else:
            assert not review["reference"]["error"] and review["reference"]["page"] == 1
        draft = case["name"].endswith("-draft")
        opened = case["name"].endswith(("-open", "-return"))
        assert review["sourceOpened"] == opened
        assert bool(review["error"]) == stale
        assert review["title"] == ("My unsaved geometry" if draft else "On the Nature of Things")
        assert review["author"] == "Lucretius"
        for control in controls["controls"]:
            assert control["rect"][0] >= control["clip"][0] - 1, (case["name"], control["label"])
            assert control["rect"][2] <= control["clip"][2] + 1, (case["name"], control["label"])
        if case["name"].endswith("-open"):
            verify_render_events(controls)
            assert controls["currentReaderPage"] == 1
            assert any(event["event"] == "installed" and event["document"] == "demo-0"
                       and event["page"] == 1 for event in controls["readerRenderEvents"])
            assert any(c["enabled"] and c["label"] == "Back to metadata" for c in controls["controls"])
        elif "small" not in case["name"]:
            links = [c for c in controls["controls"] if c["label"] in
                     ("Open page 2 in reader", "Abrir página 2 en el lector")]
            assert len(links) == 1 and links[0]["enabled"] == (not draft and not stale)
        database = root / "build/demo-store/captures" / str(controls["captureProcessId"]) / "export-preview.sqlite3"
        with sqlite3.connect(database.resolve().as_uri() + "?mode=ro", uri=True) as db:
            db.row_factory = sqlite3.Row
            row = dict(db.execute("SELECT path,fingerprint,file_size,file_modified_at,page_count,title,author,title_override,author_override,source_title,source_author,last_page_index,reading_progress FROM documents WHERE id='demo-0'").fetchone())
            analysis = dict(db.execute("SELECT status,suggested_title,suggested_author FROM document_enrichment WHERE document_id='demo-0'").fetchone())
            evidence = json.loads(db.execute("SELECT value FROM app_metadata WHERE key='rust.frontMatterEvidence.v1:demo-0'").fetchone()[0])
            assert db.execute("SELECT count(*) FROM library_search").fetchone()[0] == 0
        path = Path(row["path"])
        assert path.name == "front-matter-evidence-later.pdf"
        assert digest(path) == hashes[path.name].upper()
        data = path.read_bytes()
        fingerprint = hashlib.sha256(f"{len(data)}:{hashlib.sha256(data[:65536]).hexdigest()}".encode()).hexdigest()
        assert row["fingerprint"] == fingerprint
        assert evidence["source"] == {"fingerprint": fingerprint, "size": len(data),
                                      "modified": row["file_modified_at"], "pages": 2}
        assert evidence["version"] == 1 and evidence["analyzerVersion"] == 2 and evidence["hasText"]
        for field, value in (("title", "Geometry of Light"), ("author", "Ada Lovelace")):
            assert evidence[field]["candidate"]["value"] == value
            assert evidence[field]["candidate"]["page"] == 1
        assert row["title"] == "On the Nature of Things" and row["author"] == "Lucretius"
        assert row["source_title"] == "Untitled" and row["source_author"] == "Unknown"
        assert row["title_override"] == ("Newer title" if stale else None)
        assert row["author_override"] == ("Newer author" if stale else None)
        assert analysis == {"status": "completed" if stale else "review",
                            "suggested_title": None if stale else "Geometry of Light",
                            "suggested_author": None if stale else "Ada Lovelace"}
        if not opened:
            assert row["last_page_index"] == 0 and row["reading_progress"] == 0.5
            assert not controls["readerRenderEvents"]
        print(f"PASS {case['name']}: native page evidence, source identity, draft ownership and SQLite decisions")
    for name, expected in hashes.items():
        assert digest(root / "build/rust-fixtures" / name) == expected.upper(), name


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("executable", type=Path)
    parser.add_argument("--source-hash-file", type=Path, required=True)
    args = parser.parse_args()
    verify(args.directory, args.executable, args.source_hash_file)
