// Mtime-based change detection for builder commands
// Public API for skip detection and success recording

use anyhow::{Context, Result};
use camino_fs::{Utf8Path, Utf8PathBuf};
use std::collections::HashMap;
use std::time::UNIX_EPOCH;

pub use lock::FileLock;
pub use traits::{InputFiles, OutputFiles};

mod lock;
mod traits;

/// Decision whether to skip or execute a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipDecision {
    /// Command should skip execution - inputs unchanged since last build.
    Skip {
        /// Human-readable reason for skip (logged to user).
        /// Format: "all N inputs unchanged since YYYY-MM-DD HH:MM"
        reason: String,
    },

    /// Command should execute - inputs changed, outputs missing, or first build.
    Execute {
        /// Human-readable reason for execution (logged to user).
        /// Examples: "No previous build", "Input changed: path/to/file", "1 output missing"
        reason: String,
    },
}

/// Check if command should skip execution based on mtime comparison.
///
/// # Arguments
/// * `cmd` - Command implementing InputFiles + OutputFiles traits
/// * `cmd_name` - Name for TSV file (e.g., "sass" → "sass-mtimes.tsv")
/// * `output_dir` - Base directory for relative paths and TSV storage
///
/// # Returns
/// * `Ok(SkipDecision::Skip)` - Command should skip, inputs unchanged
/// * `Ok(SkipDecision::Execute)` - Command should run (inputs changed, outputs missing, or first build)
/// * `Err(_)` - File I/O error, invalid TSV, or lock timeout
pub fn should_skip<C: InputFiles + OutputFiles>(
    cmd: &C,
    cmd_name: &str,
    output_dir: &Utf8Path,
) -> Result<SkipDecision> {
    let tsv_path = output_dir.join(format!("{}-mtimes.tsv", cmd_name));

    // 1. Load previous build records
    let previous_mtimes = if tsv_path.exists() {
        read_mtimes(&tsv_path)?
    } else {
        return Ok(SkipDecision::Execute {
            reason: "No previous build".to_string(),
        });
    };

    // 2. Get current input files and their mtimes
    let input_files = cmd.input_files();
    let mut current_mtimes = HashMap::new();
    for path in &input_files {
        let mtime = get_mtime_nanos(path)?;
        let rel_path = path.strip_prefix(output_dir).unwrap_or(path).to_owned();
        current_mtimes.insert(rel_path, mtime);
    }

    // 3. Check for input changes
    if current_mtimes.len() != previous_mtimes.len() {
        return Ok(SkipDecision::Execute {
            reason: format!(
                "{} inputs added/removed",
                (current_mtimes.len() as i64 - previous_mtimes.len() as i64).abs()
            ),
        });
    }

    for (path, current_mtime) in &current_mtimes {
        match previous_mtimes.get(path) {
            None => {
                return Ok(SkipDecision::Execute {
                    reason: format!("New input: {}", path),
                });
            }
            Some(&prev_mtime) => {
                if *current_mtime != prev_mtime {
                    return Ok(SkipDecision::Execute {
                        reason: format!("Input changed: {}", path),
                    });
                }
            }
        }
    }

    // 4. Check for missing outputs
    let output_files = cmd.output_files();
    for path in &output_files {
        if !path.exists() {
            return Ok(SkipDecision::Execute {
                reason: format!("Output missing: {}", path),
            });
        }
    }

    // 5. All checks passed - skip execution
    let last_build = previous_mtimes.values().max().copied().unwrap_or(0);
    let timestamp = format_timestamp(last_build);

    Ok(SkipDecision::Skip {
        reason: format!(
            "all {} inputs unchanged since {}",
            input_files.len(),
            timestamp
        ),
    })
}

/// Record successful build completion by updating mtime TSV file.
///
/// Call this AFTER command succeeds to update TSV with current input mtimes.
///
/// # Arguments
/// * `cmd` - Command that just completed successfully
/// * `cmd_name` - Name for TSV file (must match should_skip() call)
/// * `output_dir` - Base directory (must match should_skip() call)
///
/// # Returns
/// * `Ok(())` - TSV file updated successfully
/// * `Err(_)` - File I/O error or invalid paths
pub fn record_success<C: InputFiles + OutputFiles>(
    cmd: &C,
    cmd_name: &str,
    output_dir: &Utf8Path,
) -> Result<()> {
    let tsv_path = output_dir.join(format!("{}-mtimes.tsv", cmd_name));

    // Collect current input mtimes
    let input_files = cmd.input_files();
    let mut records = HashMap::new();

    for path in input_files {
        let mtime = get_mtime_nanos(&path)?;
        let rel_path = path.strip_prefix(output_dir).unwrap_or(&path).to_owned();
        records.insert(rel_path, mtime);
    }

    // Write to TSV
    write_mtimes(&records, &tsv_path)?;

    Ok(())
}

// Internal helper functions

fn read_mtimes(tsv_path: &Utf8Path) -> Result<HashMap<Utf8PathBuf, u128>> {
    let content = fs_err::read_to_string(tsv_path)
        .with_context(|| format!("Failed to read TSV from {}", tsv_path))?;

    let mut records = HashMap::new();
    for (line_num, line) in content.lines().enumerate() {
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid TSV at {}:{} - expected 2 fields, got {}",
                tsv_path,
                line_num + 1,
                parts.len()
            );
        }

        let path = Utf8PathBuf::from(parts[0]);
        let mtime: u128 = parts[1]
            .parse()
            .with_context(|| format!("Invalid mtime at {}:{}", tsv_path, line_num + 1))?;

        records.insert(path, mtime);
    }

    Ok(records)
}

fn write_mtimes(records: &HashMap<Utf8PathBuf, u128>, tsv_path: &Utf8Path) -> Result<()> {
    let mut lines: Vec<String> = records
        .iter()
        .map(|(p, mtime)| format!("{}\t{}", p, mtime))
        .collect();

    lines.sort(); // Deterministic order

    fs_err::write(tsv_path, lines.join("\n"))
        .with_context(|| format!("Failed to write TSV to {}", tsv_path))?;

    Ok(())
}

fn get_mtime_nanos(path: &Utf8Path) -> Result<u128> {
    let metadata =
        fs_err::metadata(path).with_context(|| format!("Failed to read metadata for {}", path))?;

    let mtime = metadata
        .modified()
        .context("Filesystem doesn't support modification time")?;

    match mtime.duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(duration.as_nanos()),
        Err(e) => {
            anyhow::bail!("Invalid modification time for {}: {}", path, e);
        }
    }
}

fn format_timestamp(nanos: u128) -> String {
    // Simple formatter: "YYYY-MM-DD HH:MM"
    let secs = (nanos / 1_000_000_000) as i64;
    let datetime =
        time::OffsetDateTime::from_unix_timestamp(secs).unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]")
        .expect("Invalid format string");

    datetime
        .format(&format)
        .unwrap_or_else(|_| "unknown".to_string())
}
