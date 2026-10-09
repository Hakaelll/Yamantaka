"""Verify the pinned PDFium wrapper and its narrowly recorded local patches."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1] / "third_party" / "pdfium-render"
metadata = json.loads((root / "UPSTREAM.json").read_text(encoding="utf-8"))
assert metadata["registryArchiveSha256"] == "8948a803616a9e936b15a6637af2cd48c5fb8ae0fcdeb3c32eac3a540e255a19"
assert metadata["package"] == "pdfium-render" and metadata["version"] == "0.9.4"
expected_patches = {"Cargo.toml", "src/pdf/action/uri.rs", "src/error.rs", "src/pdf/document/page/object/content_mark.rs"}
patches = {item["path"]: item["sha256"] for item in metadata["localPatches"]}
assert set(patches) == expected_patches
seen = set()
for item in metadata["files"]:
    path = root / item["path"]
    assert path.resolve().is_relative_to(root.resolve())
    assert item["path"] not in seen
    seen.add(item["path"])
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    assert actual == patches.get(item["path"], item["sha256"]), item["path"]
assert set(patches).issubset(seen)
actual_files = {path.relative_to(root).as_posix() for path in (root / "src").rglob("*") if path.is_file()}
assert actual_files == {name for name in seen if name.startswith("src/")}
print(f"Verified {len(seen)} pinned upstream files and {len(patches)} recorded local patches.")
