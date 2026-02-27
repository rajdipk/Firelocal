# FireLocal - Complete User & Developer Guide

**Version:** 1.0.0  
**Last Updated:** February 2025  
**Status:** Production Ready

---

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Installation Guide](#installation-guide)
4. [Core Concepts](#core-concepts)
5. [API Reference](#api-reference)
6. [Security Rules](#security-rules)
7. [Examples](#examples)
8. [Performance Tuning](#performance-tuning)
9. [Production Deployment](#production-deployment)
10. [Troubleshooting](#troubleshooting)
11. [FAQ](#faq)

---

## Introduction

Welcome to FireLocal! This comprehensive guide will help you understand, install, and use FireLocal effectively, whether you're a beginner just starting out or an advanced developer building production applications.

### Who Should Read This?

- **Beginners**: Start with [Getting Started](#getting-started) and [Installation Guide](#installation-guide)
- **Web Developers**: Jump to [Web (WASM) Integration](#installation-guide#webassembly-wasm-integration)
- **Mobile Developers**: Jump to [Installation Guide](#installation-guide) for Android/iOS
- **System Architects**: Review [Core Concepts](#core-concepts) and [Performance](#performance-tuning)
- **DevOps Engineers**: See [Production Deployment](#production-deployment) and [Troubleshooting](#troubleshooting)

---

## Getting Started

FireLocal is an **offline-first database** that provides a Firestore-compatible API for local data persistence. It's designed for applications that need to work offline, sync data when online, and provide a seamless user experience regardless of network conditions.

### Quick Start Example

```rust
use firelocal_core::FireLocal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database
    let mut db = FireLocal::new("./my_database")?;
    
    // Store data
    db.put("users/alice".to_string(), 
           serde_json::to_vec(&serde_json::json!({
               "name": "Alice",
               "age": 30
           }))?)?;
    
    // Retrieve data
    if let Some(data) = db.get("users/alice")? {
        let user: serde_json::Value = serde_json::from_slice(&data)?;
        println!("User: {}", user);
    }
    
    Ok(())
}
```

---

## Installation Guide

### Rust Core Library

```bash
# Add to Cargo.toml
[dependencies]
firelocal-core = { git = "https://github.com/rajdipk/Firelocal.git" }

# Or install from crates.io (when published)
cargo add firelocal-core
```

### JavaScript/Node.js

```bash
# Install npm package
npm install @firelocal/node

# Usage
const { FireLocal } = require('@firelocal/node');
const db = new FireLocal('./database');
```

### WebAssembly (WASM)

```bash
# Install npm package
npm install firelocal-wasm

# Usage in browser
import { FireLocal } from 'firelocal-wasm';
const db = new FireLocal('./database');
```

### CLI Tool

```bash
# Install from source
cargo install --path /path/to/Firelocal/firelocal-cli

# Or from crates.io (when published)
cargo install firelocal-cli

# Basic usage
firelocal-cli put users/alice '{"name":"Alice","age":30}'
firelocal-cli get users/alice
```

### Python Bindings

```bash
# Install from PyPI (when published)
pip install firelocal

# Development installation
cd bindings/python
pip install -e .
```

### Dart Bindings

```bash
# Add to pubspec.yaml
dependencies:
  firelocal:
    git: https://github.com/rajdipk/Firelocal.git

# Install dependencies
dart pub get
```

### C#/.NET Bindings

```bash
# Add NuGet package (when published)
dotnet add package FireLocal

# Or reference local project
dotnet add reference ../bindings/dotnet/FireLocal.csproj
```

---

## Core Concepts

### Database Structure

FireLocal uses an LSM-Tree (Log-Structured Merge Tree) architecture:

- **WAL (Write-Ahead Log)**: All writes first go here for durability
- **Memtable**: In-memory sorted structure for recent writes
- **SST (Sorted String Table)**: Immutable files for long-term storage
- **Compaction**: Background process to merge SST files and remove tombstones

### ACID Properties

- **Atomicity**: Transactions are all-or-nothing
- **Consistency**: Database always moves from one valid state to another
- **Isolation**: Concurrent transactions don't interfere with each other
- **Durability**: Committed changes survive system crashes

### Security Model

FireLocal implements a Firebase-compatible security rules engine:

```javascript
service cloud.firestore {
  match /databases/{database}/documents {
    match /users/{userId} {
      allow read, write: if request.auth != null && request.auth.uid == userId;
    }
    match /posts/{postId} {
      allow read: if true;
      allow write: if request.auth != null;
    }
  }
}
```

---

## API Reference

### Core Rust API

#### Database Operations

```rust
// Create/open database
let mut db = FireLocal::new("./path")?;

// Write document
db.put("collection/doc".to_string(), data)?;

// Read document
let data = db.get("collection/doc")?; // Returns Option<Vec<u8>>

// Delete document
db.delete("collection/doc".to_string())?;

// Query documents
let results = db.query("collection/*")?;

// Batch operations
let batch = db.batch();
batch.set("users/alice".to_string(), alice_data);
batch.set("users/bob".to_string(), bob_data);
db.commit_batch(batch)?;

// Transaction
let txn = db.transaction();
txn.read("users/alice");
txn.set("users/alice".to_string(), new_data);
txn.validate(|path| get_version(path), |path| get_data(path))?;
txn.commit()?;
```

#### Listeners

```rust
// Listen for changes
let listener_id = db.listen(
    QueryAst::parse("users/*")?,
    |docs| println!("Documents changed: {:?}", docs)
);

// Stop listening
db.unlisten(listener_id);
```

### Security Configuration

```rust
// Load security rules
db.load_rules(r#"
  service cloud.firestore {
    match /databases/{database}/documents {
      match /{document=**} {
        allow read, write: if true;
      }
    }
  }
"#)?;

// Configure security limits (via environment variables)
// FIRELOCAL_MAX_REQUESTS_PER_MINUTE=1000
// FIRELOCAL_MAX_DOCUMENT_SIZE=10485760
// FIRELOCAL_AUTH_ENABLED=true
// FIRELOCAL_RATE_LIMIT_ENABLED=true
```

---

## Security Rules

### Rule Syntax

FireLocal uses Firebase's security rules syntax:

```javascript
service cloud.firestore {
  match /databases/{database}/documents {
    // Public read access
    match /public/{doc} {
      allow read: if true;
      allow write: if false;
    }
    
    // User-specific access
    match /users/{userId} {
      allow read, write: if 
        request.auth != null && 
        request.auth.uid == userId;
    }
    
    // Role-based access
    match /admin/{doc} {
      allow read, write: if 
        request.auth != null && 
        request.auth.token.admin == true;
    }
  }
}
```

### Built-in Functions

- `request.auth` - Authentication information
- `request.time` - Current timestamp
- `request.resource` - Resource being accessed
- `resource.data` - Existing document data
- `request.method` - HTTP method (get, post, put, delete)

### Security Features

- **Authentication**: User identity verification
- **Authorization**: Rule-based access control
- **Input Validation**: Path and data sanitization
- **Rate Limiting**: Configurable request limits
- **Audit Logging**: Security event tracking

---

## Examples

### Web Application

```javascript
import { FireLocal } from 'firelocal-wasm';

class UserService {
  constructor() {
    this.db = new FireLocal('./user_data');
  }
  
  async createUser(userData) {
    const userId = this.generateId();
    await this.db.put(`users/${userId}`, JSON.stringify(userData));
    return userId;
  }
  
  async getUser(userId) {
    const data = await this.db.get(`users/${userId}`);
    return data ? JSON.parse(data) : null;
  }
  
  async updateUser(userId, updates) {
    const current = await this.getUser(userId);
    if (!current) throw new Error('User not found');
    
    const updated = { ...current, ...updates };
    await this.db.put(`users/${userId}`, JSON.stringify(updated));
  }
}
```

### CLI Application

```rust
use firelocal_core::FireLocal;
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Put { key: String, value: String },
    Get { key: String },
    Delete { key: String },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut db = FireLocal::new("./data")?;
    
    match cli.command {
        Commands::Put { key, value } => {
            db.put(key, value.into_bytes())?;
            println!("✅ Stored {}", key);
        }
        Commands::Get { key } => {
            if let Some(data) = db.get(&key)? {
                println!("{}", String::from_utf8(data)?);
            } else {
                println!("❌ Not found");
            }
        }
        Commands::Delete { key } => {
            db.delete(key)?;
            println!("🗑️ Deleted {}", key);
        }
    }
    
    Ok(())
}
```

---

## Performance Tuning

### Benchmarks

FireLocal has been benchmarked extensively:

| Operation | Performance | Notes |
|-----------|-------------|---------|
| **Read** | 411,271 ops/sec | Extremely fast due to LSM-Tree |
| **Write** | 31.46 ops/sec | Limited by disk I/O and WAL |
| **Mixed** | 63.00 ops/sec | Realistic workload |
| **Large Docs** | 32.62 ops/sec | 1MB+ documents |

### Optimization Tips

1. **Batch Operations**: Use batches for multiple writes
2. **Appropriate Transactions**: Keep transactions small
3. **Regular Compaction**: Prevents SST file accumulation
4. **Memory Settings**: Adjust memtable size for your workload
5. **Storage Type**: Use SSD for better WAL performance

### Monitoring

```rust
// Health checks
let health = db.health_check();
println!("Database healthy: {}", health.is_healthy());

// Performance metrics
let metrics = db.get_metrics();
println!("Read ops: {}", metrics.read_operations);
println!("Write ops: {}", metrics.write_operations);
println!("Cache hit rate: {}", metrics.cache_hit_rate);
```

---

## Production Deployment

### Environment Configuration

```bash
# Security limits
export FIRELOCAL_MAX_REQUESTS_PER_MINUTE=1000
export FIRELOCAL_MAX_DOCUMENT_SIZE=10485760
export FIRELOCAL_MAX_PATH_DEPTH=32

# Feature flags
export FIRELOCAL_AUTH_ENABLED=true
export FIRELOCAL_RATE_LIMIT_ENABLED=true
export FIRELOCAL_AUDIT_LOGGING=true
```

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/firelocal-cli /usr/local/bin/
EXPOSE 8080
CMD ["firelocal-cli"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: firelocal
spec:
  replicas: 3
  selector:
    matchLabels:
      app: firelocal
  template:
    metadata:
      labels:
        app: firelocal
    spec:
      containers:
      - name: firelocal
        image: firelocal:latest
        env:
        - name: FIRELOCAL_MAX_REQUESTS_PER_MINUTE
          value: "1000"
        - name: FIRELOCAL_AUTH_ENABLED
          value: "true"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

### Monitoring Setup

```rust
use firelocal_core::monitoring::*;

// Set up monitoring
let monitor = Monitoring::new()
    .with_health_check_interval(Duration::from_secs(30))
    .with_metrics_collection(true)
    .with_alert_thresholds(AlertThresholds {
        error_rate: 0.05,  // 5% error rate
        response_time: Duration::from_millis(1000),
        memory_usage: 0.8,  // 80% memory
    });

monitor.start(&db);
```

### Backup Strategy

```bash
#!/bin/bash
# Backup script
BACKUP_DIR="/backups/firelocal"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup
tar -czf "$BACKUP_DIR/backup_$DATE.tar.gz" \
    --exclude='wal.log.lock' \
    ./database/

# Keep last 7 days
find "$BACKUP_DIR" -name "backup_*.tar.gz" -mtime +7 -delete

echo "Backup completed: backup_$DATE.tar.gz"
```

---

## Troubleshooting

### Common Issues

#### Database Won't Open

```bash
# Check permissions
ls -la ./database/

# Check for lock files
find ./database/ -name "*.lock"

# Remove stale locks (carefully)
rm ./database/wal.log.lock
```

#### Performance Issues

```rust
// Check for excessive WAL size
let wal_size = std::fs::metadata("./database/wal.log")?.len();
if wal_size > 100_000_000 {  // 100MB
    println!("Warning: Large WAL file, consider compaction");
    db.compact()?;
}

// Check SST file count
let sst_count = std::fs::read_dir("./database/")?
    .filter(|entry| entry.as_ref().ok()
        .map(|e| e.file_name().to_string_lossy().starts_with("sst_"))
    .count();
    
if sst_count > 10 {
    println!("Warning: Many SST files, consider compaction");
}
```

#### Memory Issues

```rust
// Monitor memory usage
let metrics = db.get_memory_metrics();
println!("Memtable size: {} bytes", metrics.memtable_size);
println!("Cache size: {} bytes", metrics.cache_size);

// Clear caches if needed
if metrics.cache_size > 100_000_000 {  // 100MB
    db.clear_caches();
}
```

### Error Recovery

FireLocal includes automatic recovery mechanisms:

1. **WAL Recovery**: Automatically replays WAL after crash
2. **Mutex Poison Recovery**: Handles poisoned mutexes with data validation
3. **File Corruption Detection**: Validates file integrity on startup
4. **Graceful Degradation**: Continues operation with limited functionality

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=firelocal_core=debug

# Run with detailed output
firelocal-cli --verbose get users/alice
```

---

## FAQ

### General Questions

**Q: Is FireLocal compatible with Firebase?**  
A: FireLocal provides a Firestore-compatible API, but it's completely offline. No Firebase connection is required.

**Q: How does FireLocal handle concurrent access?**  
A: FireLocal uses proper mutex locking and file-based locking to ensure thread safety and prevent race conditions.

**Q: Can FireLocal be used in production?**  
A: Yes! FireLocal is production-ready with enterprise-grade security, monitoring, and error handling.

### Performance Questions

**Q: Why are writes slower than reads?**  
A: Writes must go to WAL for durability, then eventually to SST files. Reads can directly access optimized SST files.

**Q: How can I improve performance?**  
A: Use batch operations, keep transactions small, ensure regular compaction, and use SSD storage.

### Security Questions

**Q: How do security rules work?**  
A: FireLocal implements Firebase's security rules engine, allowing fine-grained access control based on user authentication and document data.

**Q: Is data encrypted?**  
A: FireLocal focuses on API compatibility. Encryption should be handled at the application level if required.

### Technical Questions

**Q: What happens if the application crashes?**  
A: FireLocal automatically recovers from WAL on next startup, ensuring no data loss.

**Q: Can I use FireLocal in the browser?**  
A: Yes! FireLocal provides WebAssembly bindings for browser usage with IndexedDB persistence.

**Q: How large can documents be?**  
A: Default limit is 10MB, configurable via `FIRELOCAL_MAX_DOCUMENT_SIZE` environment variable.

---

## Getting Help

- **Documentation**: [Complete Guide](README.md)
- **GitHub Repository**: [FireLocal](https://github.com/rajdipk/Firelocal)
- **Issues**: [Bug Reports](https://github.com/rajdipk/Firelocal/issues)
- **Discussions**: [Community Forum](https://github.com/rajdipk/Firelocal/discussions)
- **Examples**: [Sample Projects](https://github.com/rajdipk/Firelocal/tree/main/examples)

---

<div align="center">
  <p><strong>Thank you for choosing FireLocal! 🚀</strong></p>
  <p>Built with ❤️ for the offline-first community</p>
</div>

✅ **Firestore-Compatible API** - Familiar API for Firebase developers  
✅ **Offline-First** - Works without internet connection  
✅ **Multi-Platform** - Rust, JavaScript, Dart, Python, .NET, WASM support  
✅ **LSM-Tree Storage** - Efficient write-optimized storage engine  
✅ **ACID Transactions** - Atomic, consistent, isolated, durable operations  
✅ **Security Rules** - Firebase-compatible security rules engine  
✅ **Input Validation** - Comprehensive validation for security  
✅ **Rate Limiting** - Built-in rate limiting capabilities  
✅ **CLI Tools** - Interactive shell and management commands  
✅ **Batch Operations** - Atomic multi-document writes  
✅ **FieldValue Helpers** - `serverTimestamp()`, `increment()`, `arrayUnion()`, etc.

### Use Cases

- **Mobile Applications**: Offline-capable mobile apps (Flutter, React Native)
- **Desktop Applications**: Electron, Tauri, or native desktop apps
- **Edge Computing**: IoT devices and edge servers
- **Development/Testing**: Local Firestore emulator alternative
- **Offline-First Web Apps**: Progressive Web Apps (PWAs)
- **Progressive Web Apps**: Offline-capable web applications
- **Hybrid Apps**: React Native, Flutter, Electron applications

---

## Architecture Overview
 
FireLocal is built on a unified, cross-platform architecture that runs the **exact same Rust Core on all platforms**, including Web.
 
```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│    (Rust, JavaScript, Dart, Python, .NET, CLI, WASM)       │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    FireLocal Core (Rust)                    │
│  ┌───────────────────────────────────────────────────────┐  │
│  │           API Layer (Firestore-like)                  │  │
│  │  • CollectionReference  • DocumentReference           │  │
│  │  • Query  • WriteBatch  • Transaction                 │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │         Storage Engine (LSM-Tree Architecture)        │  │
│  │  ┌──────────┬──────────┬──────────┬──────────────┐   │  │
│  │  │   WAL    │ Memtable │  SSTable │  Compaction  │   │  │
│  │  └──────────┴──────────┴──────────┴──────────────┘   │  │
│  └───────────────────────────────────────────────────────┘  │
│                             │                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │               Storage Interface (Trait)               │  │
│  │  • open() • create() • read() • write() • remove()    │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────┐
│                   Platform Adapters                         │
├───────────────────────────┬─────────────────────────────────┤
│    StdStorage (Native)    │     MemoryStorage (Web/WASM)    │
│  (Windows, Linux, Mac,    │     (Browser, Edge Workers)     │
│   Android, iOS)           │                                 │
├───────────────────────────┼─────────────────────────────────┤
│      Real Filesystem      │     In-Memory Virtual FS        │
│  .firelocal/data/wal.log  │     (Backed by IndexedDB/       │
│  .firelocal/data/*.sst    │      LocalStorage snapshot)     │
└───────────────────────────┴─────────────────────────────────┘
```
 
### Component Breakdown
 
#### 1. **Unified Core & Storage Interface**
 
FireLocal uses a pluggable storage backend:
-   **Native (Mobile/Desktop)**: Uses `StdStorage` which maps directly to the OS filesystem (`std::fs`) for maximum performance and reliability.
-   **Web (WASM)**: Uses `MemoryStorage`, a Virtual File System (VFS) that runs entirely in memory. This allows the full complex LSM-tree architecture to run inside a browser tab without needing direct filesystem access.

#### 2. **Index Engine**

Provides fast document lookups and query execution:

- **Basic Index**: Field-level inverted index for equality queries.
- **Composite Index**: Multi-field indexes for complex queries (coming soon).
- **Query Scoping**: Queries are strictly scoped to a collection. You must specify the collection (e.g., `db.collection("users").where(...)`) to use indexes correctly.
- **Query Operators**: `==`, `in`, `array-contains`, `<`, `>`, `<=`, `>=`.

#### 3. **Rules Engine**

Firebase-compatible security rules for access control:

- Pattern matching on document paths
- Conditional allow/deny statements
- Context-aware evaluation (user auth, request data)

#### 4. Validation & Constraints
1. **Path Validation**
   - Must be non-empty and valid UTF-8.
   - Maximum depth: 32 segments (e.g., `a/b/c/...`).
   - restricted characters: `.` (period) at start/end, `__` (double underscore).
   - Maximum length: 4096 bytes.

2. **Data Validation**
   - Documents must be valid JSON objects.
   - Keys must be strings.
   - Maximum document size: 1 MiB (soft limit).

#### 5. Rules Engine
- **Development Mode**: If no rules are loaded, FireLocal defaults to **ALLOW ALL** for convenience.
- **Production Mode**: Once rules are loaded, the behavior switches to **DEFAULT DENY**. You must explicitly allow operations.

#### 5. **API Layer**

Firestore-compatible API for familiar development experience:

- Collections and Documents
- Queries and Listeners
- Batch Operations
- Transactions

#### 6. **Language Bindings**

Native bindings for multiple programming languages:

- **Rust**: Direct API access (zero-cost abstraction)
- **JavaScript/Node.js**: NAPI bindings for Node.js, WASM for browsers
- **Dart**: FFI bindings for Flutter apps
- **Python**: ctypes bindings
- **.NET**: P/Invoke bindings for C#

---

## Data Storage Locations

Knowing where your data lives is critical for backups, debugging, and platform integration.

| Platform | Storage Type | Default Location / Mechanism | Notes |
|----------|--------------|------------------------------|-------|
| **Desktop (Windows/Mac/Linux)** | File System | `./<path_arg>/` (e.g., `./my-data/`) | Creates `.sst`, `.wal`, and `.lock` files directly in the specified folder. |
| **Mobile (Android/iOS)** | File System | App Documents Directory + `/<path_arg>/` | Use `path_provider` in Flutter to get the doc dir, then append your specific db path. |
| **Web (Browser)** | Virtual FS (RAM + LocalStorage) | `localStorage` key: `firelocal_<path_arg>` | **Warning**: Data persists in Browser Storage. Clearing browser cache/data **WILL delete** the database. Limit ~5-10MB. |
| **Server (Node/Python)** | File System | Relative to CWD + `/<path_arg>/` | same as Desktop. |

> [!IMPORTANT]
> **Web Persistence**: On the web, FireLocal simulates a filesystem in memory and flushes snapshots to `localStorage`. It does **NOT** write to the user's hard drive directly. Large datasets (>5MB) may hit browser quotas.

---

## Getting Started

### Prerequisites

- **For Rust**: Rust 1.70+ and Cargo
- **For JavaScript**: Node.js 16+ or modern browser
- **For Dart**: Dart SDK 3.0+ or Flutter 3.0+
- **For Python**: Python 3.8+
- **.NET**: .NET 6.0+

### Quick Start (5 Minutes)

#### Option 1: Using Rust

#### Option 1: Using Rust (Systems/Backend)

**Initialization:**
```rust
use firelocal_core::FireLocal;

// Standard initialization
let mut db = FireLocal::new("./my-data").expect("Failed to init DB");

// The path "./my-data" will be created if it doesn't exist.
// It will contain the Write-Ahead Log (WAL) and SSTable files.
```

**Basic Usage:**
```rust
    
    // Write data
    let data = br#"{"name": "Alice", "age": 30}"#;
    db.put("users/alice".to_string(), data.to_vec())?;
    
    // Read data
    if let Some(value) = db.get("users/alice") {
        println!("Found: {}", String::from_utf8_lossy(&value));
    }
    
    // Delete data
    db.delete("users/alice".to_string())?;
    
    Ok(())
}
```

#### Option 2: Using JavaScript/Node.js

**Initialization:**
```javascript
const { FireLocal } = require('@firelocal/node');

// Initialize database
const db = new FireLocal('./my-data');
// Note: Relative path from where the script is run
```

**Basic Usage:**
```javascript
// Write data
db.put('users/alice', JSON.stringify({
    name: 'Alice',
    age: 30
}));

// Read data
const data = db.get('users/alice');
console.log('Found:', JSON.parse(data));

// Delete data
db.delete('users/alice');

// Always close when done
db.close();
```

#### Option 3: Using Dart (Flutter Desktop/Mobile)

**Initialization:**
```dart
import 'package:firelocal_dart/firelocal_dart.dart';
import 'package:path_provider/path_provider.dart';

Future<void> initDb() async {
  // Good Practice: Use Application Documents Directory
  final docDir = await getApplicationDocumentsDirectory();
  final dbPath = '${docDir.path}/my_firelocal_db';
  
  final db = FireLocal(dbPath);
  // ... usage ...
}
```

**Basic Usage:**
```dart
void main() async {
  // ... assuming db is initialized ...
  
  // Write data (accepts Map directly)
  await db.put('users/alice', {'name': 'Alice', 'age': 30});
  
  // Read data
  final data = await db.get('users/alice');
  print('Found: $data');
  
  // Delete data
  await db.delete('users/alice');
  
  db.close();
}
```

#### Option 4: Using Dart (Flutter Web with WASM)

**Initialization:**
```dart
import 'package:firelocal_dart/firelocal_dart.dart';

void main() async {
  // For web, path is a key in LocalStorage
  final db = FireLocal('my_app_data'); 
  
  // ... usage is identical to Mobile/Desktop ...
  
  await db.put('users/alice', {'name': 'Alice', 'age': 30});
  // ...
}
```

---

## Windows Desktop Integration

### Installation for Windows

#### Prerequisites

- Windows 10 or later
- Visual Studio 2019+ or MinGW
- Rust 1.70+ (for development)
- Flutter 3.0+ (for Flutter apps)

#### Step 1: Add Dependency

**For Flutter Desktop:**

```yaml
# pubspec.yaml
dependencies:
  flutter:
    sdk: flutter
  firelocal_dart:
    path: ../firelocal/bindings/dart
```

#### Step 2: Build Native Library

The native library (`firelocal_core.dll`) is automatically built when you run:

```bash
flutter run -d windows
```

#### Step 3: Run Your App

```bash
flutter run -d windows
```

### Windows Features

✅ **Native Performance** - Direct Rust library access  
✅ **Full Storage** - Unlimited disk space  
✅ **Offline Support** - Works without internet  
✅ **Data Persistence** - Survives app restart  
✅ **ACID Transactions** - Reliable operations  

### Windows Example

```dart
import 'package:flutter/material.dart';
import 'package:firelocal_dart/firelocal_dart.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({Key? key}) : super(key: key);

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  late FireLocal db;
  String? data;

  @override
  void initState() {
    super.initState();
    _initDatabase();
  }

  Future<void> _initDatabase() async {
    try {
      // Initialize FireLocal for Windows
      db = FireLocal('./firelocal_data');
      
      // Load existing data
      final result = await db.get('app/config');
      setState(() {
        data = result?.toString();
      });
    } catch (e) {
      print('Error: $e');
    }
  }

  Future<void> _saveData() async {
    try {
      await db.put('app/config', {
        'theme': 'dark',
        'language': 'en',
        'timestamp': DateTime.now().toIso8601String(),
      });
      
      final result = await db.get('app/config');
      setState(() {
        data = result?.toString();
      });
    } catch (e) {
      print('Error: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('FireLocal Windows')),
        body: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Text('Data: ${data ?? "Loading..."}'),
              ElevatedButton(
                onPressed: _saveData,
                child: const Text('Save Data'),
              ),
            ],
          ),
        ),
      ),
    );
  }

  @override
  void dispose() {
    db.close();
    super.dispose();
  }
}
```

---

## Web (WASM) Integration

### What is WASM?

WebAssembly (WASM) is a binary format that allows running compiled code in web browsers with near-native performance. FireLocal uses WASM to provide offline-first database capabilities in web applications.

### Installation for Web

#### Prerequisites

- Flutter 3.0+
- Modern web browser (Chrome 74+, Firefox 79+, Safari 14.1+, Edge 79+)
- No additional dependencies needed

#### Step 1: Add Dependency

```yaml
# pubspec.yaml
dependencies:
  flutter:
    sdk: flutter
  firelocal_dart:
    path: ../firelocal/bindings/dart
  firelocal_wasm:
    path: ../firelocal/firelocal-wasm
```

#### Step 2: Create Web Entry Point

Create `lib/main_web.dart` for web-specific code:

```dart
import 'package:flutter/material.dart';
import 'dart:convert';

void main() {
  runApp(const MyWebApp());
}

class MyWebApp extends StatefulWidget {
  const MyWebApp({Key? key}) : super(key: key);

  @override
  State<MyWebApp> createState() => _MyWebAppState();
}

class _MyWebAppState extends State<MyWebApp> {
  List<Map<String, dynamic>> todos = [];

  @override
  void initState() {
    super.initState();
    _loadTodos();
  }

  Future<void> _loadTodos() async {
    // In production, load from WASM/localStorage
    setState(() {});
  }

  Future<void> _addTodo(String title) async {
    final todo = {
      'id': DateTime.now().millisecondsSinceEpoch.toString(),
      'title': title,
      'completed': false,
      'createdAt': DateTime.now().toIso8601String(),
    };
    
    todos.add(todo);
    
    // Save to localStorage via WASM
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'FireLocal Web',
      theme: ThemeData(useMaterial3: true),
      home: Scaffold(
        appBar: AppBar(title: const Text('FireLocal Web (WASM)')),
        body: ListView.builder(
          itemCount: todos.length,
          itemBuilder: (context, index) {
            final todo = todos[index];
            return ListTile(
              title: Text(todo['title']),
              subtitle: Text(todo['createdAt']),
            );
          },
        ),
      ),
    );
  }
}
```

#### Step 3: Run on Web

```bash
# Run on Chrome
flutter run -d chrome --target=lib/main_web.dart

# Run on Firefox
flutter run -d firefox --target=lib/main_web.dart

# Build for production
flutter build web --release --target=lib/main_web.dart
```

### WASM Features

✅ **Browser Compatible** - Works in all modern browsers  
✅ **Offline Support** - Full offline-first capability  
✅ **localStorage Persistence** - 5-10MB storage per domain  
✅ **Fast Performance** - Near-native speed  
✅ **Zero Server Required** - Completely client-side  

### WASM Storage

**localStorage (Default):**
- Limit: 5-10MB per domain
- Persistent across sessions
- Synchronous access
- Good for small to medium datasets

**Future: IndexedDB**
- Limit: 50MB+
- Asynchronous access
- Better for large datasets
- Can be added later

### Browser Support

| Browser | Version | Status |
|---------|---------|--------|
| Chrome | 74+ | ✅ Supported |
| Firefox | 79+ | ✅ Supported |
| Safari | 14.1+ | ✅ Supported |
| Edge | 79+ | ✅ Supported |
| Opera | 61+ | ✅ Supported |

### WASM Example: Todo App

```dart
import 'package:flutter/material.dart';
import 'dart:convert';

void main() {
  runApp(const TodoApp());
}

class TodoApp extends StatefulWidget {
  const TodoApp({Key? key}) : super(key: key);

  @override
  State<TodoApp> createState() => _TodoAppState();
}

class _TodoAppState extends State<TodoApp> {
  final List<Todo> todos = [];
  final TextEditingController controller = TextEditingController();

  void addTodo(String title) {
    if (title.isEmpty) return;
    
    final todo = Todo(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      title: title,
      completed: false,
      createdAt: DateTime.now(),
    );
    
    setState(() {
      todos.add(todo);
    });
    
    controller.clear();
    _saveTodos();
  }

  void toggleTodo(int index) {
    setState(() {
      todos[index].completed = !todos[index].completed;
    });
    _saveTodos();
  }

  void deleteTodo(int index) {
    setState(() {
      todos.removeAt(index);
    });
    _saveTodos();
  }

  void _saveTodos() {
    // Save to localStorage via WASM
    final json = jsonEncode(todos.map((t) => t.toJson()).toList());
    // In production: db.put('todos/list', json);
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'FireLocal Todo',
      home: Scaffold(
        appBar: AppBar(title: const Text('Todos (WASM)')),
        body: Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(16),
              child: TextField(
                controller: controller,
                decoration: InputDecoration(
                  hintText: 'Add a todo...',
                  suffixIcon: IconButton(
                    icon: const Icon(Icons.add),
                    onPressed: () => addTodo(controller.text),
                  ),
                ),
                onSubmitted: addTodo,
              ),
            ),
            Expanded(
              child: ListView.builder(
                itemCount: todos.length,
                itemBuilder: (context, index) {
                  final todo = todos[index];
                  return ListTile(
                    leading: Checkbox(
                      value: todo.completed,
                      onChanged: (_) => toggleTodo(index),
                    ),
                    title: Text(
                      todo.title,
                      style: TextStyle(
                        decoration: todo.completed
                            ? TextDecoration.lineThrough
                            : null,
                      ),
                    ),
                    trailing: IconButton(
                      icon: const Icon(Icons.delete),
                      onPressed: () => deleteTodo(index),
                    ),
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }
}

class Todo {
  final String id;
  final String title;
  bool completed;
  final DateTime createdAt;

  Todo({
    required this.id,
    required this.title,
    required this.completed,
    required this.createdAt,
  });

  Map<String, dynamic> toJson() => {
    'id': id,
    'title': title,
    'completed': completed,
    'createdAt': createdAt.toIso8601String(),
  };

  factory Todo.fromJson(Map<String, dynamic> json) => Todo(
    id: json['id'],
    title: json['title'],
    completed: json['completed'],
    createdAt: DateTime.parse(json['createdAt']),
  );
}
```

---

## Core Concepts

### 1. Documents and Collections

FireLocal organizes data in a hierarchical structure similar to Firestore:

```
users/                    ← Collection
  alice                   ← Document
  bob                     ← Document
  charlie/                ← Document
    posts/                ← Subcollection
      post1               ← Document
      post2               ← Document
```

**Document Paths:**

- Format: `collection/document` or `collection/document/subcollection/document`
- Examples: `users/alice`, `users/alice/posts/post1`

**Document Data:**

- Stored as JSON objects
- Supports nested structures
- Field types: string, number, boolean, array, object, null

### 2. Write-Ahead Log (WAL)

Every write operation is first recorded in the WAL before being applied:

```
1. Client calls put("users/alice", data)
2. Write to WAL (durable on disk)
3. Write to Memtable (in-memory)
4. Return success to client
```

**Benefits:**

- **Durability**: Data survives crashes
- **Recovery**: Replay WAL on startup
- **Performance**: Sequential writes are fast

### 3. Memtable and SSTable

**Memtable** (In-Memory):
- Stores recent writes
- Sorted for efficient range queries
- Flushed to SSTable when full
- Provides O(log n) access

**SSTable** (Disk):
- Immutable sorted files
- Multiple levels for efficient compaction
- Provides persistent storage
- Searched in reverse order (newest first)

### 4. Compaction

Background process that optimizes storage:

```
Before Compaction:
SST Level 0: [file1, file2, file3]
SST Level 1: [file4, file5]
SST Level 2: [file6]

After Compaction:
SST Level 0: [file7]
SST Level 1: [file8]
SST Level 2: [file9]
```

**Benefits:**
- Reduces file count
- Removes tombstones
- Reclaims disk space
- Improves read performance

---

## Installation Guide

### Rust

```bash
# Add to your Cargo.toml
cargo add firelocal-core

# Or manually add to Cargo.toml
[dependencies]
firelocal-core = "1.0"
anyhow = "1.0"
```

### JavaScript/Node.js

```bash
npm install @firelocal/node
# or
yarn add @firelocal/node
```

### Python

```bash
pip install firelocal
```

### Dart/Flutter

```bash
flutter pub add firelocal_dart
```

### .NET

```bash
dotnet add package FireLocal
```

### WASM (Browser)

```bash
npm install firelocal-wasm
```

---

## Database Structure

### How Data is Organized

FireLocal stores data in a hierarchical structure similar to Firestore:

```
database/
├── users/
│   ├── alice
│   │   ├── name: "Alice"
│   │   ├── age: 30
│   │   └── posts/
│   │       ├── post1
│   │       │   ├── title: "Hello"
│   │       │   └── content: "World"
│   │       └── post2
│   │           ├── title: "Second"
│   │           └── content: "Post"
│   └── bob
│       ├── name: "Bob"
│       └── age: 25
├── posts/
│   ├── post1
│   │   ├── title: "Hello"
│   │   └── author: "alice"
│   └── post2
│       ├── title: "Second"
│       └── author: "alice"
└── settings/
    └── config
        ├── theme: "dark"
        └── language: "en"
```

### Path Naming Conventions

- **Paths** use forward slashes: `users/alice`
- **Subcollections** use the same format: `users/alice/posts/post1`
- **Path segments** can contain letters, numbers, hyphens, underscores
- **Maximum path length** is 1024 characters
- **Case-sensitive** - "Users" and "users" are different

### Data Types

FireLocal stores JSON data. Supported types:

- **String** - Text data
- **Number** - Integers and floats
- **Boolean** - true/false
- **Array** - Lists of values
- **Object** - Nested JSON objects
- **null** - Null values

### Example Document

```json
{
  "name": "Alice",
  "age": 30,
  "email": "alice@example.com",
  "active": true,
  "tags": ["developer", "rust", "firebase"],
  "profile": {
    "bio": "Software engineer",
    "location": "San Francisco",
    "links": {
      "github": "https://github.com/alice",
      "twitter": "@alice"
    }
  },
  "created": 1702080000000,
  "updated": 1702166400000
}
```

---

## How It Works

### Storage Engine (LSM-Tree)

FireLocal uses a Log-Structured Merge-Tree (LSM-Tree) architecture for efficient storage:

```
Write Operation:
1. Write to WAL (Write-Ahead Log) - ensures durability
2. Write to Memtable (in-memory) - fast access
3. When memtable is full, flush to SST file
4. Periodically compact SST files

Read Operation:
1. Check Memtable first (fastest)
2. If not found, check SST files (disk)
3. Return result or null if not found

Delete Operation:
1. Write tombstone to WAL
2. Mark as deleted in Memtable
3. Tombstones removed during compaction
```

---

## API Reference

### Core Methods

#### Database Creation
Initialize a connection to the database.

| Language | Syntax | Notes |
|----------|--------|-------|
| **Rust** | `let db = FireLocal::new("./path")?;` | Returns `Result<FireLocal>` |
| **JS** | `const db = new FireLocal("./path");` | Synchronous constructor |
| **Dart** | `final db = FireLocal("./path");` | Sync/Async depending on platform |
| **Python** | `db = FireLocal("./path")` | Returns object |

#### Write Operations

**Put (Create/Update):**
Creates or overwrites a document at the specified path.

*   **Rust**: `db.put("col/doc".to_string(), data_vec)?`
*   **JS**: `db.put("col/doc", json_string)`
*   **Dart**: `await db.put("col/doc", map_data)`
*   **Python**: `db.put("col/doc", dict_data)`

**Batch Put:**
Atomic write of multiple documents.

*   **Rust**:
    ```rust
    let mut batch = db.batch();
    batch.set("users/a".into(), data1);
    db.commit_batch(&batch)?;
    ```
*   **JS**: `const batch = db.batch(); batch.set(...); db.commit(batch);`
*   **Dart**: `final batch = db.batch(); batch.set(...); await db.commit(batch);`
*   **Python**: `batch = db.batch(); batch.set(...); db.commit(batch)`

**Update (Merge):**
Updates specific fields without overwriting the entire document.

*   **Rust**: `batch.update("path".into(), partial_data)`
*   **JS/Dart/Python**: Same API as `batch.set` but merge behavior (coming in v1.1).

#### Read Operations

**Get Single Document:**
Retrieves a document by its path. Returns `null` (or None) if not found.

*   **Rust**: `db.get("col/doc")` -> `Option<Vec<u8>>`
*   **JS**: `db.get("col/doc")` -> `string | null` (JSON string)
*   **Dart**: `await db.get("col/doc")` -> `Map<String, dynamic>?`
*   **Python**: `db.get("col/doc")` -> `dict | None`

**Query Documents:**
Queries a collection.

*   **Rust**:
    ```rust
    let query = QueryAst::new("users"); // + .where_eq(...)
    let results = db.query(&query)?;
    ```
*   **JS/Dart/Python**:
    ```javascript
    // Syntax is similar across bindings
    db.collection("users").where("age", ">", 18).get()
    ```

#### Delete Operations

**Delete Single Document:**
Removes a document by its path.

*   **Rust**: `db.delete("col/doc".into())?`
*   **JS**: `db.delete("col/doc")`
*   **Dart**: `await db.delete("col/doc")`
*   **Python**: `db.delete("col/doc")`

**Batch Delete:**
Atomic deletion of multiple documents.

*   **Rust**: `batch.delete("col/doc".into())`
*   **JS/Dart/Python**: `batch.delete("col/doc")` (part of WriteBatch)

#### Maintenance Operations

**Flush Memtable:**
```rust
db.flush()?;
```

**Compact Database:**
```rust
let stats = db.compact()?;
println!("Files before: {}", stats.files_before);
println!("Files after: {}", stats.files_after);
```

**Load Rules:**
```rust
db.load_rules(rules_string)?;
```

### FieldValue Helpers

Special values for common operations:

**Server Timestamp:**
```rust
use firelocal_core::field_value::FieldValue;

let data = serde_json::json!({
    "name": "Alice",
    "created": FieldValue::server_timestamp()
});
```

**Increment:**
```rust
let data = serde_json::json!({
    "count": FieldValue::increment(1)
});
```

**Array Union:**
```rust
let data = serde_json::json!({
    "tags": FieldValue::array_union(vec![
        json!("rust"),
        json!("database")
    ])
});
```

**Array Remove:**
```rust
let data = serde_json::json!({
    "tags": FieldValue::array_remove(vec![
        json!("old_tag")
    ])
});
```

**Delete Field:**
```rust
let data = serde_json::json!({
    "field_to_delete": FieldValue::delete()
});
```

### Transactions

**Optimistic Concurrency Control:**
```rust
db.run_transaction(|txn, db| {
    // Read current value
    let current = db.get("counter")?;
    
    // Modify value
    let new_value = increment_value(current)?;
    
    // Write back
    txn.set("counter".to_string(), new_value);
    
    Ok(())
})?;
```

---

## Security Rules

### What are Security Rules?

Security rules control who can read and write data. They use Firestore's rule syntax:

```
service cloud.firestore {
  match /databases/{database}/documents {
    // Rules here
  }
}
```

### Basic Rules

**Allow All:**
```
match /{document=**} {
  allow read, write: if true;
}
```

**Deny All:**
```
match /{document=**} {
  allow read, write: if false;
}
```

**Public Read, Authenticated Write:**
```
match /{document=**} {
  allow read: if true;
  allow write: if request.auth != null;
}
```

### Collection-Level Rules

**User-Specific Access:**
```
match /users/{userId} {
  allow read, write: if request.auth.uid == userId;
}
```

**Admin-Only Access:**
```
match /admin/{document=**} {
  allow read, write: if request.auth.token.admin == true;
}
```

### Field-Level Rules

**Protect Sensitive Fields:**
```
match /users/{userId} {
  allow read: if request.auth.uid == userId;
  allow write: if request.auth.uid == userId
    && !request.resource.data.keys().hasAny(['email', 'password']);
}
```

### Loading Rules

To enforce security, load your ruleset string into the database instance.

*   **Rust**: `db.load_rules(rules_string)?;`
*   **JS**: `db.loadRules(rules_string);`
*   **Dart**: `await db.loadRules(rules_string);`
*   **Python**: `db.load_rules(rules_string)`

---

## Examples

### Example 1: User Management

**Create User:**
```rust
let user_data = serde_json::json!({
    "name": "Alice",
    "email": "alice@example.com",
    "age": 30,
    "created": chrono::Utc::now().timestamp_millis()
});

db.put(
    "users/alice".to_string(),
    serde_json::to_vec(&user_data)?
)?;
```

**Update User:**
```rust
let mut batch = db.batch();
batch.update(
    "users/alice".to_string(),
    serde_json::json!({
        "age": 31,
        "updated": chrono::Utc::now().timestamp_millis()
    }).to_string().into_bytes()
);
db.commit_batch(&batch)?;
```

**Delete User:**
```rust
db.delete("users/alice".to_string())?;
```

### Example 2: Blog Application

**Create Post:**
```rust
let post = serde_json::json!({
    "title": "My First Post",
    "content": "Hello, World!",
    "author": "alice",
    "created": chrono::Utc::now().timestamp_millis(),
    "tags": ["hello", "world"],
    "published": true
});

db.put(
    "posts/post1".to_string(),
    serde_json::to_vec(&post)?
)?;
```

**Add Comment:**
```rust
let comment = serde_json::json!({
    "author": "bob",
    "text": "Great post!",
    "created": chrono::Utc::now().timestamp_millis()
});

db.put(
    "posts/post1/comments/comment1".to_string(),
    serde_json::to_vec(&comment)?
)?;
```

### Example 3: Batch Operations

**Bulk Import:**
```rust
let mut batch = db.batch();

for (i, user) in users.iter().enumerate() {
    batch.set(
        format!("users/user{}", i),
        serde_json::to_vec(user)?
    );
}

db.commit_batch(&batch)?;
```

**Atomic Update:**
```rust
let mut batch = db.batch();

// Debit from account A
batch.update(
    "accounts/A".to_string(),
    serde_json::json!({
        "balance": -100
    }).to_string().into_bytes()
);

// Credit to account B
batch.update(
    "accounts/B".to_string(),
    serde_json::json!({
        "balance": 100
    }).to_string().into_bytes()
);

// Both succeed or both fail
db.commit_batch(&batch)?;
```

---

## Advanced Topics

### Transactions

**Optimistic Concurrency Control:**
```rust
db.run_transaction(|txn, db| {
    // Read
    let current = db.get("counter")?;
    let value = serde_json::from_slice::<i32>(&current)?;
    
    // Modify
    let new_value = value + 1;
    
    // Write
    txn.set("counter".to_string(), serde_json::to_vec(&new_value)?);
    
    Ok(())
})?;
```

### Listeners (Real-time Updates)

**Subscribe to Changes:**
```rust
let listener_id = db.listen(
    QueryAst::new("users"),
    Box::new(|docs| {
        println!("Documents changed: {:?}", docs);
    })
);

// Later, unsubscribe
db.unregister_listener(listener_id);
```

### Sync Operations

**Push to Remote:**
```rust
db.sync_push("users/alice")?;
```

**Pull from Remote:**
```rust
db.sync_pull("users/alice")?;
```

### Input Validation

**Validate Paths:**
```rust
use firelocal_core::validation;

validation::validate_path("users/alice")?;
```

**Validate Data:**
```rust
validation::validate_json(data)?;
validation::validate_data_size(data)?;
```

**Rate Limiting:**
```rust
let limiter = validation::RateLimiter::new(100, 60); // 100 req/min
limiter.check()?;
```

---

## Production Readiness Checklist

Before deploying your application with FireLocal, verify the following:

1.  [ ] **Data Persistence**: Confirmed the database path is pointing to a persistent volume/directory (especially for Docker/Containers).
2.  [ ] **Security Rules**: **Critical**. Ensure you call `load_rules()` on startup. Without rules, FireLocal defaults to **Allow All** (Dev Mode).
3.  [ ] **Backup Strategy**: Implement a cron job or scheduled task to `export` data or snapshot the database directory.
4.  [ ] **Compaction**: Schedule `compact()` calls during off-peak hours to manage disk space.
5.  [ ] **Error Handling**: Ensure your application handles `PermissionDenied` and `InvalidInput` errors gracefully.
6.  [ ] **Web Storage**: If using WASM, verify your data fits within Browser Quotas (simulating >10MB can be risky).

---

## Production Deployment

FireLocal is enterprise-ready with comprehensive production deployment capabilities. This section covers deployment strategies, security configurations, monitoring, and operational best practices.

### 🚀 Quick Production Setup

#### 1. System Requirements

**Minimum:**
- CPU: 2 cores, RAM: 4GB, Storage: 10GB SSD
- Rust 1.70+, OpenSSL dev libraries

**Recommended:**
- CPU: 4+ cores, RAM: 8GB+, Storage: 50GB+ SSD
- Dedicated monitoring and backup systems

#### 2. Installation

```bash
# Install from source
cargo install --path . --root /opt/firelocal

# Or download pre-built binaries
wget https://github.com/firelocal/firelocal/releases/latest/firelocal-linux.tar.gz
tar -xzf firelocal-linux.tar.gz -C /opt/firelocal
```

#### 3. Configuration

Create `/etc/firelocal/config.toml`:

```toml
[database]
data_path = "/var/lib/firelocal"
max_document_size = 10485760  # 10MB
memtable_threshold_mb = 64

[security]
authentication_enabled = true
rate_limit_enabled = true
max_requests_per_minute = 1000
audit_logging_enabled = true

[logging]
level = "info"
file_path = "/var/log/firelocal/firelocal.log"
```

#### 4. Service Setup

```bash
# Create firelocal user
sudo useradd -r -s /bin/false firelocal

# Setup directories
sudo mkdir -p /var/lib/firelocal /var/log/firelocal /etc/firelocal
sudo chown firelocal:firelocal /var/lib/firelocal /var/log/firelocal /etc/firelocal

# Install systemd service
sudo cp scripts/firelocal.service /etc/systemd/system/
sudo systemctl enable firelocal
sudo systemctl start firelocal
```

### 🔒 Security Configuration

FireLocal includes enterprise-grade security features:

#### Authentication & Authorization
```rust
use firelocal_core::security::{SecurityAuditor, SecurityContext};

// Create security auditor
let auditor = SecurityAuditor::new(security_config);

// Check permissions
let context = SecurityContext::authenticated("user123", vec!["reader".to_string()]);
auditor.check_permission(&context, "read", "users/alice")?;
```

#### Rate Limiting
```toml
[security]
rate_limit_enabled = true
max_requests_per_minute = 1000
blocked_ips = ["192.168.1.100"]
```

#### Input Sanitization
- Path traversal protection
- Prototype pollution prevention
- JSON validation and sanitization
- Size limits and depth validation

### 📊 Monitoring & Observability

#### Health Checks
```bash
curl http://localhost:9090/health
```

Response:
```json
{
  "status": "healthy",
  "uptime": 86400,
  "checks": {
    "database": {"status": "healthy"},
    "storage": {"status": "healthy"},
    "memory": {"status": "healthy"}
  }
}
```

#### Performance Metrics
```bash
curl http://localhost:9090/metrics
```

#### Logging Configuration
```rust
use firelocal_core::logging::{init_logging, LoggingConfig};

let config = LoggingConfig {
    level: "info".to_string(),
    file_path: Some("/var/log/firelocal/firelocal.log".to_string()),
    max_file_size: 10 * 1024 * 1024, // 10MB
    max_files: 5,
};

init_logging(&config)?;
```

### 🐳 Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/firelocal-server /usr/local/bin/
EXPOSE 8080 8443 9090
CMD ["firelocal-server", "--config", "/etc/firelocal/config.toml"]
```

```yaml
# docker-compose.yml
version: '3.8'
services:
  firelocal:
    build: .
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - firelocal_data:/var/lib/firelocal
      - ./config:/etc/firelocal
    environment:
      - FIRELOCAL_LOG_LEVEL=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9090/health"]
      interval: 30s
```

### ☸️ Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: firelocal
spec:
  replicas: 3
  selector:
    matchLabels:
      app: firelocal
  template:
    metadata:
      labels:
        app: firelocal
    spec:
      containers:
      - name: firelocal
        image: firelocal:latest
        ports:
        - containerPort: 8080
        - containerPort: 9090
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 10
```

### 📈 Performance Optimization

#### Configuration Tuning
```toml
[performance]
memtable_threshold_mb = 128
write_buffer_size_mb = 64
block_cache_size_mb = 256
wal_sync_mode = "normal"
compaction_style = "universal"
```

#### Benchmarking
```bash
# Run performance benchmarks
cargo bench --bench performance

# Generate HTML report
cargo bench --bench performance -- --output-format html
```

### 🔧 Operational Best Practices

#### Backup Strategy
```bash
#!/bin/bash
# Automated backup script
BACKUP_DIR="/backup/firelocal"
DATE=$(date +%Y%m%d_%H%M%S)

# Stop service for consistent backup
systemctl stop firelocal

# Backup data
tar -czf "$BACKUP_DIR/$DATE/data.tar.gz" -C /var/lib/firelocal .

# Start service
systemctl start firelocal

# Clean old backups
find "$BACKUP_DIR" -type d -mtime +7 -exec rm -rf {} \;
```

#### Monitoring Setup
- **Prometheus**: Metrics collection
- **Grafana**: Visualization and alerting
- **ELK Stack**: Log aggregation
- **Jaeger**: Distributed tracing

#### Security Hardening
```bash
# Firewall configuration
ufw allow 8080/tcp  # HTTP API
ufw allow 8443/tcp  # HTTPS API
ufw allow 9090/tcp  # Health checks

# SSL/TLS setup
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout /etc/firelocal/server.key \
    -out /etc/firelocal/server.crt
```

### 🚨 Troubleshooting

#### Common Issues

**High Memory Usage:**
```bash
# Check memory usage
ps aux | grep firelocal

# Tune memory settings
# Reduce memtable_threshold_mb in config
```

**Slow Performance:**
```bash
# Check I/O stats
iostat -x 1

# Monitor disk space
df -h /var/lib/firelocal
```

**Connection Issues:**
```bash
# Check service status
systemctl status firelocal

# View logs
journalctl -u firelocal -f
```

#### Debug Mode
```bash
# Enable debug logging
export FIRELOCAL_LOG_LEVEL=debug
systemctl restart firelocal
```

### 📚 Additional Resources

- **Complete Deployment Guide**: See `DEPLOYMENT.md` for detailed instructions
- **API Reference**: [API Documentation](#api-reference)
- **Security Guide**: [Security Rules](#security-rules)
- **Performance Tuning**: [Performance Guide](#performance-tuning)
- **Troubleshooting**: [Troubleshooting](#troubleshooting)

For enterprise support and consulting, contact enterprise@firelocal.dev

---

## Performance Tuning

### Best Practices

1. **Use Batch Operations** - Combine multiple writes
2. **Run Compaction** - Periodically optimize storage
3. **Flush Regularly** - Ensure data is written to disk
4. **Limit Document Size** - Keep documents reasonably sized
5. **Use Appropriate Paths** - Organize data hierarchically

### Benchmarks

| Operation | Time | Notes |
|-----------|------|-------|
| Put | ~1ms | With WAL |
| Get | ~0.5ms | From memtable |
| Delete | ~1ms | Tombstone marking |
| Batch (100 ops) | ~10ms | Single flush |
| Compaction | Variable | Depends on data size |

### Memory Usage

- **Memtable** - ~10MB per 100k documents
- **SST Files** - Depends on data size
- **Index** - ~1MB per 100k documents
- **Total** - Typically <100MB for most applications

### Disk Usage

- **WAL** - ~1KB per operation
- **SST Files** - Depends on data size
- **Indexes** - ~10% of data size
- **Total** - Typically 1-2x data size

---

## Troubleshooting

### Common Issues

#### Issue: "Database lock poisoned"

**Cause:** Database mutex became poisoned due to panic in another thread

**Solution:**
1. Close all connections to the database
2. Restart the application
3. Check for panics in logs

#### Issue: "Rules check failed"

**Cause:** Security rules denied the operation

**Solution:**
1. Check your security rules
2. Verify the operation is allowed
3. Check the document path

#### Issue: "Failed to open database"

**Cause:** Database directory doesn't exist or is not writable

**Solution:**
1. Ensure directory exists: `mkdir -p ./mydata`
2. Check write permissions: `chmod 755 ./mydata`
3. Check disk space

#### Issue: "Invalid JSON data"

**Cause:** Data is not valid JSON

**Solution:**
1. Validate JSON before writing
2. Use `serde_json` for serialization
3. Check for special characters

#### Issue: Slow Performance

**Cause:** Large memtable or many SST files

**Solution:**
1. Run compaction: `db.compact()?`
2. Flush memtable: `db.flush()?`
3. Check document sizes
4. Use batch operations

---

## FAQ

### Q: Can I use FireLocal in production?

**A:** Yes! FireLocal is production-ready with:
- Comprehensive error handling
- ACID transactions
- Security rules
- Durability guarantees
- Multi-platform support

### Q: How much data can FireLocal store?

**A:** Theoretically unlimited, practically limited by:
- Available disk space
- Available RAM (for memtable)
- Performance requirements

### Q: Is FireLocal thread-safe?

**A:** Yes, with proper locking:
- Use Arc<Mutex<FireLocal>> for shared access
- Each operation is atomic
- Transactions provide isolation

### Q: Can I sync with Firebase?

**A:** Not yet, but it's on the roadmap. Currently:
- You can export/import data
- Manual sync is possible
- Cloud sync coming in v1.1

### Q: What about encryption?

**A:** Encryption is on the roadmap:
- Currently: No built-in encryption
- Workaround: Encrypt data before storing
- v1.1: Built-in encryption support

### Q: How do I backup my data?

**A:** Several options:
1. Copy the database directory
2. Export to JSON: `firelocal export --output backup.json`
3. Use system backup tools

### Q: Can I use multiple databases?

**A:** Yes:
```rust
let db1 = FireLocal::new("./db1")?;
let db2 = FireLocal::new("./db2")?;
```

### Q: What's the maximum document size?

**A:** Practically unlimited, but:
- Recommended: <1MB per document
- Larger documents slow down operations
- Consider splitting large documents

### Q: How do I handle migrations?

**A:** Options:
1. Write migration scripts
2. Use batch operations
3. Export/import data
4. Version your schema

### Q: Is there a REST API?

**A:** Not yet, but:
- Language bindings available
- REST API coming in v1.2
- You can build your own wrapper

---

## Getting Help

- **Documentation**: [README.md](README.md)
- **Issues**: [GitHub Issues](https://github.com/rajdipk/Firelocal/issues)
- **Discussions**: [GitHub Discussions](https://github.com/rajdipk/Firelocal/discussions)
- **Examples**: [examples/](examples/) directory
- **Security**: [SECURITY_AUDIT.md](SECURITY_AUDIT.md)

---

## Appendix

### Glossary

- **LSM-Tree**: Log-Structured Merge-Tree, a write-optimized data structure
- **WAL**: Write-Ahead Log, ensures durability
- **Memtable**: In-memory sorted table
- **SSTable**: Sorted String Table, immutable on-disk file
- **Compaction**: Process of merging SSTables
- **Tombstone**: Marker for deleted entry
- **ACID**: Atomicity, Consistency, Isolation, Durability
- **OCC**: Optimistic Concurrency Control

### File Structure

```
project/
├── .env                      # Configuration
├── .firelocal/
│   └── data/
│       ├── wal.log          # Write-ahead log
│       ├── uuid1.sst        # SSTable file
│       ├── uuid2.sst        # SSTable file
│       └── ...
└── your-app/
```

### Version History

- **1.0.0** (2024-12-08): Production Release
  - Core storage engine
  - Input validation
  - Rate limiting
  - Security audit
  - Comprehensive testing
  - Complete documentation
  - Multi-language bindings
  - CLI tools

### Contributing

We welcome contributions! Please see:

- GitHub: <https://github.com/rajdipk/Firelocal>
- Issues: <https://github.com/rajdipk/Firelocal/issues>
- Pull Requests: <https://github.com/rajdipk/Firelocal/pulls>

### License

MIT License - see LICENSE file for details.

---

**Made with ❤️ using Rust 🦀**

For more information, visit: <https://github.com/rajdipk/Firelocal>

**Last Updated:** December 8, 2025  
**Version:** 1.0.0  
**Status:** Production Ready
