@echo off
chcp 65001 >nul
title Spotify Playlist Genre Organizer

echo ========================================================
echo  🎵 Spotify Playlist Genre Organizer Starting...
echo ========================================================

cd /d "%~dp0"

if exist "%~dp0PlaylistOrganizer.exe" (
    echo Starting standalone executable: PlaylistOrganizer.exe
    "%~dp0PlaylistOrganizer.exe" %*
    if %errorlevel% neq 0 (
        echo.
        echo Application exited with an error.
        pause
    )
    exit /b %errorlevel%
)

set PYTHON_EXE=
if exist "%~dp0.venv\Scripts\python.exe" (
    set "PYTHON_EXE=%~dp0.venv\Scripts\python.exe"
) else if exist "%LOCALAPPDATA%\Python\bin\python.exe" (
    set "PYTHON_EXE=%LOCALAPPDATA%\Python\bin\python.exe"
) else (
    where py >nul 2>nul
    if %errorlevel% equ 0 (
        set "PYTHON_EXE=py"
    ) else (
        where python >nul 2>nul
        if %errorlevel% equ 0 (
            set "PYTHON_EXE=python"
        )
    )
)

if "%PYTHON_EXE%"=="" (
    echo [ERROR] Python executable not found!
    echo Please install Python 3.9+ from https://www.python.org/
    pause
    exit /b 1
)

echo Using Python: %PYTHON_EXE%
"%PYTHON_EXE%" run.py %*

if %errorlevel% neq 0 (
    echo.
    echo Application exited with an error.
    pause
)
