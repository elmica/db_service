# Running as a Windows service with NSSM

This service is designed to run on the POS machine (Windows) and read from the Aldelo Jet database in a **read-only** way, then push order data to Convex.

## Prerequisites

- **Windows** with the Aldelo POS Jet (.mdb) database path known.
- **Microsoft Access Database Engine** (ODBC driver) installed. Use the same bitness as the built executable (e.g. 64-bit driver for 64-bit build).
  - Download: [Microsoft Access Database Engine Redistributable](https://www.microsoft.com/en-us/download/details.aspx?id=54920)
- **NSSM** (Non-Sucking Service Manager): [nssm.cc](https://nssm.cc) — extract and use `nssm.exe` (64-bit from win64 folder if you built a 64-bit exe).

## Build (on Windows or cross-compile)

```bash
cargo build --release
```

Binary: `target\release\aldelo-convex-sync.exe`

## Install the service

1. Create a directory for config and cursor, e.g. `C:\AldeloSync\`.
2. Copy `config.toml.example` (from repo root) to `C:\AldeloSync\config.toml` and edit:
   - `mdb_path`: full path to the Aldelo .mdb file.
   - `convex_url`: your Convex ingestion endpoint URL.
   - `cursor_path`: e.g. `C:\AldeloSync\cursor.json`.
3. Install the service (run Command Prompt or PowerShell as Administrator):

```batch
nssm install AldeloConvexSync "C:\path\to\aldelo-convex-sync.exe"
```

4. In NSSM GUI (or via command line):
   - **Path**: path to `aldelo-convex-sync.exe`.
   - **Startup directory**: `C:\AldeloSync` (or where config.toml and cursor live).
   - **Arguments**: optional — if you pass a config path, use: `C:\AldeloSync\config.toml`. Otherwise the app will look for `config.toml` in the startup directory.
   - **Environment**: add if needed:
     - `CONVEX_URL=https://your-deployment.convex.site/api/...`
     - `CONVEX_API_KEY=your-secret-key`

5. **Restart** tab: set “Restart service after failure” and delay (e.g. 5 seconds).

6. **I/O** tab (optional): redirect stdout/stderr to log files for debugging, e.g. `C:\AldeloSync\stdout.log` and `C:\AldeloSync\stderr.log`.

7. Start the service:

```batch
nssm start AldeloConvexSync
```

## Permissions

- The service account (usually Local System or a dedicated user) must have:
  - **Read** access to the .mdb file and its folder.
  - **Write** access to the directory containing the cursor file (so it can create/update `cursor.json`).
- Do **not** grant write access to the .mdb file or Aldelo data directory; the service must only read.

## Uninstall

```batch
nssm stop AldeloConvexSync
nssm remove AldeloConvexSync confirm
```
