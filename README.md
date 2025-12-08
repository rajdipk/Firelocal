<div align="center">
  <img src="assets/firelocal.png" alt="FireLocal Logo" width="200"/>
  <h1>FireLocal</h1>
  <p>
    <strong>Offline-first database with Firestore API compatibility</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/firelocal-core">
      <img src="https://img.shields.io/crates/v/firelocal-core.svg" alt="Crates.io">
    </a>
    <a href="https://github.com/rajdipk/firelocal/actions">
      <img src="https://github.com/rajdipk/firelocal/actions/workflows/ci.yml/badge.svg" alt="Build Status">
    </a>
    <a href="https://opensource.org/licenses/MIT">
      <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT">
    </a>
  </p>
</div>

FireLocal is a production-ready, offline-first database that provides Firestore-compatible APIs for local data persistence. Perfect for mobile apps, desktop applications, and any scenario requiring local-first architecture.

## 📖 Documentation

For complete documentation, guides, and API reference, please visit:

📚 **[FireLocal Documentation](DOCUMENTATION.md)**

## ✨ Key Features

- **Firestore-Compatible API** - Familiar API for Firebase developers
- **Offline-First** - Works without internet connection
- **Multi-Platform** - Rust, JavaScript, Dart, Python support
- **ACID Transactions** - Reliable data operations
- **Security Rules** - Firebase-compatible security rules
- **Efficient Storage** - LSM-Tree based storage engine
- **CLI Tools** - Manage your database from the command line

## 🚀 Quick Start

1. **Install** the appropriate package for your platform:

   ```bash
   # Rust
   cargo add firelocal-core

   # JavaScript/Node.js
   npm install firelocal

   # Python
   pip install firelocal

   # Dart/Flutter
   flutter pub add firelocal_dart
   ```

2. **Basic Usage** (Rust example):

   ```rust
   use firelocal_core::FireLocal;
   use anyhow::Result;

   fn main() -> Result<()> {
       // Create or open a database
       let mut db = FireLocal::new("./mydata")?;

       // Write data
       db.put("users/alice".to_string(), br#"{"name":"Alice"}"#.to_vec())?;

       // Read data
       if let Some(data) = db.get("users/alice") {
           println!("User: {}", String::from_utf8_lossy(&data));
       }
       
       Ok(())
   }
   ```

## 📚 Learn More

- [Getting Started Guide](DOCUMENTATION.md#getting-started)
- [API Reference](DOCUMENTATION.md#api-reference)
- [Configuration Options](DOCUMENTATION.md#configuration)
- [Security Rules](DOCUMENTATION.md#security--rules)
- [Performance Tuning](DOCUMENTATION.md#performance--optimization)

## 📦 Language Bindings

FireLocal supports multiple programming languages:

- [Rust](bindings/rust/README.md) - Core implementation and primary API
- [JavaScript/Node.js](bindings/js/README.md) - N-API bindings for Node.js
- [Python](bindings/python/README.md) - Python bindings using PyO3
- [Dart/Flutter](bindings/dart/README.md) - FFI bindings for Flutter apps

## 🔧 CLI Tools

Manage your FireLocal databases from the command line:

```bash
# Initialize a new project
firelocal init

# Start interactive shell
firelocal shell

# Get help
firelocal --help

# Show database info
firelocal info

# Run database compaction
firelocal compact
```

## 🏗️ Architecture

FireLocal is built with a modular architecture:

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

## 🚀 Performance

- **Writes**: O(log n) with WAL durability
- **Reads**: O(log n) from memtable + SST
- **Batches**: Single WAL flush for all operations
- **Transactions**: Minimal overhead with version checking

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on how to get started.

## 📄 License

FireLocal is [MIT licensed](LICENSE).

## 🙏 Acknowledgments

- Inspired by Firebase Firestore and its ecosystem
- Built with ❤️ using Rust
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

- **Issues**: [GitHub Issues](https://github.com/rajdipk/Firelocal/issues)
- **Documentation**: See `docs/` directory
- **Examples**: See `examples/` directory

## Acknowledgments

Built with Rust 🦀 for performance and reliability.
