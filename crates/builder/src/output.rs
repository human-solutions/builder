use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Output {
    artifacts: Vec<String>,
}
