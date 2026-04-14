@echo off
setlocal

cd /d "%~dp0"

where wsl >nul 2>nul
if errorlevel 1 (
    echo [error] WSL is not available on this machine.
    pause
    exit /b 1
)

echo [info] Starting local ARM64 test in WSL...
wsl bash -lc "cd \"$(wslpath '%CD%')\" && ./test_arm64.sh"
set "STATUS=%ERRORLEVEL%"

echo.
echo [info] Launcher exit code: %STATUS%
pause
exit /b %STATUS%