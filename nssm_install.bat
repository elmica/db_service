@echo off
REM Install Aldelo Convex Sync (Python) as a Windows service using NSSM.
REM Run as Administrator. Adjust paths to match your setup.

set SERVICE_NAME=AldeloConvexSync
REM Option A: Use exe (download from GitHub Actions artifact)
set EXE_PATH=C:\AldeloSync\aldelo-convex-sync.exe
REM Option B: Use Python script
set PYTHON_PATH=python
set SCRIPT_PATH=%~dp0sync_service\aldelo_convex_sync.py
set WORK_DIR=C:\AldeloSync
set CONFIG_PATH=C:\AldeloSync\config.toml

REM Check NSSM is on PATH or set full path, e.g. set NSSM=C:\tools\nssm.exe
where nssm >nul 2>&1 || set NSSM=nssm.exe

echo Installing service %SERVICE_NAME%
echo Exe: %EXE_PATH% (or use Python: %PYTHON_PATH% %SCRIPT_PATH%)
echo Working directory: %WORK_DIR%

REM Use exe if it exists, else Python script
if exist "%EXE_PATH%" (
  %NSSM% install %SERVICE_NAME% "%EXE_PATH%"
) else (
  %NSSM% install %SERVICE_NAME% "%PYTHON_PATH%" "%SCRIPT_PATH%"
)
%NSSM% set %SERVICE_NAME% AppDirectory "%WORK_DIR%"
REM Optional: set CONVEX_URL and CONVEX_API_KEY in AppEnvironment
REM %NSSM% set %SERVICE_NAME% AppEnvironmentExtra CONVEX_URL=https://... CONVEX_API_KEY=... ALDELO_SYNC_CONFIG=%CONFIG_PATH%

echo.
echo Configure Restart and I/O in: nssm edit %SERVICE_NAME%
echo Then: nssm start %SERVICE_NAME%
