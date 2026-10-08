"""Extract literal SQL from the frozen Flutter migration implementation.

Development-only adapter generation, never a runtime Dart dependency. Keep the
original source until parity is approved; commit generated SQL for review.
"""
from pathlib import Path
import json
import re
import textwrap

root = Path(__file__).resolve().parent.parent
source = (root / "lib/storage/database/app_database.dart").read_text(encoding="utf-8")
source = source.split("void _migrate(", 1)[1].split("  void _transaction(", 1)[0]
literal = re.compile(r"'''(.*?)'''|\"\"\"(.*?)\"\"\"|'((?:\\.|[^'\\])*)'|\"((?:\\.|[^\"\\])*)\"", re.S)
statements = []
position = 0
for call in re.finditer(r"\.execute\s*\(", source):
    pos = call.end()
    parts = []
    while True:
        match = literal.match(source, pos + len(source[pos:]) - len(source[pos:].lstrip()))
        if not match:
            break
        value = next(value for value in match.groups() if value is not None)
        value = re.sub(r"\\(['\"\\])", r"\1", value)
        parts.append(value)
        pos = match.end()
    if not parts:
        raise ValueError("Non-literal migration SQL")
    sql = textwrap.dedent("".join(parts)).strip()
    statements.append(sql + ";")
    version = re.fullmatch(r"PRAGMA user_version = (\d+)", sql)
    if version:
        target = root / "rust/storage/migrations" / f"{int(version[1]):02d}.sql"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("-- Extracted from Flutter AppDatabase._migrate; preserve persisted semantics.\n" + "\n\n".join(statements) + "\n", encoding="utf-8")
        print(f"{target.name}: {len(statements)} statements")
        statements = []
if statements:
    raise ValueError("SQL remains after the final version boundary")

backup = (root / "lib/storage/backup/local_backup_service.dart").read_text(encoding="utf-8")
contracts = {}
bodies = dict(re.findall(r"const (_\w+Columns) = <String>\[(.*?)\];", backup, re.S))
def resolve(name):
    if name in contracts:
        return contracts[name]
    body = bodies[name]
    values = []
    for reference, quoted in re.findall(r"\.\.\.(_\w+Columns)|'([^']+)'", body):
        values.extend(resolve(reference) if reference else [quoted])
    contracts[name] = values
    return values
for name in bodies:
    resolve(name)
restore = backup.split("void _replaceTables(", 1)[1].split("Future<void> _restoreSafetyDatabase", 1)[0]
table_contracts = dict(re.findall(r"_restoreTable\(\s*'([^']+)',\s*(_\w+Columns)", restore))
versions = {}
for version in range(1, 10):
    values = {table: contracts[name] for table, name in table_contracts.items()}
    values["documents"] = contracts["_documentColumns" if version >= 9 else "_v8DocumentColumns" if version >= 4 else "_v3DocumentColumns" if version >= 3 else "_legacyDocumentColumns"]
    values["document_preferences"] = contracts["_preferenceColumns" if version >= 6 else "_legacyPreferenceColumns"]
    values["annotations"] = contracts["_annotationColumns" if version >= 9 else "_v8AnnotationColumns" if version >= 8 else "_v7AnnotationColumns"]
    values["workspaces"] = contracts["_workspaceColumns" if version >= 8 else "_v7WorkspaceColumns"]
    values["workspace_tabs"] = contracts["_workspaceTabColumns" if version >= 8 else "_v7WorkspaceTabColumns"]
    values["operation_journal"] = contracts["_journalColumns" if version >= 9 else "_v8JournalColumns"]
    versions[str(version)] = values
(root / "rust/storage/migrations/backup-contracts.json").write_text(json.dumps(versions, indent=2) + "\n", encoding="utf-8")
print(f"backup-contracts.json: {len(table_contracts)} tables, nine historical formats")
