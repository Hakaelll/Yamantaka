"""Recolor the tracked Yamantaka mark without changing its geometry.

The source pixels are read from ``HEAD`` so running this script repeatedly is
stable.  Every output pixel is produced at the same coordinate as its source;
only chroma changes.  Local luminance, alpha, dimensions, and ICO frame sizes
are retained so the engraving and ray silhouette stay intact.
"""

from __future__ import annotations

import io
import json
import math
import subprocess
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
PURPLE = (181, 138, 255)  # AppColorTokens.librarySelectionAccent (#B58AFF).
CHROMA_STRENGTH = 0.92
LUMA_WEIGHTS = (0.2126, 0.7152, 0.0722)

PNG_OUTPUTS = (
    ("web/icons/Icon-512.png", "web/icons/Icon-512.png"),
    ("web/icons/Icon-192.png", "web/icons/Icon-192.png"),
    ("web/icons/Icon-maskable-512.png", "web/icons/Icon-maskable-512.png"),
    ("web/icons/Icon-maskable-192.png", "web/icons/Icon-maskable-192.png"),
    ("web/favicon.png", "web/favicon.png"),
    ("web/icons/Icon-512.png", "assets/branding/yamantaka-logo-purple.png"),
)
ICO_OUTPUTS = {
    "web/favicon.ico": "web/favicon.ico",
    "windows/runner/resources/app_icon.ico": (
        "windows/runner/resources/app_icon.ico"
    ),
}


def tracked_bytes(path: str) -> bytes:
    return subprocess.check_output(
        ["git", "show", f"HEAD:{path}"], cwd=ROOT
    )


def luma(red: int, green: int, blue: int) -> float:
    return (
        LUMA_WEIGHTS[0] * red
        + LUMA_WEIGHTS[1] * green
        + LUMA_WEIGHTS[2] * blue
    )


def recolor(source: Image.Image) -> Image.Image:
    """Apply purple chroma while preserving per-pixel perceived luminance."""
    rgba = source.convert("RGBA")
    target_luma = luma(*PURPLE)
    chroma = tuple(component - target_luma for component in PURPLE)
    output: list[tuple[int, int, int, int]] = []

    for red, green, blue, alpha in rgba.getdata():
        value = luma(red, green, blue)
        max_scale = CHROMA_STRENGTH
        for delta in chroma:
            if delta > 0:
                max_scale = min(max_scale, (255 - value) / delta)
            elif delta < 0:
                max_scale = min(max_scale, value / -delta)
        scale = max(0.0, max_scale)
        recolored = tuple(
            max(0, min(255, round(value + scale * delta)))
            for delta in chroma
        )
        output.append((*recolored, alpha))

    result = Image.new("RGBA", rgba.size)
    result.putdata(output)
    return result


def proof(source: Image.Image, result: Image.Image) -> dict[str, object]:
    source_rgba = source.convert("RGBA")
    result_rgba = result.convert("RGBA")
    source_pixels = list(source_rgba.getdata())
    result_pixels = list(result_rgba.getdata())
    source_luma = [luma(*pixel[:3]) for pixel in source_pixels]
    result_luma = [luma(*pixel[:3]) for pixel in result_pixels]
    source_mean = sum(source_luma) / len(source_luma)
    result_mean = sum(result_luma) / len(result_luma)
    numerator = sum(
        (left - source_mean) * (right - result_mean)
        for left, right in zip(source_luma, result_luma, strict=True)
    )
    source_variance = sum((value - source_mean) ** 2 for value in source_luma)
    result_variance = sum((value - result_mean) ** 2 for value in result_luma)
    correlation = numerator / math.sqrt(source_variance * result_variance)
    max_luma_delta = max(
        abs(left - right)
        for left, right in zip(source_luma, result_luma, strict=True)
    )
    return {
        "dimensionsIdentical": source_rgba.size == result_rgba.size,
        "alphaIdentical": all(
            left[3] == right[3]
            for left, right in zip(source_pixels, result_pixels, strict=True)
        ),
        "coordinateMapping": "1:1; no crop, resize, warp, or regeneration",
        "luminanceCorrelation": round(correlation, 9),
        "maximumLuminanceDelta": round(max_luma_delta, 4),
    }


def write_png(source_path: str, output_path: str) -> dict[str, object]:
    source = Image.open(io.BytesIO(tracked_bytes(source_path)))
    result = recolor(source)
    destination = ROOT / output_path
    destination.parent.mkdir(parents=True, exist_ok=True)
    if source.mode == "RGB":
        result = result.convert("RGB")
    result.save(destination, optimize=True)
    return {
        "source": f"HEAD:{source_path}",
        "output": output_path,
        "size": list(source.size),
        **proof(source, result),
    }


def write_ico(source_path: str, output_path: str) -> dict[str, object]:
    source = Image.open(io.BytesIO(tracked_bytes(source_path)))
    sizes = sorted(source.ico.sizes(), key=lambda size: size[0] * size[1])
    source_frames = [source.ico.getimage(size).convert("RGBA") for size in sizes]
    result_frames = [recolor(frame) for frame in source_frames]
    largest = result_frames[-1]
    destination = ROOT / output_path
    destination.parent.mkdir(parents=True, exist_ok=True)
    largest.save(
        destination,
        format="ICO",
        sizes=sizes,
        append_images=result_frames[:-1],
    )
    written = Image.open(destination)
    return {
        "source": f"HEAD:{source_path}",
        "output": output_path,
        "frameSizesIdentical": set(sizes) == written.ico.sizes(),
        "frames": [
            {"size": list(size), **proof(before, after)}
            for size, before, after in zip(
                sizes, source_frames, result_frames, strict=True
            )
        ],
    }


def main() -> None:
    report = {
        "method": (
            "Per-pixel chroma substitution from tracked HEAD assets; local "
            "luminance and every source coordinate are preserved."
        ),
        "targetColor": "#B58AFF",
        "png": [
            write_png(source_path, output_path)
            for source_path, output_path in PNG_OUTPUTS
        ],
        "ico": [
            write_ico(source_path, output_path)
            for source_path, output_path in ICO_OUTPUTS.items()
        ],
    }
    report_path = ROOT / "build/visual-evidence/v0.8.0/branding/pixel-proof.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(report_path)


if __name__ == "__main__":
    main()
