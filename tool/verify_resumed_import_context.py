"""Inspect the owned native resumed-import fixture and its persisted rule context."""
import argparse
import json
from pathlib import Path
import sqlite3
ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("controls", type=Path)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    controls = json.loads(args.controls.read_text(encoding="utf-8-sig"))
    if not controls.get("synthetic") or controls["libraryDocumentCount"] != 11:
        raise ValueError("Expected the completed synthetic resumed-import case")
    stores = list((ROOT / "build/demo-store/captures").glob(str(controls["captureProcessId"]) + "-*"))
    if len(stores) != 1:
        raise ValueError("Could not uniquely identify the owned synthetic store")
    database = stores[0] / "export-preview.sqlite3"
    with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True) as db:
        payload = json.loads(db.execute("SELECT summary_json FROM operation_journal WHERE id='rust-import-rule-context-v1' AND operation_kind='rustImportContext.v1'").fetchone()[0])
        ids = payload["ids"]
        if payload["version"] != 1 or len(ids) != 2 or "demo-0" not in ids or sum(identity.startswith("demo-") for identity in ids) != 1:
            raise ValueError("Resumed context omitted the prior member or included unrelated catalogue records")
        for identity in ids:
            if db.execute("SELECT count(*) FROM documents WHERE id=?", (identity,)).fetchone()[0] != 1:
                raise ValueError("A resumed context member is not a unique catalogue record")
        if db.execute("PRAGMA user_version").fetchone()[0] != 11:
            raise ValueError("Main library schema changed")
    with sqlite3.connect(Path(str(database) + ".imports.sqlite3").as_uri() + "?mode=ro", uri=True) as recovery:
        if recovery.execute("PRAGMA user_version").fetchone()[0] != 2 or recovery.execute("SELECT count(*) FROM import_jobs").fetchone()[0] != 0:
            raise ValueError("Recovery journal did not migrate or finish the job")
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps({"synthetic": True, "catalogueRecords": 11, "contextRecords": 2, "priorCheckpointMemberRetained": True, "newMemberRetained": True, "unrelatedCatalogueRecordsExcluded": True, "recoveryJobsFinished": True, "recoverySchemaVersion": 2, "mainSchemaVersion": 11}, indent=2) + "\n", encoding="utf-8")
    print("Native resumed import retained its prior and new rule-context members.")


if __name__ == "__main__":
    main()
