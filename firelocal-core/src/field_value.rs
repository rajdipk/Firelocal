use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// FieldValue represents special Firestore field transformations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum FieldValue {
    /// Set field to server timestamp
    ServerTimestamp,

    /// Increment a numeric field by the given amount
    Increment(i64),

    /// Add elements to an array (union)
    ArrayUnion(Vec<Value>),

    /// Remove elements from an array
    ArrayRemove(Vec<Value>),

    /// Delete the field
    Delete,
}

impl FieldValue {
    /// Create a server timestamp field value
    pub fn server_timestamp() -> Self {
        FieldValue::ServerTimestamp
    }

    /// Create an increment field value
    pub fn increment(n: i64) -> Self {
        FieldValue::Increment(n)
    }

    /// Create an array union field value
    pub fn array_union(elements: Vec<Value>) -> Self {
        FieldValue::ArrayUnion(elements)
    }

    /// Create an array remove field value
    pub fn array_remove(elements: Vec<Value>) -> Self {
        FieldValue::ArrayRemove(elements)
    }

    /// Create a delete field value
    pub fn delete() -> Self {
        FieldValue::Delete
    }

    /// Apply this field value transformation to an existing value
    pub fn apply(&self, existing: Option<&Value>) -> Option<Value> {
        match self {
            FieldValue::ServerTimestamp => {
                // Get current timestamp in milliseconds
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64;
                Some(Value::Number(now.into()))
            }

            FieldValue::Increment(delta) => {
                let current = existing.and_then(|v| v.as_i64()).unwrap_or(0);
                Some(Value::Number((current + delta).into()))
            }

            FieldValue::ArrayUnion(elements) => {
                let mut result = if let Some(Value::Array(arr)) = existing {
                    arr.clone()
                } else {
                    Vec::new()
                };

                // Add elements that don't already exist
                for elem in elements {
                    if !result.contains(elem) {
                        result.push(elem.clone());
                    }
                }

                Some(Value::Array(result))
            }

            FieldValue::ArrayRemove(elements) => {
                let mut result = if let Some(Value::Array(arr)) = existing {
                    arr.clone()
                } else {
                    Vec::new()
                };

                // Remove matching elements
                result.retain(|v| !elements.contains(v));

                Some(Value::Array(result))
            }

            FieldValue::Delete => None,
        }
    }
}

/// Helper to detect and process FieldValue operations in a document
pub fn process_field_values(
    data: &mut serde_json::Map<String, Value>,
    existing_data: Option<&serde_json::Map<String, Value>>,
) {
    let mut updates = Vec::new();
    let mut deletions = Vec::new();

    for (key, value) in data.iter() {
        // Check if this is a FieldValue operation
        if let Ok(field_value) = serde_json::from_value::<FieldValue>(value.clone()) {
            let existing_value = existing_data.and_then(|d| d.get(key));

            if let Some(new_value) = field_value.apply(existing_value) {
                updates.push((key.clone(), new_value));
            } else {
                deletions.push(key.clone());
            }
        }
    }

    // Apply updates
    for (key, value) in updates {
        data.insert(key, value);
    }

    // Apply deletions
    for key in deletions {
        data.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_server_timestamp() {
        let fv = FieldValue::server_timestamp();
        let result = fv.apply(None);
        assert!(result.is_some());
        assert!(result.unwrap().is_number());
    }

    #[test]
    fn test_increment() {
        let fv = FieldValue::increment(5);

        // Increment from 10 to 15
        let existing = json!(10);
        let result = fv.apply(Some(&existing));
        assert_eq!(result, Some(json!(15)));

        // Increment from nothing to 5
        let result = fv.apply(None);
        assert_eq!(result, Some(json!(5)));
    }

    #[test]
    fn test_array_union() {
        let fv = FieldValue::array_union(vec![json!("a"), json!("b")]);

        // Union with existing array
        let existing = json!(["a", "c"]);
        let result = fv.apply(Some(&existing));
        assert_eq!(result, Some(json!(["a", "c", "b"])));

        // Union with no existing array
        let result = fv.apply(None);
        assert_eq!(result, Some(json!(["a", "b"])));
    }

    #[test]
    fn test_array_remove() {
        let fv = FieldValue::array_remove(vec![json!("b")]);

        let existing = json!(["a", "b", "c"]);
        let result = fv.apply(Some(&existing));
        assert_eq!(result, Some(json!(["a", "c"])));
    }

    #[test]
    fn test_delete() {
        let fv = FieldValue::delete();
        let existing = json!({"key": "value"});
        let result = fv.apply(Some(&existing));
        assert_eq!(result, None);
    }

    #[test]
    fn test_process_field_values() {
        let mut data = serde_json::Map::new();
        data.insert(
            "count".to_string(),
            json!({"type": "Increment", "value": 1}),
        );
        data.insert("name".to_string(), json!("Alice"));

        let mut existing = serde_json::Map::new();
        existing.insert("count".to_string(), json!(5));

        process_field_values(&mut data, Some(&existing));

        assert_eq!(data.get("count"), Some(&json!(6)));
        assert_eq!(data.get("name"), Some(&json!("Alice")));
    }
}
