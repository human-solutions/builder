use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::Output;

#[derive(TypedBuilder, Debug, Serialize, Deserialize)]
pub struct LocalizedCmd {
    #[builder(setter(into))]
    pub input_dir: Utf8PathBuf,

    /// File extensions that should be processed when searching for files in the input directory
    #[builder(setter(into))]
    pub file_extension: String,

    pub output: Vec<Output>,
}
