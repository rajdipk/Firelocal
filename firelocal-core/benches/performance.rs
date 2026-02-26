use criterion::{black_box, criterion_group, criterion_main, Criterion};
use firelocal_core::FireLocal;

/// Benchmark configuration
const TEST_DATA: &str = r#"{"name":"test","value":42}"#;

/// Benchmark basic operations
fn bench_basic_operations(c: &mut Criterion) {
    c.bench_function("put_operation", |b| {
        b.iter(|| {
            let test_dir = format!("bench_put_{}", uuid::Uuid::new_v4());
            let mut db = FireLocal::new(&test_dir).unwrap();

            let key = format!("users/user_{}", uuid::Uuid::new_v4());
            let data = TEST_DATA.as_bytes().to_vec();

            db.put(key, data).unwrap();

            // Cleanup
            let _ = std::fs::remove_dir_all(test_dir);
        });
    });

    c.bench_function("get_operation", |b| {
        b.iter(|| {
            let test_dir = format!("bench_get_{}", uuid::Uuid::new_v4());
            let mut db = FireLocal::new(&test_dir).unwrap();

            let key = format!("users/user_{}", uuid::Uuid::new_v4());
            let data = TEST_DATA.as_bytes().to_vec();
            db.put(key.clone(), data).unwrap();

            black_box(db.get(&key).unwrap());

            // Cleanup
            let _ = std::fs::remove_dir_all(test_dir);
        });
    });

    c.bench_function("delete_operation", |b| {
        b.iter(|| {
            let test_dir = format!("bench_delete_{}", uuid::Uuid::new_v4());
            let mut db = FireLocal::new(&test_dir).unwrap();

            let key = format!("users/user_{}", uuid::Uuid::new_v4());
            let data = TEST_DATA.as_bytes().to_vec();
            db.put(key.clone(), data).unwrap();

            db.delete(key).unwrap();

            // Cleanup
            let _ = std::fs::remove_dir_all(test_dir);
        });
    });
}

criterion_group!(benches, bench_basic_operations);
criterion_main!(benches);
