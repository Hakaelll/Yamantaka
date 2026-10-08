"""Create synthetic PDFium acceptance inputs, never use personal PDFs."""
from pathlib import Path
from io import BytesIO
import hashlib
import json
from reportlab.pdfgen import canvas
from reportlab.lib.utils import ImageReader
from PIL import Image, ImageDraw
from pypdf import PdfReader, PdfWriter

root = Path(__file__).resolve().parent.parent / "build/rust-fixtures"
root.mkdir(parents=True, exist_ok=True)

def textual():
    data = BytesIO()
    c = canvas.Canvas(data, pagesize=(600, 800))
    c.setTitle("Yamantaka native acceptance")
    c.setAuthor("Synthetic fixture")
    c.drawString(50, 700, "Yamantaka Rust: text, links and vector geometry")
    c.linkURL("https://example.invalid", (50, 695, 330, 715))
    c.rect(50, 50, 500, 550)
    c.showPage()
    c.setPageSize((800, 600))
    c.drawString(50, 500, "Mixed landscape page")
    c.showPage()
    c.save()
    return data.getvalue()

normal = textual()
(root / "textual.pdf").write_bytes(normal)
reader = PdfReader(BytesIO(normal))
writer = PdfWriter()
for page in reader.pages:
    writer.add_page(page)
writer.pages[0].rotate(90)
with (root / "rotated.pdf").open("wb") as file:
    writer.write(file)
writer = PdfWriter()
for page in reader.pages:
    writer.add_page(page)
writer.encrypt("fixture-password", algorithm="RC4-128")
with (root / "protected.pdf").open("wb") as file:
    writer.write(file)
writer = PdfWriter()
for index in range(5000):
    writer.add_blank_page(width=600 if index % 2 == 0 else 800, height=800 if index % 2 == 0 else 600)
with (root / "5000-pages.pdf").open("wb") as file:
    writer.write(file)
image = Image.new("RGB", (600, 800), "white")
draw = ImageDraw.Draw(image)
draw.rectangle((60, 100, 540, 700), fill=(50, 50, 70))
data = BytesIO()
c = canvas.Canvas(data, pagesize=(600, 800))
c.drawImage(ImageReader(image), 0, 0, width=600, height=800)
c.showPage()
c.save()
(root / "scanned.pdf").write_bytes(data.getvalue())
(root / "damaged.pdf").write_bytes(b"%PDF-1.7\ninvalid malformed body\n")
manifest = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(root.glob("*.pdf"))}
(root / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
print(f"Created {len(manifest)} synthetic PDF fixtures, including 5,000 pages.")
