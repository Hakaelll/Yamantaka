"""Package and independently verify the native Windows review bundle."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
TARGET = "x86_64-pc-windows-msvc"
PDFIUM_SHA = "79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def require(value, message):
    if not value:
        raise ValueError(message)


def pe_info(path):
    data = path.read_bytes()
    pe = struct.unpack_from("<I", data, 0x3c)[0]
    require(data[pe:pe + 4] == b"PE\0\0", "Invalid PE header")
    machine, count = struct.unpack_from("<HH", data, pe + 4)
    optional_size = struct.unpack_from("<H", data, pe + 20)[0]
    optional = pe + 24
    require(machine == 0x8664 and struct.unpack_from("<H", data, optional)[0] == 0x20b, "Expected a Windows x64 PE image")
    sections = [struct.unpack_from("<IIII", data, optional + optional_size + index * 40 + 8) for index in range(count)]

    def offset(rva):
        if rva < struct.unpack_from("<I", data, optional + 60)[0]:
            return rva
        for virtual_size, virtual_address, raw_size, raw_offset in sections:
            if virtual_address <= rva < virtual_address + max(virtual_size, raw_size):
                return raw_offset + rva - virtual_address
        raise ValueError("PE address is outside image sections")

    def name(rva):
        start = offset(rva)
        end = data.find(b"\0", start, start + 256)
        require(end >= 0, "Invalid imported library name")
        result = data[start:end].decode("ascii").lower()
        require(re.fullmatch(r"[a-z0-9_.-]+\.dll", result), "Unexpected imported library path")
        return result

    libraries = set()
    for directory, size, name_field in [(1, 20, 12), (13, 32, 4)]:
        rva = struct.unpack_from("<I", data, optional + 112 + directory * 8)[0]
        if not rva:
            continue
        entry = offset(rva)
        while any(data[entry:entry + size]):
            if directory == 13:
                require(struct.unpack_from("<I", data, entry)[0] & 1, "Unsupported VA-based delayed imports")
            libraries.add(name(struct.unpack_from("<I", data, entry + name_field)[0]))
            entry += size
    return {"machine": "AMD64", "subsystem": struct.unpack_from("<H", data, optional + 68)[0], "imports": sorted(libraries)}


def command(args):
    return subprocess.check_output(args, cwd=ROOT, text=True, encoding="utf-8")


def json_file(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def source_inputs():
    paths = set((ROOT / "rust").rglob("*.rs")) | set((ROOT / "rust").rglob("Cargo.toml"))
    paths.update((ROOT / "third_party/pdfium-render").rglob("*.rs"))
    for crate in ["egui", "epaint"]:
        source = ROOT / "third_party" / crate
        paths.update(source.rglob("*.rs"))
        paths.update(path for path in source.iterdir() if path.is_file() and path.name in {"Cargo.toml", "LICENSE-MIT", "LICENSE-APACHE", "YAMANTAKA-PATCH.md"})
    paths.update(ROOT / name for name in [
        "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".gitattributes", "LICENSE",
        "third_party/pdfium-render/Cargo.toml", "assets/branding/yamantaka.ico",
        "assets/branding/yamantaka-logo-purple.png", "assets/fonts/source-serif/SourceSerif4Display-Regular.ttf",
        "tool/package_rust_windows.ps1", "tool/rust_windows_bundle.py", "tool/fetch_rust_licenses.py",
        "tool/rust_license_sources.json", "docs/release/RUST-WINDOWS-BUNDLE.txt",
    ])
    return [{"path": path.relative_to(ROOT).as_posix(), "sha256": digest(path)} for path in sorted(paths)]


def package(output, portable, cargo):
    release_parent = (ROOT / "build/release").resolve()
    require(output != release_parent and output.is_relative_to(release_parent), "Use a named directory below build/release")
    require(not output.exists(), "Package destination already exists; choose a fresh review directory")
    subprocess.check_call([sys.executable, str(ROOT / "tool/generate_brand_assets.py"), "--check"], cwd=ROOT)
    subprocess.check_call([sys.executable, str(ROOT / "tool/fetch_rust_licenses.py"), "--check"], cwd=ROOT)
    sources = source_inputs()
    build_environment = os.environ.copy()
    require(build_environment.get("USERPROFILE"), "Build-profile environment is unavailable")
    build_environment.update({
        "CARGO_ENCODED_RUSTFLAGS": "\x1f".join([
            "-C", "target-feature=+crt-static", "--remap-path-prefix=" + str(ROOT) + "=/_yamantaka",
            "--remap-path-prefix=" + build_environment["USERPROFILE"] + "=/_user",
        ]),
        "CARGO_BUILD_JOBS": "2", "CARGO_INCREMENTAL": "0",
    })
    subprocess.check_call([
        cargo, "build", "--release", "--target", TARGET, "-p", "yamantaka",
        "--bin", "yamantaka-rust", "--locked", "--offline",
    ], cwd=ROOT, env=build_environment)
    require(source_inputs() == sources, "Inputs changed during compilation; rebuild before packaging")
    executable = ROOT / "target" / TARGET / "release/yamantaka-rust.exe"
    pdfium = ROOT / "build/native/pdfium-7881"
    require(executable.is_file(), "Build the native Windows Release executable first")
    require(digest(pdfium / "bin/pdfium.dll") == PDFIUM_SHA, "Pinned PDFium hash mismatch")
    require(pe_info(executable)["subsystem"] == 2, "A debug/console executable cannot be packaged as Release")
    metadata = json.loads(command([cargo, "metadata", "--locked", "--offline", "--filter-platform", TARGET, "--format-version", "1"]))
    packages = {item["id"]: item for item in metadata["packages"]}
    graph = {item["id"]: item for item in metadata["resolve"]["nodes"]}
    root_id = next(item["id"] for item in packages.values() if item["name"] == "yamantaka")

    def edges(identity):
        return [dep["pkg"] for dep in graph[identity]["deps"] if any(kind["kind"] != "dev" for kind in dep["dep_kinds"])]

    pending, reachable = [root_id], set()
    while pending:
        identity = pending.pop()
        if identity not in reachable:
            reachable.add(identity)
            pending.extend(edges(identity))
    runtime, pending = set(), [root_id]
    while pending:
        identity=pending.pop()
        if identity not in runtime:
            runtime.add(identity)
            for dependency in graph[identity]["deps"]:
                if any(kind["kind"] is None for kind in dependency["dep_kinds"]) and not any("proc-macro" in target["kind"] for target in packages[dependency["pkg"]]["targets"]):
                    pending.append(dependency["pkg"])
    version = packages[root_id]["version"]
    status = "review-candidate-not-final-release" if "-" in version else "stable-release"
    require((ROOT / "LICENSE").read_text(encoding="utf-8").startswith("MIT License"),"The approved application license is not present")
    require(packages[root_id].get("license") == "MIT", "Application license differs from the approved MIT terms")
    output.mkdir(parents=True)
    bundle = output / "yamantaka"
    bundle.mkdir()
    shutil.copyfile(executable, bundle / "yamantaka.exe")
    shutil.copyfile(pdfium / "bin/pdfium.dll", bundle / "pdfium.dll")
    shutil.copyfile(ROOT / "Cargo.lock", bundle / "Cargo.lock")
    shutil.copyfile(ROOT / "LICENSE", bundle / "LICENSE")
    shutil.copyfile(ROOT / "docs/release/RUST-WINDOWS-BUNDLE.txt", bundle / "README.txt")
    if portable:
        (bundle / ".yamantaka-rust-portable").write_bytes(b"")
    lock=tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    checksums={(item["name"],item["version"]):item.get("checksum") for item in lock["package"] if item.get("checksum")}
    components, dependencies, refs, missing = [], [], {}, []
    license_sources = json.loads((ROOT / "tool/rust_license_sources.json").read_text(encoding="utf-8"))["sources"]
    notices = ["Yamantaka native Windows component inventory", "", "The Cargo inventory includes normal and build dependencies for Windows x64; dev-only dependencies are excluded.", "Yamantaka project code is MIT licensed. Third-party components retain their own terms.", ""]
    for identity in sorted(reachable, key=lambda identity: (packages[identity]["name"], packages[identity]["version"])):
        item = packages[identity]
        ref = "pkg:cargo/" + item["name"] + "@" + item["version"]
        require(ref not in refs.values(), "Duplicate component identity requires source disambiguation")
        refs[identity] = ref
        component = {"type": "application" if identity == root_id else "library", "bom-ref": ref, "name": item["name"], "version": item["version"], "purl": ref}
        component["scope"]="required" if identity in runtime else "excluded"
        if identity not in runtime:
            component["properties"]=[{"name":"yamantaka:dependency-role","value":"build-or-proc-macro"}]
        if checksums.get((item["name"],item["version"])):
            component["hashes"]=[{"alg":"SHA-256","content":checksums[(item["name"],item["version"]) ]}]
        if item.get("license"):
            component["licenses"] = [{"expression": item["license"]}]
        else:
            component.setdefault("properties",[]).append({"name": "yamantaka:license-status", "value": "not-declared"})
        components.append(component)
        source = Path(item["manifest_path"]).parent
        if source.is_relative_to(ROOT / "third_party"):
            component.setdefault("properties", []).append({"name": "yamantaka:source-override", "value": source.relative_to(ROOT).as_posix()})
        workspace = source.is_relative_to(ROOT / "rust")
        candidates = set()
        if not workspace:
            for path in source.rglob("*"):
                if path.is_file() and (re.search(r"(^|[-_.])(license|licence|copying|notice|unlicense|copyright)([-._]|$)", path.name, re.I) or path.name.upper() in {"OFL.TXT", "UFL.TXT"}):
                    candidates.add(path)
            for folder in [source / "licenses", source / "LICENSES"]:
                if folder.is_dir():
                    candidates.update(path for path in folder.rglob("*") if path.is_file())
            if item.get("license_file"):
                candidates.add(source / item["license_file"])
            if not candidates:
                key = item["name"] + "@" + item["version"]
                upstream = license_sources.get(key)
                if upstream:
                    vcs = json.loads((source / ".cargo_vcs_info.json").read_text(encoding="utf-8"))
                    require(vcs["git"]["sha1"] == upstream["url"].split("/")[-2], "License source differs from the crate revision: " + key)
                    require(upstream["selectedLicense"] in (item.get("license") or "").split(" OR "), "Selected upstream terms differ from the crate license: " + key)
                    expected = upstream["sha256"]
                    require(re.fullmatch(r"[a-f0-9]{64}", expected), "Invalid license cache hash")
                    require(Path(upstream["filename"]).name == upstream["filename"], "Unexpected license filename")
                    cached = ROOT / "build/native/cargo-license-cache" / (expected + ".txt")
                    require(cached.is_file() and digest(cached) == expected, "Fetch the pinned upstream license text first: " + key)
                    destination = bundle / "licenses/cargo" / (item["name"] + "-" + item["version"]) / upstream["filename"]
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(cached, destination)
                    component.setdefault("externalReferences", []).append({"type": "license", "url": upstream["url"]})
                    component["licenses"] = [{"expression": upstream["selectedLicense"]}]
                else:
                    missing.append(key)
        for path in sorted(candidates):
            require(path.resolve().is_relative_to(source.resolve()), "License file escaped its package")
            require(path.stat().st_size <= 2 * 1024 * 1024, "Unexpectedly large license file")
            destination = bundle / "licenses/cargo" / (item["name"] + "-" + item["version"]) / path.relative_to(source)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, destination)
        terms = component.get("licenses", [{}])[0].get("expression", "not declared; see bundled notice")
        notices.append(item["name"] + " " + item["version"] + ": " + terms)
    require(not missing, "Dependency license text missing: " + ", ".join(missing))
    for identity in sorted(reachable, key=lambda identity: refs[identity]):
        dependencies.append({"ref": refs[identity], "dependsOn": sorted({refs[dep] for dep in edges(identity)})})
    for path in [pdfium / "LICENSE", *sorted((pdfium / "licenses").rglob("*"))]:
        if path.is_file():
            destination = bundle / "licenses/pdfium" / path.relative_to(pdfium)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, destination)
    font = bundle / "licenses/fonts/SourceSerif4-OFL.txt"
    font.parent.mkdir(parents=True)
    shutil.copyfile(ROOT / "assets/fonts/source-serif/LICENSE.md", font)
    sysroot = Path(command([str(Path(cargo).with_name("rustc.exe")), "--print", "sysroot"]).strip())
    rust_docs = sysroot / "share/doc/rust"
    rust_license = bundle / "licenses/rust-standard-library"
    rust_license.mkdir(parents=True)
    shutil.copyfile(rust_docs / "COPYRIGHT-library.html", rust_license / "COPYRIGHT-library.html")
    for path in (rust_docs / "licenses").glob("*.txt"):
        destination = rust_license / "licenses" / path.name
        destination.parent.mkdir(exist_ok=True)
        shutil.copyfile(path, destination)
    notices.extend(["", "Crate notice sets can also describe optional code or data not linked by this target.", "PDFium 151.0.7881.0: see licenses/pdfium.", "Source Serif 4: SIL Open Font License 1.1, see licenses/fonts.", "Rust standard library: see licenses/rust-standard-library; the toolchain notice catalog can also describe components for other platforms.", "Windows system libraries and statically linked Microsoft runtime components retain their platform terms."])
    (bundle / "THIRD-PARTY-NOTICES.txt").write_text("\n".join(notices) + "\n", encoding="utf-8")
    pdfium_ref, font_ref, rust_ref = "pkg:generic/pdfium@151.0.7881.0", "pkg:generic/source-serif-4@4", "pkg:generic/rust-standard-library@1.97.1"
    components.extend([{"type": "library", "bom-ref": pdfium_ref, "name": "PDFium", "version": "151.0.7881.0", "hashes": [{"alg": "SHA-256", "content": PDFIUM_SHA}], "licenses": [{"license": {"name": "Bundled PDFium third-party terms"}}]}, {"type": "library", "bom-ref": font_ref, "name": "Source Serif 4", "licenses": [{"license": {"id": "OFL-1.1"}}]}, {"type": "library", "bom-ref": rust_ref, "name": "Rust standard library", "version": "1.97.1", "licenses": [{"expression": "MIT OR Apache-2.0"}]}])
    next(item for item in dependencies if item["ref"] == refs[root_id])["dependsOn"].extend([pdfium_ref, font_ref, rust_ref])
    application = next(item for item in components if item["bom-ref"] == refs[root_id])
    json_file(bundle / "SBOM.cdx.json", {"bomFormat": "CycloneDX", "specVersion": "1.5", "version": 1, "metadata": {"component": application, "properties": [{"name": "yamantaka:target", "value": TARGET}, {"name": "yamantaka:status", "value": status}]}, "components": [item for item in components if item != application], "dependencies": dependencies})
    require(source_inputs() == sources, "Inputs changed during packaging; rebuild the candidate")
    files = [{"path": path.relative_to(bundle).as_posix(), "sha256": digest(path), "bytes": path.stat().st_size} for path in sorted(bundle.rglob("*")) if path.is_file()]
    runtime_changes = command(["git", "status", "--porcelain=v1", "--untracked-files=normal", "--", "rust", "third_party/pdfium-render", "third_party/egui", "third_party/epaint", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "assets/branding", "assets/fonts", "assets/branding/yamantaka.ico"])
    json_file(output / "package-manifest.json", {"format": "yamantaka-rust-windows-package-v1", "version": version, "target": TARGET, "status": status, "portable": portable, "applicationLicense": "MIT", "cargoLockSha256": digest(ROOT / "Cargo.lock"), "baseSourceCommit": command(["git", "rev-parse", "HEAD"]).strip(), "runtimeSourcesModified": bool(runtime_changes.strip()), "sources":sources, "authenticode":"not-verified", "files": files})
    archive = output / ("yamantaka-" + version + "-windows-x64" + ("-portable" if portable else "") + ".zip")
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zipped:
        for path in [output / "package-manifest.json", *sorted(bundle.rglob("*"))]:
            if path.is_file():
                info = zipfile.ZipInfo(path.relative_to(output).as_posix(), date_time=(1980, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                zipped.writestr(info, path.read_bytes(), compresslevel=9)
    (output / "SHA256SUMS.txt").write_text(digest(archive) + "  " + archive.name + "\n" + digest(output / "package-manifest.json") + "  package-manifest.json\n", encoding="ascii")
    return verify(output)


def verify(output):
    release_parent = (ROOT / "build/release").resolve()
    require(output != release_parent and output.is_relative_to(release_parent), "Use a named directory below build/release")
    manifest = json.loads((output / "package-manifest.json").read_text(encoding="utf-8"))
    require(manifest["format"] == "yamantaka-rust-windows-package-v1" and manifest["target"] == TARGET, "Unexpected package format")
    bundle = output / "yamantaka"
    expected = {item["path"]: item for item in manifest["files"]}
    require(len(expected) == len(manifest["files"]), "Duplicate file entries in the manifest")
    actual = {path.relative_to(bundle).as_posix() for path in bundle.rglob("*") if path.is_file()}
    require(actual == set(expected), "Bundle file list differs from the manifest")
    profile = os.environ.get("USERPROFILE", "")
    require(profile, "Build-profile environment is unavailable for the privacy check")
    profile_forms={profile,profile.replace(chr(92),"/"),json.dumps(profile)[1:-1]}
    private = [value.encode(encoding).lower() for value in profile_forms if value for encoding in ["utf-8", "utf-16-le"]]
    binaries = {}
    for name, item in expected.items():
        path = bundle / name
        require(digest(path) == item["sha256"] and path.stat().st_size == item["bytes"], "Bundle hash mismatch: " + name)
        require(path.suffix.lower() not in [".pdf", ".sqlite3", ".sqlite", ".db", ".pdb"], "Personal, database or debug file found in bundle")
        data = path.read_bytes().lower()
        require(not any(marker in data for marker in private), "Build-machine profile path found in bundle: " + name)
        if path.suffix.lower() in [".exe", ".dll"]:
            binaries[name] = pe_info(path)
    require(set(binaries) == {"yamantaka.exe", "pdfium.dll"}, "Unexpected native binary inventory")
    require(binaries["yamantaka.exe"]["subsystem"] == 2, "Expected a GUI Release executable")
    require(digest(bundle / "pdfium.dll") == PDFIUM_SHA, "PDFium is not the pinned binary")
    for binary in binaries.values():
        for library in binary["imports"]:
            require(not re.match(r"(?:vcruntime|msvcp|concrt)\d", library), "External Visual C++ runtime must be packaged explicitly: " + library)
            require(library in binaries or library.startswith(("api-ms-", "ext-ms-")) or (Path(os.environ["SystemRoot"]) / "System32" / library).is_file(), "Unresolved native import: " + library)
    sbom = json.loads((bundle / "SBOM.cdx.json").read_text())
    require(sbom["bomFormat"] == "CycloneDX" and sbom["specVersion"] == "1.5", "SBOM format mismatch")
    references = {sbom["metadata"]["component"]["bom-ref"], *(item["bom-ref"] for item in sbom["components"])}
    for dependency in sbom["dependencies"]:
        require(dependency["ref"] in references and set(dependency["dependsOn"]).issubset(references), "SBOM dependency refers to an absent component")
    archives=list(output.glob("yamantaka-*-windows-x64*.zip"))
    require(len(archives)==1,"Expected exactly one package archive")
    archive=archives[0]
    with zipfile.ZipFile(archive) as zipped:
        names = zipped.namelist()
        require(len(names) == len(set(names)) and set(names) == {"package-manifest.json", *("yamantaka/" + name for name in expected)}, "Archive inventory differs from bundle")
        for name in names:
            require(zipped.read(name) == (output / name).read_bytes(), "Archive bytes differ from verified files")
    sums = dict(line.split("  ", 1)[::-1] for line in (output / "SHA256SUMS.txt").read_text().splitlines())
    require(sums[archive.name] == digest(archive) and sums["package-manifest.json"] == digest(output / "package-manifest.json"), "Published checksums differ")
    report = {"verified": True, "status": manifest["status"], "version": manifest["version"], "files": len(expected), "components": len(references), "pdfiumHashPinned": True, "machineProfilePathsAbsent": True, "archiveMatchesBundle": True, "archiveSha256": digest(archive), "binaries": binaries, "applicationLicense": manifest["applicationLicense"]}
    json_file(output / "verification.json", report)
    print(json.dumps({key: report[key] for key in ["verified", "status", "version", "files", "components", "applicationLicense"]}))
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--portable", action="store_true")
    parser.add_argument("--verify-only", action="store_true")
    parser.add_argument("--cargo", default="cargo")
    args = parser.parse_args()
    output = args.output.resolve()
    verify(output) if args.verify_only else package(output, args.portable, args.cargo)
