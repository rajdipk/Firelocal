# 📚 FireLocal Platform-Specific Documentation

This guide provides detailed documentation for each platform and language binding of FireLocal.

## 🦀 **Rust Core Library**

### **Installation**
```bash
# Add to Cargo.toml
[dependencies]
firelocal-core = { git = "https://github.com/rajdipk/Firelocal.git" }

# Or install from local path
firelocal-core = { path = "/home/pikachu/projects/Firelocal/firelocal-core" }
```

### **Usage**
```rust
use firelocal_core::FireLocal;
use anyhow::Result;

fn main() -> Result<()> {
    let mut db = FireLocal::new("./my_database")?;
    
    // Store data
    db.put("users/alice".to_string(), 
           serde_json::to_vec(&user_data)?)?;
    
    // Retrieve data
    if let Some(data) = db.get("users/alice") {
        let user: User = serde_json::from_slice(&data)?;
        println!("User: {}", user.name);
    }
    
    Ok(())
}
```

### **Features**
- ✅ Full CRUD operations
- ✅ ACID transactions
- ✅ LSM-Tree storage engine
- ✅ Write-ahead logging (WAL)
- ✅ Automatic compaction
- ✅ Security rules engine
- ✅ Performance monitoring
- ✅ Health checks

### **Performance**
- **Write**: ~30 ops/sec
- **Read**: ~400K ops/sec
- **Memory**: Efficient LSM-Tree structure
- **Storage**: Compressed SST files

---

## 🟢 **JavaScript/Node.js**

### **Installation**
```bash
# From npm (when published)
npm install @firelocal/node

# From source
git clone https://github.com/rajdipk/Firelocal.git
cd Firelocal/bindings/js
npm install
npm run build
```

### **Usage**
```javascript
const { FireLocal } = require('@firelocal/node');

// Initialize database
const db = new FireLocal('./my_database');

// Store data
db.put('users/alice', JSON.stringify({
    name: 'Alice',
    email: 'alice@example.com',
    age: 28
}));

// Retrieve data
const userData = db.get('users/alice');
if (userData) {
    const user = JSON.parse(userData);
    console.log('User:', user.name);
}

// Batch operations
const batch = db.batch();
batch.set('users/bob', { name: 'Bob', age: 30 });
batch.set('users/charlie', { name: 'Charlie', age: 25 });
batch.commit();
```

### **API Reference**
```javascript
// Constructor
const db = new FireLocal(databasePath);

// Core operations
db.put(key, jsonString);        // Store document
db.get(key);                    // Retrieve document (string or null)
db.delete(key);                 // Delete document

// Batch operations
const batch = db.batch();
batch.set(key, data);           // Add set operation
batch.update(key, data);        // Add update operation
batch.delete(key);              // Add delete operation
batch.commit();                 // Execute batch atomically

// Database management
db.loadRules(rulesString);      // Load security rules
db.compact();                   // Run compaction
db.flush();                     // Flush to disk
```

### **Platform Support**
- ✅ Windows (x64)
- ✅ macOS (x64, arm64)
- ✅ Linux (x64, arm64)

### **Requirements**
- Node.js 14+
- NAPI-RS native bindings

---

## 🐍 **Python**

### **Installation**
```bash
# From PyPI (when published)
pip install firelocal

# From source
git clone https://github.com/rajdipk/Firelocal.git
cd Firelocal/bindings/python
pip install -e .
```

### **Usage**
```python
import firelocal
import json

# Initialize database
db = firelocal.FireLocal('./my_database')

# Store data
user_data = {
    'name': 'Alice',
    'email': 'alice@example.com',
    'age': 28
}
db.put('users/alice', json.dumps(user_data))

# Retrieve data
user_json = db.get('users/alice')
if user_json:
    user = json.loads(user_json)
    print(f"User: {user['name']}")

# Batch operations
batch = db.batch()
batch.set('users/bob', {'name': 'Bob', 'age': 30})
batch.set('users/charlie', {'name': 'Charlie', 'age': 25})
batch.commit()

# Context manager
with firelocal.FireLocal('./temp_db') as db:
    db.put('test', {'value': 42})
    # Automatically closed
```

### **API Reference**
```python
# Constructor
db = firelocal.FireLocal(database_path)

# Core operations
db.put(key, dict_data)          # Store document
db.get(key)                    # Retrieve document (dict or None)
db.delete(key)                 # Delete document

# Batch operations
batch = db.batch()
batch.set(key, data)           # Add set operation
batch.update(key, data)        # Add update operation
batch.delete(key)              # Add delete operation
batch.commit()                 # Execute batch atomically

# Database management
db.load_rules(rules_string)     # Load security rules
stats = db.compact()            # Run compaction
db.flush()                     # Flush to disk
```

### **Platform Support**
- ✅ Windows (x64)
- ✅ macOS (x64, arm64)
- ✅ Linux (x64, arm64)

### **Requirements**
- Python 3.8+
- ctypes (built-in)
- FireLocal shared library

---

## 🌐 **WebAssembly (Browser)**

### **Installation**
```bash
# From npm (when published)
npm install firelocal-wasm

# From source
git clone https://github.com/rajdipk/Firelocal.git
cd Firelocal/firelocal-wasm
wasm-pack build --target web
```

### **Usage**
```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import init, { FireLocal } from './pkg/firelocal_wasm.js';
        
        async function run() {
            await init();
            
            // Initialize database (uses IndexedDB)
            const db = new FireLocal('my_app_db');
            
            // Store data
            db.put('users/alice', JSON.stringify({
                name: 'Alice',
                email: 'alice@example.com'
            }));
            
            // Retrieve data
            const userData = db.get('users/alice');
            console.log('User:', JSON.parse(userData));
        }
        
        run();
    </script>
</head>
<body>
    <h1>FireLocal WASM Demo</h1>
</body>
</html>
```

### **API Reference**
```javascript
// Constructor
const db = new FireLocal(database_name);

// Core operations
db.put(key, jsonString);        // Store document
db.get(key);                    // Retrieve document (string or null)
db.delete(key);                 // Delete document

// Batch operations
const batch = db.batch();
batch.set(key, data);           // Add set operation
batch.update(key, data);        // Add update operation
batch.delete(key);              // Add delete operation
batch.commit();                 // Execute batch atomically
```

### **Features**
- ✅ IndexedDB persistence
- ✅ Same API as Node.js
- ✅ Browser compatibility
- ✅ Offline-first capability

### **Requirements**
- Modern browser with IndexedDB support
- WebAssembly support

---

## 🎯 **Dart/Flutter**

### **Installation**
```bash
# From pub.dev (when published)
dart pub add firelocal

# From source
git clone https://github.com/rajdipk/Firelocal.git
cd Firelocal/bindings/dart
dart pub get
```

### **Usage**
```dart
import 'package:firelocal/firelocal.dart';

void main() async {
  // Initialize database
  final db = FireLocal('./my_database');
  
  // Store data
  final userData = {
    'name': 'Alice',
    'email': 'alice@example.com',
    'age': 28
  };
  db.put('users/alice', jsonEncode(userData));
  
  // Retrieve data
  final userJson = db.get('users/alice');
  if (userJson != null) {
    final user = jsonDecode(userJson);
    print('User: ${user['name']}');
  }
  
  // Batch operations
  final batch = db.batch();
  batch.set('users/bob', {'name': 'Bob', 'age': 30});
  batch.set('users/charlie', {'name': 'Charlie', 'age': 25});
  batch.commit();
}
```

### **API Reference**
```dart
// Constructor
final db = FireLocal(database_path);

// Core operations
db.put(key, jsonString);         // Store document
db.get(key);                     // Retrieve document (String or null)
db.delete(key);                  // Delete document

// Batch operations
final batch = db.batch();
batch.set(key, data);            // Add set operation
batch.update(key, data);         // Add update operation
batch.delete(key);               // Add delete operation
batch.commit();                  // Execute batch atomically
```

### **Platform Support**
- ✅ Android
- ✅ iOS
- ✅ macOS
- ✅ Linux
- ✅ Windows

### **Requirements**
- Dart 2.17+
- Flutter 3.0+ (for Flutter apps)
- dart:ffi library

---

## 🔷 **C#/.NET**

### **Installation**
```bash
# From NuGet (when published)
dotnet add package FireLocal

# From source
git clone https://github.com/rajdipk/Firelocal.git
cd Firelocal/bindings/dotnet
dotnet add reference FireLocal.csproj
```

### **Usage**
```csharp
using FireLocal;
using System.Text.Json;

// Initialize database
var db = new FireLocal("./my_database");

// Store data
var userData = new {
    Name = "Alice",
    Email = "alice@example.com",
    Age = 28
};
db.Put("users/alice", JsonSerializer.Serialize(userData));

// Retrieve data
var userJson = db.Get("users/alice");
if (userJson != null) {
    var user = JsonSerializer.Deserialize<dynamic>(userJson);
    Console.WriteLine($"User: {user.Name}");
}

// Batch operations
var batch = db.CreateBatch();
batch.Set("users/bob", JsonSerializer.Serialize(new { Name = "Bob", Age = 30 }));
batch.Set("users/charlie", JsonSerializer.Serialize(new { Name = "Charlie", Age = 25 }));
batch.Commit();
```

### **API Reference**
```csharp
// Constructor
var db = new FireLocal(database_path);

// Core operations
db.Put(key, jsonString);          // Store document
db.Get(key);                      // Retrieve document (string or null)
db.Delete(key);                   // Delete document

// Batch operations
var batch = db.CreateBatch();
batch.Set(key, data);             // Add set operation
batch.Update(key, data);          // Add update operation
batch.Delete(key);                // Add delete operation
batch.Commit();                   // Execute batch atomically

// Database management
db.LoadRules(rulesString);        // Load security rules
var stats = db.Compact();         // Run compaction
db.Flush();                       // Flush to disk
```

### **Platform Support**
- ✅ Windows (x64)
- ✅ Linux (x64)
- ✅ macOS (x64)

### **Requirements**
- .NET 6.0+
- P/Invoke native library

---

## 💻 **CLI Tool**

### **Installation**
```bash
# From source
git clone https://github.com/rajdipk/Firelocal.git
cd Firelocal/firelocal-cli
cargo install --path .

# Or from crates.io (when published)
cargo install firelocal-cli
```

### **Usage**
```bash
# Initialize database
firelocal-cli init ./my_db

# Store data
firelocal-cli put users/alice '{"name":"Alice","email":"alice@example.com"}'

# Retrieve data
firelocal-cli get users/alice

# List keys (if available)
firelocal-cli list

# Delete data
firelocal-cli delete users/alice

# Load security rules
firelocal-cli rules rules.json

# Compact database
firelocal-cli compact

# Database statistics
firelocal-cli stats
```

### **Commands**
- `init <path>` - Initialize new database
- `put <key> <json>` - Store JSON document
- `get <key>` - Retrieve document
- `delete <key>` - Delete document
- `list` - List all keys
- `rules <file>` - Load security rules
- `compact` - Run compaction
- `stats` - Show database statistics

---

## 🚀 **Deployment & Production**

### **Production Considerations**

#### **Security**
- Load security rules before allowing operations
- Use authentication in production
- Validate input data
- Enable audit logging

#### **Performance**
- Use batch operations for multiple writes
- Compact database regularly
- Monitor WAL size
- Use appropriate cache sizes

#### **Reliability**
- Flush data before shutdown
- Implement backup strategies
- Monitor disk space
- Handle errors gracefully

#### **Scaling**
- Multiple database instances for different data types
- Sharding by key prefix
- Read replicas (future feature)
- Distributed transactions (future feature)

### **Best Practices**

#### **Database Design**
```rust
// Good: Organized by type
users/user_001
users/user_002
products/prod_001
products/prod_002
orders/order_001

// Bad: Mixed types
data/item_1
data/item_2
misc/entry_1
```

#### **Batch Operations**
```javascript
// Good: Use batches for multiple operations
const batch = db.batch();
batch.set('users/alice', aliceData);
batch.set('users/bob', bobData);
batch.set('users/charlie', charlieData);
batch.commit();

// Bad: Individual operations
db.put('users/alice', aliceData);
db.put('users/bob', bobData);
db.put('users/charlie', charlieData);
```

#### **Error Handling**
```python
try:
    db.put('users/alice', user_data)
except Exception as e:
    logger.error(f"Failed to store user: {e}")
    # Handle error appropriately
```

---

## 📊 **Performance Benchmarks**

### **Rust Core**
- **Write**: 31.46 ops/sec
- **Read**: 411,271.81 ops/sec
- **Mixed**: 63.00 ops/sec
- **Large Docs**: 32.62 ops/sec

### **JavaScript/Node.js**
- **Write**: ~25-30 ops/sec
- **Read**: ~350K-400K ops/sec
- **Memory**: ~50-100MB base + data

### **Python**
- **Write**: ~20-25 ops/sec
- **Read**: ~300K-350K ops/sec
- **Memory**: ~60-120MB base + data

### **WASM**
- **Write**: ~15-20 ops/sec
- **Read**: ~200K-250K ops/sec
- **Memory**: Browser-dependent

---

## 🔧 **Troubleshooting**

### **Common Issues**

#### **Library Loading Errors**
```bash
# Python: Ensure shared library is in path
export LD_LIBRARY_PATH=/path/to/firelocal/lib

# Node.js: Check platform-specific binary
ls -la *.node

# Dart: Verify ffi library is available
dart --version
```

#### **Permission Errors**
```bash
# Check file permissions
ls -la database_path/

# Fix permissions
chmod 755 database_path/
```

#### **Memory Issues**
```rust
// Monitor memory usage
db.compact();  // Free up space
```

### **Debug Mode**
```bash
# Enable debug logging
RUST_LOG=debug firelocal-cli get key

# Python debug
import logging
logging.basicConfig(level=logging.DEBUG)
```

---

## 📞 **Support & Community**

- **GitHub Issues**: https://github.com/rajdipk/Firelocal/issues
- **Documentation**: https://github.com/rajdipk/Firelocal/blob/main/DOCUMENTATION.md
- **Examples**: https://github.com/rajdipk/Firelocal/tree/main/examples

---

**This documentation covers all currently supported platforms and provides comprehensive guidance for using FireLocal in any environment!** 🚀
