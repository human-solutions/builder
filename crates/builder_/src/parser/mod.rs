mod str_divider;
mod tables;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result};
use serde_json::Value;
use str_divider::StrDivider;
use tables::Tables;

pub fn parse<P>(path: P) -> Result<Tables>
where
    P: AsRef<Path>,
{
    let file = File::open(path).context("Failed to open builder configuration file")?;

    let reader = BufReader::new(file);

    process_lines(reader.lines())
}

fn process_lines<L>(lines: L) -> Result<Tables>
where
    L: Iterator<Item = std::result::Result<String, std::io::Error>>,
{
    let mut buffer = String::new();
    let mut tables = Tables::default();
    let mut key = String::new();
    let dividers = ["[[", "[", "]]", "]"];

    for (line_number, line) in lines.enumerate() {
        let line = line.context(format!("Failed to read line at {line_number}"))?;
        let line = line.trim();

        if line.is_empty() {
            if buffer.is_empty() {
                continue;
            }

            let value: Value = toml::from_str(&buffer).context(format!(
                "Failed to parse the configuration file at line {line_number}"
            ))?;

            if key.is_empty() {
                tables
                    .insert_array(key.clone(), value)
                    .context("Failed to insert table array")?;
            } else {
                tables.insert(key.clone(), value);
            }

            key.clear();
            buffer.clear();
        } else {
            let mut divider = StrDivider::new(line, &dividers);

            if let Some(part) = divider.next() {
                if part == "[[" {
                    key = divider
                        .next()
                        .ok_or(anyhow::Error::msg(format!(
                            "Failed to retrieve array table key at {line_number}",
                        )))?
                        .to_string();
                    tables
                        .insert_empty_vec(key.clone())
                        .context(format!("Failed to initialize table array at {line_number}"))?;
                } else if part == "[" {
                    key = divider
                        .next()
                        .ok_or(anyhow::Error::msg(format!(
                            "Failed to retrieve table key at {line_number}"
                        )))?
                        .to_string();
                } else {
                    if divider.next().is_some() {
                        anyhow::bail!(
                            "Found unexpected part divider '{part}' at line {line_number}: {line}",
                        );
                    }

                    buffer.push_str(&format!("{}\n", line));
                }
            } else {
                buffer.push_str(&format!("{}\n", line));
            }
        }
    }

    if !buffer.is_empty() {
        let value: Value =
            toml::from_str(&buffer).context("Failed to parse the configuration at end of file")?;

        if key.is_empty() {
            tables
                .insert_array(key.clone(), value)
                .context("Failed to insert table array")?;
        } else {
            tables.insert(key.clone(), value);
        }
    }

    Ok(tables)
}

#[test]
fn parse_config() {
    use std::io::Cursor;

    let toml = r###"
[[githook.post-commit]]
install = "true"

[phase.assembly.target.profile.plugin.action]
var = "value"

[[githook.post-commit]]
script = "echo 'Hello, world!'"

    "###;

    let cursor = Cursor::new(toml);
    let reader = BufReader::new(cursor);

    let tables = process_lines(reader.lines()).unwrap();

    println!("{}", tables.string());

    panic!()
}
