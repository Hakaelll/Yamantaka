"""Create an isolated, synthetic study PDF for native contents-panel checks."""
from pathlib import Path
import hashlib
import json
from reportlab.pdfgen import canvas

root = Path(__file__).resolve().parents[1] / "build" / "rust-fixtures"
root.mkdir(parents=True, exist_ok=True)
target = root / "outline.pdf"
pdf = canvas.Canvas(str(target), pagesize=(600, 800), invariant=1)
pdf.setTitle("The art of attention")
pdf.setAuthor("Synthetic study collection")
chapters = [
    ("The art of attention", "Begin with a question", "Attention gives a reading session its shape."),
    ("Reading with intention", "Make room for thought", "A good question stays with us beyond the page."),
    ("A reader's notebook", "Keep what matters", "Notes connect today's reading to tomorrow's ideas."),
]
for page, (title, subtitle, passage) in enumerate(chapters):
    pdf.setFillColorRGB(.98, .97, .94)
    pdf.rect(0, 0, 600, 800, fill=1, stroke=0)
    pdf.setFillColorRGB(.15, .16, .17)
    pdf.setFont("Times-Roman", 32)
    pdf.drawString(64, 670, title)
    pdf.setFont("Helvetica", 11)
    pdf.drawString(64, 630, "A PERSONAL STUDY COMPANION")
    pdf.setFont("Times-Roman", 20)
    pdf.drawString(64, 530, subtitle)
    pdf.setFont("Times-Roman", 14)
    pdf.drawString(64, 485, passage)
    pdf.drawString(64, 455, "Read slowly. Return to a passage. Let the idea settle.")
    pdf.bookmarkPage(f"chapter-{page}", fit="Fit")
    pdf.addOutlineEntry(title, f"chapter-{page}", level=0)
    pdf.bookmarkPage(f"passage-{page}", fit="XYZ", left=64, top=550, zoom=1.5)
    pdf.addOutlineEntry(subtitle, f"passage-{page}", level=1)
    pdf.showPage()
pdf.save()
target.with_suffix(".provenance.json").write_text(json.dumps({
    "synthetic": True, "pages": 3, "source": "tool/create_rust_outline_fixture.py",
    "sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
}, indent=2) + "\n", encoding="utf-8")
