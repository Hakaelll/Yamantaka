import importlib.util
import json
import struct
import unittest
import zlib
from pathlib import Path

spec = importlib.util.spec_from_file_location("checkpoint", Path(__file__).resolve().parents[1] / "check_public_checkpoint.py")
checkpoint = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checkpoint)


def chunk(kind, payload):
    return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)


def png(metadata):
    # Byte-level parser fixture, not a screenshot or a publishable image.
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IDAT", b"test") + chunk(b"tEXt", b"impeccable:prompt\0" + metadata) + chunk(b"IEND", b"")


class PublicCheckpointTests(unittest.TestCase):
    def test_json_decoding_exposes_escaped_local_paths_without_echoing_them(self):
        path = "C:" + "\\Users\\" + "private_reader\\library"
        findings = checkpoint.audit_blob("docs/example.json", json.dumps({"nested": [path]}).encode())
        self.assertIn("local user path", findings)
        self.assertNotIn(path, " ".join(findings))

    def test_contact_tokens_and_private_keys_are_categories_not_values(self):
        examples = [
            ("reader" + "@example.test", "email in file"),
            ("ghp_" + "a" * 36, "credential token"),
            ("-----BEGIN " + "OPENSSH PRIVATE KEY-----", "private key"),
        ]
        for value, expected in examples:
            findings = checkpoint.audit_blob("docs/example.txt", value.encode())
            self.assertIn(expected, findings)
            self.assertNotIn(value, " ".join(findings))

    def test_provenance_after_image_data_is_inspected(self):
        name = "docs/screenshots/rust-example/reader.png"
        self.assertEqual(checkpoint.audit_blob(name, png(b"Native synthetic capture")), set())
        private = "Native synthetic capture: " + "C:" + "\\Users\\" + "private_reader\\image"
        self.assertIn("local user path", checkpoint.audit_blob(name, png(private.encode())))
        self.assertIn("image lacks synthetic capture provenance", checkpoint.audit_blob(name, png(b"Personal photograph")))

    def test_corrupt_unknown_and_trailing_metadata_cannot_be_silently_skipped(self):
        name = "docs/screenshots/rust-example/reader.png"
        data = png(b"Native synthetic capture")
        self.assertIn("Unexpected data after PNG end", checkpoint.audit_blob(name, data + b"private"))
        damaged = bytearray(data)
        damaged[20] ^= 1
        self.assertIn("Invalid PNG chunk CRC", checkpoint.audit_blob(name, damaged))
        extra = b"\x89PNG\r\n\x1a\n" + chunk(b"eXIf", b"unknown") + chunk(b"IEND", b"")
        self.assertIn("Unexpected compressed/international/EXIF metadata", checkpoint.audit_blob(name, extra))

    def test_library_files_editor_settings_and_unverified_binaries_require_review(self):
        for name in ["original.pdf", "library.sqlite3", ".vscode/settings.json", "build/package.zip"]:
            self.assertIn("non-public/generated/private data path", checkpoint.audit_blob(name, b"{}"))
        self.assertIn("unexpected binary", checkpoint.audit_blob("document.bin", b"\xff\x00"))
        self.assertIn("unverified font binary", checkpoint.audit_blob(checkpoint.FONT_PATH, b"unknown font"))


if __name__ == "__main__":
    unittest.main()
