use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::Output;

#[derive(TypedBuilder, Debug, Serialize, Deserialize)]
pub struct FontForgeCmd {
    /// Input sfd file path
    #[builder(setter(into))]
    pub font_file: Utf8PathBuf,

    pub output: Vec<Output>,
}
