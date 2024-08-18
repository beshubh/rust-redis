use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct Data {
    pub value: String,
    pub exp: Option<Instant>,
}

#[derive(Clone)]
pub struct Store {
    data: Arc<Mutex<HashMap<String, Data>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let data = self.data.lock().unwrap();
        data.get(key).and_then(|Data { value, exp }| match exp {
            Some(exp) if exp.clone() <= Instant::now() => None,
            _ => Some(value.clone()),
        })
    }

    pub fn set(&self, key: String, value: String, px: Option<u64>) {
        let mut data = self.data.lock().unwrap();
        let value = Data {
            value,
            exp: px.map(|px| Instant::now() + std::time::Duration::from_millis(px)),
        };
        data.insert(key, value);
    }
}
