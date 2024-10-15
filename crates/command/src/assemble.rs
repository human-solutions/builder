use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssembleCmd {
    /// Input sfd file path
    pub localized: Vec<Utf8PathBuf>,

    /// Path to one of the asset files. If there are other
    /// versions of it (e.g. compressed), then they'll be
    /// automatically detected.
    pub files: Vec<Utf8PathBuf>,

    /// Where to write the generated code. If not provided, it will be printed to stdout.
    pub out_file: Option<Utf8PathBuf>,

    pub no_brotli: bool,

    pub no_gzip: bool,

    pub no_uncompressed: bool,
}
