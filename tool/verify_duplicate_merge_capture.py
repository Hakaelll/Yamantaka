"""Read the owned synthetic catalogue behind a completed native merge capture."""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3

ROOT = Path(__file__).resolve().parents[1]

def normal_path(path):
    value = str(path.resolve(strict=True))
    slash = chr(92)
    prefix = slash * 2 + "?" + slash
    unc = prefix + "UNC" + slash
    if value.startswith(unc):
        value = slash * 2 + value[len(unc):]
    elif value.startswith(prefix):
        value = value[len(prefix):]
    return Path(value)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("controls", type=Path)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    controls = json.loads(args.controls.read_text(encoding="utf-8-sig"))
    if not controls.get("synthetic") or not controls["duplicates"]["merge"]["merged"]:
        raise ValueError("Expected a completed synthetic merge capture")
    stores = list((ROOT / "build/demo-store/captures").glob(str(controls["captureProcessId"]) + "-*"))
    if len(stores) != 1:
        raise ValueError("Synthetic capture store could not be identified uniquely")
    store = normal_path(stores[0])
    with sqlite3.connect((store / "export-preview.sqlite3").as_uri() + "?mode=ro", uri=True) as db:
        db.row_factory = sqlite3.Row
        records = list(db.execute("SELECT * FROM documents ORDER BY id"))
        if [row["id"] for row in records] != ["demo-0", "demo-2"]:
            raise ValueError("Unexpected remaining catalogue identities")
        journal = list(db.execute("SELECT * FROM operation_journal WHERE operation_kind='rustDuplicateMerge.v1' AND status='completed'"))
        if len(journal) != 1:
            raise ValueError("Expected one durable consolidation journal")
        journal = journal[0]
        summary, undo = json.loads(journal["summary_json"]), json.loads(journal["undo_json"])
        history = db.execute("SELECT * FROM duplicate_merge_history WHERE id=?", (journal["id"],)).fetchone()
        if not history or history["kept_document_id"] != "demo-0" or history["incoming_document_id"] != "demo-1" or history["status"] != "completed":
            raise ValueError("Consolidation history is inconsistent")
        backup = normal_path(Path(history["backup_path"]))
        if not backup.is_relative_to(store) or undo["backupPath"] != str(backup):
            raise ValueError("Backup escaped the owned synthetic store")
        digest = hashlib.sha256(backup.read_bytes()).hexdigest()
        if digest != summary["backupSha256"] or digest != undo["sha256"]:
            raise ValueError("Backup bytes do not match the committed receipt")
        before = json.loads(backup.read_text(encoding="utf-8"))
        original = {row["id"]: row for row in before["tables"]["documents"]}
        incoming = original["demo-1"]
        expected_title = incoming.get("title_override") or incoming.get("title") or incoming.get("source_title") or incoming["file_name"]
        if records[0]["title_override"] != expected_title or records[0]["path"] != original["demo-0"]["path"] or records[0]["fingerprint"] != original["demo-0"]["fingerprint"]:
            raise ValueError("The chosen title or kept source identity was not persisted")
        for document in ["demo-0", "demo-1"]:
            source = normal_path(Path(original[document]["path"]))
            if not source.is_relative_to(store) or hashlib.sha256(source.read_bytes()).hexdigest() != summary["sourceSha256"]:
                raise ValueError("An original synthetic PDF changed")
        if db.execute("PRAGMA foreign_key_check").fetchone() is not None:
            raise ValueError("Consolidation left invalid relationships")
    report = {"version": 1, "synthetic": True, "remainingRecords": 2, "incomingTitlePersisted": True, "keptSourceIdentityPreserved": True, "originalPdfHashesUnchanged": True, "durableHistoryAndJournalAgree": True, "backupHashMatchesReceipt": True, "foreignKeysValid": True}
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("Native consolidation verified against its stored catalogue, backup and original PDF hashes.")

if __name__ == "__main__":
    main()
