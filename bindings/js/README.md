# FireLocal for Node.js

Node.js/JavaScript bindings for FireLocal - an offline-first database with Firestore API compatibility.

## Installation

```bash
npm install @firelocal/node
```

## Quick Start

```javascript
const { FireLocal } = require('@firelocal/node');

// Create database
const db = new FireLocal('./data');

// Load security rules
db.loadRules('service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }');

// Write documents
db.put('users/alice', JSON.stringify({ name: 'Alice', age: 30 }));

// Read documents
const user = JSON.parse(db.get('users/alice'));
console.log(user);

// Delete documents
db.delete('users/alice');

// Batch operations
const batch = db.batch();
batch.set('users/bob', JSON.stringify({ name: 'Bob' }));
batch.set('users/charlie', JSON.stringify({ name: 'Charlie' }));
batch.delete('users/old');
batch.commit();

// Compaction
const stats = db.compact();
console.log(`Saved ${(stats.size_before - stats.size_after) / stats.size_before * 100}% space`);

// Cleanup
db.close();
```

## TypeScript Support

```typescript
import { FireLocal, WriteBatch, CompactionStats } from '@firelocal/node';

const db = new FireLocal('./data');
const user = JSON.parse(db.get('users/alice') || '{}');
```

## API Reference

### FireLocal

#### Constructor

```javascript
new FireLocal(path: string)
```

Create a new database instance.

**Parameters:**
- `path` - Path to the database directory

**Example:**
```javascript
const db = new FireLocal('./my_database');
```

#### loadRules(rules: string): void

Load Firestore-compatible security rules.

**Parameters:**
- `rules` - Firestore security rules string

**Example:**
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

#### put(key: string, value: string): void

Write a document.

**Parameters:**
- `key` - Document path (e.g., "users/alice")
- `value` - Document data as JSON string

**Example:**
```javascript
db.put('users/alice', JSON.stringify({
  name: 'Alice',
  age: 30,
  email: 'alice@example.com'
}));
```

#### get(key: string): string | null

Read a document.

**Parameters:**
- `key` - Document path

**Returns:**
- Document data as JSON string, or null if not found

**Example:**
```javascript
const data = db.get('users/alice');
if (data) {
  const user = JSON.parse(data);
  console.log(user.name);
}
```

#### delete(key: string): void

Delete a document.

**Parameters:**
- `key` - Document path

**Example:**
```javascript
db.delete('users/alice');
```

#### batch(): WriteBatch

Create a new write batch for atomic operations.

**Returns:**
- WriteBatch instance

**Example:**
```javascript
const batch = db.batch();
batch.set('users/alice', JSON.stringify({ name: 'Alice' }));
batch.set('users/bob', JSON.stringify({ name: 'Bob' }));
batch.commit();
```

#### compact(): CompactionStats

Run compaction to merge SST files and remove tombstones.

**Returns:**
- CompactionStats object with metrics

**Example:**
```javascript
const stats = db.compact();
console.log(`Files before: ${stats.files_before}`);
console.log(`Files after: ${stats.files_after}`);
console.log(`Space saved: ${stats.size_before - stats.size_after} bytes`);
```

#### flush(): void

Flush memtable to SST file.

**Example:**
```javascript
db.flush();
```

#### close(): void

Close the database.

**Example:**
```javascript
db.close();
```

### WriteBatch

#### set(path: string, data: string): WriteBatch

Add a set operation to the batch.

**Parameters:**
- `path` - Document path
- `data` - Document data as JSON string

**Returns:**
- This WriteBatch instance (for chaining)

#### update(path: string, data: string): WriteBatch

Add an update operation to the batch.

**Parameters:**
- `path` - Document path
- `data` - Document data as JSON string

**Returns:**
- This WriteBatch instance (for chaining)

#### delete(path: string): WriteBatch

Add a delete operation to the batch.

**Parameters:**
- `path` - Document path

**Returns:**
- This WriteBatch instance (for chaining)

#### commit(): void

Commit the batch atomically.

All operations in the batch will be committed together. If any operation fails, the entire batch will be rolled back.

### CompactionStats

Interface containing compaction statistics:

```typescript
interface CompactionStats {
  files_before: number;        // Number of files before compaction
  files_after: number;         // Number of files after compaction
  entries_before: number;      // Number of entries before compaction
  entries_after: number;       // Number of entries after compaction
  tombstones_removed: number;  // Number of tombstones removed
  size_before: number;         // Size before compaction (bytes)
  size_after: number;          // Size after compaction (bytes)
}
```

## Platform Support

- Windows (x64)
- macOS (Intel and Apple Silicon)
- Linux (x64 and ARM64)

## Error Handling

All operations may throw errors. Use try-catch blocks:

```javascript
try {
  db.put('users/alice', JSON.stringify({ name: 'Alice' }));
} catch (error) {
  console.error('Failed to write document:', error.message);
}
```

Common errors:

- **PermissionDenied** - Security rules denied the operation
- **NotFound** - Document not found
- **InvalidData** - Invalid JSON data
- **Other** - Database operation failed

## Security Rules

FireLocal supports Firestore-compatible security rules. Rules are evaluated for every read and write operation.

Example rules:

```javascript
db.loadRules(`
  service cloud.firestore {
    match /databases/{database}/documents {
      // Allow public read, authenticated write
      match /public/{document=**} {
        allow read: if true;
        allow write: if request.auth != null;
      }
      
      // Allow users to read/write their own documents
      match /users/{userId} {
        allow read, write: if request.auth.uid == userId;
      }
      
      // Deny all other access
      match /{document=**} {
        allow read, write: if false;
      }
    }
  }
`);
```

## Performance Tips

1. **Use Batch Operations** - Batch multiple writes for better performance
2. **Run Compaction** - Periodically run compaction to reclaim space
3. **Flush Memtable** - Call flush() to ensure data is written to disk
4. **Limit Document Size** - Keep documents reasonably sized

## Examples

### User Management

```javascript
const db = new FireLocal('./data');

// Create user
db.put('users/alice', JSON.stringify({
  name: 'Alice',
  email: 'alice@example.com',
  created: Date.now()
}));

// Update user
const user = JSON.parse(db.get('users/alice'));
user.email = 'alice.new@example.com';
db.put('users/alice', JSON.stringify(user));

// Delete user
db.delete('users/alice');
```

### Batch Operations

```javascript
const batch = db.batch();

// Add multiple users
batch.set('users/alice', JSON.stringify({ name: 'Alice' }));
batch.set('users/bob', JSON.stringify({ name: 'Bob' }));
batch.set('users/charlie', JSON.stringify({ name: 'Charlie' }));

// Delete old user
batch.delete('users/old_user');

// Commit all at once
batch.commit();
```

### Data Export

```javascript
function exportDatabase(db, outputPath) {
  const fs = require('fs');
  const data = {};
  
  // Note: This is a simple example
  // In production, you'd need to implement proper iteration
  
  fs.writeFileSync(outputPath, JSON.stringify(data, null, 2));
}
```

## Troubleshooting

### Database Lock Error

If you see "Database lock poisoned" errors, the database may be corrupted. Try:

1. Close all connections to the database
2. Delete the database directory
3. Recreate the database

### Permission Denied Errors

If you get "Security rules check failed" errors:

1. Check your security rules
2. Ensure rules allow the operation
3. Verify the document path is correct

### Performance Issues

If the database is slow:

1. Run compaction: `db.compact()`
2. Flush memtable: `db.flush()`
3. Check document sizes
4. Use batch operations for multiple writes

## License

MIT

## Support

For issues, questions, or contributions, visit:
https://github.com/rajdipk/Firelocal
