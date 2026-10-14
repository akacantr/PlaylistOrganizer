@echo off
chcp 65001 >nul
title Spotify Playlist Genre Organizer

echo ========================================================
echo  🎵 Spotify Playlist Genre Organizer Baslatiliyor...
echo ========================================================

set PYTHON_EXE=
if exist "%LOCALAPPDATA%\Python\bin\python.exe" (
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
    echo [HATA] Python calistirilabilir dosyasi bulunamadi!
    pause
    exit /b 1
)

echo Python bulundu: %PYTHON_EXE%
"%PYTHON_EXE%" run.py

if %errorlevel% neq 0 (
    echo.
    echo Bir hata olustu.
    pause
)
