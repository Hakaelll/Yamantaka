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
    "android/app/src/main/res/mipmap-hdpi/ic_launcher.png": "ab8d686a93175ea4432a3a1d9b0456010e0b7051ba8f8b5128a42903bf6c11a0",
    "android/app/src/main/res/mipmap-mdpi/ic_launcher.png": "9e4aae94ab91a650057148ed86bc8aa73912815539fa0bda7a165bc383f74c3b",
    "android/app/src/main/res/mipmap-xhdpi/ic_launcher.png": "f2fb078835296ff560bb01b5559c15be46055112ec8be80a826af0fed3d8d087",
    "android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png": "3d360083ef9cdf3859b813c5da90db5938e2713429b92334466e58d2cd41b9f7",
    "android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png": "5392ad7c2fc20e9f5589cb183602ad3f708025620bf6fd892c90b9a3c5dbf283",
    "assets/branding/yamantaka-logo-purple.png": "26f8bb346276be029b9e54c0fc901e5f13bd7752387281bcb27cfc3cd176a107",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-1024x1024@1x.png": "70e74667bc443a37f48c397dbccdd8f1f5eb2b13c85e885ab67229b36c636e79",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@1x.png": "7fdc5eb5182123868dfde4b71c23b55b665698346b6d1de4669b586682f83003",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@2x.png": "35fcafd0f1ef0302e65e0d65426be0da7877374df3aea069451721a114f15c85",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-20x20@3x.png": "b7120590caaf0bbe7bbf758a689ad14e8112db6dbe5aa4805c4e3231afc6f27c",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@1x.png": "dea22f9dafd0c6ad215ecafbb72f70e4c290de735ffb2252be0ca77baf6792d7",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@2x.png": "8553768231b7b3efe181ea12b544c9b1c3f2388b857809812dccdaaf27af2c2c",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-29x29@3x.png": "56933fa952bdaac5a00f4af6e7d31cdba11a8aaa82564bd6aad94cc9c43bbdd6",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@1x.png": "35fcafd0f1ef0302e65e0d65426be0da7877374df3aea069451721a114f15c85",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@2x.png": "3c27c42058a9bad8475204969bcd0152f4ccd166a5c9e6090d0f749f99cd064b",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-40x40@3x.png": "023512f2a29d7ab096b24b9b7e7d53b591d74280813078a68a878e7b9700059c",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@2x.png": "023512f2a29d7ab096b24b9b7e7d53b591d74280813078a68a878e7b9700059c",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-60x60@3x.png": "04a0c2287fed2ffb7e9a1b8d49a1cba171d307bbfe5694b8d487fcbd13aa0bd1",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@1x.png": "e23b818b078202ec8c73ac33e3f0b82fae786fce0b58c574326c69f4b691b342",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-76x76@2x.png": "59799d2c88d1c095724fc4f3783fbe330c8758375f55e1e979424074310e0e01",
    "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-83.5x83.5@2x.png": "1cca607345810e8ee45600a0cc55f632f388eb8d8e5016cc3d3180c506e5b64a",
    "web/favicon.ico": "6e14442b93a8dc08f6e30e6242773c0e4116187227e660cf13cc8704d4321e31",
    "web/favicon.png": "5392ad7c2fc20e9f5589cb183602ad3f708025620bf6fd892c90b9a3c5dbf283",
    "web/icons/Icon-192.png": "5392ad7c2fc20e9f5589cb183602ad3f708025620bf6fd892c90b9a3c5dbf283",
    "web/icons/Icon-512.png": "26f8bb346276be029b9e54c0fc901e5f13bd7752387281bcb27cfc3cd176a107",
    "web/icons/Icon-maskable-192.png": "c48fa4bfedab3bac9467fdd2aad9471e01a5848a24da2198ee481f5ef0fdaebb",
    "web/icons/Icon-maskable-512.png": "1d80d40c638a05939f38efa292517a50b37012f4525c2f39ddd3984aa99d4a7d",
    "windows/runner/resources/app_icon.ico": "6e14442b93a8dc08f6e30e6242773c0e4116187227e660cf13cc8704d4321e31"
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
