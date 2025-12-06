# FireLocal WASM

WebAssembly bindings for FireLocal - run offline-first database in the browser!

## Installation

```bash
npm install firelocal-wasm
```

## Quick Start

```javascript
import init, { FireLocal, serverTimestamp, increment } from 'firelocal-wasm';

async function main() {
  // Initialize WASM
  await init();

  // Create database (uses IndexedDB)
  const db = new FireLocal('my-app-db');

  // Write documents
  await db.put('users/alice', {
    name: 'Alice',
    age: 30,
    created: serverTimestamp()
  });

  // Read documents
  const user = await db.get('users/alice');
  console.log(user);

  // Batch operations
  const batch = db.batch();
  batch.set('users/bob', { name: 'Bob' });
  batch.set('users/charlie', { name: 'Charlie' });
  await batch.commit();

  // Compaction
  const stats = await db.compact();
  console.log(`Removed ${stats.tombstones_removed} tombstones`);
}

main();
```

## FieldValue Helpers

```javascript
import { serverTimestamp, increment, arrayUnion } from 'firelocal-wasm';

await db.put('posts/1', {
  title: 'Hello WASM!',
  views: increment(1),
  tags: arrayUnion(['wasm', 'rust']),
  updated: serverTimestamp()
});
```

## Browser Support

- Chrome/Edge 87+
- Firefox 78+
- Safari 14+

Requires WebAssembly and IndexedDB support.

## API Reference

### FireLocal

- `new FireLocal(path: string)` - Create database
- `put(key: string, value: object): Promise<void>` - Write document
- `get(key: string): Promise<object>` - Read document
- `delete(key: string): Promise<void>` - Delete document
- `batch(): WriteBatch` - Create batch
- `compact(): Promise<CompactionStats>` - Run compaction

### WriteBatch

- `set(path: string, data: object)` - Add set operation
- `delete(path: string)` - Add delete operation
- `commit(): Promise<void>` - Commit atomically

### FieldValue Functions

- `serverTimestamp()` - Current timestamp
- `increment(n: number)` - Increment field
- `arrayUnion(elements: any[])` - Add to array
- `arrayRemove(elements: any[])` - Remove from array
- `deleteField()` - Delete field

## Building from Source

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build for web
npm run build

# Build for Node.js
npm run build:nodejs

# Run tests
npm test
```

## License

MIT License
