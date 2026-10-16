"""Verify native cover recovery after occupied PDF slots become available."""
import argparse
import sqlite3
from pathlib import Path
from verify_rust_metadata_review import digest, read_json


def verify(directory):
    root = Path(__file__).resolve().parent.parent
    manifest = read_json(directory / "capture-manifest.json")
    assert manifest["synthetic"]
    assert {c["name"] for c in manifest["captures"]} == {"cover-capacity-retry-dark", "cover-capacity-retry-light-es"}
    for source in manifest["sources"]:
        assert digest(root / source["path"]) == source["sha256"].upper(), "UI source changed after capture"
    fixture_root = root / "build/rust-fixtures/covers"
    fixture_manifest = read_json(fixture_root / "manifest.json")
    assert fixture_manifest["synthetic"]
    hashes = {item["file"]: item["sha256"].upper() for item in fixture_manifest["pdfs"]}
    for case in manifest["captures"]:
        for file_key, hash_key in (("image", "imageSha256"), ("controls", "controlsSha256")):
            assert digest(directory / case[file_key]) == case[hash_key].upper()
        controls = read_json(directory / case["controls"])
        covers = controls["libraryCovers"]
        assert controls["synthetic"] and controls["pagePresenter"] == "native-wgpu"
        assert covers["blockedBeforeRelease"] == covers["ready"] == covers["resident"] == 10
        assert covers["capacityPreview"] == 3 and covers["capacityRevision"] > 0 and covers["pending"] == 0
        assert covers["maximumVisible"] == 16
        expected = {f"demo-{index}": 0 for index in range(10)}
        assert covers["sourcePages"] == expected
        database = root / "build/demo-store/captures" / str(controls["captureProcessId"]) / "export-preview.sqlite3"
        with sqlite3.connect(database.resolve().as_uri() + "?mode=ro", uri=True) as db:
            rows = db.execute("SELECT d.id,d.path,d.reading_progress,c.source_page_index,c.selection_mode,c.algorithm_version FROM documents d JOIN document_covers c ON c.document_id=d.id ORDER BY d.id").fetchall()
            assert db.execute("SELECT count(*) FROM library_search").fetchone()[0] == 0
        assert len(rows) == 10
        for index, (document, source, progress, page, mode, version) in enumerate(rows):
            assert document == f"demo-{index}" and page == expected[document]
            assert mode == "automatic" and version == 3 and progress == (0.5 if index == 0 else 0.0)
            source = Path(source)
            assert source.samefile(fixture_root / f"cover-{index}.pdf")
            assert digest(source) == hashes[source.name]
    print("Verified two native recovery captures: ten blocked covers become ten PDF textures, persisted choices match, consent and original PDF hashes remain intact.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    verify(parser.parse_args().directory)
