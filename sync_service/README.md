# Aldelo → Convex sync service

Windows service (Rust) that reads orders from an Aldelo Jet/MDB database **read-only** and syncs them to Convex. Designed to run under NSSM with exponential backoff when the database is locked.

- **Read-only**: Never writes to the .mdb file.
- **Incremental**: Tracks `last_order_id` in a local cursor file; only new orders are read and sent.
- **Resilient**: Configurable poll interval; quick retries then exponential backoff on MDB open/read errors; cursor advanced only after successful Convex send.

See [NSSM.md](NSSM.md) for installing and running as a Windows service.

## Build

**On Windows** (or with a Windows VM):

```bash
cargo build --release
```

**From a Mac or Linux** (e.g. Apple Silicon): push to GitHub and run the **Build Windows sync service** workflow (Actions tab). You can trigger it manually via “Run workflow” or on push/PR when `sync_service/` changes. After the run, download the `aldelo-convex-sync-windows-x64` artifact to get `aldelo-convex-sync.exe`. Copy it to your POS machine and use NSSM to install the service.

On non-Windows hosts the crate builds but MDB read returns an error at runtime (ODBC is Windows-only); the CI build runs on `windows-latest` so the artifact is a working Windows executable.

## Config

Copy `../config.toml.example` to `config.toml` (or pass path as first argument). Set `CONVEX_URL` and optionally `CONVEX_API_KEY` in the environment.

## Convex

Your Convex backend must expose an HTTP endpoint that accepts POST with an `OrderBatch` JSON body (see `order_data.rs` for the shape). The service sends batches after each successful read; implement idempotent upserts by `OrderID` and related IDs.
