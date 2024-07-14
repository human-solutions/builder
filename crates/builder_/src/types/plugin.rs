use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;

use super::phase::Phase;

pub struct Plugin {
    name: String,
    prebuild: Vec<Action>,
    postbuild: Vec<Action>,
}

impl From<(String, (Vec<Action>, Vec<Action>))> for Plugin {
    fn from(value: (String, (Vec<Action>, Vec<Action>))) -> Self {
        todo!()
    }
}

struct Action {
    name: String,
    specs: Vec<Spec>,
}

struct Spec {
    assembly: Option<String>,
    target: Option<String>,
    profile: Option<String>,
    output: Option<Value>,
}
