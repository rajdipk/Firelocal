# FireLocal - Complete User & Developer Guide

**Version:** 1.0.0  
**Last Updated:** December 8, 2025  
**Status:** Production Ready

---

## Table of Contents

1. [Introduction](#introduction)
2. [What is FireLocal?](#what-is-firelocal)
3. [Architecture Overview](#architecture-overview)
4. [Getting Started](#getting-started)
5. [Core Concepts](#core-concepts)
6. [Installation Guide](#installation-guide)
7. [Database Structure](#database-structure)
8. [How It Works](#how-it-works)
9. [API Reference](#api-reference)
10. [Security Rules](#security-rules)
11. [Examples](#examples)
12. [Advanced Topics](#advanced-topics)
13. [Performance Tuning](#performance-tuning)
14. [Troubleshooting](#troubleshooting)
15. [FAQ](#faq)

---

## Introduction

Welcome to FireLocal! This comprehensive guide will help you understand, install, and use FireLocal effectively, whether you're a beginner just starting out or an advanced developer building production applications.

### Who Should Read This?

- **Beginners**: Start with [What is FireLocal?](#what-is-firelocal) and [Getting Started](#getting-started)
- **Mobile/Web Developers**: Jump to [Installation Guide](#installation-guide) for your platform
- **System Architects**: Review [Architecture Overview](#architecture-overview) and [Performance](#performance-tuning)
- **DevOps Engineers**: See [Database Structure](#database-structure) and [Troubleshooting](#troubleshooting)

---

## What is FireLocal?

FireLocal is an **offline-first database** that provides a Firestore-compatible API for local data persistence. It's designed for applications that need to work offline, sync data when online, and provide a seamless user experience regardless of network conditions.

### Key Features

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

FireLocal is built on a layered architecture that separates concerns and provides flexibility:

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│    (Rust, JavaScript, Dart, Python, .NET, CLI, WASM)       │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    Language Bindings                        │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐  │
│  │   Rust   │    JS    │   Dart   │  Python  │  .NET    │  │
│  │   API    │  (NAPI)  │  (FFI)   │ (ctypes) │ (P/Invoke)│ │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    FireLocal Core (Rust)                    │
│  ┌───────────────────────────────────────────────────────┐  │
│  │           API Layer (Firestore-like)                  │  │
│  │  • CollectionReference  • DocumentReference           │  │
│  │  • Query  • WriteBatch  • Transaction                 │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌──────────────┬──────────────┬──────────────┬─────────┐  │
│  │ Rules Engine │ Index Engine │ Sync Adapter │ Config  │  │
│  │ (Firebase    │ (Basic +     │ (Pluggable)  │ (.env)  │  │
│  │  compatible) │  Composite)  │              │         │  │
│  └──────────────┴──────────────┴──────────────┴─────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │         Storage Engine (LSM-Tree Architecture)        │  │
│  │  ┌──────────┬──────────┬──────────┬──────────────┐   │  │
│  │  │   WAL    │ Memtable │  SSTable │  Compaction  │   │  │
│  │  │ (Write-  │ (In-     │ (Sorted  │  (Background │   │  │
│  │  │  Ahead   │  Memory  │  String  │   Merging)   │   │  │
│  │  │   Log)   │  Table)  │  Table)  │              │   │  │
│  │  └──────────┴──────────┴──────────┴──────────────┘   │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    File System                              │
│  • .firelocal/data/wal.log                                  │
│  • .firelocal/data/*.sst                                    │
│  • .env (configuration)                                     │
└─────────────────────────────────────────────────────────────┘
```

### Component Breakdown

#### 1. **Storage Engine (LSM-Tree)**

The foundation of FireLocal uses a Log-Structured Merge-Tree (LSM-Tree) architecture:

- **Write-Ahead Log (WAL)**: All writes are first appended to a log file for durability
- **Memtable**: In-memory sorted structure for fast writes and recent reads
- **SSTable**: Immutable sorted files on disk for persistent storage
- **Compaction**: Background process that merges SSTables and removes deleted entries

**Benefits:**

- Fast writes (append-only to WAL)
- Crash recovery (replay WAL on startup)
- Efficient storage (compaction removes tombstones)

#### 2. **Index Engine**

Provides fast document lookups and query execution:

- **Basic Index**: Field-level inverted index for equality queries
- **Composite Index**: Multi-field indexes for complex queries
- **Query Operators**: `==`, `in`, `array-contains`, `<`, `>`, `<=`, `>=`

#### 3. **Rules Engine**

Firebase-compatible security rules for access control:

- Pattern matching on document paths
- Conditional allow/deny statements
- Context-aware evaluation (user auth, request data)

#### 4. Simplified Validation

FireLocal uses minimal validation to maximize flexibility:

1. **Path Validation**
   - Must be non-empty
   - Maximum 4096 characters
   - All characters allowed except control characters

2. **Data Validation**
   - Must be non-empty
   - Must be valid UTF-8
   - Maximum 100MB per document
   - No strict JSON validation for better performance

3. **Security Rules**
   - Optional - only enforced if rules are loaded
   - Maximum 1MB for rules
   - No strict format requirements

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

## Getting Started

### Prerequisites

- **For Rust**: Rust 1.70+ and Cargo
- **For JavaScript**: Node.js 16+ or modern browser
- **For Dart**: Dart SDK 3.0+ or Flutter 3.0+
- **For Python**: Python 3.8+
- **.NET**: .NET 6.0+

### Quick Start (5 Minutes)

#### Option 1: Using Rust

```rust
use firelocal_core::FireLocal;

fn main() -> anyhow::Result<()> {
    // Create database
    let mut db = FireLocal::new("./my-data")?;
    
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

#### Option 2: Using JavaScript

```javascript
const { FireLocal } = require('@firelocal/node');

const db = new FireLocal('./my-data');

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

db.close();
```

#### Option 3: Using Dart

```dart
import 'package:firelocal_dart/firelocal_dart.dart';

void main() async {
  final db = FireLocal('./my-data');
  
  // Write data
  await db.put('users/alice', {'name': 'Alice', 'age': 30});
  
  // Read data
  final data = await db.get('users/alice');
  print('Found: $data');
  
  // Delete data
  await db.delete('users/alice');
  
  db.close();
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

**Rust:**
```rust
let mut db = FireLocal::new("./mydata")?;
let mut db = FireLocal::new_with_config("./mydata")?;
```

**JavaScript:**
```javascript
const db = new FireLocal('./mydata');
```

**Python:**
```python
db = FireLocal('./mydata')
```

**Dart:**
```dart
final db = FireLocal('./mydata');
```

#### Write Operations

**Put (Create/Update):**
```rust
db.put("users/alice".to_string(), data)?;
```

**Batch Put:**
```rust
let mut batch = db.batch();
batch.set("users/alice".to_string(), data1);
batch.set("users/bob".to_string(), data2);
db.commit_batch(&batch)?;
```

**Update (Merge):**
```rust
let mut batch = db.batch();
batch.update("users/alice".to_string(), partial_data);
db.commit_batch(&batch)?;
```

#### Read Operations

**Get Single Document:**
```rust
if let Some(data) = db.get("users/alice") {
    println!("{}", String::from_utf8_lossy(&data));
}
```

**Query Documents:**
```rust
let query = QueryAst::new("users");
let results = db.query(&query)?;
```

#### Delete Operations

**Delete Single Document:**
```rust
db.delete("users/alice".to_string())?;
```

**Batch Delete:**
```rust
let mut batch = db.batch();
batch.delete("users/alice".to_string());
batch.delete("users/bob".to_string());
db.commit_batch(&batch)?;
```

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

**Rust:**
```rust
let rules = r#"
service cloud.firestore {
  match /databases/{database}/documents {
    match /users/{userId} {
      allow read, write: if request.auth.uid == userId;
    }
  }
}
"#;
db.load_rules(rules)?;
```

**JavaScript:**
```javascript
db.loadRules(`
  service cloud.firestore {
    match /databases/{database}/documents {
      match /users/{userId} {
        allow read, write: if request.auth.uid == userId;
      }
    }
  }
`);
```

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
