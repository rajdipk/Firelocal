use crate::index::{QueryAst, QueryOperator};
use crate::model::Document;
use crate::FireLocal;
use serde_json::Value;
use std::sync::Arc;

pub struct CollectionReference {
    db: Arc<std::sync::Mutex<FireLocal>>,
    path: String,
}

impl CollectionReference {
    pub fn new(db: Arc<std::sync::Mutex<FireLocal>>, path: String) -> Self {
        Self { db, path }
    }

    pub fn doc(&self, id: &str) -> DocumentReference {
        DocumentReference {
            db: self.db.clone(),
            path: format!("{}/{}", self.path, id),
        }
    }

    // Simple Query builder
    pub fn where_eq(&self, field: &str, value: Value) -> Query {
        let q = QueryAst {
            collection: Some(self.path.clone()),
            field: field.to_string(),
            operator: QueryOperator::Equal(value),
        };
        Query {
            db: self.db.clone(),
            ast: q,
        }
    }
}

pub struct DocumentReference {
    db: Arc<std::sync::Mutex<FireLocal>>,
    path: String,
}

impl DocumentReference {
    pub fn set(&self, data: Value) -> anyhow::Result<()> {
        let mut db = self
            .db
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        // Construct full document with path
        let doc = Document {
            path: self.path.clone(),
            fields: data
                .as_object()
                .ok_or(anyhow::anyhow!("Data must be an object"))?
                .clone(),
            version: 0,
        };

        let bytes = doc.to_json()?.into_bytes();
        db.put(self.path.clone(), bytes)?;
        Ok(())
    }

    pub fn get(&self) -> Option<Document> {
        let db = match self.db.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Database lock poisoned, attempting recovery");
                poisoned.into_inner()
            }
        };
        if let Some(bytes) = db.get(&self.path) {
            if let Ok(s) = std::str::from_utf8(&bytes) {
                return Document::from_json(s).ok();
            }
        }
        None
    }

    pub fn delete(&self) -> anyhow::Result<()> {
        let mut db = self
            .db
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        db.delete(self.path.clone())?;
        Ok(())
    }
}

pub struct Query {
    db: Arc<std::sync::Mutex<FireLocal>>,
    ast: QueryAst,
}

impl Query {
    pub fn get(&self) -> anyhow::Result<Vec<Document>> {
        let db = self
            .db
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        Ok(db.query(&self.ast)?)
    }

    pub fn on_snapshot<F>(&self, callback: F) -> u64
    where
        F: Fn(Vec<Document>) + Send + Sync + 'static,
    {
        let mut db = match self.db.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Database lock poisoned, attempting recovery");
                poisoned.into_inner()
            }
        };
        db.listen(self.ast.clone(), Box::new(callback))
    }
}
