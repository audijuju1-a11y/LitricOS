@echo off
rem LitricOS as an app: double-click to open the desktop in its own window.
rem
rem The window is a WSLg surface, so the console this starts has to stay open --
rem closing it is how the machine is powered off. Ctrl+C in this window sends
rem SIGINT to QEMU, which is an abrupt power cut; use the desktop's own shutdown
rem from inside the guest when it matters.
setlocal
title LitricOS
cd /d "%~dp0"
if not exist "tools\desktop-app.sh" (
  echo LitricOS: tools\desktop-app.sh not found next to this file.
  pause
  exit /b 1
)
wsl.exe -d archlinux -- bash -lc "exec bash tools/desktop-app.sh %*"
set rc=%ERRORLEVEL%
if not "%rc%"=="0" (
  echo.
  echo LitricOS exited with status %rc%.
  pause
)
endlocal
