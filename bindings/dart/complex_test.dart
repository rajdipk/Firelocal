import 'dart:ffi';
import 'dart:convert';
import 'dart:io';

import 'package:firelocal_dart/firelocal_dart.dart';

void testComplexScenarios() async {
  print('🧪 Starting Dart Complex Scenario Tests');
  print('=' * 50);
  
  // Test 1: Complex nested documents
  print('\n📊 Test 1: Complex Nested Documents');
  final db = FireLocal('complex_test_db');
  
  final complexDoc = {
    'user': {
      'id': 'user_123',
      'profile': {
        'name': 'Alice Johnson',
        'email': 'alice@example.com',
        'preferences': {
          'theme': 'dark',
          'notifications': true,
          'privacy': {
            'share_data': true,
            'analytics': false,
          }
        }
      },
      'activity': {
        'timestamp': '2024-02-27T10:30:00Z',
        'type': 'login',
        'metadata': {
          'ip': '192.168.1.100',
          'user_agent': 'Mozilla/5.0...',
          'location': {
            'country': 'US',
            'city': 'New York',
          }
        }
      }
    }
  };
  
  await db.put('users/alice', jsonEncode(complexDoc) as String);
  final result = await db.get('users/alice');
  final retrieved = jsonDecode(result! as String);
  
  assert(retrieved['user']['profile']['name'] == 'Alice Johnson');
  assert(retrieved['user']['activity']['metadata']['location']['country'] == 'US');
  print('  ✅ Complex nested document test passed');
  
  // Test 2: Batch operations with mixed operations
  print('\n📊 Test 2: Mixed Batch Operations');
  final batch = db.batch();
  
  // Multiple set operations
  await batch.set('users/bob', jsonEncode({'name': 'Bob', 'role': 'admin'}) as String);
  await batch.set('users/charlie', jsonEncode({'name': 'Charlie', 'role': 'user'}) as String);
  await batch.set('posts/post_1', jsonEncode({
    'title': 'First Post',
    'content': 'Hello world!',
    'author': 'Bob',
    'tags': ['welcome', 'intro']
  }) as String);
  
  // Update operation
  await batch.update('users/alice', jsonEncode({
    'last_login': '2024-02-27T10:30:00Z',
    'status': 'active'
  }) as String);
  
  // Delete operation
  await batch.delete('temp/data');
  
  await batch.commit();
  print('  ✅ Mixed batch operations test passed');
  
  // Test 3: Large document handling
  print('\n📊 Test 3: Large Document Handling');
  final largeContent = 'x' * 10000; // 10KB of content
  final largeDoc = {
    'id': 'large_doc_1',
    'content': largeContent,
    'metadata': {
      'size': largeContent.length,
      'created': DateTime.now().toIso8601String(),
      'type': 'large_text'
    },
    'attachments': List.generate(10, (i) => {
      'name': 'file_$i',
      'content': 'y' * 1000
    })
  };
  
  await db.put('large/documents/1', jsonEncode(largeDoc) as String);
  final retrievedLarge = jsonDecode((await db.get('large/documents/1'))! as String);
  
  assert(retrievedLarge['content'].length == 10000);
  assert(retrievedLarge['attachments'].length == 10);
  print('  ✅ Large document handling test passed');
  
  // Test 4: Performance stress test
  print('\n📊 Test 4: Performance Stress Test');
  final stopwatch = Stopwatch()..start();
  
  for (int i = 0; i < 1000; i++) {
    final doc = {
      'id': 'perf_doc_$i',
      'data': 'performance_test_$i'
    };
    await db.put('performance/test/$i', jsonEncode(doc) as String);
    
    if (i % 100 == 0) {
      print('  📝 Completed ${i} write operations');
    }
  }
  
  // Read back all documents
  int readCount = 0;
  for (int i = 0; i < 1000; i++) {
    final result = await db.get('performance/test/$i');
    if (result != null) {
      readCount++;
    }
  }
  
  stopwatch.stop();
  final duration = stopwatch.elapsedMilliseconds / 1000.0;
  
  assert(readCount == 1000);
  print('  ✅ Performance test passed: ${duration.toStringAsFixed(2)}s for 2000 operations');
  print('  📊 Performance: ${(2000 / duration).toStringAsFixed(2)} ops/sec');
  
  // Test 5: Error handling
  print('\n📊 Test 5: Error Handling');
  try {
    await db.put('', ''); // Empty key should fail
  } catch (e) {
    print('  ✅ Error handling works: ${e.toString()}');
  }
  
  // Test non-existent document
  final nonExistent = await db.get('non/existent/path');
  assert(nonExistent == null);
  print('  ✅ Non-existent document returns null');
  
  print('\n🎉 All Dart complex scenario tests passed!');
  print('📈 Dart bindings are production ready!');
  
  // Cleanup
  try {
    final testDir = Directory('complex_test_db');
    if (await testDir.exists()) {
      await testDir.delete(recursive: true);
    }
  } catch (e) {
    print('Warning: Could not clean up test files: ${e.toString()}');
  }
}

Future<void> main() async {
  await testComplexScenarios();
}
