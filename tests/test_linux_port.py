import os
import sys
import subprocess
from pathlib import Path
import pytest
from app import config

BASE_DIR = Path(__file__).resolve().parent.parent


class TestLinuxPort:
    @pytest.mark.skipif(sys.platform == "win32", reason="POSIX executable bit checks apply to POSIX only")
    def test_start_sh_exists_and_executable(self):
        start_sh = BASE_DIR / "start.sh"
        assert start_sh.exists(), "start.sh does not exist"
        assert os.access(start_sh, os.X_OK), "start.sh is not executable"

    @pytest.mark.skipif(sys.platform == "win32", reason="POSIX executable bit checks apply to POSIX only")
    def test_install_desktop_sh_exists_and_executable(self):
        script = BASE_DIR / "install-desktop.sh"
        assert script.exists(), "install-desktop.sh does not exist"
        assert os.access(script, os.X_OK), "install-desktop.sh is not executable"

    def test_desktop_file_syntax(self):
        desktop_file = BASE_DIR / "playlist-organizer.desktop"
        assert desktop_file.exists(), "playlist-organizer.desktop does not exist"
        content = desktop_file.read_text(encoding="utf-8")
        assert "[Desktop Entry]" in content
        assert "Type=Application" in content
        assert "Exec=" in content
        assert "Icon=" in content

    def test_systemd_service_syntax(self):
        service_file = BASE_DIR / "playlist-organizer.service"
        assert service_file.exists(), "playlist-organizer.service does not exist"
        content = service_file.read_text(encoding="utf-8")
        assert "[Unit]" in content
        assert "[Service]" in content
        assert "[Install]" in content

    def test_dockerfile_and_compose_exist(self):
        assert (BASE_DIR / "Dockerfile").exists()
        assert (BASE_DIR / "docker-compose.yml").exists()

    def test_run_py_help(self):
        result = subprocess.run(
            [sys.executable, str(BASE_DIR / "run.py"), "--help"],
            capture_output=True,
            text=True
        )
        assert result.returncode == 0
        assert "--host" in result.stdout
        assert "--port" in result.stdout
        assert "--no-browser" in result.stdout

    def test_config_base_dir(self):
        base_dir = config.get_base_dir()
        assert base_dir.exists()
        assert (base_dir / "app").exists()

    @pytest.mark.skipif(sys.platform == "win32", reason="Linux shell script execution requires POSIX environment")
    def test_start_sh_binary_dispatch(self, tmp_path):
        import shutil
        script = tmp_path / "start.sh"
        shutil.copy2(BASE_DIR / "start.sh", script)
        script.chmod(0o755)

        mock_bin = tmp_path / "playlist-organizer"
        mock_bin.write_text("#!/bin/sh\necho 'MOCK_BINARY_RUN: '$@\n")
        mock_bin.chmod(0o755)

        res = subprocess.run([str(script), "--test-arg", "123"], capture_output=True, text=True)
        assert res.returncode == 0
        assert "MOCK_BINARY_RUN: --test-arg 123" in res.stdout

    def test_start_bat_contains_binary_dispatch(self):
        bat = (BASE_DIR / "start.bat").read_text(encoding="utf-8")
        assert "PlaylistOrganizer.exe" in bat
        assert "%*" in bat

    def test_install_desktop_uninstall_flag(self):
        content = (BASE_DIR / "install-desktop.sh").read_text(encoding="utf-8")
        assert "--uninstall" in content
        assert "rm -f" in content

    @pytest.mark.skipif(sys.platform == "win32", reason="Linux shell script execution requires POSIX environment")
    def test_start_sh_help(self):
        res = subprocess.run([str(BASE_DIR / "start.sh"), "--help"], capture_output=True, text=True)
        assert res.returncode == 0
        assert "--host" in res.stdout
        assert "--port" in res.stdout

    def test_appimage_packaging_support(self):
        from scripts import package_release
        assert hasattr(package_release, "package_appimage")
        assert hasattr(package_release, "package_windows_release")
        assert hasattr(package_release, "package_linux_release")

    def test_release_workflow_structure(self):
        workflow = (BASE_DIR / ".github" / "workflows" / "release.yml").read_text(encoding="utf-8")
        assert "build-linux:" in workflow
        assert "build-windows:" in workflow
        assert "publish-release:" in workflow
        assert "PlaylistOrganizer-*-windows-x64.exe" in workflow
        assert "PlaylistOrganizer-*-linux-x86_64.AppImage" in workflow
        assert "SHA256SUMS.txt" in workflow
        assert "softprops/action-gh-release@v2" in workflow

    def test_package_release_version_override(self, monkeypatch):
        import importlib
        from scripts import package_release
        monkeypatch.setenv("RELEASE_TAG", "v2.5.0")
        importlib.reload(package_release)
        assert package_release.VERSION == "v2.5.0"
        monkeypatch.delenv("RELEASE_TAG", raising=False)
        importlib.reload(package_release)
        assert package_release.VERSION == "v1.0.0"

    def test_package_windows_release_with_binary(self, monkeypatch, tmp_path):
        import shutil
        from scripts import package_release
        pkg_name = f"PlaylistOrganizer-{package_release.VERSION}-windows-x64"
        exe_name = f"{pkg_name}.exe"
        zip_name = f"{pkg_name}.zip"

        rust_target = BASE_DIR / "target" / "release"
        rust_target.mkdir(parents=True, exist_ok=True)
        fake_exe = rust_target / "playlist-organizer.exe"
        fake_exe.write_text("MZ_MOCK_WINDOWS_PE")
        try:
            zip_res = package_release.package_windows_release()
            assert zip_res.exists()
            assert (package_release.DIST_DIR / exe_name).exists()
            assert (package_release.DIST_DIR / f"{exe_name}.sha256").exists()
            assert (package_release.DIST_DIR / f"{zip_name}.sha256").exists()
        finally:
            if fake_exe.exists():
                fake_exe.unlink()
            for artifact in (
                package_release.DIST_DIR / exe_name,
                package_release.DIST_DIR / f"{exe_name}.sha256",
                package_release.DIST_DIR / zip_name,
                package_release.DIST_DIR / f"{zip_name}.sha256",
            ):
                if artifact.exists():
                    artifact.unlink()
            staging_dir = package_release.STAGING_DIR / pkg_name
            if staging_dir.exists():
                shutil.rmtree(staging_dir, ignore_errors=True)

    def test_package_linux_release_clean(self):
        import shutil
        from scripts import package_release
        pkg_name = f"PlaylistOrganizer-{package_release.VERSION}-linux-x86_64"
        tar_name = f"{pkg_name}.tar.gz"
        try:
            tar_res = package_release.package_linux_release("playlist-organizer")
            assert tar_res.exists()
            assert (package_release.DIST_DIR / f"{tar_name}.sha256").exists()
        finally:
            for artifact in (
                package_release.DIST_DIR / tar_name,
                package_release.DIST_DIR / f"{tar_name}.sha256",
            ):
                if artifact.exists():
                    artifact.unlink()
            staging_dir = package_release.STAGING_DIR / pkg_name
            if staging_dir.exists():
                shutil.rmtree(staging_dir, ignore_errors=True)

    def test_package_windows_portable_clean(self):
        import shutil
        from scripts import package_release
        pkg_name = f"PlaylistOrganizer-{package_release.VERSION}-windows-portable"
        zip_name = f"{pkg_name}.zip"
        try:
            zip_res = package_release.package_windows_portable()
            assert zip_res.exists()
            assert (package_release.DIST_DIR / f"{zip_name}.sha256").exists()
        finally:
            for artifact in (
                package_release.DIST_DIR / zip_name,
                package_release.DIST_DIR / f"{zip_name}.sha256",
            ):
                if artifact.exists():
                    artifact.unlink()
            staging_dir = package_release.STAGING_DIR / pkg_name
            if staging_dir.exists():
                shutil.rmtree(staging_dir, ignore_errors=True)

