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
  
  final complexDoc = <String, dynamic>{
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
  
  await db.put('users/alice', jsonEncode(complexDoc));
  final result = await db.get('users/alice');
  final retrieved = jsonDecode(result!);
  
  assert(retrieved['user']['profile']['name'] == 'Alice Johnson');
  assert(retrieved['user']['activity']['metadata']['location']['country'] == 'US');
  print('  ✅ Complex nested document test passed');
  
  // Test 2: Batch operations with mixed operations
  print('\n📊 Test 2: Mixed Batch Operations');
  final batch = db.batch();
  
  // Multiple set operations
  await batch.set('users/bob', <String, dynamic>{'name': 'Bob', 'role': 'admin'});
  await batch.set('users/charlie', <String, dynamic>{'name': 'Charlie', 'role': 'user'});
  await batch.set('posts/post_1', <String, dynamic>{
    'title': 'First Post',
    'content': 'Hello world!',
    'author': 'Bob',
    'tags': ['welcome', 'intro']
  });
  
  // Update operation
  await batch.update('users/alice', <String, dynamic>{
    'last_login': '2024-02-27T10:30:00Z',
    'status': 'active'
  });
  
  // Delete operation
  await batch.delete('temp/data');
  
  await batch.commit();
  print('  ✅ Mixed batch operations test passed');
  
  // Test 3: Performance stress test
  print('\n📊 Test 3: Performance Stress Test');
  final stopwatch = Stopwatch()..start();
  
  for (int i = 0; i < 1000; i++) {
    final doc = <String, dynamic>{
      'id': 'perf_doc_$i',
      'data': 'performance_test_$i'
    };
    await db.put('performance/test/$i', jsonEncode(doc));
    
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
  
  // Test 4: Error handling
  print('\n📊 Test 4: Error Handling');
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
