"""Generate deterministic synthetic metadata and front-matter PDF cases."""

import hashlib
import json
from pathlib import Path
from reportlab.pdfgen import canvas
from pypdf import PdfReader, PdfWriter


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
for name, title, author, repeated, logo, dense in [
    ("front-matter-missing", "Untitled", "Unknown", True, False, False),
    ("front-matter-conflict", "Old project title", "Old Person", True, False, False),
    ("front-matter-logo", "Geometry of Light", "Ada Lovelace", False, True, False),
    ("front-matter-logo-missing", "", "", True, True, False),
    ("front-matter-single", "", "", False, False, False),
    ("front-matter-body", "Contents and Context", "", False, False, True),
    ("front-matter-ambiguous", "", "", False, False, False),
    ("front-matter-ambiguous-metadata", "Geometry of Light", "Ada Lovelace", False, False, False),
]:
    path = root / f"{name}.pdf"
    pdf = canvas.Canvas(str(path), pagesize=(600, 800), invariant=1)
    pdf.setTitle(title)
    pdf.setAuthor(author)
    for _ in range(2 if repeated else 1):
        if logo:
            pdf.setFont("Helvetica", 60)
            pdf.drawString(50, 730, "STUDY PRESS")
        pdf.setFont("Helvetica", 36)
        pdf.drawString(90, 640, "Chapter One" if dense else "Geometry of Light")
        pdf.setFont("Helvetica", 16)
        pdf.drawString(100, 595, "By the river" if dense else "By Ada Lovelace")
        if name.startswith("front-matter-ambiguous"):
            pdf.drawString(100, 570, "By Charles Babbage")
        pdf.setFont("Helvetica", 10)
        for index in range(32 if dense else 2):
            pdf.drawString(100, 540 - index * 14, "Synthetic front matter acceptance" if not dense else f"Body line {index}: ordinary running text.")
        pdf.showPage()
    pdf.save()
    records.append({"file": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
for rotation in (90, 180, 270):
    path = root / f"front-matter-rotated-{rotation}.pdf"
    writer = PdfWriter(clone_from=PdfReader(root / "front-matter-missing.pdf"))
    for page in writer.pages:
        page.rotate(rotation)
    writer.write(path)
    records.append({"file": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
path = root / "front-matter-work-limit.pdf"
pdf = canvas.Canvas(str(path), pagesize=(600, 800), invariant=1)
pdf.setTitle("Native work limit")
pdf.setAuthor("Known Institute")
pdf.drawString(20, 700, "word " * 4000)
pdf.showPage()
pdf.save()
records.append({"file": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
path = root / "front-matter-evidence-later.pdf"
pdf = canvas.Canvas(str(path), pagesize=(600, 800), invariant=1)
pdf.setTitle("Untitled")
pdf.setAuthor("Unknown")
pdf.showPage()
pdf.setFont("Helvetica", 36)
pdf.drawString(90, 640, "Geometry of Light")
pdf.setFont("Helvetica", 16)
pdf.drawString(100, 595, "By Ada Lovelace")
pdf.showPage()
pdf.save()
records.append({"file": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
(root / "metadata-manifest.json").write_text(
    json.dumps({"synthetic": True, "files": records}, indent=2), encoding="utf-8"
)
print(f"Created {len(records)} deterministic synthetic metadata/front-matter PDFs.")
