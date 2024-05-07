use core::hash::Hash;
use std::hash::Hasher;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Value(serde_json::Value);

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_value(&self.0, state);
    }
}

fn hash_value<H: Hasher>(value: &serde_json::Value, state: &mut H) {
    match value {
        serde_json::Value::Null => {
            "null".hash(state);
        }
        serde_json::Value::Bool(value) => {
            value.hash(state);
        }
        serde_json::Value::Number(value) => {
            value.hash(state);
        }
        serde_json::Value::String(value) => {
            value.hash(state);
        }
        serde_json::Value::Array(values) => {
            values.iter().for_each(|value| hash_value(value, state))
        }
        serde_json::Value::Object(values) => {
            values.iter().for_each(|(key, value)| {
                key.hash(state);
                hash_value(value, state);
            });
        }
    }
}
