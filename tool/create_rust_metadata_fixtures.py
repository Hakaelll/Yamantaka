"""Generate deterministic metadata-only variants of a synthetic PDF."""

import hashlib
import json
from pathlib import Path
from reportlab.pdfgen import canvas


root = Path(__file__).resolve().parent.parent / "build/rust-fixtures"
root.mkdir(parents=True, exist_ok=True)
cases = {
    "metadata-tool": ("Microsoft Word - Thesis.docx", "Unknown"),
    "metadata-unicode": ("  Óptica\t y señales  ", "  Instituto de Óptica  "),
    "metadata-legitimate": ("1984", "Anonymous"),
}
records = []
for name, (title, author) in cases.items():
    path = root / f"{name}.pdf"
    pdf = canvas.Canvas(str(path), pagesize=(600, 800), invariant=1)
    pdf.setTitle(title)
    pdf.setAuthor(author)
    pdf.drawString(50, 700, "Synthetic import metadata acceptance")
    pdf.showPage()
    pdf.save()
    records.append({"file": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
(root / "metadata-manifest.json").write_text(
    json.dumps({"synthetic": True, "files": records}, indent=2), encoding="utf-8"
)
print(f"Created {len(records)} deterministic synthetic metadata PDFs.")
