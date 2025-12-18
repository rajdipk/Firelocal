use crate::index::{IndexProvider, QueryAst, QueryOperator};
use crate::model::Document;
use anyhow::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

// Map: CollectionGroup -> FieldName -> Value -> Set<DocPath>
// Very naive storage for M2
type InvertedIndex = HashMap<String, HashMap<String, HashMap<String, HashSet<String>>>>;

pub struct BasicIndexProvider {
    index: Arc<RwLock<InvertedIndex>>,
}

impl BasicIndexProvider {
    pub fn new() -> Self {
        Self {
            index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn value_to_key(v: &Value) -> String {
        match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => v.to_string(), // Arrays/Objects serialized not ideal but works for unique key
        }
    }

    pub fn query_equal(&self, collection: &str, field: &str, value: &Value) -> Result<Vec<String>> {
        let index = self.index.read().unwrap();

        let empty_map = HashMap::new();
        let col_index = index.get(collection).unwrap_or(&empty_map);

        let val_key = Self::value_to_key(value);
        let field_index = col_index.get(field);

        let docs = match field_index {
            Some(f_map) => f_map.get(&val_key).cloned().unwrap_or_default(),
            None => HashSet::new(),
        };

        let result: Vec<String> = docs.into_iter().collect();
        Ok(result)
    }
}

impl IndexProvider for BasicIndexProvider {
    fn on_put(&self, doc_path: &str, doc: &Document) -> Result<()> {
        let mut index = self.index.write().unwrap();

        // Extract collection group from path
        // path: collection/doc/subcol/doc...
        // For M2 simplification, assume root collection is group
        let parts: Vec<&str> = doc_path.split('/').collect();
        let collection = if !parts.is_empty() {
            parts[0]
        } else {
            "default"
        };

        // For each field in doc, update index
        // Naive: doesn't handle nested fields well nicely without flattening
        for (field, value) in &doc.fields {
            let val_key = Self::value_to_key(value);

            index
                .entry(collection.to_string())
                .or_default()
                .entry(field.clone())
                .or_default()
                .entry(val_key)
                .or_default()
                .insert(doc_path.to_string());
        }
        Ok(())
    }

    fn on_delete(&self, doc_path: &str) -> Result<()> {
        // Naive delete: we'd need the old doc to know which entries to remove.
        // Or we scan everything.
        // Real system: Read old doc from store first OR store reverse index (doc -> keys).
        // For M2 MVP: we might just leave stale entries or scan?
        // TDD says "updated on document put/delete".
        // Let's brute force remove for now (SLOW but correct-ish).
        let mut index = self.index.write().unwrap();

        for col_map in index.values_mut() {
            for field_map in col_map.values_mut() {
                for docs in field_map.values_mut() {
                    docs.remove(doc_path);
                }
            }
        }
        Ok(())
    }

    fn query(&self, query_ast: &QueryAst) -> Result<Vec<String>> {
        // For now, only support Equal operator
        match &query_ast.operator {
            QueryOperator::Equal(value) => {
                // Extract collection from query AST
                let collection = query_ast.collection.as_deref().unwrap_or("default");
                self.query_equal(collection, &query_ast.field, value)
            }
            _ => {
                // Other operators not yet implemented
                Ok(Vec::new())
            }
        }
    }
}
