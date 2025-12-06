use crate::model::Document;
use crate::sync::RemoteStore;
use reqwest::blocking::Client;
use serde_json::Value;
use std::env;

pub struct FirebaseClient {
    client: Client,
    project_id: String,
    auth_token: Option<String>,
}

impl FirebaseClient {
    pub fn new() -> Self {
        dotenv::dotenv().ok();

        let project_id =
            env::var("FIREBASE_PROJECT_ID").unwrap_or_else(|_| "test-project".to_string());

        Self {
            client: Client::new(),
            project_id,
            auth_token: env::var("FIREBASE_AUTH_TOKEN").ok(),
        }
    }

    fn base_url(&self) -> String {
        format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents",
            self.project_id
        )
    }
}

impl RemoteStore for FirebaseClient {
    fn push(&self, doc: &Document) -> Result<(), String> {
        let url = format!("{}/{}", self.base_url(), doc.path);

        let firestore_fields = map_to_firestore_json(&doc.fields);
        let body = serde_json::json!({
            "fields": firestore_fields
        });

        let mut req = self.client.patch(&url).json(&body);
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let resp = req.send().map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("Remote error: {}", resp.status()))
        }
    }

    fn pull(&self, path: &str) -> Result<Option<Document>, String> {
        let url = format!("{}/{}", self.base_url(), path);
        let mut req = self.client.get(&url);
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let resp = req.send().map_err(|e| e.to_string())?;
        if resp.status() == 404 {
            return Ok(None);
        }
        if !resp.status().is_success() {
            return Err(format!("Remote error: {}", resp.status()));
        }

        let json: Value = resp.json().map_err(|e| e.to_string())?;

        if let Some(fields_map) = json.get("fields").and_then(|f| f.as_object()) {
            let simple_fields = map_from_firestore_json(fields_map);
            return Ok(Some(Document {
                path: path.to_string(),
                fields: simple_fields,
                version: 0,
            }));
        }

        Ok(None)
    }
}

fn map_to_firestore_json(
    fields: &serde_json::Map<String, Value>,
) -> serde_json::Map<String, Value> {
    let mut out = serde_json::Map::new();
    for (k, v) in fields {
        let typed_val = match v {
            Value::String(s) => serde_json::json!({ "stringValue": s }),
            Value::Number(n) => {
                if n.is_i64() {
                    serde_json::json!({ "integerValue": n.to_string() })
                } else {
                    serde_json::json!({ "doubleValue": n.as_f64() })
                }
            }
            Value::Bool(b) => serde_json::json!({ "booleanValue": b }),
            _ => serde_json::json!({ "stringValue": "unsupported" }),
        };
        out.insert(k.clone(), typed_val);
    }
    out
}

fn map_from_firestore_json(
    fields: &serde_json::Map<String, Value>,
) -> serde_json::Map<String, Value> {
    let mut out = serde_json::Map::new();
    for (k, v) in fields {
        if let Some(obj) = v.as_object() {
            if let Some(s) = obj.get("stringValue") {
                out.insert(k.clone(), s.clone());
            } else if let Some(n) = obj.get("integerValue") {
                let num = n.as_str().unwrap().parse::<i64>().unwrap_or(0);
                out.insert(k.clone(), serde_json::json!(num));
            } else if let Some(b) = obj.get("booleanValue") {
                out.insert(k.clone(), b.clone());
            }
        }
    }
    out
}
