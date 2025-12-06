pub mod basic_index;
pub mod composite;

use crate::model::Document;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Query operators for advanced filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryOperator {
    /// Equal to value
    Equal(Value),
    /// Value in array
    In(Vec<Value>),
    /// Value not in array
    NotIn(Vec<Value>),
    /// Array contains value
    ArrayContains(Value),
    /// Array contains any of values
    ArrayContainsAny(Vec<Value>),
    /// Less than
    LessThan(Value),
    /// Less than or equal
    LessThanOrEqual(Value),
    /// Greater than
    GreaterThan(Value),
    /// Greater than or equal
    GreaterThanOrEqual(Value),
}

impl QueryOperator {
    /// Check if a value matches this operator
    pub fn matches(&self, value: &Value) -> bool {
        match self {
            QueryOperator::Equal(v) => value == v,
            QueryOperator::In(values) => values.contains(value),
            QueryOperator::NotIn(values) => !values.contains(value),
            QueryOperator::ArrayContains(v) => {
                if let Value::Array(arr) = value {
                    arr.contains(v)
                } else {
                    false
                }
            }
            QueryOperator::ArrayContainsAny(values) => {
                if let Value::Array(arr) = value {
                    values.iter().any(|v| arr.contains(v))
                } else {
                    false
                }
            }
            QueryOperator::LessThan(v) => compare_values(value, v) < 0,
            QueryOperator::LessThanOrEqual(v) => compare_values(value, v) <= 0,
            QueryOperator::GreaterThan(v) => compare_values(value, v) > 0,
            QueryOperator::GreaterThanOrEqual(v) => compare_values(value, v) >= 0,
        }
    }
}

/// Compare two JSON values
fn compare_values(a: &Value, b: &Value) -> i32 {
    match (a, b) {
        (Value::Number(n1), Value::Number(n2)) => {
            let f1 = n1.as_f64().unwrap_or(0.0);
            let f2 = n2.as_f64().unwrap_or(0.0);
            if f1 < f2 {
                -1
            } else if f1 > f2 {
                1
            } else {
                0
            }
        }
        (Value::String(s1), Value::String(s2)) => {
            if s1 < s2 {
                -1
            } else if s1 > s2 {
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// Trait for index providers
pub trait IndexProvider: Send + Sync {
    /// Called when a document is put
    fn on_put(&self, doc_path: &str, doc: &Document) -> Result<()>;

    /// Called when a document is deleted
    fn on_delete(&self, doc_path: &str) -> Result<()>;

    /// Query documents using the index
    fn query(&self, query: &QueryAst) -> Result<Vec<String>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAst {
    pub field: String,
    pub operator: QueryOperator,
}
