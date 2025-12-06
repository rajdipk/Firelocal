# FireLocal Python Bindings

Python bindings for FireLocal - an offline-first database with Firestore API compatibility.

## Installation

```bash
pip install firelocal
```

## Quick Start

```python
from firelocal import FireLocal, server_timestamp, increment

# Create database
db = FireLocal("./data")

# Write documents
db.put("users/alice", {
    "name": "Alice",
    "age": 30,
    "created": server_timestamp()
})

# Read documents
user = db.get("users/alice")
print(user)

# Batch operations
batch = db.batch()
batch.set("users/bob", {"name": "Bob"})
batch.set("users/charlie", {"name": "Charlie"})
batch.commit()

# Compaction
stats = db.compact()
print(f"Saved {stats.size_reduction_percent:.1f}% space")
```

## FieldValue Helpers

```python
from firelocal import (
    server_timestamp,
    increment,
    array_union,
    array_remove,
    delete_field,
)

# Server timestamp
db.put("posts/1", {
    "title": "Hello",
    "created_at": server_timestamp()
})

# Increment counter
db.put("stats/views", {
    "count": increment(1)
})

# Array operations
db.put("users/alice", {
    "tags": array_union(["python", "rust"])
})
```

## API Reference

### FireLocal

- `__init__(path: str)` - Create database instance
- `put(key: str, value: dict)` - Write document
- `get(key: str) -> dict` - Read document
- `delete(key: str)` - Delete document
- `batch() -> WriteBatch` - Create write batch
- `compact() -> CompactionStats` - Run compaction
- `flush()` - Flush memtable to SST

### WriteBatch

- `set(path: str, data: dict)` - Add set operation
- `update(path: str, data: dict)` - Add update operation
- `delete(path: str)` - Add delete operation
- `commit()` - Commit batch atomically

## Development

```bash
# Install dev dependencies
pip install -e ".[dev]"

# Run tests
pytest tests/

# Run with coverage
pytest --cov=firelocal tests/
```

## License

MIT License
