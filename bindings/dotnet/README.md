# FireLocal for .NET

.NET bindings for FireLocal - an offline-first database with Firestore API compatibility.

## Installation

```bash
dotnet add package FireLocal
```

Or via NuGet Package Manager:

```
Install-Package FireLocal
```

## Quick Start

```csharp
using FireLocal;
using System.Collections.Generic;

// Create database
using var db = new FireLocalDb("./data");

// Write documents
db.Put("users/alice", new Dictionary<string, object>
{
    ["name"] = "Alice",
    ["age"] = 30,
    ["created"] = FieldValue.ServerTimestamp()
});

// Read documents
var user = db.Get("users/alice");
Console.WriteLine(user["name"]); // "Alice"

// Delete documents
db.Delete("users/alice");

// Batch operations
var batch = db.Batch();
batch.Set("users/bob", new Dictionary<string, object> { ["name"] = "Bob" });
batch.Set("users/charlie", new Dictionary<string, object> { ["name"] = "Charlie" });
batch.Delete("users/old");
batch.Commit();

// Compaction
var stats = db.Compact();
Console.WriteLine($"Saved {stats.SizeReductionPercent:F1}% space");
```

## FieldValue Helpers

```csharp
using FireLocal;

// Server timestamp
db.Put("posts/1", new Dictionary<string, object>
{
    ["title"] = "Hello .NET!",
    ["created"] = FieldValue.ServerTimestamp()
});

// Increment counter
db.Put("stats/views", new Dictionary<string, object>
{
    ["count"] = FieldValue.Increment(1)
});

// Array operations
db.Put("users/alice", new Dictionary<string, object>
{
    ["tags"] = FieldValue.ArrayUnion("csharp", "dotnet")
});
```

## API Reference

### FireLocalDb

- `FireLocalDb(string path)` - Create database instance
- `void Put(string key, Dictionary<string, object> value)` - Write document
- `Dictionary<string, object>? Get(string key)` - Read document
- `void Delete(string key)` - Delete document
- `WriteBatch Batch()` - Create write batch
- `void CommitBatch(WriteBatch batch)` - Commit batch
- `CompactionStats Compact()` - Run compaction
- `void Flush()` - Flush memtable to SST
- `void Dispose()` - Close database

### WriteBatch

- `WriteBatch Set(string path, Dictionary<string, object> data)` - Add set operation
- `WriteBatch Update(string path, Dictionary<string, object> data)` - Add update operation
- `WriteBatch Delete(string path)` - Add delete operation
- `void Commit()` - Commit atomically

### FieldValue

- `ServerTimestamp()` - Current server time
- `Increment(long n)` - Increment numeric field
- `ArrayUnion(params object[] elements)` - Add unique elements
- `ArrayRemove(params object[] elements)` - Remove elements
- `Delete()` - Delete field

## Platform Support

- .NET 6.0+
- .NET Framework 4.7.2+
- .NET Standard 2.0+
- Xamarin.iOS / Xamarin.Android
- .NET MAUI
- Unity (with IL2CPP)

## MAUI Integration

```csharp
using FireLocal;
using Microsoft.Maui.Storage;

public class DataService
{
    private readonly FireLocalDb _db;

    public DataService()
    {
        var dbPath = Path.Combine(FileSystem.AppDataDirectory, "firelocal");
        _db = new FireLocalDb(dbPath);
    }

    public void SaveUser(string id, string name)
    {
        _db.Put($"users/{id}", new Dictionary<string, object>
        {
            ["name"] = name,
            ["updated"] = FieldValue.ServerTimestamp()
        });
    }
}
```

## Development

```bash
# Build
dotnet build

# Run tests
dotnet test

# Create NuGet package
dotnet pack
```

## License

MIT License
