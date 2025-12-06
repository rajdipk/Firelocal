import 'firelocal_base.dart';

enum _OperationType { set, update, delete }

class _BatchOperation {
  final _OperationType type;
  final String path;
  final Map<String, dynamic>? data;

  _BatchOperation.set(this.path, this.data) : type = _OperationType.set;
  _BatchOperation.update(this.path, this.data) : type = _OperationType.update;
  _BatchOperation.delete(this.path)
      : type = _OperationType.delete,
        data = null;
}

/// Atomic write batch
///
/// Example:
/// ```dart
/// final batch = db.batch();
/// batch.set('users/alice', {'name': 'Alice'});
/// batch.set('users/bob', {'name': 'Bob'});
/// batch.delete('users/charlie');
/// await db.commitBatch(batch);
/// ```
class WriteBatch {
  final FireLocal _db;
  final List<_BatchOperation> _operations = [];

  WriteBatch(this._db);

  /// Add a set operation to the batch
  WriteBatch set(String path, Map<String, dynamic> data) {
    _operations.add(_BatchOperation.set(path, data));
    return this;
  }

  /// Add an update operation to the batch
  WriteBatch update(String path, Map<String, dynamic> data) {
    _operations.add(_BatchOperation.update(path, data));
    return this;
  }

  /// Add a delete operation to the batch
  WriteBatch delete(String path) {
    _operations.add(_BatchOperation.delete(path));
    return this;
  }

  /// Commit the batch atomically
  Future<void> commit() async {
    await _db.commitBatch(this);
  }
}
