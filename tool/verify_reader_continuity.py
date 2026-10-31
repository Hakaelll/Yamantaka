"""Probe ordinary restart using isolated copies of a synthetic comparison capture."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import sqlite3
import subprocess
import uuid


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def same_comparison(expected, actual):
    assert actual is not None, "Comparison was not restored"
    assert expected.keys() == actual.keys(), "Comparison fields changed"
    for key, value in expected.items():
        if key in ("zoom", "ratio"):
            assert abs(value - actual[key]) < 0.00001, key
        elif key == "point":
            assert all(abs(a - b) < 0.0001 for a, b in zip(value, actual[key], strict=True)), key
        else:
            assert value == actual[key], key


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--executable", required=True, type=Path)
    parser.add_argument("--capture-directory", required=True, type=Path)
    parser.add_argument("--source-case", default="continuity-comparison-book")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    captures = (repo / "build/demo-store/captures").resolve(strict=True)
    output = args.capture_directory.resolve(strict=True)
    assert output.is_relative_to(repo / "build"), "Evidence must stay in build"
    executable = args.executable.resolve(strict=True)
    assert re.fullmatch(r"[a-z0-9-]+", args.source_case), "Unexpected source case"
    source_controls = read(output / f"{args.source_case}.controls.json")
    assert source_controls["synthetic"] is True
    token = source_controls["continuityStoreToken"]
    assert re.fullmatch(r"[0-9]+-[0-9a-f-]{36}", token), "Unexpected fixture token"
    source = (captures / token).resolve(strict=True)
    assert source.parent == captures
    expected = read(output / f"{args.source_case}.expected-comparison.json")
    comparison = expected["comparison"]
    assert expected["synthetic"] and comparison["primary"] != comparison["document"]
    files = ("export-preview.sqlite3", "preferences.json", "export-preview.reader-continuity.json")
    before = {name: digest(source / name) for name in files}
    results = []
    for mode in ("normal", "missing", "disabled"):
        fixture = captures / f"continuity-{uuid.uuid4()}"
        fixture.mkdir()
        with sqlite3.connect(f"file:{(source / files[0]).as_posix()}?mode=ro", uri=True) as original:
            with sqlite3.connect(fixture / files[0]) as copy:
                original.backup(copy)
                if mode == "missing":
                    changed = copy.execute("UPDATE documents SET is_missing=1 WHERE id=?", (comparison["document"],))
                    assert changed.rowcount == 1
        for name in files[1:]:
            shutil.copyfile(source / name, fixture / name)
        preferences = read(fixture / "preferences.json")
        preferences.setdefault("rust", {})["restoreLastSession"] = mode != "disabled"
        (fixture / "preferences.json").write_text(json.dumps(preferences), encoding="utf-8")
        image = output / f"continuity-restart-{mode}.png"
        controls_path = image.with_suffix(".controls.json")
        for owned in (image, controls_path):
            owned.unlink(missing_ok=True)
        with image.with_suffix(".stderr.log").open("w", encoding="utf-8") as errors:
            process = subprocess.Popen(
                [str(executable), "--demo", f"--capture={image}", "--continuity-resume",
                 f"--continuity-store={fixture}", "--es"], cwd=repo,
                stdout=subprocess.DEVNULL, stderr=errors,
                creationflags=subprocess.CREATE_NO_WINDOW,
            )
            try:
                assert process.wait(timeout=45) == 0, f"Native restart failed: {mode}"
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait()
        actual = read(controls_path)
        continuity = actual["readerContinuity"]
        assert actual["synthetic"] and not actual["graphicsRuntimeFallback"]
        assert continuity["loaded"] and not continuity["restoring"]
        assert actual["graphicsRecoveryRestorePreference"] == (mode != "disabled")
        if mode == "disabled":
            assert not actual["readerTabs"]["tabs"] and actual["reference"] is None
        else:
            assert actual["currentReaderPage"] == expected["primaryPage"]
            active = next(tab for tab in actual["readerTabs"]["tabs"] if tab["id"] == actual["readerTabs"]["active"])
            assert abs(active["zoom"] - expected["primaryZoom"]) < 0.00001
            if mode == "normal":
                same_comparison(comparison, continuity["comparison"])
                assert continuity["retainedComparison"] is None and actual["reference"]["images"] > 0
            else:
                assert actual["reference"] is None
                same_comparison(comparison, continuity["retainedComparison"])
                same_comparison(comparison, read(fixture / files[2])["comparison"])
        notes = expected.get("pageNotes", {})
        if notes.get("primary") and notes.get("reference"):
            recovered = actual["pageNoteRecovery"]
            assert recovered["retained"] == {"normal": 0, "missing": 1, "disabled": 2}[mode]
            assert recovered["primary"] == (None if mode == "disabled" else notes["primary"])
            assert recovered["reference"] == (notes["reference"] if mode == "normal" else None)
            persisted = read(fixture / files[2])["page_notes"]
            assert sorted(persisted, key=lambda note: note["reference"]) == [notes["primary"], notes["reference"]]
        results.append({"mode": mode, "passed": True, "artifacts": {
            path.name: digest(path) for path in (image, controls_path)
        }})
        print(f"PASS continuity restart: {mode}", flush=True)
    assert before == {name: digest(source / name) for name in files}, "Source fixture changed"
    report = {"synthetic": True, "executableSha256": digest(executable),
              "sourceUnchanged": True, "cases": results}
    (output / "continuity-restart-report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
