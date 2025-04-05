use std::collections::HashMap;

use liblog::{Extractor, Injector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transaction {
    pub cx: HashMap<String, String>,
}
impl Transaction {
    pub fn with_source<S: ToString>(mut self, source: S) -> Self {
        self.cx
            .insert("transaction.source".to_string(), source.to_string());
        self
    }

    pub fn with_intent<S: ToString>(mut self, intent: S) -> Self {
        self.cx
            .insert("transaction.intent".to_string(), intent.to_string());
        self
    }

    /// Pull tran properties _out_ of the context
    pub fn inject(&mut self) {
        liblog::inject(self);
    }

    /// Push tran properties _into_ the trace context
    pub fn extract(&mut self) -> &mut Self {
        let this = liblog::extract(self);

        this
    }
}
impl Injector for Transaction {
    fn set(&mut self, key: &str, value: String) {
        self.cx.insert(key.to_string(), value);
    }
}
impl Extractor for Transaction {
    fn get(&self, key: &str) -> Option<&str> {
        self.cx.get(key).map(|s| s.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.cx.keys().map(|s| s.as_str()).collect()
    }
}
