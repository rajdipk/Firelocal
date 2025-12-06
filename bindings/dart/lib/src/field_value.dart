/// FieldValue helpers for special operations
library;

/// Get current server timestamp
Map<String, dynamic> serverTimestamp() {
  return {
    '_firelocal_field_value': 'serverTimestamp',
    'value': DateTime.now().millisecondsSinceEpoch,
  };
}

/// Increment a numeric field
///
/// Args:
///   n: Amount to increment by
Map<String, dynamic> increment(int n) {
  return {
    '_firelocal_field_value': 'increment',
    'value': n,
  };
}

/// Add elements to an array (unique)
///
/// Args:
///   elements: Elements to add
Map<String, dynamic> arrayUnion(List<dynamic> elements) {
  return {
    '_firelocal_field_value': 'arrayUnion',
    'value': elements,
  };
}

/// Remove elements from an array
///
/// Args:
///   elements: Elements to remove
Map<String, dynamic> arrayRemove(List<dynamic> elements) {
  return {
    '_firelocal_field_value': 'arrayRemove',
    'value': elements,
  };
}

/// Delete a field from a document
Map<String, dynamic> deleteField() {
  return {
    '_firelocal_field_value': 'delete',
  };
}
