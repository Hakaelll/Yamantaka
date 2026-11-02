"""Fetch hash-pinned upstream notices omitted from the published crate archives."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Verify cached sources without network access")
    args = parser.parse_args()
    entries = json.loads((ROOT / "tool/rust_license_sources.json").read_text(encoding="utf-8"))["sources"]
    cache = ROOT / "build/native/cargo-license-cache"
    cache.mkdir(parents=True, exist_ok=True)
    for entry in entries.values():
        expected = entry["sha256"]
        if not re.fullmatch(r"[a-f0-9]{64}", expected) or not re.fullmatch(
            r"https://raw\.githubusercontent\.com/[\w-]+/[\w.-]+/[a-f0-9]{40}/LICENSE(?:-MIT|-APACHE)?", entry["url"]
        ):
            raise ValueError("License sources must use a pinned upstream commit and SHA-256")
        destination = cache / (expected + ".txt")
        if destination.is_file() and hashlib.sha256(destination.read_bytes()).hexdigest() == expected:
            continue
        if args.check:
            raise ValueError("Pinned license text is absent or changed; run fetch_rust_licenses.py without --check")
        with urllib.request.urlopen(entry["url"], timeout=30) as response:
            data = response.read(2 * 1024 * 1024 + 1)
        if len(data) > 2 * 1024 * 1024 or hashlib.sha256(data).hexdigest() != expected:
            raise ValueError("Upstream license bytes differ from the reviewed source")
        destination.write_bytes(data)
    print(f"Pinned license sources ready for {len(entries)} packages")


if __name__ == "__main__":
    main()
