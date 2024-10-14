use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[builder(field_defaults(default))]
pub struct UniffiCmd {
    #[builder(setter(into))]
    pub udl_file: Utf8PathBuf,

    /// Where to generate the bindings
    #[builder(setter(into))]
    pub out_dir: Utf8PathBuf,

    /// the .dylib or .so file to generate bindings for
    /// normally in target/debug or target/release
    #[builder(setter(into))]
    pub built_lib_file: Utf8PathBuf,

    #[builder(setter(into))]
    pub library_name: String,

    pub swift: bool,

    pub kotlin: bool,
}
