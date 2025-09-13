use fs_err::File;
use std::io::{BufReader, BufWriter};

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

/// Get section type name and size information for debug output
fn get_section_debug_info(section: &Section) -> (String, Option<usize>) {
    let section_name = match section {
        Section::Custom(_) => {
            if let Some(custom) = as_custom_section(section) {
                format!("Custom({})", custom.name())
            } else {
                "Custom(invalid)".to_string()
            }
        }
        Section::Type(_) => "Type".to_string(),
        Section::Import(_) => "Import".to_string(),
        Section::Function(_) => "Function".to_string(),
        Section::Table(_) => "Table".to_string(),
        Section::Memory(_) => "Memory".to_string(),
        Section::Global(_) => "Global".to_string(),
        Section::Export(_) => "Export".to_string(),
        Section::Start(_) => "Start".to_string(),
        Section::Element(_) => "Element".to_string(),
        Section::DataCount(_) => "DataCount".to_string(),
        Section::Code(_) => "Code".to_string(),
        Section::Data(_) => "Data".to_string(),
    };

    // Try to get size information by checking if we can get raw bytes
    let size = match section {
        Section::Custom(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Type(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Import(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Function(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Table(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Memory(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Global(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Export(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Start(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Element(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::DataCount(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Code(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
        Section::Data(blob) => blob.try_as_raw().ok().map(|bytes| bytes.len()),
    };

    (section_name, size)
}

/// Debug output all WASM sections and their sizes
///
/// Example output:
/// ```text
/// === WASM Sections Debug (original module) ===
/// Total sections: 8
///   0: Type (142 bytes)
///   1: Import (89 bytes)
///   2: Function (45 bytes)
///   3: Memory (3 bytes)
///   4: Global (15 bytes)
///   5: Export (234 bytes)
///   6: Code (12856 bytes)
///   7: Custom(.debug_info) (3456 bytes)
/// Total measured section size: 16840 bytes (8/8 sections)
/// === End WASM Sections Debug ===
/// ```
fn debug_wasm_sections(module: &Module, context: &str) {
    log::debug!("=== WASM Sections Debug ({}) ===", context);
    log::debug!("Total sections: {}", module.sections.len());

    let mut total_size = 0usize;
    let mut sections_with_size = 0usize;

    for (index, section) in module.sections.iter().enumerate() {
        let (section_name, size) = get_section_debug_info(section);

        if let Some(size_bytes) = size {
            log::debug!("  {}: {} ({} bytes)", index, section_name, size_bytes);
            total_size += size_bytes;
            sections_with_size += 1;
        } else {
            log::debug!("  {}: {} (size unknown)", index, section_name);
        }
    }

    if sections_with_size > 0 {
        log::debug!(
            "Total measured section size: {} bytes ({}/{} sections)",
            total_size,
            sections_with_size,
            module.sections.len()
        );
    }
    log::debug!("=== End WASM Sections Debug ===");
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

    // Debug output: Original module sections
    debug_wasm_sections(&module, "original module");

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

    let build_num = Uuid::new_v4();
    log::info!("Using build id: {build_num}");

    let build_id = build_num.as_bytes().to_vec();
    module
        .sections
        .push(CustomSection::BuildId(build_id.clone()).into());

    // Write the complete module (including debug info) to the debug file
    log::debug!("Write _debug.wasm file {}", wasm_debug_path);
    module
        .encode_into(BufWriter::new(File::create(wasm_debug_path)?))
        .map_err(|e| anyhow::anyhow!("Failed to encode debug WASM file: {}", e))?;

    // Strip debug sections from main file
    // The name section is human-readable names for functions, types, etc.
    module
        .sections
        .retain(|section| !is_strippable_section(section, true));

    let is_adjacent =
        matches!((wasm_debug_path.parent(), wasm_path.parent()), (Some(p1), Some(p2)) if p1 == p2);

    // Add external debug info reference if we have a debug file
    if is_adjacent {
        let file_name = wasm_debug_path.file_name().unwrap().to_string();
        log::info!("Adding external debug info file name: {}", file_name);

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
