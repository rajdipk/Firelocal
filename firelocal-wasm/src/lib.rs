use firelocal_core::store::io::MemoryStorage;
use firelocal_core::FireLocal as FireLocalCore;
use std::sync::Arc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

// We need to wrap the Core instance.
// Since FireLocalCore is not thread-safe by default (RefCell/etc internal?),
// but we defined it with Send+Sync components mainly (Arc<Mutex>).
// However, wasm_bindgen requires structs to be opaque pointers effectively.
// We can store it in a Mutex assuming single-threaded WASM context usually,
// but Rust checks bounds.

#[wasm_bindgen]
pub struct FireLocal {
    inner: Arc<Mutex<FireLocalCore<MemoryStorage>>>,
}

#[wasm_bindgen]
impl FireLocal {
    #[wasm_bindgen(constructor)]
    pub fn new(path: String) -> Result<FireLocal, JsValue> {
        log(&format!(
            "🔥 FireLocal WASM (Core Powered) initialized at: {}",
            path
        ));

        // Initialize Core with MemoryStorage
        // In the future, we can back MemoryStorage with IndexedDB by loading/saving snapshots.
        let storage = MemoryStorage::new();

        let db = FireLocalCore::new_with_storage(path, storage)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(FireLocal {
            inner: Arc::new(Mutex::new(db)),
        })
    }

    /// Write a document
    #[wasm_bindgen]
    pub fn put(&self, key: String, value: JsValue) -> Result<(), JsValue> {
        let json_str = js_sys::JSON::stringify(&value)
            .map_err(|_| JsValue::from_str("Failed to stringify value"))?;

        let value_str = json_str
            .as_string()
            .ok_or_else(|| JsValue::from_str("Invalid JSON"))?;

        let mut db = self.inner.lock().unwrap();
        db.put(key.clone(), value_str.into_bytes())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // log(&format!("✅ Put: {}", key));
        Ok(())
    }

    /// Read a document
    #[wasm_bindgen]
    pub fn get(&self, key: String) -> Result<JsValue, JsValue> {
        let db = self.inner.lock().unwrap();

        if let Some(bytes) = db.get(&key) {
            let s = std::str::from_utf8(&bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
            let parsed =
                js_sys::JSON::parse(s).map_err(|_| JsValue::from_str("Failed to parse JSON"))?;
            Ok(parsed)
        } else {
            Ok(JsValue::NULL)
        }
    }

    /// Delete a document
    #[wasm_bindgen]
    pub fn delete(&self, key: String) -> Result<(), JsValue> {
        let mut db = self.inner.lock().unwrap();
        db.delete(key)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Run compaction
    #[wasm_bindgen]
    pub async fn compact(&self) -> Result<JsValue, JsValue> {
        log("Running compaction (Stub)");
        // let db = self.inner.lock().unwrap();
        // let stats = db.compact().map_err(|e| JsValue::from_str(&e.to_string()))?;
        // For now just return empty object
        Ok(js_sys::Object::new().into())
    }
}
