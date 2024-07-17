use serde_json::Value;

pub enum ValueWrapper {
    Array(Vec<Value>),
    Single(Value),
}

impl Default for ValueWrapper {
    fn default() -> Self {
        Self::Single(Value::default())
    }
}
