use std::collections::BTreeMap;

#[derive(Clone)]
pub enum Entry {
    Put(Vec<u8>),
    Delete,
}

pub struct Memtable {
    map: BTreeMap<String, Entry>,
    size_approx: usize,
}

impl Memtable {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            size_approx: 0,
        }
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) {
        self.size_approx += key.len() + value.len();
        self.map.insert(key, Entry::Put(value));
    }

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        match self.map.get(key) {
            Some(Entry::Put(val)) => Some(val),
            _ => None,
        }
    }

    pub fn delete(&mut self, key: String) {
        self.size_approx += key.len(); // Tombstone size approximation
        self.map.insert(key, Entry::Delete);
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Entry)> {
        self.map.iter()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
