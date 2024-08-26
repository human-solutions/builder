use serde_json::Value;

pub trait IntoVecString {
    fn into_vec_string(&self, key: &str) -> Vec<String>;
}

impl IntoVecString for &Value {
    fn into_vec_string(&self, key: &str) -> Vec<String> {
        self.get(key)
            .and_then(Value::as_array)
            .map(|t| {
                t.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
}
