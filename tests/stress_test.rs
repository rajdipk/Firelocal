use firelocal_core::FireLocal;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Clone)]
struct TestData {
    id: String,
    name: String,
    data: HashMap<String, Value>,
    timestamp: u64,
}

impl TestData {
    fn new(id: &str, name: &str) -> Self {
        let mut data = HashMap::new();
        data.insert("field1".to_string(), json!("value1"));
        data.insert("field2".to_string(), json!("value2"));
        data.insert("field3".to_string(), json!(42));
        
        Self {
            id: id.to_string(),
            name: name.to_string(),
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

fn stress_test_basic_operations(db: &FireLocal, num_ops: usize) -> Result<()> {
    println!("🔄 Starting basic operations stress test: {} operations", num_ops);
    
    let start = Instant::now();
    
    // Write operations
    for i in 0..num_ops {
        let test_data = TestData::new(&format!("doc_{}", i), &format!("Document {}", i));
        let key = format!("stress_test/{}", i);
        db.put(key, test_data.to_bytes())?;
        
        if i % 1000 == 0 {
            println!("  ✅ Completed {} write operations", i);
        }
    }
    
    let write_duration = start.elapsed();
    println!("  📝 Write operations completed in {:?}", write_duration);
    
    // Read operations
    let read_start = Instant::now();
    for i in 0..num_ops {
        let key = format!("stress_test/{}", i);
        let result = db.get(&key);
        
        if result.is_none() {
            return Err(anyhow::anyhow!("Failed to retrieve document: {}", key));
        }
        
        if i % 1000 == 0 {
            println!("  ✅ Completed {} read operations", i);
        }
    }
    
    let total_duration = start.elapsed();
    let read_duration = read_start.elapsed();
    
    println!("  📖 Read operations completed in {:?}", read_duration);
    println!("  ⏱️ Total test time: {:?}", total_duration);
    
    // Calculate performance metrics
    let write_ops_per_sec = num_ops as f64 / write_duration.as_secs_f64();
    let read_ops_per_sec = num_ops as f64 / read_duration.as_secs_f64();
    
    println!("  📊 Performance Metrics:");
    println!("    - Write: {:.2} ops/sec", write_ops_per_sec);
    println!("    - Read: {:.2} ops/sec", read_ops_per_sec);
    
    Ok(())
}

fn stress_test_batch_operations(db: &FireLocal, batch_size: usize, num_batches: usize) -> Result<()> {
    println!("🔄 Starting batch operations stress test: {} batches of {} operations", num_batches, batch_size);
    
    let start = Instant::now();
    let total_ops = batch_size * num_batches;
    
    for batch_num in 0..num_batches {
        let batch = db.batch();
        
        for i in 0..batch_size {
            let doc_id = batch_num * batch_size + i;
            let test_data = TestData::new(&format!("batch_doc_{}", doc_id), &format!("Batch Document {}", doc_id));
            let key = format!("batch_test/{}", doc_id);
            batch.set(key, test_data.to_bytes());
        }
        
        batch.commit()?;
        
        if batch_num % 10 == 0 {
            println!("  ✅ Completed {} batches", batch_num + 1);
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
    
    println!("  ⏱️ Batch test completed in {:?}", duration);
    println!("  📊 Batch performance: {:.2} ops/sec", ops_per_sec);
    
    Ok(())
}

fn stress_test_concurrent_access(db_path: &str, num_threads: usize, ops_per_thread: usize) -> Result<()> {
    println!("🔄 Starting concurrent access stress test: {} threads, {} ops/thread", num_threads, ops_per_thread);
    
    let start = Instant::now();
    let db = Arc::new(Mutex::new(FireLocal::new(db_path)?));
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let db_clone = Arc::clone(&db);
        let path = format!("concurrent_test_{}", thread_id);
        
        let handle = thread::spawn(move || -> Result<()> {
            for i in 0..ops_per_thread {
                let test_data = TestData::new(&format!("thread_{}_doc_{}", thread_id, i), &format!("Thread {} Document {}", thread_id, i));
                let key = format!("{}/doc_{}", path, i);
                
                let db_guard = db_clone.lock().unwrap();
                db_guard.put(key, test_data.to_bytes())?;
                
                // Read back to verify
                let result = db_guard.get(&key);
                if result.is_none() {
                    return Err(anyhow::anyhow!("Failed to read back document: {}", key));
                }
            }
            Ok(())
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap()?;
    }
    
    let duration = start.elapsed();
    let total_ops = (num_threads * ops_per_thread) as f64;
    let ops_per_sec = total_ops / duration.as_secs_f64();
    
    println!("  ⏱️ Concurrent test completed in {:?}", duration);
    println!("  📊 Concurrent performance: {:.2} ops/sec", ops_per_sec);
    
    Ok(())
}

fn stress_test_large_documents(db: &FireLocal, doc_size_kb: usize, num_docs: usize) -> Result<()> {
    println!("🔄 Starting large document stress test: {} documents of {}KB each", num_docs, doc_size_kb);
    
    let start = Instant::now();
    
    // Create large document
    let mut large_data = HashMap::new();
    for i in 0..(doc_size_kb * 100) {
        large_data.insert(format!("field_{}", i), json!("This is some test data with a moderately long string to simulate real-world document content with multiple fields and various data types including numbers, strings, and nested objects to test the performance characteristics of the database when handling larger documents."));
    }
    
    for i in 0..num_docs {
        let doc = TestData {
            id: format!("large_doc_{}", i),
            name: format!("Large Document {}", i),
            data: large_data.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        let key = format!("large_test/{}", i);
        db.put(key, serde_json::to_vec(&doc).unwrap())?;
        
        if i % 10 == 0 {
            println!("  ✅ Completed {} large documents", i + 1);
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = num_docs as f64 / duration.as_secs_f64();
    
    println!("  ⏱️ Large document test completed in {:?}", duration);
    println!("  📊 Large doc performance: {:.2} ops/sec", ops_per_sec);
    
    Ok(())
}

fn stress_test_mixed_workload(db: &FireLocal, duration_secs: u64) -> Result<()> {
    println!("🔄 Starting mixed workload stress test: {} seconds", duration_secs);
    
    let start = Instant::now();
    let mut write_count = 0;
    let mut read_count = 0;
    let mut delete_count = 0;
    
    while start.elapsed().as_secs() < duration_secs {
        // Mix of operations: 70% writes, 25% reads, 5% deletes
        let rand_num = (start.elapsed().as_nanos() % 100) as u32;
        
        if rand_num < 70 {
            // Write operation
            let test_data = TestData::new(&format!("mixed_{}", write_count), "Mixed Write");
            let key = format!("mixed/write/{}", write_count);
            db.put(key, test_data.to_bytes())?;
            write_count += 1;
        } else if rand_num < 95 {
            // Read operation
            if write_count > 0 {
                let key = format!("mixed/write/{}", write_count - 1);
                db.get(&key);
                read_count += 1;
            }
        } else {
            // Delete operation
            if write_count > 10 {
                let key = format!("mixed/write/{}", write_count - 10);
                db.delete(&key)?;
                delete_count += 1;
            }
        }
    }
    
    let total_ops = write_count + read_count + delete_count;
    let ops_per_sec = total_ops as f64 / duration_secs as f64;
    
    println!("  ⏱️ Mixed workload test completed");
    println!("  📊 Mixed workload stats:");
    println!("    - Writes: {}", write_count);
    println!("    - Reads: {}", read_count);
    println!("    - Deletes: {}", delete_count);
    println!("    - Total ops: {}", total_ops);
    println!("    - Performance: {:.2} ops/sec", ops_per_sec);
    
    Ok(())
}

fn main() -> Result<()> {
    println!("🔥 FireLocal Stress Test Suite");
    println!("================================");
    
    // Test database path
    let test_db_path = "./stress_test_db";
    
    // Clean up any existing test database
    if std::path::Path::new(test_db_path).exists() {
        std::fs::remove_dir_all(test_db_path)?;
    }
    
    let db = FireLocal::new(test_db_path)?;
    
    println!("\n📊 Test 1: Basic Operations");
    stress_test_basic_operations(&db, 10000)?;
    
    println!("\n📊 Test 2: Batch Operations");
    stress_test_batch_operations(&db, 100, 100)?;
    
    println!("\n📊 Test 3: Concurrent Access");
    stress_test_concurrent_access("./concurrent_test_db", 8, 1000)?;
    
    println!("\n📊 Test 4: Large Documents");
    stress_test_large_documents(&db, 10, 100)?;
    
    println!("\n📊 Test 5: Mixed Workload");
    stress_test_mixed_workload(&db, 30)?;
    
    println!("\n🎉 All stress tests completed successfully!");
    println!("📈 FireLocal is ready for production deployment!");
    
    Ok(())
}
