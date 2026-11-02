"""Compatibility entry point for the corporate brand asset generator."""
import runpy
from pathlib import Path

if __name__ == "__main__":
    runpy.run_path(str(Path(__file__).with_name("generate_brand_assets.py")), run_name="__main__")
