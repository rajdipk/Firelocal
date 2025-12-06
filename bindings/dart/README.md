# FireLocal Dart

Dart/Flutter bindings for FireLocal - an offline-first database with Firestore API compatibility.

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  firelocal_dart: ^0.1.0
```

## Quick Start

```dart
import 'package:firelocal_dart/firelocal_dart.dart';

void main() async {
  // Create database
  final db = FireLocal('./data');

  // Write documents
  await db.put('users/alice', {
    'name': 'Alice',
    'age': 30,
    'created': serverTimestamp(),
  });

  // Read documents
  final user = await db.get('users/alice');
  print(user);

  // Batch operations
  final batch = db.batch();
  batch.set('users/bob', {'name': 'Bob'});
  batch.set('users/charlie', {'name': 'Charlie'});
  await batch.commit();

  // Compaction
  final stats = await db.compact();
  print('Saved ${stats.sizeReductionPercent.toStringAsFixed(1)}% space');

  // Cleanup
  db.dispose();
}
```

## FieldValue Helpers

```dart
import 'package:firelocal_dart/firelocal_dart.dart';

// Server timestamp
await db.put('posts/1', {
  'title': 'Hello',
  'created_at': serverTimestamp(),
});

// Increment counter
await db.put('stats/views', {
  'count': increment(1),
});

// Array operations
await db.put('users/alice', {
  'tags': arrayUnion(['dart', 'flutter']),
});
```

## API Reference

### FireLocal

- `FireLocal(String path)` - Create database instance
- `Future<void> put(String key, Map value)` - Write document
- `Future<Map?> get(String key)` - Read document
- `Future<void> delete(String key)` - Delete document
- `WriteBatch batch()` - Create write batch
- `Future<CompactionStats> compact()` - Run compaction
- `Future<void> flush()` - Flush memtable to SST
- `void dispose()` - Close database

### WriteBatch

- `WriteBatch set(String path, Map data)` - Add set operation
- `WriteBatch update(String path, Map data)` - Add update operation
- `WriteBatch delete(String path)` - Add delete operation
- `Future<void> commit()` - Commit batch atomically

### FieldValue Functions

- `serverTimestamp()` - Current server time
- `increment(int n)` - Increment numeric field
- `arrayUnion(List elements)` - Add unique elements
- `arrayRemove(List elements)` - Remove elements
- `deleteField()` - Delete field

## Flutter Integration

```dart
import 'package:flutter/material.dart';
import 'package:firelocal_dart/firelocal_dart.dart';

class MyApp extends StatefulWidget {
  @override
  _MyAppState createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  late FireLocal db;

  @override
  void initState() {
    super.initState();
    db = FireLocal('./app_data');
  }

  @override
  void dispose() {
    db.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // Your UI here
  }
}
```

## Development

```bash
# Get dependencies
dart pub get

# Run tests
dart test

# Run with coverage
dart test --coverage
```

## License

MIT License
