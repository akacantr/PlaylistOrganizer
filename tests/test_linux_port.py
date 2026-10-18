import os
import sys
import subprocess
from pathlib import Path
from app import config

BASE_DIR = Path(__file__).resolve().parent.parent


class TestLinuxPort:
    def test_start_sh_exists_and_executable(self):
        start_sh = BASE_DIR / "start.sh"
        assert start_sh.exists(), "start.sh does not exist"
        assert os.access(start_sh, os.X_OK), "start.sh is not executable"

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

    def test_start_sh_help(self):
        res = subprocess.run([str(BASE_DIR / "start.sh"), "--help"], capture_output=True, text=True)
        assert res.returncode == 0
        assert "--host" in res.stdout
        assert "--port" in res.stdout
