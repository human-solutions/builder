use anyhow::Result;
use serde_json::Value;

use crate::types::ValueWrapper;

pub struct TableEntry {
    pub key: String,
    pub value: ValueWrapper,
}

#[derive(Default)]
pub struct Tables(Vec<TableEntry>);

impl Tables {
    pub fn insert(&mut self, key: String, val: Value) {
        if let Some(entry) = self.0.iter_mut().find(|entry| entry.key == key) {
            if let ValueWrapper::Array(vec) = &mut entry.value {
                vec.push(val);
                return;
            }

            entry.value = ValueWrapper::Single(val);
            return;
        }

        self.0.push(TableEntry {
            key,
            value: ValueWrapper::Single(val),
        });
    }

    pub fn into_iter(self) -> std::vec::IntoIter<TableEntry> {
        self.0.into_iter()
    }

    pub fn insert_empty_vec(&mut self, key: String) -> Result<()> {
        if let Some(entry) = self.0.iter_mut().find(|entry| entry.key == key) {
            if let ValueWrapper::Array(_) = &entry.value {
                // NOTE: if the key already exists for an array, maybe insert an empty entry ?
                return Ok(());
            }

            anyhow::bail!(
                "Tried to insert an empty array, but key '{key}' already exists as a single value"
            );
        }

        self.0.push(TableEntry {
            key,
            value: ValueWrapper::Array(Vec::new()),
        });

        Ok(())
    }

    pub fn insert_array(&mut self, key: String, val: Value) -> Result<()> {
        if let Some(entry) = self.0.iter_mut().find(|entry| entry.key == key) {
            if let ValueWrapper::Array(vec) = &mut entry.value {
                vec.push(val);
                return Ok(());
            }

            anyhow::bail!(
                "Tried to push into array, but key '{key}' already exists as a single value"
            );
        }

        self.0.push(TableEntry {
            key,
            value: ValueWrapper::Array(vec![val]),
        });

        Ok(())
    }

    #[cfg(test)]
    pub fn string(&self) -> String {
        let mut s = self
            .0
            .iter()
            .fold(String::from("Table:["), |acc, entry| match &entry.value {
                ValueWrapper::Array(vec) => {
                    let vec_str = vec
                        .iter()
                        .map(|val| val.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");

                    format!("{}\n- {}: [{}]", acc, entry.key, vec_str)
                }
                ValueWrapper::Single(val) => {
                    format!("{}\n- {}: {}", acc, entry.key, val)
                }
            });

        s.push_str("\n]");

        s
    }
}

#[test]
fn table_insert_single() {
    let mut tables = Tables::default();

    tables.insert("key1".to_string(), Value::String("value".to_string()));
    tables.insert("key2".to_string(), Value::String("value2".to_string()));
    tables.insert("key1".to_string(), Value::String("value3".to_string()));

    assert_eq!(
        tables.string(),
        r#"Table:[
- key1: "value3"
- key2: "value2"
]"#,
        "Should be able to insert single values"
    );
}

#[test]
fn table_insert_array() {
    let mut tables = Tables::default();

    tables
        .insert_array("key1".to_string(), Value::String("value".to_string()))
        .unwrap();
    tables
        .insert_array("key2".to_string(), Value::String("value2".to_string()))
        .unwrap();
    tables
        .insert_array("key1".to_string(), Value::String("value3".to_string()))
        .unwrap();

    assert_eq!(
        tables.string(),
        r#"Table:[
- key1: ["value", "value3"]
- key2: ["value2"]
]"#,
        "Should be able to insert arrays"
    );
}

#[test]
fn table_insert_any() {
    let mut tables = Tables::default();

    tables
        .insert_array("key1".to_string(), Value::String("value".to_string()))
        .unwrap();
    tables.insert("key2".to_string(), Value::String("value2".to_string()));
    tables
        .insert_array("key1".to_string(), Value::String("value3".to_string()))
        .unwrap();

    assert_eq!(
        tables.string(),
        r#"Table:[
- key1: ["value", "value3"]
- key2: "value2"
]"#,
        "Should be able to insert single values and arrays"
    );
}

#[test]
fn table_insert_error() {
    let mut tables = Tables::default();

    tables.insert("key1".to_string(), Value::String("value".to_string()));

    let insert = tables.insert_array("key1".to_string(), Value::String("value2".to_string()));

    assert!(
        insert.is_err(),
        "Inserting an array in place of a single value should fail"
    );
}
