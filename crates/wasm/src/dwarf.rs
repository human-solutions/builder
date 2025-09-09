use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

use anyhow::bail;
use camino_fs::*;
use uuid::Uuid;
use wasmbin::{
    Module,
    sections::{CustomSection, Section},
};

fn as_custom_section(section: &Section) -> Option<&CustomSection> {
    section.try_as()?.try_contents().ok()
}
///
/// Returns `true` if this section should be stripped.
fn is_strippable_section(section: &Section, strip_names: bool) -> bool {
    as_custom_section(section).is_some_and(|section| match section {
        CustomSection::Name(_) => strip_names,
        other => other.name().starts_with(".debug_"),
    })
}

/// Split DWARF debug symbols from a WASM file
pub fn split_debug_symbols(
    wasm_path: &Utf8PathBuf,
    wasm_debug_path: &Utf8PathBuf,
) -> anyhow::Result<()> {
    log::debug!("Reading WASM file {}", wasm_path);
    let mut module = Module::decode_from(BufReader::new(File::open(wasm_path)?))
        .map_err(|e| anyhow::anyhow!("Failed to decode WASM module: {}", e))?;

    let mut has_dwarf = false;
    let mut has_build_id = false;
    for section in module.sections.iter().filter_map(as_custom_section) {
        // Check for DWARF sections
        if section.name().starts_with(".debug_") {
            has_dwarf = true;
        }
        if matches!(section, CustomSection::BuildId(_)) {
            has_build_id = true;
        }
    }

    if !has_dwarf {
        bail!("No debug symbols found in WASM file");
    }
    if has_build_id {
        bail!("WASM file already has a build id");
    }

    let build_id = Uuid::new_v4().as_bytes().to_vec();
    module
        .sections
        .push(CustomSection::BuildId(build_id.clone()).into());

    // Write the complete module (including debug info) to the debug file
    log::debug!("Write _debug.wasm file {}", wasm_debug_path);
    module
        .encode_into(BufWriter::new(File::create(&wasm_debug_path)?))
        .map_err(|e| anyhow::anyhow!("Failed to encode debug WASM file: {}", e))?;

    // Strip debug sections from main file
    // The name section is human-readable names for functions, types, etc.
    module
        .sections
        .retain(|section| !is_strippable_section(section, true));

    let is_adjacent = match (wasm_debug_path.parent(), wasm_path.parent()) {
        (Some(p1), Some(p2)) if p1 == p2 => true,
        _ => false,
    };
    // Add external debug info reference if we have a debug file
    if is_adjacent {
        let file_name = wasm_debug_path.file_name().unwrap().to_string();
        module
            .sections
            .push(CustomSection::ExternalDebugInfo(file_name.into()).into());
    }

    // Write the main module
    log::debug!("Write stripped .wasm file {}", wasm_path);
    module
        .encode_into(BufWriter::new(File::create(wasm_path)?))
        .map_err(|e| anyhow::anyhow!("Failed to encode main WASM file: {}", e))?;
    Ok(())
}
