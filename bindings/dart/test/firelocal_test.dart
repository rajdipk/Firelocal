import 'package:firelocal_dart/firelocal_dart.dart';
import 'package:test/test.dart';
import 'dart:io';

void main() {
  final dbPath = "tmp_dart_test_db";

  setUp(() {
    if (Directory(dbPath).existsSync()) {
      Directory(dbPath).deleteSync(recursive: true);
    }
  });

  tearDown(() {
    if (Directory(dbPath).existsSync()) {
      Directory(dbPath).deleteSync(recursive: true);
    }
  });

  test('FireLocal Lifecycle', () async {
    final db = FireLocal(dbPath);

    // Load rules
    db.loadRules(
        "service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }");

    // Put
    final key = "doc1";
    final val = {"foo": "bar", "num": 123};
    await db.put(key, val);

    // Get
    final res = await db.get(key);
    expect(res, isNotNull);
    expect(res!['foo'], equals("bar"));
    expect(res['num'], equals(123));

    // Get missing
    final missing = await db.get("missing");
    expect(missing, isNull);

    db.close();
  });
}
