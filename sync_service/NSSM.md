# Running as a Windows service with NSSM

This service is designed to run on the POS machine (Windows) and read from the Aldelo Jet database in a **read-only** way, then push order data to Convex.

## Prerequisites

- **Windows** with the Aldelo POS Jet (.mdb) database path known.
- **Python 3.9+** installed and in PATH.
- **Microsoft Access Database Engine** (ODBC driver) installed. Use the same bitness as Python (e.g. 64-bit driver for 64-bit Python).
  - Download: [Microsoft Access Database Engine Redistributable](https://www.microsoft.com/en-us/download/details.aspx?id=54920)
- **NSSM** (Non-Sucking Service Manager): [nssm.cc](https://nssm.cc) — extract and use `nssm.exe` (64-bit from win64 folder for 64-bit Python).

## Install dependencies

```bash
cd sync_service
pip install -r requirements.txt
```

## Install the service

1. Create a directory for config and cursor, e.g. `C:\AldeloSync\`.
2. Copy `sync_service/config.toml.example` to `C:\AldeloSync\config.toml` and edit:
   - `mdb_path`: full path to the Aldelo .mdb file.
   - `convex_url`: your Convex ingestion endpoint URL.
   - `cursor_path`: e.g. `C:\AldeloSync\cursor.json`.
3. Install the service (run Command Prompt or PowerShell as Administrator):

**Option A: Standalone executable** (recommended — no Python needed)
```batch
nssm install AldeloConvexSync "C:\AldeloSync\aldelo-convex-sync.exe"
```

**Option B: Python script**
```batch
nssm install AldeloConvexSync "C:\Python311\python.exe" "C:\path\to\sync_service\aldelo_convex_sync.py"
```

Adjust paths to match your setup.

4. In NSSM GUI (or via command line):
   - **Path**: `python.exe` (or full path).
   - **Arguments**: full path to `aldelo_convex_sync.py`.
   - **Startup directory**: `C:\AldeloSync` (or where config.toml and cursor live). The script looks for `config.toml` in the current directory by default; set `ALDELO_SYNC_CONFIG=C:\AldeloSync\config.toml` in environment if needed.
   - **Environment**: add if needed:
     - `CONVEX_URL=https://your-deployment.convex.site/api/...`
     - `CONVEX_API_KEY=your-secret-key`
     - `ALDELO_SYNC_CONFIG=C:\AldeloSync\config.toml`

5. **Restart** tab: set "Restart service after failure" and delay (e.g. 5 seconds).

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

## Troubleshooting: "Disk or network error" (-1022)

If you get `[Microsoft][ODBC Microsoft Access Driver] Disk or network error`:

1. **Run manually first** — Open Command Prompt, `cd` to `C:\AldeloSync`, run `python aldelo_convex_sync.py`. If it works manually but fails as a service, it's a service-account issue.

2. **Use a user account instead of Local System** — The Jet driver creates temp files. Local System may have permission issues. In NSSM: **Log on** tab → select "This account" → enter a user account that has read access to the .mdb (e.g. the same user who runs Aldelo or the Python bridge).

3. **Verify the path** — Ensure `mdb_path` in config.toml is an absolute path and the file exists: `C:\Chatham\ChathamSandwich.mdb`. Use double backslashes in TOML: `mdb_path = "C:\\Chatham\\ChathamSandwich.mdb"`.

4. **File locking** — If Aldelo has the .mdb open exclusively, the sync cannot connect. Ensure Aldelo allows shared read access, or run the sync when Aldelo is idle.

## Uninstall

```batch
nssm stop AldeloConvexSync
nssm remove AldeloConvexSync confirm
```
