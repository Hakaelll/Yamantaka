"""Verify last completed import context across two real native processes."""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "build/verification/import-context-restart")
    args = parser.parse_args()
    executable = args.executable.resolve(strict=True)
    output = args.output.resolve()
    if not output.is_relative_to(ROOT / "build"):
        raise ValueError("Output must remain in workspace build")
    output.mkdir(parents=True, exist_ok=True)
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = 0

    def capture(name, extra):
        path = output / (name + ".png")
        with (output / (name + ".stdout.log")).open("w") as stdout, (output / (name + ".stderr.log")).open("w") as stderr:
            subprocess.run([str(executable), "--demo", "--saved-rules", "--rule-simulation", "--rule-import-context", "--reduce-motion", "--capture=" + str(path), *extra], cwd=ROOT, startupinfo=startup, timeout=60, check=True, stdout=stdout, stderr=stderr)
        controls = json.loads(path.with_suffix(".controls.json").read_text(encoding="utf-8-sig"))
        result = controls["ruleSimulation"]
        if not controls.get("synthetic") or controls["libraryDocumentCount"] != 13 or controls["libraryVisibleFavorites"] != 0 or result["state"] != "complete" or result["matched"] != 3 or result["scanned"] != 3 or any(identity.startswith("demo-") for identity in result["ids"]):
            raise ValueError("Native import context simulation matched the wrong scope")
        return controls, result["ids"]

    first, expected = capture("completed-import", ["--folder-import"])
    stores = list((ROOT / "build/demo-store/captures").glob(str(first["captureProcessId"]) + "-*"))
    if len(stores) != 1:
        raise ValueError("Synthetic store must be identified uniquely")
    store = stores[0].resolve(strict=True)
    with sqlite3.connect((store / "export-preview.sqlite3").as_uri() + "?mode=ro", uri=True) as db:
        payload = json.loads(db.execute("SELECT summary_json FROM operation_journal WHERE id='rust-import-rule-context-v1' AND operation_kind='rustImportContext.v1'").fetchone()[0])
        if payload["version"] != 1 or sorted(payload["ids"]) != sorted(expected):
            raise ValueError("Committed import context differs from the completed native batch")
    _, restored = capture("restarted", ["--continuity-resume", "--continuity-store=" + str(store)])
    if restored != expected:
        raise ValueError("Restart changed the imported identities")
    report = {"synthetic": True, "catalogueRecords": 13, "importedRecords": 3, "durableJournalMatchesBatch": True, "separateNativeProcessRestoredExactScope": True, "ruleActionsAutomaticallyExecuted": False, "executableSha256": hashlib.sha256(executable.read_bytes()).hexdigest()}
    sources = ["rust/storage/src/import_rule_context.rs", "rust/application/src/runtime.rs", "rust/domain/src/lib.rs", "rust/storage/src/database.rs", "rust/ui/src/app.rs", "rust/ui/src/app/operations.rs", "rust/ui/src/app/automation/view.rs"]
    report["sources"] = [{"path": name, "sha256": hashlib.sha256((ROOT / name).read_bytes()).hexdigest()} for name in sources]
    report["cargoLockSha256"] = hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest()
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("Completed import context retained its exact rule scope across native restart.")


if __name__ == "__main__":
    main()
