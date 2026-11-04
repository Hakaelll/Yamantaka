"""Render the corporate SVG into the existing application icon variants."""
from __future__ import annotations
import argparse
import hashlib
import io
import json
import xml.etree.ElementTree as ET
from pathlib import Path
from PIL import Image, ImageDraw, PngImagePlugin

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets/branding/yamantaka-logo.svg"
ICO_SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)

def master(opaque=False):
    svg = ET.fromstring(SOURCE.read_bytes())
    if svg.attrib.get("viewBox") != "0 0 512 512":
        raise ValueError("The brand source must use its canonical 512-unit canvas")
    scale = 4
    result = Image.new("RGBA", (2048, 2048), "#F2EFE8" if opaque else (0, 0, 0, 0))
    draw = ImageDraw.Draw(result)
    for node in svg:
        kind = node.tag.rsplit("}", 1)[-1]
        if kind in {"title", "desc"}:
            continue
        if kind == "rect":
            width, height, radius = (float(node.attrib[key]) * scale for key in ("width", "height", "rx"))
            draw.rounded_rectangle((0, 0, width - 1, height - 1), radius=radius, fill=node.attrib["fill"])
        elif kind == "polygon":
            points = [tuple(float(value) * scale for value in point.split(",")) for point in node.attrib["points"].split()]
            draw.polygon(points, fill=node.attrib["fill"])
        else:
            raise ValueError(f"Unsupported brand SVG element: {kind}")
    return result

def png_bytes(image):
    output = io.BytesIO()
    metadata = PngImagePlugin.PngInfo()
    metadata.add_text("Title", "Yamantaka corporate application icon")
    metadata.add_text("Source", "assets/branding/yamantaka-logo.svg")
    image.save(output, format="PNG", pnginfo=metadata, optimize=True)
    return output.getvalue()

def generated():
    transparent, opaque = master(), master(True)
    targets = {"assets/branding/yamantaka-logo-purple.png": (512, False)}
    output = {}
    for path, (size, solid) in targets.items():
        if size <= 0 or size > 1024:
            raise ValueError("Unexpected application icon dimensions")
        output[path] = png_bytes((opaque if solid else transparent).resize((size, size), Image.Resampling.LANCZOS))
    ico = io.BytesIO()
    transparent.resize((256, 256), Image.Resampling.LANCZOS).save(ico, format="ICO", sizes=[(size, size) for size in ICO_SIZES])
    output["assets/branding/yamantaka.ico"] = ico.getvalue()
    manifest = {
        "version": 1, "identity": "open-folio-v3", "source": SOURCE.relative_to(ROOT).as_posix(),
        "sourceSha256": hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
        "background": "#F2EFE8", "foreground": "#101112", "icoSizes": list(ICO_SIZES),
        "files": [{"path": path, "sha256": hashlib.sha256(data).hexdigest()} for path, data in sorted(output.items())],
    }
    output["assets/branding/manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    return output, manifest

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    output, manifest = generated()
    changed = [name for name, data in output.items() if not (ROOT / name).is_file() or (ROOT / name).read_bytes() != data]
    if not args.check:
        for name, data in output.items():
            (ROOT / name).write_bytes(data)
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps({"status": "fail" if args.check and changed else "pass", "files": len(manifest["files"]), "mismatches": changed if args.check else [], "sourceSha256": manifest["sourceSha256"]}, indent=2) + "\n", encoding="utf-8")
    if args.check and changed:
        raise SystemExit("Brand assets differ from the canonical SVG: " + ", ".join(changed))
    with Image.open(io.BytesIO(output["assets/branding/yamantaka.ico"])) as icon:
        if icon.ico.sizes() != {(size, size) for size in ICO_SIZES}:
            raise SystemExit("Windows icon frames do not match the required sizes")
    print(f"Brand assets {'verified' if args.check else 'generated'}: {len(manifest['files'])}")

if __name__ == "__main__":
    main()
