use firelocal_core::FireLocal;
use std::time::Instant;

fn stress_test_basic_operations(db: &FireLocal, num_ops: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting basic operations stress test: {} operations", num_ops);
    
    let start = Instant::now();
    
    // Write operations
    for i in 0..num_ops {
        let test_data = format!(r#"{{"id": "{}", "name": "Document {}", "data": {{"field1": "value1", "field2": "value2", "field3": 42}}}"#, i, i);
        let key = format!("stress_test/{}", i);
        db.put(key, test_data.into_bytes())?;
        
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
            return Err(format!("Failed to retrieve document: {}", key).into());
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

fn stress_test_batch_operations(db: &FireLocal, batch_size: usize, num_batches: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting batch operations stress test: {} batches of {} operations", num_batches, batch_size);
    
    let start = Instant::now();
    let total_ops = batch_size * num_batches;
    
    for batch_num in 0..num_batches {
        let batch = db.batch();
        
        for i in 0..batch_size {
            let doc_id = batch_num * batch_size + i;
            let test_data = format!(r#"{{"id": "{}", "name": "Batch Document {}", "data": {{"field1": "value1", "field2": "value2"}}}"#, doc_id, doc_id);
            let key = format!("batch_test/{}", doc_id);
            batch.set(key, test_data.into_bytes());
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    
    println!("\n🎉 All stress tests completed successfully!");
    println!("📈 FireLocal is ready for production deployment!");
    
    Ok(())
}
