"""Check real PDF reanalysis captures against native evidence and SQLite."""
import argparse
import hashlib
import sqlite3
from pathlib import Path
from verify_rust_metadata_review import digest, read_json


def verify(directory, source_hash_file):
    root = Path(__file__).resolve().parent.parent
    manifest = read_json(directory / "capture-manifest.json")
    names = {"metadata-reanalysis-" + suffix for suffix in
             ("dark", "light-es", "manual", "limit", "small-200")}
    assert manifest["synthetic"] and {case["name"] for case in manifest["captures"]} == names
    sources = {item["File"]: item["Sha256"] for item in read_json(source_hash_file)}
    for source in manifest["sources"]:
        assert digest(root / source["path"]) == source["sha256"].upper(), "UI source changed after capture"
    for case in manifest["captures"]:
        for file_key, hash_key in (("image", "imageSha256"), ("controls", "controlsSha256")):
            assert digest(directory / case[file_key]) == case[hash_key].upper()
        controls = read_json(directory / case["controls"])
        review = controls["metadataReview"]
        assert controls["synthetic"] and not review["pending"] and not review["analyzing"] and not review["saved"]
        limited = case["name"].endswith("-limit")
        manual = case["name"].endswith("-manual")
        assert review["analyzed"] == (not limited) and bool(review["error"]) == limited
        for control in controls["controls"]:
            assert control["rect"][0] >= control["clip"][0] - 1
            assert control["rect"][2] <= control["clip"][2] + 1, (case["name"], control["label"])
        assert any(c["enabled"] and c["label"] in ("Analyze PDF again", "Volver a analizar el PDF") for c in controls["controls"])
        database = root / "build/demo-store/captures" / str(controls["captureProcessId"]) / "export-preview.sqlite3"
        with sqlite3.connect(database.resolve().as_uri() + "?mode=ro", uri=True) as db:
            db.row_factory = sqlite3.Row
            row = dict(db.execute("SELECT path,title,author,source_title,source_author,title_override,author_override,fingerprint,reading_progress,last_page_index FROM documents WHERE id='demo-0'").fetchone())
            analysis = dict(db.execute("SELECT status,suggested_title,suggested_author FROM document_enrichment WHERE document_id='demo-0'").fetchone())
            evidence = db.execute("SELECT value FROM app_metadata WHERE key='rust.frontMatterEvidence.v1:demo-0'").fetchone()
            assert db.execute("SELECT count(*) FROM library_search").fetchone()[0] == 0
        path = Path(row.pop("path"))
        assert digest(path) == sources[path.name]
        data = path.read_bytes()
        partial = hashlib.sha256(data[:65536]).hexdigest()
        assert row.pop("fingerprint") == hashlib.sha256(f"{len(data)}:{partial}".encode()).hexdigest()
        assert row == {"title": "On the Nature of Things", "author": "Lucretius", "source_title": "Untitled", "source_author": "Unknown", "title_override": "Manual geometry" if manual else None, "author_override": "Manual author" if manual else None, "reading_progress": 0.5, "last_page_index": 0}
        assert review["title"] == ("Manual geometry" if manual else "On the Nature of Things")
        assert review["author"] == ("Manual author" if manual else "Lucretius")
        assert analysis == {"status": "completed" if manual else "review", "suggested_title": None if manual else "Geometry of Light", "suggested_author": None if manual else "Ada Lovelace"}
        if limited:
            assert evidence is None
        else:
            import json
            record = json.loads(evidence[0])
            assert record["analyzerVersion"] == 3 and record["hasText"]
            assert record["title"]["kind"] == "review" and record["author"]["kind"] == "review"
            assert record["title"]["candidate"]["value"] == "Geometry of Light"
            assert record["author"]["candidate"]["value"] == "Ada Lovelace"
    print("Verified five real reanalysis states, accepted and manual values, native evidence, bounded failure, control width, consent, and original PDF hashes.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--source-hash-file", type=Path, required=True)
    args = parser.parse_args()
    verify(args.directory, args.source_hash_file)
