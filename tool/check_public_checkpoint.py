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


# Exact binaries rendered from the owner-requested corporate SVG, reviewed locally.
BRAND_ASSET_HASHES = {
    "android/app/src/main/res/mipmap-hdpi/ic_launcher.png": "9d99a4d9c20b4047b99767be2deee895abfad04c0da5202bee9c93f72fc454cd",
    "android/app/src/main/res/mipmap-mdpi/ic_launcher.png": "220ea1df8ba3524206b54246e6ee49b402b2ae83409b46b6ae15b8a8eb9d7509",
    "android/app/src/main/res/mipmap-xhdpi/ic_launcher.png": "c755d6c73d690d8015f47844dc91e2d9f70f545d444ad0bd5f42739779b1681b",
    "android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png": "b6fc64c235b9870d110ae0cbb9b33f0f24aa1ae36ce094b066bc2bc9aeb79cb1",
    "android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png": "0c9b87e5d671ec0b216c4eb82371ea6ff0327342965df96fd02f9f80453e431a",
    "assets/branding/yamantaka-logo-purple.png": "0367c684444a7a06a56e2bca826681f3db41ecfd3ccc465c35f64eb580b593e2",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-1024x1024@1x.png": "b4e845930514ead00535e4628e1fa56f4048041265341f2a0dde9a9b3332a286",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@1x.png": "e26a03a0c8fee998212087abcb4250fdc93f35ee5b9076373026867afee300a9",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@2x.png": "a1477d0b91b521cfc635626a7c17250eb7efc9a4ee42cb98771d832ede298f15",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@3x.png": "b1561dbaccae322f6a9820d08eb268619d4418ee8d04ea564668f17a329cf76b",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@1x.png": "0f6c26efb6ac1c06c4cf8ba2fefea2092d366f6c85bc22762a93952e49fc76db",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@2x.png": "8a71124bd86d05304217484c2c8046af41da800ef80780e1cd144afa28b02ee4",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@3x.png": "a901e409a0fdd96912493e144c8f071af49d64a53c0058e826f5746fac23a9fe",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@1x.png": "a1477d0b91b521cfc635626a7c17250eb7efc9a4ee42cb98771d832ede298f15",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@2x.png": "62a5695647addee3350cef87bea7e2cf1a15c11891e28fb1e0e0a5031771c4aa",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@3x.png": "1c458e3d19195286c8ff153261b42db2144b962ab6e13bac9e1323efa1247fb2",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@2x.png": "1c458e3d19195286c8ff153261b42db2144b962ab6e13bac9e1323efa1247fb2",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@3x.png": "5df023f7c3e1f4bfffa3447f265d5f2ce5f688ecea0109384cb1e84f3d12eee2",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@1x.png": "a1e7afc4d0d4bc6ae2d31f236e541505270b91766aed37b902facc142c45c612",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@2x.png": "f7c8adc7d7b695fe7361491d62e62566d348ad623141794088d1e99376039c54",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-83.5x83.5@2x.png": "2295c59402aaaf3b9dc7cc1bba8bddef6662d7cb32809f5a373cfc2a17dafc3b",
    "web/favicon.ico": "f38afc05ec30ac9b44cfb8051e262e2e6916e95171dddf66de131e74ae816e81",
    "web/favicon.png": "0c9b87e5d671ec0b216c4eb82371ea6ff0327342965df96fd02f9f80453e431a",
    "web/icons/Icon-192.png": "0c9b87e5d671ec0b216c4eb82371ea6ff0327342965df96fd02f9f80453e431a",
    "web/icons/Icon-512.png": "0367c684444a7a06a56e2bca826681f3db41ecfd3ccc465c35f64eb580b593e2",
    "web/icons/Icon-maskable-192.png": "a77293c630261c34f030af934b0f8b988fff0f8f8854e510f6e19c39e6a5f11a",
    "web/icons/Icon-maskable-512.png": "95460d464dd866bc5985c884adef40f81444179aae438117ddc4c410280867f5",
    "windows/runner/resources/app_icon.ico": "f38afc05ec30ac9b44cfb8051e262e2e6916e95171dddf66de131e74ae816e81"
}

def text_findings(text):
    # Also inspect serialized Windows paths with doubled JSON backslashes.
    values = [text]
    for _ in range(3):
        values.append(values[-1].replace("\\\\", "\\"))
    launcher_names = {PurePosixPath(name).name for name in BRAND_ASSET_HASHES if name.startswith("ios/")}
    return {
        kind for kind, pattern in PATTERNS.items()
        if any(
            kind != "email in file" or match.group() not in launcher_names
            for value in values for match in pattern.finditer(value)
        )
    }


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
    if name in BRAND_ASSET_HASHES:
        if hashlib.sha256(data).hexdigest() != BRAND_ASSET_HASHES[name]:
            findings.add("unverified corporate brand binary")
        return findings
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
