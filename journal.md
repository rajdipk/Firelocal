# FireLocal Development Journal

This journal tracks key implementation milestones and technical decisions.

## Milestone 0: Repo & Skeleton
**Status**: Complete
**Date**: 2025-12-06
...

## Milestone 1: Minimal Store (Phase 1)
**Status**: Complete
**Date**: 2025-12-06
...

## Milestone 2: Query + Index (Phase 2)
**Status**: Complete
**Date**: 2025-12-06
...

## Milestone 3: API Parity + Snapshots (Phase 3)
**Status**: Complete
**Date**: 2025-12-06
...

## Milestone 4: Rules Engine
**Status**: Complete
**Date**: 2025-12-06
...

## Milestone 5: Sync Adapter (Phase 4)
**Status**: Complete
**Date**: 2025-12-06

### Summary
Implemented a pluggable sync architecture with `SyncManager` and `RemoteStore` trait. Added a `FirebaseClient` using `reqwest` for Firestore interaction.

### Architecture
- **Abstraction**: `Box<dyn RemoteStore>` allows swapping the backend (e.g., Mock for tests, Firebase for prod).
- **Client**: `sync/firebase.rs` uses `reqwest` (blocking) to hitting standard Firestore REST endpoints (`patch` for set/update, `get` for retrieve).
- **Core Integration**: `sync_push` reads local and pushes remote. `sync_pull` reads remote and uses `put` to write local (triggering index/WAL/listeners).

### Key Changes
- Added dependencies: `reqwest` (blocking/json), `dotenv`.
- `FireLocal` core now holds an optional (swappable) `SyncManager`.
- Added `tests/sync_test.rs` using `MockRemoteStore` to verify the sync flow logic without network interaction.

### Developer Notes
- **Authentication**: `FirebaseClient` expects `FIREBASE_PROJECT_ID` and `FIREBASE_AUTH_TOKEN` in `.env`.
- **Mapping**: Simplistic JSON mapping implemented. Complex Firestore types (geo, ref, timestamp) are not fully handled in type mapping yet.

## Milestone 6: CLI & Release
**Status**: Complete
**Date**: 2025-12-06

### Summary
Developed `firelocal-cli`, a comprehensive command-line tool for project initialization and database interaction. Addressed critical persistence issues to ensure data durability.

### Architecture
- **CLI Crate**: `firelocal-cli` in workspace. Built with `clap` (args) and `rustyline` (REPL).
- **Commands**: 
  - `init`: Create project structure.
  - `put`/`get`/`del`: CRUD operations.
  - `flush`: Manually trigger Memtable -> SST flush (for testing).
  - `shell`: Interactive session.
- **Persistence Fix**: 
  - `FireLocal::new` now scans directory for `.sst` files, loading them into `SstReader`s.
  - WAL is replayed on startup to populate Memtable with recent changes.
  - `get` logic updated to query Memtable then SSTables (newest first).

### Key Changes
- **Refactor**: Changed `FireLocal::get` to return `Option<Vec<u8>>` (owned) instead of `Option<&[u8]>` to support reading from disk (SSTs).
- **Core Update**: Added `ssts` vector to `FireLocal` struct.
- **FFI**: Updated `firelocal_get` to handle the new return type stability.

### Developer Notes
- **Verification**: `basic_test.rs` and manual CLI tests confirmed data survives process restart and WAL deletion (SST path).
