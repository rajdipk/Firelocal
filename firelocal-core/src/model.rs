use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub path: String,
    pub fields: Map<String, Value>,
    #[serde(default)]
    pub version: u64,
    // TODO: Add create_time, update_time for M3/M4
}

impl Document {
    pub fn from_json(json_str: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json_str)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}
