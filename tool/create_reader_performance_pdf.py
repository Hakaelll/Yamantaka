"""Generate a PDF of raster-only pages for the reader performance probe."""

import argparse
from io import BytesIO
from pathlib import Path
from random import Random

from PIL import Image, ImageDraw
from reportlab.lib.utils import ImageReader
from reportlab.pdfgen import canvas


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    width, height = 1800, 2400
    document = canvas.Canvas(str(args.output), pagesize=(600, 800), invariant=True)
    document.setTitle("Yamantaka synthetic scan benchmark")
    for page in range(48):
        random = Random(page)
        noise = Image.frombytes("L", (width, height), random.randbytes(width * height))
        paper = noise.point(lambda value: 210 + value // 6).convert("RGB")
        draw = ImageDraw.Draw(paper)
        draw.text((120, 120), f"SYNTHETIC SCAN   {page + 1:02}", fill=(30, 45, 75))
        for y in range(220, 2100, 32):
            for x in range(120, random.randrange(1050, 1550), 57):
                draw.rectangle((x, y, x + random.randrange(18, 47), y + 7), fill=(48, 41, 37))
        for x, y, tint in [(120, 560, (40, 130, 190)), (850, 1230, (180, 75, 40))]:
            patch = noise.crop((x, y, x + 700, y + 460))
            photo = Image.merge("RGB", tuple(patch.point(lambda value, c=c: (value + c) // 2) for c in tint))
            paper.paste(photo, (x, y))
        jpeg = BytesIO()
        paper.save(jpeg, format="JPEG", quality=89)
        jpeg.seek(0)
        document.drawImage(ImageReader(jpeg), 0, 0, width=600, height=800)
        document.showPage()
    document.save()


if __name__ == "__main__":
    main()
