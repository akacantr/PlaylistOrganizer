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


def package_appimage(bin_name: str = "playlist-organizer"):
    print(f"📦 Assembling Linux AppImage bundle...")
    pkg_name = f"PlaylistOrganizer-{VERSION}-linux-x86_64"
    appdir = STAGING_DIR / "AppDir"
    if appdir.exists():
        shutil.rmtree(appdir)

    usr_bin = appdir / "usr" / "bin"
    usr_bin.mkdir(parents=True, exist_ok=True)
    (appdir / "usr" / "share" / "applications").mkdir(parents=True, exist_ok=True)
    (appdir / "usr" / "share" / "icons" / "hicolor" / "scalable" / "apps").mkdir(parents=True, exist_ok=True)

    pyinstaller_output = DIST_DIR / bin_name
    if pyinstaller_output.exists():
        shutil.copytree(pyinstaller_output, usr_bin, dirs_exist_ok=True)

    for item in ("run.py", "README.md", "LICENSE", ".env.example", "logo.svg", "requirements.txt"):
        src = PROJECT_ROOT / item
        if src.exists():
            shutil.copy2(src, usr_bin)

    desktop_content = """[Desktop Entry]
Version=1.0
Type=Application
Name=Spotify Playlist Genre Organizer
GenericName=Playlist Organizer
Comment=Organize, filter, and split your Spotify playlists by genre
Exec=playlist-organizer
Icon=playlist-organizer
Terminal=false
Categories=AudioVideo;Audio;Utility;
Keywords=spotify;playlist;music;genre;organizer;
StartupNotify=true
"""
    (appdir / "playlist-organizer.desktop").write_text(desktop_content, encoding="utf-8")
    (appdir / "usr" / "share" / "applications" / "playlist-organizer.desktop").write_text(desktop_content, encoding="utf-8")

    logo = PROJECT_ROOT / "logo.svg"
    if logo.exists():
        shutil.copy2(logo, appdir / "playlist-organizer.svg")
        shutil.copy2(logo, appdir / "usr" / "share" / "icons" / "hicolor" / "scalable" / "apps" / "playlist-organizer.svg")

    apprun_content = """#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export APPDIR="${HERE}"
exec "${HERE}/usr/bin/playlist-organizer" "$@"
"""
    apprun_path = appdir / "AppRun"
    apprun_path.write_text(apprun_content, encoding="utf-8")
    apprun_path.chmod(0o755)

    bin_path = usr_bin / bin_name
    if bin_path.exists():
        bin_path.chmod(0o755)

    appimage_file = DIST_DIR / f"{pkg_name}.AppImage"

    # Search for appimagetool
    appimagetool = shutil.which("appimagetool")
    if not appimagetool:
        candidates = [
            Path("/tmp/squashfs-root/AppRun"),
            PROJECT_ROOT / "squashfs-root" / "AppRun",
            PROJECT_ROOT / "appimagetool"
        ]
        for c in candidates:
            if c.exists() and os.access(c, os.X_OK):
                appimagetool = str(c)
                break

    if not appimagetool:
        print("⚠️ appimagetool not found. Staging AppDir ready, skipping AppImage squashfs build.")
        return None

    print(f"🗜️ Building AppImage using {appimagetool}...")
    env = dict(os.environ, ARCH="x86_64")
    subprocess.run([appimagetool, str(appdir), str(appimage_file)], env=env, check=True)

    sha = compute_sha256(appimage_file)
    (DIST_DIR / f"{pkg_name}.AppImage.sha256").write_text(f"{sha}  {appimage_file.name}\n")
    print(f"✅ Linux AppImage ready: {appimage_file.name} (SHA256: {sha[:12]}...)")
    return appimage_file


def package_windows_release(bin_name: str = "PlaylistOrganizer"):
    pkg_name = f"PlaylistOrganizer-{VERSION}-windows-x64"
    target_dir = STAGING_DIR / pkg_name
    target_dir.mkdir(parents=True, exist_ok=True)

    # Standalone onefile .exe
    exe_name = f"{pkg_name}.exe"
    exe_path = DIST_DIR / exe_name
    try:
        print(f"📦 Compiling standalone onefile Windows executable...")
        cmd = [
            sys.executable, "-m", "PyInstaller",
            "--noconfirm",
            "--onefile",
            "--name", pkg_name,
            "--add-data", "app/templates;app/templates",
            "--add-data", "app/static;app/static",
            "--add-data", "data;data",
            "run.py"
        ]
        subprocess.run(cmd, cwd=PROJECT_ROOT, check=True)
        if exe_path.exists():
            sha = compute_sha256(exe_path)
            (DIST_DIR / f"{exe_name}.sha256").write_text(f"{sha}  {exe_name}\n")
            print(f"✅ Windows standalone executable ready: {exe_name} (SHA256: {sha[:12]}...)")
    except Exception as e:
        print(f"⚠️ Standalone onefile compilation notice: {e}")

    print(f"📁 Assembling Windows directory release in {target_dir}...")
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
        package_appimage(bin_name)
    else:
        package_windows_release(bin_name)
    package_windows_portable()

    # Generate unified SHA256SUMS.txt
    print("📝 Generating unified SHA256SUMS.txt...")
    sums = []
    for f in DIST_DIR.glob("*"):
        if f.suffix in (".gz", ".zip", ".AppImage", ".exe"):
            sums.append(f"{compute_sha256(f)}  {f.name}\n")
    (DIST_DIR / "SHA256SUMS.txt").write_text("".join(sorted(sums)))
    print("🎉 All packaging completed successfully!")


if __name__ == "__main__":
    main()
