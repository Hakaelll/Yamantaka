"""Review exact Git blobs before publishing; never print matched private values.

This is a targeted guard, not proof that arbitrary prose/images contain no PII.
Human review and synthetic capture provenance remain required.
"""
import argparse
import hashlib
import json
import re
import struct
import subprocess
import sys
import zlib
from pathlib import PurePosixPath


PATTERNS = {
    "local user path": re.compile(r"(?:[A-Za-z]:[\\/]+(?:Users|Documents and Settings)[\\/]+[^\\/\s\"']+|/(?:home|Users)/[^/\s\"']+)", re.I),
    "email in file": re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b"),
    "private key": re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----"),
    "credential token": re.compile(r"\b(?:gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|sk-(?:proj-)?[A-Za-z0-9_-]{24,}|AKIA[0-9A-Z]{16})\b"),
}
BLOCKED_EXTENSIONS = {".pdf", ".sqlite", ".sqlite3", ".db", ".pem", ".pfx", ".key", ".env", ".zip", ".exe", ".dll"}
FONT_PATH = "assets/fonts/source-serif/SourceSerif4Display-Regular.ttf"
FONT_SHA = "a1788d674713c99e61d4541c85e5cfbc3145c4e746b0185c674a0b15f7313295"


def text_findings(text):
    # Also inspect serialized Windows paths with doubled JSON backslashes.
    values = [text]
    for _ in range(3):
        values.append(values[-1].replace("\\\\", "\\"))
    return {kind for kind, pattern in PATTERNS.items() if any(pattern.search(value) for value in values)}


def json_strings(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for key, item in value.items():
            yield key
            yield from json_strings(item)
    elif isinstance(value, list):
        for item in value:
            yield from json_strings(item)


def png_text(data):
    if not data.startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError("Invalid PNG signature")
    pos = 8
    metadata = {}
    while pos < len(data):
        if len(data) - pos < 12:
            raise ValueError("Truncated PNG chunk")
        size = struct.unpack(">I", data[pos:pos + 4])[0]
        end = pos + size + 12
        if end > len(data):
            raise ValueError("PNG chunk exceeds file")
        kind = data[pos + 4:pos + 8]
        payload = data[pos + 8:end - 4]
        crc = struct.unpack(">I", data[end - 4:end])[0]
        if zlib.crc32(kind + payload) & 0xFFFFFFFF != crc:
            raise ValueError("Invalid PNG chunk CRC")
        if kind == b"tEXt":
            key, value = payload.split(b"\0", 1)
            metadata[key.decode("latin1")] = value.decode("latin1")
        elif kind in {b"zTXt", b"iTXt", b"eXIf"}:
            # Our native capture pipeline emits plain text provenance only.
            # Unexpected metadata needs explicit inspection rather than skipping.
            raise ValueError("Unexpected compressed/international/EXIF metadata")
        pos = end
        if kind == b"IEND":
            if pos != len(data):
                raise ValueError("Unexpected data after PNG end")
            break
    return metadata


def audit_blob(name, data):
    path = PurePosixPath(name)
    findings = set()
    if path.suffix.lower() in BLOCKED_EXTENSIONS or any(part in {".vscode", ".aws", ".codex", "build", "target"} for part in path.parts):
        findings.add("non-public/generated/private data path")
    if name == FONT_PATH:
        if hashlib.sha256(data).hexdigest() != FONT_SHA:
            findings.add("unverified font binary")
        return findings
    if path.suffix.lower() == ".png":
        if not name.startswith("docs/screenshots/rust-"):
            findings.add("unexpected image")
        try:
            metadata = png_text(data)
            origin = metadata.get("impeccable:prompt", "")
            if "synthetic" not in origin.lower():
                findings.add("image lacks synthetic capture provenance")
            for value in metadata.values():
                findings.update(text_findings(value))
        except (ValueError, UnicodeError) as error:
            findings.add(str(error))
        return findings
    try:
        text = data.decode("utf-8-sig")
    except UnicodeError:
        findings.add("unexpected binary")
        return findings
    findings.update(text_findings(text))
    if path.suffix.lower() == ".json":
        try:
            for value in json_strings(json.loads(text)):
                findings.update(text_findings(value))
        except ValueError:
            findings.add("invalid JSON")
    return findings


def git(*args):
    return subprocess.check_output(["git", *args])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--base", help="Audit changed blobs between this Git ref and HEAD")
    selection.add_argument("--staged", action="store_true", help="Audit staged blobs (default)")
    parser.add_argument("--check-author", action="store_true", help="Verify HEAD uses the user's configured Git address without printing it")
    args = parser.parse_args()
    if args.base:
        names = git("diff", "--name-only", "--diff-filter=ACMR", "-z", args.base, "HEAD")
        prefix = "HEAD:"
    else:
        names = git("diff", "--cached", "--name-only", "--diff-filter=ACMR", "-z")
        prefix = ":"
    files = [name.decode("utf-8") for name in names.split(b"\0") if name]
    problems = []
    for name in files:
        for finding in sorted(audit_blob(name, git("cat-file", "blob", prefix + name))):
            problems.append((name, finding))
    if args.check_author:
        addresses = git("show", "-s", "--format=%ae%n%ce", "HEAD").decode().splitlines()
        expected = git("config", "user.email").decode().strip()
        if any(address != expected for address in addresses):
            problems.append(("commit metadata", "author or committer differs from the user's configured Git address"))
    for name, finding in problems:
        print(f"REVIEW REQUIRED: {name}: {finding}")
    print(f"Reviewed {len(files)} exact Git blobs; {len(problems)} findings.")
    return int(bool(problems))


if __name__ == "__main__":
    sys.exit(main())
