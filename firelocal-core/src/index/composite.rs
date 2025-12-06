use crate::model::Document;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Composite index for multi-field queries
pub struct CompositeIndex {
    fields: Vec<String>,
    entries: HashMap<Vec<Value>, HashSet<String>>,
}

impl CompositeIndex {
    pub fn new(fields: Vec<String>) -> Self {
        Self {
            fields,
            entries: HashMap::new(),
        }
    }

    /// Add a document to the composite index
    pub fn add(&mut self, doc: &Document) {
        let key: Vec<Value> = self
            .fields
            .iter()
            .map(|f| doc.fields.get(f).cloned().unwrap_or(Value::Null))
            .collect();

        self.entries
            .entry(key)
            .or_insert_with(HashSet::new)
            .insert(doc.path.clone());
    }

    /// Remove a document from the composite index
    pub fn remove(&mut self, doc: &Document) {
        let key: Vec<Value> = self
            .fields
            .iter()
            .map(|f| doc.fields.get(f).cloned().unwrap_or(Value::Null))
            .collect();

        if let Some(paths) = self.entries.get_mut(&key) {
            paths.remove(&doc.path);
            if paths.is_empty() {
                self.entries.remove(&key);
            }
        }
    }

    /// Query documents using the composite index
    pub fn query(&self, conditions: &[(String, Value)]) -> Vec<String> {
        // Build query key from conditions
        let mut query_key = Vec::new();
        for field in &self.fields {
            if let Some((_, value)) = conditions.iter().find(|(f, _)| f == field) {
                query_key.push(value.clone());
            } else {
                // Partial match not supported yet
                return Vec::new();
            }
        }

        // Exact match lookup
        self.entries
            .get(&query_key)
            .map(|paths| paths.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Range query support (limited implementation with HashMap)
    /// Note: This is less efficient than BTreeMap's range query
    /// For production use, consider using a sorted data structure or specialized index
    pub fn range_query(&self, start: &[Value], end: &[Value]) -> Vec<String> {
        let mut results = HashSet::new();

        // Since HashMap doesn't support range queries, we need to iterate all entries
        // and check if each key falls within the range
        for (key, paths) in &self.entries {
            if key.len() == start.len() && key.len() == end.len() {
                // Check if key is within range (lexicographically)
                // This is a simplified comparison - in production you'd want more robust comparison
                if is_in_range(key, start, end) {
                    results.extend(paths.iter().cloned());
                }
            }
        }

        results.into_iter().collect()
    }
}

/// Helper function to check if a key is within a range
/// This is a simplified implementation for demonstration
fn is_in_range(key: &[Value], start: &[Value], end: &[Value]) -> bool {
    if key.len() != start.len() || key.len() != end.len() {
        return false;
    }

    // Compare each element
    for i in 0..key.len() {
        if !value_gte(&key[i], &start[i]) || !value_lte(&key[i], &end[i]) {
            return false;
        }
    }

    true
}

/// Compare if a >= b for JSON values
fn value_gte(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(n1), Value::Number(n2)) => {
            let f1 = n1.as_f64().unwrap_or(0.0);
            let f2 = n2.as_f64().unwrap_or(0.0);
            f1 >= f2
        }
        (Value::String(s1), Value::String(s2)) => s1 >= s2,
        (Value::Bool(b1), Value::Bool(b2)) => b1 >= b2,
        _ => false,
    }
}

/// Compare if a <= b for JSON values
fn value_lte(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(n1), Value::Number(n2)) => {
            let f1 = n1.as_f64().unwrap_or(0.0);
            let f2 = n2.as_f64().unwrap_or(0.0);
            f1 <= f2
        }
        (Value::String(s1), Value::String(s2)) => s1 <= s2,
        (Value::Bool(b1), Value::Bool(b2)) => b1 <= b2,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_composite_index_add() {
        let mut index = CompositeIndex::new(vec!["age".to_string(), "city".to_string()]);

        let doc = Document {
            path: "users/alice".to_string(),
            fields: serde_json::from_value(json!({
                "age": 30,
                "city": "NYC"
            }))
            .unwrap(),
            version: 0,
        };

        index.add(&doc);

        let results = index.query(&[
            ("age".to_string(), json!(30)),
            ("city".to_string(), json!("NYC")),
        ]);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "users/alice");
    }

    #[test]
    fn test_composite_index_remove() {
        let mut index = CompositeIndex::new(vec!["age".to_string()]);

        let doc = Document {
            path: "users/alice".to_string(),
            fields: serde_json::from_value(json!({"age": 30})).unwrap(),
            version: 0,
        };

        index.add(&doc);
        index.remove(&doc);

        let results = index.query(&[("age".to_string(), json!(30))]);
        assert_eq!(results.len(), 0);
    }
}
