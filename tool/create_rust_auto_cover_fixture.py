"""Generate an explicitly synthetic PDF for native automatic cover selection."""
from pathlib import Path
from reportlab.pdfgen import canvas


def main():
    path = Path(__file__).resolve().parent.parent / "build/rust-fixtures/automatic-cover.pdf"
    path.parent.mkdir(parents=True, exist_ok=True)
    pdf = canvas.Canvas(str(path), pagesize=(400, 560), invariant=True)
    pdf.setTitle("Synthetic automatic cover acceptance fixture")
    for page in range(7):
        if page == 1:
            pdf.setFillColorRGB(0.25, 0.1, 0.2)
            pdf.rect(45, 55, 310, 450, fill=1, stroke=0)
            pdf.setFillColorRGB(1, 1, 1)
            pdf.setFont("Times-Roman", 26)
            pdf.drawString(70, 425, "A synthetic cover")
            pdf.setFont("Helvetica", 10)
            pdf.drawString(70, 390, "Native selection acceptance")
        else:
            pdf.setFillColorRGB(0, 0, 0)
            pdf.setFont("Helvetica", 10)
            pdf.drawString(35, 525, "CONTENTS: explicitly synthetic study text")
            for line in range(44):
                pdf.drawString(35, 505 - line * 10, f"{line+1}. Chapter notes and study content for this synthetic PDF.")
        pdf.showPage()
    pdf.save()
    print("Created build/rust-fixtures/automatic-cover.pdf with seven synthetic pages")


if __name__ == "__main__":
    main()
