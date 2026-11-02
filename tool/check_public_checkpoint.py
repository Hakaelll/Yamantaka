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
    "android/app/src/main/res/mipmap-hdpi/ic_launcher.png": "91132792336c30631cbd8e68d09b3057584ee2ca11556298131293c46e40ff32",
    "android/app/src/main/res/mipmap-mdpi/ic_launcher.png": "06326f51bee71e845aaf2fa1f20657a6e4abf1fae42d6cbda48a75d98697fe4e",
    "android/app/src/main/res/mipmap-xhdpi/ic_launcher.png": "d336a009bbcac5ad7f6c58c86cf3f09679ad4eb434f62d902e43267c6a00e3e2",
    "android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png": "54a5c5ce99297c253c152cfd8d0d0d67c70808accce224cc1ba086d9b90eacd2",
    "android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png": "fbc0ea7a496a83c45a4c09c72ab985ccb5166857017c133e2fd02579a7531efd",
    "assets/branding/yamantaka-logo-purple.png": "36ae243efea708f8daa286232597a75ef8c1322b876f65a750750f49e860686c",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-1024x1024@1x.png": "9319d7f3eaa2388141990802c37a24f181ee487142b91c02ba06a75e582debb2",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@1x.png": "2f70cd3f820e6e2b0ff3713dde348ea69fce87d26e8f35d6ae7795cc0cb1ef8f",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@2x.png": "75856bbccc91d9567f213d6ef03184c5c19ecedce026fe8875eb5a794177506c",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@3x.png": "1eb4f85e7d873ce2115b89676008b6dfd1dbd48be7bf27dc0a54d8840941ca14",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@1x.png": "aa362e1a7b59052c29b31a184b14b36fc3536229ab58d24cf14f23585e212e07",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@2x.png": "ef091239747f9867f9d58ce11c9d45492357bddbed377df01615404ffb1395ca",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@3x.png": "a4d29172b350698c307285f0e78fb5c3f0a40aaf4b57a9d6786559f949c11c04",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@1x.png": "75856bbccc91d9567f213d6ef03184c5c19ecedce026fe8875eb5a794177506c",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@2x.png": "081235b1a26209700728007b9874bd84e6198bea886a4f8390b81e1d6b3aec95",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@3x.png": "04a4e94a63cefdd53778ff990374b7a708a583246d6009830841544f7a8ddb95",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@2x.png": "04a4e94a63cefdd53778ff990374b7a708a583246d6009830841544f7a8ddb95",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@3x.png": "acf107a92b309c32065a37c6fdbd64b42a427bf872b859e0960b7f644a00b5b8",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@1x.png": "a511a726151b14e7b7a75e8d4fe7f48d1435f982ec4b498b41d1570cb660e3be",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@2x.png": "5f7d307834494b18f3e78a0087be9f308eb7525e55d7ee8700b4a8e9e4dc099f",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-83.5x83.5@2x.png": "6aa864125a1a1637698819775684c8a08faf4d062551b39e0a5829562d88827a",
    "web/favicon.ico": "242b29582921f24f28f09a4de77be7b99c52c923c823aada6100bb0a9c9ae578",
    "web/favicon.png": "fbc0ea7a496a83c45a4c09c72ab985ccb5166857017c133e2fd02579a7531efd",
    "web/icons/Icon-192.png": "fbc0ea7a496a83c45a4c09c72ab985ccb5166857017c133e2fd02579a7531efd",
    "web/icons/Icon-512.png": "36ae243efea708f8daa286232597a75ef8c1322b876f65a750750f49e860686c",
    "web/icons/Icon-maskable-192.png": "5fd2fc417035be17d28d7e4692c1a03e8e63f8477765d1ee4984505e01254753",
    "web/icons/Icon-maskable-512.png": "8932743bbed75bb81a1bb2dbe1d8eaaba3ed50db6cb39481b4ffd59b07ca3936",
    "windows/runner/resources/app_icon.ico": "242b29582921f24f28f09a4de77be7b99c52c923c823aada6100bb0a9c9ae578"
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
