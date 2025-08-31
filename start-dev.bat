@echo off
REM Starts backend (NestJS) and frontend (Svelte/Vite) in separate windows.
REM Run this file from the repo root by double-clicking or from a terminal.

setlocal ENABLEDELAYEDEXPANSION

call :detectPM backend BACKEND_PM
call :detectPM frontend FRONTEND_PM

echo Detected package manager (backend): %BACKEND_PM%
echo Detected package manager (frontend): %FRONTEND_PM%

echo Installing dependencies if needed...
call :install backend %BACKEND_PM%
call :install frontend %FRONTEND_PM%

REM Set backend environment variables (edit as needed)
set "SERIAL_PORT=COM4"
set "SERIAL_BAUD=115200"
echo Using SERIAL_PORT=%SERIAL_PORT% SERIAL_BAUD=%SERIAL_BAUD%

echo Launching backend (watch)...
start "backend" cmd /k "cd /d %~dp0backend && set SERIAL_PORT=%SERIAL_PORT% && set SERIAL_BAUD=%SERIAL_BAUD% && %BACKEND_PM% run start:dev"

echo Launching frontend (Vite dev)...
start "frontend" cmd /k "cd /d %~dp0frontend && %FRONTEND_PM% run dev"

echo Both processes started in their own windows.
exit /b 0

:detectPM
REM %1 = folder, %2 = out var name
set FOLDER=%1
set OUTVAR=%2
set PM=
if exist "%FOLDER%\pnpm-lock.yaml" set PM=pnpm
if exist "%FOLDER%\yarn.lock" set PM=yarn
if exist "%FOLDER%\package-lock.json" set PM=npm
if not defined PM set PM=npm
for /f "delims=" %%A in ("%PM%") do set %OUTVAR%=%%A
exit /b 0

:install
REM %1 = folder, %2 = pm
pushd %1 >nul 2>&1
if /i "%2"=="npm" (
  if not exist node_modules (echo Installing %1 deps with npm... & npm install --no-audit --no-fund)
) else if /i "%2"=="pnpm" (
  if not exist node_modules (echo Installing %1 deps with pnpm... & pnpm install)
) else if /i "%2"=="yarn" (
  if not exist node_modules (echo Installing %1 deps with yarn... & yarn install)
)
popd >nul 2>&1
exit /b 0
