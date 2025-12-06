# FireLocal

**Offline-first database with Firestore API compatibility**

FireLocal is a production-ready, offline-first database that provides Firestore-compatible APIs for local data persistence. Perfect for mobile apps, desktop applications, and any scenario requiring local-first architecture.

## Features

✅ **Firestore-Compatible API** - Drop-in replacement for offline scenarios  
✅ **Smart Configuration** - Auto-creates/populates `.env` files  
✅ **Batch Operations** - Atomic multi-document writes  
✅ **Transactions** - Optimistic concurrency control  
✅ **FieldValue Helpers** - `serverTimestamp()`, `increment()`, `arrayUnion()`, etc.  
✅ **Compaction** - Automatic disk space optimization  
✅ **Rules Engine** - Firebase-compatible security rules  
✅ **Multi-Language** - Rust, JavaScript/Node.js, Dart, Python bindings  
✅ **CLI Tools** - Interactive shell and management commands  

## Quick Start

### Installation

**Rust:**
```toml
[dependencies]
firelocal-core = "0.1"
```

**JavaScript/Node.js:**
```bash
npm install firelocal
```

**Dart:**
```yaml
dependencies:
  firelocal_dart: ^0.1.0
```

**Python:**
```bash
pip install firelocal
```

### Basic Usage

**Rust:**
```rust
use firelocal_core::FireLocal;

// Create database with auto .env configuration
let mut db = FireLocal::new_with_config("./data")?;

// Write data
db.put("users/alice".to_string(), br#"{"name":"Alice","age":30}"#.to_vec())?;

// Read data
if let Some(data) = db.get("users/alice") {
    println!("{}", String::from_utf8_lossy(&data));
}

// Delete data
db.delete("users/alice".to_string())?;
```

**JavaScript:**
```javascript
import FireLocal, { serverTimestamp, increment } from 'firelocal';

const db = new FireLocal('./data');

// Write with FieldValue
const data = {
    name: 'Alice',
    timestamp: serverTimestamp(),
    loginCount: increment(1)
};
db.put('users/alice', JSON.stringify(data));

// Batch operations
const batch = db.batch();
batch.set('users/bob', JSON.stringify({name: 'Bob'}));
batch.set('users/charlie', JSON.stringify({name: 'Charlie'}));
db.commitBatch(batch);

// Compaction
const stats = db.compact();
console.log(`Saved ${stats.tombstonesRemoved} tombstones`);
```

### CLI Usage

```bash
# Initialize project (creates .env)
firelocal init

# Show configuration
firelocal config show

# Interactive shell
firelocal shell

# Put document
firelocal put users/alice '{"name":"Alice"}'

# Get document
firelocal get users/alice

# Run compaction
firelocal compact
```

## Configuration

FireLocal auto-creates a `.env` file with smart defaults:

```env
# FireLocal Configuration
FIRELOCAL_PROJECT_ID=my-firelocal-project
FIRELOCAL_DB_PATH=./.firelocal/data
FIRELOCAL_SYNC_MODE=off
FIRELOCAL_SYNC_INTERVAL=300

# Firebase Credentials (optional, for sync)
FIREBASE_API_KEY=
FIREBASE_APP_ID=
FIREBASE_PROJECT_ID=
```

## Advanced Features

### Batch Operations

```rust
let mut batch = db.batch();
batch
    .set("users/alice".to_string(), data1)
    .set("users/bob".to_string(), data2)
    .delete("users/charlie".to_string());
db.commit_batch(&batch)?; // Atomic commit
```

### Transactions

```rust
db.run_transaction(|txn, db| {
    let data = txn.get("counter", db.get("counter"), 1);
    // Perform updates
    txn.set("counter".to_string(), new_value);
    Ok(())
})?; // Auto-validates versions
```

### FieldValue Helpers

```rust
use firelocal_core::field_value::FieldValue;

let mut data = serde_json::Map::new();
data.insert("timestamp", serde_json::to_value(FieldValue::server_timestamp())?);
data.insert("count", serde_json::to_value(FieldValue::increment(1))?);
data.insert("tags", serde_json::to_value(FieldValue::array_union(vec![json!("new")]))?);

db.put_with_field_values("doc1".to_string(), data)?;
```

### Security Rules

```rust
db.load_rules(r#"
service cloud.firestore {
  match /databases/{database}/documents {
    match /users/{userId} {
      allow read, write: if request.auth.uid == userId;
    }
  }
}
"#)?;
```

## Architecture

```
┌─────────────────────────────────────┐
│         Application Layer           │
│  (Rust, JS, Dart, Python, CLI)     │
└─────────────────────────────────────┘
                 │
┌─────────────────────────────────────┐
│         FireLocal Core              │
│  ┌─────────────────────────────┐   │
│  │  API Layer (Firestore-like) │   │
│  └─────────────────────────────┘   │
│  ┌──────────┬──────────┬─────────┐ │
│  │ Rules    │ Index    │ Sync    │ │
│  │ Engine   │ Engine   │ Adapter │ │
│  └──────────┴──────────┴─────────┘ │
│  ┌─────────────────────────────┐   │
│  │  Storage Engine (LSM-Tree)  │   │
│  │  WAL → Memtable → SSTable   │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

## Performance

- **Writes**: O(log n) with WAL durability
- **Reads**: O(log n) from memtable + SST
- **Batches**: Single WAL flush for all operations
- **Transactions**: Minimal overhead with version checking
- **Compaction**: Background SST merging

## API Reference

### Core Methods

| Method | Description |
|--------|-------------|
| `new(path)` | Create database instance |
| `new_with_config(path)` | Create with auto .env |
| `put(key, value)` | Write document |
| `get(key)` | Read document |
| `delete(key)` | Delete document |
| `batch()` | Create write batch |
| `commit_batch(batch)` | Commit batch atomically |
| `run_transaction(fn)` | Run transaction with OCC |
| `compact()` | Run compaction |
| `flush()` | Flush memtable to SST |

### FieldValue Helpers

- `serverTimestamp()` - Current server time
- `increment(n)` - Increment numeric field
- `arrayUnion(elements)` - Add unique elements
- `arrayRemove(elements)` - Remove elements
- `delete()` - Delete field

## Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --package firelocal-core

# Run with output
cargo test -- --nocapture
```

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Submit a pull request

## License

MIT License - see LICENSE file for details

## Roadmap

- [x] Core storage engine (WAL, Memtable, SST)
- [x] Basic indexing and queries
- [x] Rules engine
- [x] Configuration system
- [x] Batch operations
- [x] Transactions
- [x] FieldValue helpers
- [x] CLI tools
- [x] JavaScript bindings
- [ ] Dart bindings (in progress)
- [ ] Python bindings (in progress)
- [ ] Composite indexes
- [ ] Advanced query operators
- [ ] WASM support
- [ ] Enhanced sync modes

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/firelocal/issues)
- **Documentation**: See `docs/` directory
- **Examples**: See `examples/` directory

## Acknowledgments

Built with Rust 🦀 for performance and reliability.
