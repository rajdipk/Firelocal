use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// FireLocal WASM bindings for browser use
#[wasm_bindgen]
pub struct FireLocal {
    path: String,
    // Note: Actual implementation would use IndexedDB or similar
}

#[wasm_bindgen]
impl FireLocal {
    #[wasm_bindgen(constructor)]
    pub fn new(path: String) -> Result<FireLocal, JsValue> {
        log(&format!("Creating FireLocal at: {}", path));
        
        Ok(FireLocal { path })
    }

    /// Write a document
    #[wasm_bindgen]
    pub async fn put(&mut self, key: String, value: JsValue) -> Result<(), JsValue> {
        let json_str = js_sys::JSON::stringify(&value)
            .map_err(|e| JsValue::from_str("Failed to stringify value"))?;
        
        let value_str = json_str.as_string()
            .ok_or_else(|| JsValue::from_str("Invalid JSON"))?;
        
        log(&format!("Put: {} = {}", key, value_str));
        
        // In production, this would:
        // 1. Store in IndexedDB
        // 2. Update in-memory cache
        // 3. Trigger sync if enabled
        
        Ok(())
    }

    /// Read a document
    #[wasm_bindgen]
    pub async fn get(&self, key: String) -> Result<JsValue, JsValue> {
        log(&format!("Get: {}", key));
        
        // In production, this would:
        // 1. Check in-memory cache
        // 2. Query IndexedDB
        // 3. Return parsed JSON
        
        Ok(JsValue::NULL)
    }

    /// Delete a document
    #[wasm_bindgen]
    pub async fn delete(&mut self, key: String) -> Result<(), JsValue> {
        log(&format!("Delete: {}", key));
        
        // In production, this would:
        // 1. Remove from IndexedDB
        // 2. Update cache
        // 3. Add tombstone
        
        Ok(())
    }

    /// Create a write batch
    #[wasm_bindgen]
    pub fn batch(&self) -> WriteBatch {
        WriteBatch::new()
    }

    /// Run compaction
    #[wasm_bindgen]
    pub async fn compact(&self) -> Result<CompactionStats, JsValue> {
        log("Running compaction");
        
        Ok(CompactionStats {
            files_before: 0,
            files_after: 0,
            tombstones_removed: 0,
        })
    }
}

/// Write batch for atomic operations
#[wasm_bindgen]
pub struct WriteBatch {
    operations: Vec<String>,
}

#[wasm_bindgen]
impl WriteBatch {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WriteBatch {
        WriteBatch {
            operations: Vec::new(),
        }
    }

    #[wasm_bindgen]
    pub fn set(&mut self, path: String, data: JsValue) -> Result<(), JsValue> {
        self.operations.push(format!("set:{}", path));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn delete(&mut self, path: String) {
        self.operations.push(format!("delete:{}", path));
    }

    #[wasm_bindgen]
    pub async fn commit(&self) -> Result<(), JsValue> {
        log(&format!("Committing {} operations", self.operations.len()));
        Ok(())
    }
}

/// Compaction statistics
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct CompactionStats {
    pub files_before: u32,
    pub files_after: u32,
    pub tombstones_removed: u32,
}

/// FieldValue helpers
#[wasm_bindgen]
pub fn server_timestamp() -> JsValue {
    let ts = js_sys::Date::now() as i64;
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("_firelocal_field_value"),
        &JsValue::from_str("serverTimestamp"),
    ).unwrap();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("value"),
        &JsValue::from_f64(ts as f64),
    ).unwrap();
    obj.into()
}

#[wasm_bindgen]
pub fn increment(n: i32) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("_firelocal_field_value"),
        &JsValue::from_str("increment"),
    ).unwrap();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("value"),
        &JsValue::from_f64(n as f64),
    ).unwrap();
    obj.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_firelocal_creation() {
        let db = FireLocal::new("./test".to_string()).unwrap();
        assert_eq!(db.path, "./test");
    }
}
