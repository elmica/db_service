# Aldelo → Convex sync service

Python service that reads orders from an Aldelo Jet/MDB database **read-only** and syncs them to Convex. Designed to run under NSSM on Windows with exponential backoff when the database is locked.

- **Read-only**: Never writes to the .mdb file.
- **Incremental**: Tracks `last_order_id` in a local cursor file; only new orders are read and sent.
- **Resilient**: Configurable poll interval; quick retries then exponential backoff on MDB open/read errors; cursor advanced only after successful Convex send.

See [NSSM.md](NSSM.md) for installing and running as a Windows service.

## Setup

**Option A: Python script**
1. Install Python 3.9+ on the POS machine.
2. Install dependencies: `pip install -r requirements.txt`

**Option B: Standalone executable**
1. Download `aldelo-convex-sync.exe` from the [GitHub Actions](../../actions) artifact (Build workflow).
2. Copy it to your POS machine (e.g. `C:\AldeloSync\`).

**Config (both options)**
3. Copy `config.toml.example` (in this folder) to `config.toml` and set:
   - `mdb_path`: full path to the Aldelo .mdb file
   - `convex_url`: your Convex ingestion endpoint URL
   - `cursor_path`: path for the cursor file (e.g. `C:\AldeloSync\cursor.json`)

4. Optionally set `CONVEX_URL` and `CONVEX_API_KEY` in the environment.

## Run

**Option A: Python script**
```bash
python aldelo_convex_sync.py
```

**Option B: Standalone executable** (from GitHub Actions artifact)
```bash
aldelo-convex-sync.exe
```

Pass a config path as first argument if needed:
```bash
python aldelo_convex_sync.py C:\AldeloSync\config.toml
aldelo-convex-sync.exe C:\AldeloSync\config.toml
```

## Convex

Your Convex backend must expose an HTTP endpoint that accepts POST with an `OrderBatch` JSON body (headers, transactions, payments, refunds). See [CONVEX_ALDELO_SETUP.md](../CONVEX_ALDELO_SETUP.md) for the schema and mutation.
