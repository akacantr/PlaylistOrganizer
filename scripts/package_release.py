#!/usr/bin/env python3
"""
Release Packaging Script for Spotify Playlist Genre Organizer.
Builds standalone and portable distributions for Linux and Windows.
"""
import os
import sys
import shutil
import tarfile
import zipfile
import hashlib
import subprocess
from pathlib import Path

VERSION = "v1.0.0"
PROJECT_ROOT = Path(__file__).resolve().parent.parent
DIST_DIR = PROJECT_ROOT / "dist"
BUILD_DIR = PROJECT_ROOT / "build"
STAGING_DIR = PROJECT_ROOT / "pkg_staging"


def compute_sha256(filepath: Path) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def clean():
    print("🧹 Cleaning previous build and staging artifacts...")
    for d in (BUILD_DIR, STAGING_DIR):
        if d.exists():
            shutil.rmtree(d)
    DIST_DIR.mkdir(parents=True, exist_ok=True)


def build_pyinstaller_binary():
    print(f"📦 Compiling PyInstaller binary on {sys.platform}...")
    is_win = sys.platform == "win32"
    bin_name = "PlaylistOrganizer" if is_win else "playlist-organizer"
    sep = ";" if is_win else ":"

    cmd = [
        sys.executable, "-m", "PyInstaller",
        "--noconfirm",
        "--onedir",
        "--name", bin_name,
        "--add-data", f"app/templates{sep}app/templates",
        "--add-data", f"app/static{sep}app/static",
        "--add-data", f"data{sep}data",
        "run.py"
    ]
    subprocess.run(cmd, cwd=PROJECT_ROOT, check=True)
    return bin_name


def package_linux_release(bin_name: str):
    pkg_name = f"PlaylistOrganizer-{VERSION}-linux-x86_64"
    target_dir = STAGING_DIR / pkg_name
    target_dir.mkdir(parents=True, exist_ok=True)

    print(f"📁 Assembling Linux release in {target_dir}...")
    pyinstaller_output = DIST_DIR / bin_name
    if pyinstaller_output.exists():
        shutil.copytree(pyinstaller_output, target_dir, dirs_exist_ok=True)

    # Copy scripts and metadata
    metadata_files = (
        "run.py", "start.sh", "install-desktop.sh", "playlist-organizer.desktop",
        "playlist-organizer.service", "README.md", "LICENSE", ".env.example",
        "logo.svg", "requirements.txt"
    )
    for item in metadata_files:
        src = PROJECT_ROOT / item
        if src.exists():
            shutil.copy2(src, target_dir)

    # Ensure executable permissions
    for exe in (target_dir / bin_name, target_dir / "start.sh", target_dir / "install-desktop.sh"):
        if exe.exists():
            exe.chmod(0o755)

    # Create tar.gz archive
    archive_path = DIST_DIR / f"{pkg_name}.tar.gz"
    print(f"🗜️ Creating archive {archive_path.name}...")
    with tarfile.open(archive_path, "w:gz") as tar:
        tar.add(target_dir, arcname=pkg_name)

    sha = compute_sha256(archive_path)
    (DIST_DIR / f"{pkg_name}.tar.gz.sha256").write_text(f"{sha}  {archive_path.name}\n")
    print(f"✅ Linux release ready: {archive_path.name} (SHA256: {sha[:12]}...)")
    return archive_path


def package_windows_release(bin_name: str):
    pkg_name = f"PlaylistOrganizer-{VERSION}-windows-x64"
    target_dir = STAGING_DIR / pkg_name
    target_dir.mkdir(parents=True, exist_ok=True)

    print(f"📁 Assembling Windows binary release in {target_dir}...")
    pyinstaller_output = DIST_DIR / bin_name
    if pyinstaller_output.exists():
        shutil.copytree(pyinstaller_output, target_dir, dirs_exist_ok=True)

    for item in ("run.py", "start.bat", "README.md", "LICENSE", ".env.example", "logo.svg", "requirements.txt"):
        src = PROJECT_ROOT / item
        if src.exists():
            shutil.copy2(src, target_dir)

    archive_path = DIST_DIR / f"{pkg_name}.zip"
    print(f"🗜️ Creating archive {archive_path.name}...")
    with zipfile.ZipFile(archive_path, "w", zipfile.ZIP_DEFLATED) as zipf:
        for root, _, files in os.walk(target_dir):
            for file in files:
                abs_f = Path(root) / file
                rel_f = abs_f.relative_to(STAGING_DIR)
                zipf.write(abs_f, arcname=rel_f)

    sha = compute_sha256(archive_path)
    (DIST_DIR / f"{pkg_name}.zip.sha256").write_text(f"{sha}  {archive_path.name}\n")
    print(f"✅ Windows binary release ready: {archive_path.name} (SHA256: {sha[:12]}...)")
    return archive_path


def package_windows_portable():
    pkg_name = f"PlaylistOrganizer-{VERSION}-windows-portable"
    target_dir = STAGING_DIR / pkg_name
    target_dir.mkdir(parents=True, exist_ok=True)

    print(f"📁 Assembling Windows portable release in {target_dir}...")
    shutil.copytree(PROJECT_ROOT / "app", target_dir / "app", dirs_exist_ok=True)
    shutil.copytree(PROJECT_ROOT / "data", target_dir / "data", dirs_exist_ok=True)

    for item in ("run.py", "start.bat", "README.md", "LICENSE", ".env.example", "logo.svg", "requirements.txt"):
        src = PROJECT_ROOT / item
        if src.exists():
            shutil.copy2(src, target_dir)

    archive_path = DIST_DIR / f"{pkg_name}.zip"
    print(f"🗜️ Creating archive {archive_path.name}...")
    with zipfile.ZipFile(archive_path, "w", zipfile.ZIP_DEFLATED) as zipf:
        for root, _, files in os.walk(target_dir):
            for file in files:
                abs_f = Path(root) / file
                rel_f = abs_f.relative_to(STAGING_DIR)
                zipf.write(abs_f, arcname=rel_f)

    sha = compute_sha256(archive_path)
    (DIST_DIR / f"{pkg_name}.zip.sha256").write_text(f"{sha}  {archive_path.name}\n")
    print(f"✅ Windows portable release ready: {archive_path.name} (SHA256: {sha[:12]}...)")
    return archive_path


def main():
    clean()
    bin_name = build_pyinstaller_binary()
    if sys.platform != "win32":
        package_linux_release(bin_name)
    else:
        package_windows_release(bin_name)
    package_windows_portable()

    # Generate unified SHA256SUMS.txt
    print("📝 Generating unified SHA256SUMS.txt...")
    sums = []
    for f in DIST_DIR.glob("*"):
        if f.suffix in (".gz", ".zip"):
            sums.append(f"{compute_sha256(f)}  {f.name}\n")
    (DIST_DIR / "SHA256SUMS.txt").write_text("".join(sorted(sums)))
    print("🎉 All packaging completed successfully!")


if __name__ == "__main__":
    main()
