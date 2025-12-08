import 'dart:ffi';
import 'dart:convert';
import 'package:ffi/ffi.dart';
import 'firelocal_base.dart' as ffi;

class WriteBatch {
  final Pointer<Void> _dbHandle;
  Pointer<Void>? _handle;

  WriteBatch(this._dbHandle) {
    _handle = ffi.firelocalBatchNew(_dbHandle);
    if (_handle == nullptr) {
      throw Exception('Failed to create batch');
    }
  }

  WriteBatch set(String path, Map<String, dynamic> data) {
    final jsonStr = jsonEncode(data);
    final pathPtr = path.toNativeUtf8();
    final dataPtr = jsonStr.toNativeUtf8();

    try {
      final result = ffi.firelocalBatchSet(_handle!, pathPtr, dataPtr);
      if (result != 0) {
        throw Exception('Failed to add set operation: $path');
      }
    } finally {
      malloc.free(pathPtr);
      malloc.free(dataPtr);
    }
    return this;
  }

  WriteBatch update(String path, Map<String, dynamic> data) {
    final jsonStr = jsonEncode(data);
    final pathPtr = path.toNativeUtf8();
    final dataPtr = jsonStr.toNativeUtf8();

    try {
      final result = ffi.firelocalBatchUpdate(_handle!, pathPtr, dataPtr);
      if (result != 0) {
        throw Exception('Failed to add update operation: $path');
      }
    } finally {
      malloc.free(pathPtr);
      malloc.free(dataPtr);
    }
    return this;
  }

  WriteBatch delete(String path) {
    final pathPtr = path.toNativeUtf8();

    try {
      final result = ffi.firelocalBatchDelete(_handle!, pathPtr);
      if (result != 0) {
        throw Exception('Failed to add delete operation: $path');
      }
    } finally {
      malloc.free(pathPtr);
    }
    return this;
  }

  Future<void> commit() async {
    final result = ffi.firelocalBatchCommit(_dbHandle, _handle!);
    if (result != 0) {
      throw Exception('Failed to commit batch');
    }
  }

  void dispose() {
    if (_handle != null && _handle != nullptr) {
      ffi.firelocalBatchFree(_handle!);
      _handle = null;
    }
  }
}