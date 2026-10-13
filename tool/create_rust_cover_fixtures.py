"""Generate explicitly synthetic, two-page PDFs for native 3D cover acceptance."""
from pathlib import Path
from reportlab.pdfgen import canvas
from reportlab.lib.colors import HexColor
from reportlab.lib.utils import simpleSplit
import json, hashlib

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "build/rust-fixtures/covers"
TITLES = [
    ("On the Nature of Things", "Lucretius"),
    ("The Architecture of Thought", "Study collection"),
    ("Ways of Seeing", "John Berger"),
    ("A Field Guide to the Universe", "Study collection"),
    ("The Art of Attention", "Study collection"),
    ("Essays on Light", "Study collection"),
    ("The Shape of Ideas", "Study collection"),
    ("A Reader's Notebook", "Study collection"),
    ("Collected Fragments", "Study collection"),
    ("The Geometry of Silence", "Study collection"),
]
COLORS = ["#532E39", "#2B3D51", "#214A4D", "#3A314F", "#2C473A", "#996A45"]

def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    manifest = []
    for index, (title, author) in enumerate(TITLES):
        path = OUTPUT / f"cover-{index}.pdf"
        pdf = canvas.Canvas(str(path), pagesize=(420, 560), invariant=1)
        pdf.setTitle(f"Synthetic cover fixture: {title}")
        pdf.setAuthor("Yamantaka synthetic acceptance fixtures")
        pdf.setFillColor(HexColor(COLORS[index % len(COLORS)]))
        pdf.rect(0, 0, 420, 560, fill=1, stroke=0)
        pdf.setFillColor(HexColor("#E6DECF"))
        font = "Times-Roman" if index % 2 == 0 else "Helvetica-Bold"
        pdf.setFont(font, 35)
        for line_index, line in enumerate(simpleSplit(title, font, 35, 330)):
            pdf.drawString(42, 476 - line_index * 42, line)
        pdf.setStrokeColor(HexColor("#C9A875"))
        pdf.setLineWidth(1.5)
        for ring in range(6):
            pdf.circle(210 + ring * 4, 225, 65 + ring * 3, stroke=1, fill=0)
        pdf.setFillColor(HexColor("#E6DECF"))
        pdf.setFont("Helvetica", 17)
        pdf.drawString(42, 64, author)
        pdf.setFont("Helvetica", 9)
        pdf.drawString(42, 30, "Synthetic PDF cover · visual acceptance fixture")
        pdf.showPage()
        pdf.setFont("Helvetica", 20)
        pdf.drawString(42, 480, "Synthetic study page")
        pdf.save()
        manifest.append({"file":path.name, "sha256":hashlib.sha256(path.read_bytes()).hexdigest()})
    (OUTPUT / "manifest.json").write_text(json.dumps({"synthetic":True,"pdfs":manifest}, indent=2), encoding="utf-8")
    print("Generated ten synthetic two-page PDF cover fixtures.")

if __name__ == "__main__":
    main()
