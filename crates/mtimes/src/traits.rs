use camino_fs::Utf8PathBuf;

/// Trait for commands to declare their input file dependencies.
///
/// Implementations should return ALL files the command depends on.
/// Change detection compares mtimes of these files against previous build.
pub trait InputFiles {
    /// Returns absolute paths to all input files this command depends on.
    ///
    /// # Returns
    /// * Non-empty Vec - Input files to track
    /// * Empty Vec - No trackable inputs (command always executes)
    ///
    /// # Contract
    /// * Paths MUST be absolute
    /// * Paths SHOULD exist (non-existent triggers rebuild)
    /// * Result SHOULD be deterministic for same command state
    fn input_files(&self) -> Vec<Utf8PathBuf>;
}

/// Trait for commands to declare their expected output files.
///
/// Implementations should return ALL files the command produces.
/// Change detection checks existence of these files - missing files trigger rebuild.
pub trait OutputFiles {
    /// Returns absolute paths to all expected output files.
    ///
    /// # Returns
    /// * Non-empty Vec - Output files to verify existence
    /// * Empty Vec - Outputs are self-discovering (command always executes)
    ///
    /// # Contract
    /// * Paths MUST be absolute
    /// * Paths may not exist yet (first build)
    /// * Result SHOULD match what command actually produces
    fn output_files(&self) -> Vec<Utf8PathBuf>;
}
