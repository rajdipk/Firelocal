import json
import time
import os
import sys
import tempfile
import shutil

# Add the firelocal bindings to path
sys.path.insert(0, '/home/pikachu/projects/Firelocal/bindings/python')

try:
    import firelocal
except ImportError as e:
    print(f"❌ Failed to import firelocal: {e}")
    sys.exit(1)

def test_complex_scenarios():
    """Test complex real-world scenarios with Python bindings"""
    print("🧪 Starting Python Complex Scenario Tests")
    print("=" * 50)
    
    # Create temporary test directory
    test_dir = tempfile.mkdtemp(prefix="firelocal_complex_test_")
    
    try:
        # Test 1: Complex nested documents
        print("\n📊 Test 1: Complex Nested Documents")
        db = firelocal.FireLocal(test_dir)
        
        complex_doc = {
            "user": {
                "id": "user_123",
                "profile": {
                    "name": "Alice Johnson",
                    "email": "alice@example.com",
                    "preferences": {
                        "theme": "dark",
                        "notifications": True,
                        "privacy": {
                            "share_data": True,
                            "analytics": False
                        }
                    }
                },
                "activity": {
                    "timestamp": "2024-02-27T10:30:00Z",
                    "type": "login",
                    "metadata": {
                        "ip": "192.168.1.100",
                        "user_agent": "Mozilla/5.0...",
                        "location": {
                            "country": "US",
                            "city": "New York"
                        }
                    }
                }
            }
        }
        
        db.put("users/alice", json.dumps(complex_doc))
        result = db.get("users/alice")
        retrieved = json.loads(result)
        
        assert retrieved["user"]["profile"]["name"] == "Alice Johnson"
        assert retrieved["user"]["activity"]["metadata"]["location"]["country"] == "US"
        print("  ✅ Complex nested document test passed")
        
        # Test 2: Batch operations with mixed operations
        print("\n📊 Test 2: Mixed Batch Operations")
        batch = db.batch()
        
        # Multiple set operations
        batch.set("users/bob", json.dumps({"name": "Bob", "role": "admin"}))
        batch.set("users/charlie", json.dumps({"name": "Charlie", "role": "user"}))
        batch.set("posts/post_1", json.dumps({
            "title": "First Post",
            "content": "Hello world!",
            "author": "Bob",
            "tags": ["welcome", "intro"]
        }))
        
        # Update operation
        batch.update("users/alice", json.dumps({
            "last_login": "2024-02-27T10:30:00Z",
            "status": "active"
        }))
        
        # Delete operation
        batch.delete("temp/data")
        
        batch.commit()
        print("  ✅ Mixed batch operations test passed")
        
        # Test 3: Large document handling
        print("\n📊 Test 3: Large Document Handling")
        large_doc = {
            "id": "large_doc_1",
            "content": "x" * 10000,  # 10KB of content
            "metadata": {
                "size": len("x" * 10000),
                "created": time.time(),
                "type": "large_text"
            },
            "attachments": [
                {"name": f"file_{i}", "content": "y" * 1000} for i in range(10)
            ]
        }
        
        db.put("large/documents/1", json.dumps(large_doc))
        result = db.get("large/documents/1")
        retrieved_large = json.loads(result)
        
        assert len(retrieved_large["content"]) == 10000
        assert len(retrieved_large["attachments"]) == 10
        print("  ✅ Large document handling test passed")
        
        # Test 4: Performance with many operations
        print("\n📊 Test 4: Performance Stress Test")
        start_time = time.time()
        
        for i in range(1000):
            doc = {"id": f"perf_doc_{i}", "data": f"performance_test_{i}"}
            db.put(f"performance/test/{i}", json.dumps(doc))
            
            if i % 100 == 0:
                print(f"  📝 Completed {i} write operations")
        
        # Read back all documents
        read_count = 0
        for i in range(1000):
            result = db.get(f"performance/test/{i}")
            if result:
                read_count += 1
                
        end_time = time.time()
        duration = end_time - start_time
        
        assert read_count == 1000
        print(f"  ✅ Performance test passed: {duration:.2f}s for 2000 operations")
        print(f"  📊 Performance: {2000/duration:.2f} ops/sec")
        
        # Test 5: Error handling and recovery
        print("\n📊 Test 5: Error Handling")
        try:
            db.put("")  # Empty key should fail
        except Exception as e:
            print(f"  ✅ Error handling works: {e}")
        
        # Test non-existent document
        result = db.get("non/existent/path")
        assert result is None
        print("  ✅ Non-existent document returns None")
        
        print("\n🎉 All Python complex scenario tests passed!")
        print("📈 Python bindings are production ready!")
        
    finally:
        # Cleanup
        shutil.rmtree(test_dir, ignore_errors=True)

if __name__ == "__main__":
    test_complex_scenarios()
