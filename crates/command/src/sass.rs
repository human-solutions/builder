use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::Output;

#[derive(TypedBuilder, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[builder(field_defaults(default))]
pub struct SassCmd {
    #[builder(setter(into))]
    pub in_scss: Utf8PathBuf,

    pub optimize: bool,

    pub output: Vec<Output>,
}
