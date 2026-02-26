<div align="center" style="background-color: #f8f9fa; padding: 2rem; border-radius: 8px; margin-bottom: 2rem;">
  <img src="assets/firelocal.png" alt="FireLocal Logo" width="500" style="max-width: 100%; height: auto;"/>
  <h1 >FireLocal</h1>
  <p>
    <strong>Offline-first database with Firestore API compatibility</strong>
  </p>
  <p>
    <strong>v1.0.0</strong>
  </p>
  <p>
    <a href="https://opensource.org/licenses/MIT">
      <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT">
    </a>
  </p>
</div>

**FireLocal** is a offline-first database engine that provides Firestore-compatible APIs for local data persistence. Built with Rust for performance and reliability, it's perfect for mobile apps, desktop applications, web applications, and any scenario requiring local-first architecture with zero external dependencies.

### Why FireLocal?

-  **🚀 Production Ready** - Enterprise-grade security, monitoring, and error handling
-  **🔒 Secure by Default** - Comprehensive security framework with authentication & authorization
-  **📊 Full Observability** - Built-in logging, metrics, and health monitoring
-  **📱 Offline-First** - Works seamlessly without internet connection
-  **⚡ High Performance** - LSM-Tree storage with comprehensive benchmarks
-  **🛡️ Battle-Tested** - Extensive testing suite with 67+ tests passing
-  **🔧 Enterprise Features** - Rate limiting, audit logging, input sanitization
-  **🌐 Multi-Platform** - Rust, JavaScript, Dart, Python, WASM support
-  **🔄 Reliable** - ACID transactions with WAL durability and recovery
-  **📚 Familiar** - Firestore-compatible API for Firebase developers

## 📖 Documentation & Guides

For complete documentation, guides, and API reference, please visit:

📚 **[FireLocal Complete Documentation](DOCUMENTATION.md)** - Comprehensive guide for all users and developers

### Quick Links
- [Getting Started](DOCUMENTATION.md#getting-started) - Installation and first steps
- [API Reference](DOCUMENTATION.md#api-reference) - Complete API documentation
- [Database Structure](DOCUMENTATION.md#database-structure) - How data is organized
- [Security Rules](DOCUMENTATION.md#security-rules) - Authentication and authorization
- [Production Deployment](DOCUMENTATION.md#production-deployment) - Enterprise deployment guide
- [Examples](DOCUMENTATION.md#examples) - Code examples for all languages
- [Troubleshooting](DOCUMENTATION.md#troubleshooting) - Common issues and solutions

## ✨ Key Features

- **Firestore-Compatible API** - Familiar API for Firebase developers
- **Offline-First** - Works without internet connection
- **Multi-Platform** - Rust, JavaScript, Dart, Python, WASM support
- **ACID Transactions** - Reliable data operations with OCC
- **Security Rules** - Firebase-compatible security rules engine
- **Efficient Storage** - LSM-Tree based storage with O(log n) operations
- **CLI Tools** - Manage your database from the command line
- **Production Ready** - Battle-tested error handling and recovery

##  Quick Start (5 minutes)

### 1. Install for Your Platform

```bash
# Rust
cargo add firelocal-core

# JavaScript/Node.js
npm install @firelocal/node

# Python
pip install firelocal

# Dart/Flutter
flutter pub add firelocal_dart
```

### 2. Basic Usage

**Rust:**
```rust
use firelocal_core::FireLocal;

fn main() -> anyhow::Result<()> {
    let mut db = FireLocal::new("./mydata")?;
    
    // Write
    db.put("users/alice".to_string(), 
           br#"{"name":"Alice","age":30}"#.to_vec())?;
    
    // Read
    if let Some(data) = db.get("users/alice") {
        println!("User: {}", String::from_utf8_lossy(&data));
    }
    
    // Delete
    db.delete("users/alice".to_string())?;
    
    Ok(())
}
```

**JavaScript:**
```javascript
const { FireLocal } = require('@firelocal/node');

const db = new FireLocal('./mydata');

// Write
db.put('users/alice', JSON.stringify({ name: 'Alice', age: 30 }));

// Read
const user = JSON.parse(db.get('users/alice'));
console.log(user);

// Delete
db.delete('users/alice');

db.close();
```

**Python:**
```python
from firelocal import FireLocal

db = FireLocal('./mydata')

# Write
db.put('users/alice', {'name': 'Alice', 'age': 30})

# Read
user = db.get('users/alice')
print(user)

# Delete
db.delete('users/alice')
```

**Dart:**
```dart
import 'package:firelocal_dart/firelocal_dart.dart';

void main() async {
  final db = FireLocal('./mydata');
  
  // Write
  await db.put('users/alice', {'name': 'Alice', 'age': 30});
  
  // Read
  final user = await db.get('users/alice');
  print(user);
  
  // Delete
  await db.delete('users/alice');
  
  db.dispose();
}
```

## 📦 Language Bindings

FireLocal supports multiple programming languages with full feature parity:

| Language | Package | Status | Docs |
|----------|---------|--------|------|
| **Rust** | `firelocal-core` | ✅ Production Ready | [Docs](https://docs.rs/firelocal-core) |
| **JavaScript** | `@firelocal/node` | ✅ Production Ready | [README](bindings/js/README.md) |
| **Python** | `firelocal` | ✅ Production Ready | [README](bindings/python/README.md) |
| **Dart/Flutter** | `firelocal_dart` | ✅ Production Ready | [README](bindings/dart/README.md) |
| **.NET** | `FireLocal` | ✅ Production Ready | [README](bindings/dotnet/README.md) |
| **WASM** | `firelocal-wasm` | ✅ Production Ready | [README](bindings/wasm/README.md) |

## 🏗️ Architecture Overview

FireLocal uses a proven LSM-Tree (Log-Structured Merge-Tree) architecture:

```
┌─────────────────────────────────────────┐
│         Application Layer               │
│  (Rust, JS, Dart, Python, .NET, WASM)  │
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│         FireLocal Core Engine           │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │    API Layer (Firestore-like)   │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌──────────┬──────────┬──────────┐   │
│  │  Rules   │  Index   │  Sync    │   │
│  │  Engine  │  Engine  │ Adapter  │   │
│  └──────────┴──────────┴──────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │  Storage Engine (LSM-Tree)      │   │
│  │  ┌──────────────────────────┐   │   │
│  │  │  WAL (Write-Ahead Log)   │   │   │
│  │  └──────────────────────────┘   │   │
│  │  ┌──────────────────────────┐   │   │
│  │  │  Memtable (In-Memory)    │   │   │
│  │  └──────────────────────────┘   │   │
│  │  ┌──────────────────────────┐   │   │
│  │  │  SST Files (Disk)        │   │   │
│  │  └──────────────────────────┘   │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

## ⚡ Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| **Put** | O(log n) | Write-ahead log + memtable |
| **Get** | O(log n) | Memtable + SST lookup |
| **Delete** | O(log n) | Tombstone marking |
| **Batch** | O(n log n) | Single WAL flush |
| **Transaction** | O(n) | OCC with version checking |
| **Compaction** | O(n log n) | Background SST merging |
| **Query** | O(n) | Full scan (indexes coming) |

## 🔒 Security Features

- **Firestore-Compatible Rules** - Use familiar Firebase security rules
- **Field-Level Security** - Control access at document and field level
- **Role-Based Access** - Implement custom authorization logic
- **Audit Logging** - Track all database operations
- **Encryption Ready** - Support for encrypted storage

## 🔧 CLI Tools

Manage your FireLocal databases from the command line:

```bash
# Initialize a new project
firelocal init

# Start interactive shell
firelocal shell

# Show database info
firelocal info

# Run compaction
firelocal compact

# Export data
firelocal export --output data.json

# Import data
firelocal import --input data.json

# Get help
firelocal --help
```

## 📊 Use Cases

- **Mobile Apps** - Offline-first mobile applications
- **Desktop Apps** - Local-first desktop applications
- **Web Apps** - Browser-based applications with IndexedDB backend
- **IoT Devices** - Lightweight database for edge devices
- **Embedded Systems** - Minimal resource usage
- **Progressive Web Apps** - Offline-capable web applications
- **Hybrid Apps** - React Native, Flutter, Electron applications

## 🚀 Getting Started

1. **Read the [Complete Documentation](DOCUMENTATION.md)** - Comprehensive guide
2. **Check [Language-Specific Examples](DOCUMENTATION.md#examples)** - Code samples
3. **Review [API Reference](DOCUMENTATION.md#api-reference)** - All available methods
4. **Explore [Security Rules](DOCUMENTATION.md#security-rules)** - Authentication setup
5. **Run [Tests](DOCUMENTATION.md#testing)** - Verify installation

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## 📄 License

FireLocal is [MIT licensed](LICENSE) - free for commercial and personal use.

## 🙏 Acknowledgments

- Inspired by Firebase Firestore and its ecosystem
- Built with ❤️ using Rust for performance and reliability
- LSM-Tree architecture proven by RocksDB, LevelDB, and others

## 📞 Support & Community

- **Documentation**: [DOCUMENTATION.md](DOCUMENTATION.md)
- **Issues**: [GitHub Issues](https://github.com/rajdipk/Firelocal/issues)
- **Discussions**: [GitHub Discussions](https://github.com/rajdipk/Firelocal/discussions)
- **Examples**: [examples/](examples/) directory

## 🗺️ Roadmap

### ✅ Completed
- [x] Core storage engine (WAL, Memtable, SST)
- [x] Firestore-compatible API
- [x] Security rules engine
- [x] Batch operations
- [x] Transactions with OCC
- [x] FieldValue helpers
- [x] CLI tools
- [x] Multi-language bindings (Rust, JS, Python, Dart, .NET)
- [x] WASM support
- [x] Production-ready error handling

### 🚀 Planned
- [ ] Composite indexes
- [ ] Advanced query operators
- [ ] Real-time sync with cloud
- [ ] Replication support
- [ ] Sharding support
- [ ] GraphQL API
- [ ] REST API

---

**Ready to get started?** → [Read the Complete Documentation](DOCUMENTATION.md)
