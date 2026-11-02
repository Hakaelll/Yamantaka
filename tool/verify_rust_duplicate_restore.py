"""Verify native consolidation, backup installation and a separate process restart."""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def normal_path(path):
    value = str(Path(path).resolve())
    slash = chr(92)
    prefix = slash * 2 + "?" + slash
    if value.startswith(prefix + "UNC" + slash):
        value = slash * 2 + value[len(prefix + "UNC" + slash):]
    elif value.startswith(prefix):
        value = value[len(prefix):]
    return Path(value)


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "build/verification/duplicate-restore-native")
    args = parser.parse_args()
    executable = args.executable.resolve(strict=True)
    output = args.output.resolve()
    require(output.is_relative_to(ROOT / "build"), "Verification output must remain inside workspace build")
    output.mkdir(parents=True, exist_ok=True)
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = 0

    def run(extra, name):
        with (output / (name + ".stdout.log")).open("w", encoding="utf-8") as stdout, (output / (name + ".stderr.log")).open("w", encoding="utf-8") as stderr:
            subprocess.run([str(executable), "--demo", *extra], cwd=ROOT, stdout=stdout, stderr=stderr, startupinfo=startup, timeout=60, check=True)

    capture = output / "install.png"
    run(["--duplicates", "--duplicates-merge", "--duplicates-merge-history-apply", "--capture=" + str(capture)], "install")
    result = read_json(capture.with_suffix(".restore-result.json"))
    require(all(result.get(key) is True for key in ["synthetic", "installed", "safetyExists", "consolidationRestore"]), "Native restore did not complete")
    require(result["books"] == 3 and result["safetyBooks"] == 2, "Unexpected native restore record counts")
    location = read_json(capture.with_suffix(".restore-result.location.json"))
    store, safety = (normal_path(location[key]) for key in ["store", "safety"])
    owned = (ROOT / "build/demo-store/captures").resolve(strict=True)
    require(all(path != owned and path.is_relative_to(owned) for path in [store, safety]), "Native restore escaped the owned synthetic stores")
    with sqlite3.connect((safety / "export-preview.sqlite3").as_uri() + "?mode=ro", uri=True) as before:
        summary, undo = before.execute("SELECT summary_json,undo_json FROM operation_journal WHERE operation_kind='rustDuplicateMerge.v1' AND status='completed'").fetchone()
        summary, undo = json.loads(summary), json.loads(undo)
    # The pre-merge JSON moved with the previous library; verify its original receipt hash.
    backup_name = normal_path(undo["backupPath"]).relative_to(store)
    backup = safety / backup_name
    require(hashlib.sha256(backup.read_bytes()).hexdigest() == undo["sha256"], "Pre-merge backup changed during installation")
    original = read_json(backup)
    with sqlite3.connect((store / "yamantaka.sqlite3").as_uri() + "?mode=ro", uri=True) as restored:
        restored.row_factory = sqlite3.Row
        tables = {row[0] for row in restored.execute("SELECT name FROM sqlite_master WHERE type='table'")}
        for table, expected in original["tables"].items():
            require(table in tables, "Restored backup table is missing")
            actual = [dict(row) for row in restored.execute('SELECT * FROM "' + table.replace('"', '""') + '"')]
            key = lambda row: json.dumps(row, sort_keys=True)
            require(sorted(actual, key=key) == sorted(expected, key=key), "Restored table differs from the pre-merge snapshot: " + table)
        require(restored.execute("PRAGMA foreign_key_check").fetchone() is None, "Restored foreign keys are invalid")
    for document in original["tables"]["documents"]:
        source = normal_path(document["path"])
        require(source.is_relative_to(owned) and not source.is_relative_to(store) and not source.is_relative_to(safety), "Expected an external owned synthetic PDF")
        if document["id"] in ["demo-0", "demo-1"]:
            require(hashlib.sha256(source.read_bytes()).hexdigest() == summary["sourceSha256"], "An original PDF changed")
    reopened = output / "reopened.png"
    run(["--continuity-resume", "--continuity-store=" + str(store), "--pdf-covers", "--reduce-motion", "--capture=" + str(reopened)], "reopened")
    controls = read_json(reopened.with_suffix(".controls.json"))
    require(controls.get("synthetic") and controls["libraryDocumentCount"] == 3, "The separate native process did not reopen the restored library")
    report = {"synthetic": True, "installed": True, "restoredBooks": 3, "safetyBooks": 2, "snapshotTablesEqual": True, "foreignKeysValid": True, "backupHashPreserved": True, "originalDuplicatePdfHashesUnchanged": True, "separateNativeProcessReopened": True, "executableSha256": hashlib.sha256(executable.read_bytes()).hexdigest()}
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("Native consolidation restore and separate process restart verified.")


if __name__ == "__main__":
    main()
