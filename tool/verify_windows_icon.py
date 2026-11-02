"""Verify the application's actual PE icon resources against its generated ICO."""
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
from pathlib import Path
import struct


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    parser.add_argument("--icon", type=Path, default=Path(__file__).resolve().parents[1] / "windows/runner/resources/app_icon.ico")
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.LoadLibraryExW.argtypes = [wintypes.LPCWSTR, wintypes.HANDLE, wintypes.DWORD]
    kernel.LoadLibraryExW.restype = wintypes.HMODULE
    kernel.FindResourceW.argtypes = [wintypes.HMODULE, ctypes.c_void_p, ctypes.c_void_p]
    kernel.FindResourceW.restype = ctypes.c_void_p
    kernel.SizeofResource.argtypes = [wintypes.HMODULE, ctypes.c_void_p]
    kernel.SizeofResource.restype = wintypes.DWORD
    kernel.LoadResource.argtypes = [wintypes.HMODULE, ctypes.c_void_p]
    kernel.LoadResource.restype = ctypes.c_void_p
    kernel.LockResource.argtypes = [ctypes.c_void_p]
    kernel.LockResource.restype = ctypes.c_void_p
    kernel.FreeLibrary.argtypes = [wintypes.HMODULE]
    kernel.FreeLibrary.restype = wintypes.BOOL
    module = kernel.LoadLibraryExW(str(args.executable.resolve()), None, 0x22)
    if not module:
        raise ctypes.WinError(ctypes.get_last_error())
    def resource(identifier, kind):
        entry = kernel.FindResourceW(module, identifier, kind)
        if not entry:
            raise ValueError("Expected Windows icon resource is missing")
        size = kernel.SizeofResource(module, entry)
        pointer = kernel.LockResource(kernel.LoadResource(module, entry))
        if not pointer or not size:
            raise ValueError("Windows icon resource could not be read")
        return ctypes.string_at(pointer, size)
    try:
        ico = args.icon.read_bytes()
        group = resource(1, 14)
        if struct.unpack_from("<HHH", ico) != struct.unpack_from("<HHH", group):
            raise ValueError("Executable icon group differs from the source ICO")
        count = struct.unpack_from("<H", group, 4)[0]
        sizes = []
        for index in range(count):
            width, height, colors, reserved, planes, depth, size, identifier = struct.unpack_from("<BBBBHHIH", group, 6 + 14 * index)
            iw, ih, ic, ir, ip, ib, length, offset = struct.unpack_from("<BBBBHHII", ico, 6 + 16 * index)
            # RC sets an unspecified ICO plane count to one without changing pixels.
            if (width, height, colors, reserved, depth, size) != (iw, ih, ic, ir, ib, length) or planes != max(1, ip) or resource(identifier, 3) != ico[offset:offset + length]:
                raise ValueError("Embedded icon pixels differ from the approved ICO")
            sizes.append(width or 256)
        report = {"version": 1, "status": "pass", "executableSha256": hashlib.sha256(args.executable.read_bytes()).hexdigest(), "icoSha256": hashlib.sha256(ico).hexdigest(), "embeddedSizes": sizes, "allEmbeddedFramesMatch": True}
        if args.report:
            args.report.parent.mkdir(parents=True, exist_ok=True)
            args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print("Windows executable icon verified: " + ", ".join(map(str, sizes)))
    finally:
        kernel.FreeLibrary(module)

if __name__ == "__main__":
    main()
