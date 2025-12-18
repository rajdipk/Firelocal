import 'dart:ffi';
import 'dart:convert';
import 'dart:io';
import 'package:ffi/ffi.dart';
import 'write_batch.dart';
import 'compaction_stats.dart';

// Type definitions for FFI
typedef FireLocalOpenNative = Pointer<Void> Function(Pointer<Utf8>);
typedef FireLocalOpen = Pointer<Void> Function(Pointer<Utf8>);

typedef FireLocalDestroyNative = Void Function(Pointer<Void>);
typedef FireLocalDestroy = void Function(Pointer<Void>);

typedef FireLocalLoadRulesNative = Int32 Function(Pointer<Void>, Pointer<Utf8>);
typedef FireLocalLoadRules = int Function(Pointer<Void>, Pointer<Utf8>);

typedef FireLocalPutNative = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>);
typedef FireLocalPut = int Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>);

typedef FireLocalGetNative = Pointer<Utf8> Function(
    Pointer<Void>, Pointer<Utf8>);
typedef FireLocalGet = Pointer<Utf8> Function(Pointer<Void>, Pointer<Utf8>);

typedef FireLocalDeleteNative = Int32 Function(Pointer<Void>, Pointer<Utf8>);
typedef FireLocalDelete = int Function(Pointer<Void>, Pointer<Utf8>);

typedef FireLocalFreeStringNative = Void Function(Pointer<Utf8>);
typedef FireLocalFreeString = void Function(Pointer<Utf8>);

typedef FireLocalBatchNewNative = Pointer<Void> Function(Pointer<Void>);
typedef FireLocalBatchNew = Pointer<Void> Function(Pointer<Void>);

typedef FireLocalBatchSetNative = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>);
typedef FireLocalBatchSet = int Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>);

typedef FireLocalBatchUpdateNative = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>);
typedef FireLocalBatchUpdate = int Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>);

typedef FireLocalBatchDeleteNative = Int32 Function(
    Pointer<Void>, Pointer<Utf8>);
typedef FireLocalBatchDelete = int Function(Pointer<Void>, Pointer<Utf8>);

typedef FireLocalBatchCommitNative = Int32 Function(
    Pointer<Void>, Pointer<Void>);
typedef FireLocalBatchCommit = int Function(Pointer<Void>, Pointer<Void>);

typedef FireLocalBatchFreeNative = Void Function(Pointer<Void>);
typedef FireLocalBatchFree = void Function(Pointer<Void>);

typedef FireLocalCompactNative = Pointer<Utf8> Function(Pointer<Void>);
typedef FireLocalCompact = Pointer<Utf8> Function(Pointer<Void>);

typedef FireLocalFlushNative = Int32 Function(Pointer<Void>);
typedef FireLocalFlush = int Function(Pointer<Void>);

// Load native library
DynamicLibrary _loadLibrary() {
  if (Platform.isWindows) {
    // Try to load from multiple locations
    try {
      // First try relative path from Flutter app
      return DynamicLibrary.open('firelocal_core.dll');
    } catch (e) {
      // Try absolute path from FireLocal project
      try {
        return DynamicLibrary.open(
            r'D:\projects\firelocal\target\release\firelocal_core.dll');
      } catch (e2) {
        // Try debug build
        return DynamicLibrary.open(
            r'D:\projects\firelocal\target\debug\firelocal_core.dll');
      }
    }
  } else if (Platform.isMacOS) {
    return DynamicLibrary.open('libfirelocal_core.dylib');
  } else {
    return DynamicLibrary.open('libfirelocal_core.so');
  }
}

final DynamicLibrary _lib = _loadLibrary();

// Bind FFI functions
final FireLocalOpen firelocalOpen = _lib
    .lookup<NativeFunction<FireLocalOpenNative>>('firelocal_open')
    .asFunction();

final FireLocalDestroy firelocalDestroy = _lib
    .lookup<NativeFunction<FireLocalDestroyNative>>('firelocal_destroy')
    .asFunction();

final FireLocalLoadRules firelocalLoadRules = _lib
    .lookup<NativeFunction<FireLocalLoadRulesNative>>('firelocal_load_rules')
    .asFunction();

final FireLocalPut firelocalPut = _lib
    .lookup<NativeFunction<FireLocalPutNative>>('firelocal_put_resource')
    .asFunction();

final FireLocalGet firelocalGet = _lib
    .lookup<NativeFunction<FireLocalGetNative>>('firelocal_get_resource')
    .asFunction();

final FireLocalDelete firelocalDelete = _lib
    .lookup<NativeFunction<FireLocalDeleteNative>>('firelocal_delete')
    .asFunction();

final FireLocalFreeString firelocalFreeString = _lib
    .lookup<NativeFunction<FireLocalFreeStringNative>>('firelocal_free_string')
    .asFunction();

final FireLocalBatchNew firelocalBatchNew = _lib
    .lookup<NativeFunction<FireLocalBatchNewNative>>('firelocal_batch_new')
    .asFunction();

final FireLocalBatchSet firelocalBatchSet = _lib
    .lookup<NativeFunction<FireLocalBatchSetNative>>('firelocal_batch_set')
    .asFunction();

final FireLocalBatchUpdate firelocalBatchUpdate = _lib
    .lookup<NativeFunction<FireLocalBatchUpdateNative>>(
        'firelocal_batch_update')
    .asFunction();

final FireLocalBatchDelete firelocalBatchDelete = _lib
    .lookup<NativeFunction<FireLocalBatchDeleteNative>>(
        'firelocal_batch_delete')
    .asFunction();

final FireLocalBatchCommit firelocalBatchCommit = _lib
    .lookup<NativeFunction<FireLocalBatchCommitNative>>(
        'firelocal_batch_commit')
    .asFunction();

final FireLocalBatchFree firelocalBatchFree = _lib
    .lookup<NativeFunction<FireLocalBatchFreeNative>>('firelocal_batch_free')
    .asFunction();

final FireLocalCompact firelocalCompact = _lib
    .lookup<NativeFunction<FireLocalCompactNative>>('firelocal_compact')
    .asFunction();

final FireLocalFlush firelocalFlush = _lib
    .lookup<NativeFunction<FireLocalFlushNative>>('firelocal_flush')
    .asFunction();

/// FireLocal database instance
///
/// Example:
/// ```dart
/// final db = FireLocal('./data');
/// await db.put('users/alice', {'name': 'Alice', 'age': 30});
/// final doc = await db.get('users/alice');
/// print(doc);
/// ```
class FireLocal {
  final String path;
  Pointer<Void>? _handle;

  FireLocal(this.path) {
    final pathPtr = path.toNativeUtf8();
    try {
      _handle = firelocalOpen(pathPtr);
      if (_handle == nullptr) {
        throw Exception('Failed to open database at $path');
      }
    } finally {
      malloc.free(pathPtr);
    }
  }

  /// Load security rules
  ///
  /// Args:
  ///   rules: Firestore security rules string
  void loadRules(String rules) {
    final rulesPtr = rules.toNativeUtf8();
    try {
      final result = firelocalLoadRules(_handle!, rulesPtr);
      if (result != 0) {
        throw Exception('Failed to load rules');
      }
    } finally {
      malloc.free(rulesPtr);
    }
  }

  /// Write a document
  ///
  /// Args:
  ///   key: Document path (e.g., "users/alice")
  ///   value: Document data as Map
  Future<void> put(String key, Map<String, dynamic> value) async {
    final jsonStr = jsonEncode(value);
    final keyPtr = key.toNativeUtf8();
    final dataPtr = jsonStr.toNativeUtf8();
    try {
      final result = firelocalPut(_handle!, keyPtr, dataPtr);
      if (result != 0) {
        throw Exception('Failed to put document: $key');
      }
    } finally {
      malloc.free(keyPtr);
      malloc.free(dataPtr);
    }
  }

  /// Read a document
  ///
  /// Args:
  ///   key: Document path
  ///
  /// Returns:
  ///   Document data or null if not found
  Future<Map<String, dynamic>?> get(String key) async {
    final keyPtr = key.toNativeUtf8();
    try {
      final result = firelocalGet(_handle!, keyPtr);
      if (result != nullptr) {
        final jsonStr = result.toDartString();
        firelocalFreeString(result);
        return jsonDecode(jsonStr) as Map<String, dynamic>;
      }
      return null; // Explicitly return null when no document is found
    } finally {
      malloc.free(keyPtr);
    }
  }

  /// Delete a document
  ///
  /// Args:
  ///   key: Document path
  Future<void> delete(String key) async {
    final keyPtr = key.toNativeUtf8();

    try {
      final result = firelocalDelete(_handle!, keyPtr);
      if (result != 0) {
        throw Exception('Failed to delete document: $key');
      }
    } finally {
      malloc.free(keyPtr);
    }
  }

  /// Create a new write batch
  ///
  /// Returns:
  ///   WriteBatch instance
  WriteBatch batch() {
    return WriteBatch(_handle!);
  }

  /// Run compaction to merge SST files and remove tombstones
  ///
  /// Returns:
  ///   CompactionStats with before/after metrics
  Future<CompactionStats> compact() async {
    final result = firelocalCompact(_handle!);
    if (result != nullptr) {
      final jsonStr = result.toDartString();
      firelocalFreeString(result);
      final data = jsonDecode(jsonStr) as Map<String, dynamic>;
      return CompactionStats(
        filesBefore: data['files_before'] as int,
        filesAfter: data['files_after'] as int,
        entriesBefore: data['entries_before'] as int,
        entriesAfter: data['entries_after'] as int,
        tombstonesRemoved: data['tombstones_removed'] as int,
        sizeBefore: data['size_before'] as int,
        sizeAfter: data['size_after'] as int,
      );
    } else {
      throw Exception('Compaction failed');
    }
  }

  /// Flush memtable to SST file
  Future<void> flush() async {
    final result = firelocalFlush(_handle!);
    if (result != 0) {
      throw Exception('Flush failed');
    }
  }

  /// Close the database (alias for dispose)
  void close() {
    dispose();
  }

  /// Close the database
  void dispose() {
    if (_handle != null && _handle != nullptr) {
      firelocalDestroy(_handle!);
      _handle = nullptr;
    }
  }
}
