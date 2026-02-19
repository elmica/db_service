@echo off
REM Install Aldelo Convex Sync as a Windows service using NSSM.
REM Run as Administrator. Adjust paths to match your setup.

set SERVICE_NAME=AldeloConvexSync
set EXE_PATH=%~dp0sync_service\target\release\aldelo-convex-sync.exe
set WORK_DIR=C:\AldeloSync
set CONFIG_PATH=C:\AldeloSync\config.toml

REM Check NSSM is on PATH or set full path, e.g. set NSSM=C:\tools\nssm.exe
where nssm >nul 2>&1 || set NSSM=nssm.exe

echo Installing service %SERVICE_NAME%
echo Exe: %EXE_PATH%
echo Working directory: %WORK_DIR%

%NSSM% install %SERVICE_NAME% "%EXE_PATH%"
%NSSM% set %SERVICE_NAME% AppDirectory "%WORK_DIR%"
%NSSM% set %SERVICE_NAME% AppParameters "%CONFIG_PATH%"
REM Optional: set CONVEX_URL and CONVEX_API_KEY in AppEnvironment
REM %NSSM% set %SERVICE_NAME% AppEnvironmentExtra CONVEX_URL=https://... CONVEX_API_KEY=...

echo.
echo Configure Restart and I/O in: nssm edit %SERVICE_NAME%
echo Then: nssm start %SERVICE_NAME%
