/*
Copyright 2024 San Francisco Compute Company

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

//! Common parsing utilities and helper functions

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref STORAGE_SIZE_RE: Regex = Regex::new(r"(\d+(?:\.\d+)?)\s*(B|K|M|G|T)B?").unwrap();
    pub static ref NETWORK_SPEED_RE: Regex = Regex::new(r"Speed:\s+(\S+)").unwrap();
    pub static ref DMIDECODE_VALUE_RE: Regex = Regex::new(r"^\s*([^:]+):\s*(.+)$").unwrap();
}

/// Parse a size string (e.g., "16GB", "2.5TB") to bytes
///
/// # Arguments
/// * `size_str` - Size string to parse
///
/// # Returns
/// * `Ok(u64)` - Size in bytes
/// * `Err(String)` - Parse error description
pub fn parse_size_to_bytes(size_str: &str) -> Result<u64, String> {
    if size_str.trim().is_empty() || size_str == "Unknown" {
        return Ok(0);
    }

    // Handle macOS format like "2.0 TB (2001111162880 Bytes)"
    if let Some(start) = size_str.find('(') {
        let end_pos = size_str
            .find(" Bytes)")
            .or_else(|| size_str.find(" bytes)"));
        if let Some(end) = end_pos {
            let bytes_str = &size_str[start + 1..end];
            if let Ok(bytes) = bytes_str.replace(",", "").parse::<u64>() {
                return Ok(bytes);
            }
        }
    }

    let size_str = size_str.replace(" ", "").to_uppercase();

    if let Some(captures) = STORAGE_SIZE_RE.captures(&size_str) {
        let number: f64 = captures[1]
            .parse()
            .map_err(|_| format!("Invalid number in size: {}", &captures[1]))?;
        let unit = &captures[2];

        let multiplier = match unit {
            "B" => 1,
            "K" => 1024,
            "M" => 1024 * 1024,
            "G" => 1024 * 1024 * 1024,
            "T" => 1024_u64.pow(4),
            _ => return Err(format!("Unknown unit: {unit}")),
        };

        Ok((number * multiplier as f64) as u64)
    } else {
        Err(format!("Unable to parse size: {size_str}"))
    }
}

/// Extract a value from dmidecode-style output
///
/// # Arguments
/// * `output` - Raw dmidecode output
/// * `key` - Key to search for (e.g., "Vendor", "Version")
///
/// # Returns
/// * `Ok(String)` - Extracted value
/// * `Err(String)` - Key not found or parse error
pub fn extract_dmidecode_value(output: &str, key: &str) -> Result<String, String> {
    for line in output.lines() {
        if let Some(captures) = DMIDECODE_VALUE_RE.captures(line) {
            let line_key = captures[1].trim();
            let value = captures[2].trim();

            if line_key.eq_ignore_ascii_case(key) {
                return Ok(value.to_string());
            }
        }
    }
    Err(format!("Key '{key}' not found in dmidecode output"))
}

/// Parse a key-value pair from system output
///
/// # Arguments
/// * `line` - Line to parse (e.g., "CPU Model: Intel Core i7")
/// * `separator` - Separator character (usually ':')
///
/// # Returns
/// * `Ok((String, String))` - Key-value pair
/// * `Err(String)` - Parse error
pub fn parse_key_value(line: &str, separator: char) -> Result<(String, String), String> {
    if let Some(pos) = line.find(separator) {
        let key = line[..pos].trim().to_string();
        let value = line[pos + 1..].trim().to_string();
        Ok((key, value))
    } else {
        Err(format!("No separator '{separator}' found in line: {line}"))
    }
}

/// Clean and normalize a string value
///
/// # Arguments
/// * `value` - Raw string value
///
/// # Returns
/// * Cleaned string value
pub fn clean_value(value: &str) -> String {
    value
        .trim()
        .replace("  ", " ") // Replace multiple spaces with single space
        .replace("\t", " ") // Replace tabs with spaces
        .to_string()
}

/// Parse boolean-like strings to actual booleans
///
/// # Arguments
/// * `value` - String value (e.g., "yes", "true", "1", "enabled")
///
/// # Returns
/// * `Ok(bool)` - Parsed boolean value
/// * `Err(String)` - Parse error
pub fn parse_boolean(value: &str) -> Result<bool, String> {
    match value.trim().to_lowercase().as_str() {
        "yes" | "true" | "1" | "on" | "enabled" | "active" => Ok(true),
        "no" | "false" | "0" | "off" | "disabled" | "inactive" => Ok(false),
        _ => Err(format!("Cannot parse '{value}' as boolean")),
    }
}

/// Convert bytes to human-readable format
///
/// # Arguments
/// * `bytes` - Number of bytes
///
/// # Returns
/// * Human-readable string (e.g., "16.0 GB", "2.5 TB")
pub fn bytes_to_human_readable(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_to_bytes() {
        assert_eq!(
            parse_size_to_bytes("16GB").unwrap(),
            16 * 1024 * 1024 * 1024
        );
        assert_eq!(
            parse_size_to_bytes("2.5TB").unwrap(),
            (2.5 * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64
        );
        assert_eq!(parse_size_to_bytes("512MB").unwrap(), 512 * 1024 * 1024);
        assert_eq!(parse_size_to_bytes("Unknown").unwrap(), 0);
        assert_eq!(parse_size_to_bytes("").unwrap(), 0);
    }

    #[test]
    fn test_parse_macos_size() {
        let macos_size = "2.0 TB (2001111162880 Bytes)";
        assert_eq!(parse_size_to_bytes(macos_size).unwrap(), 2001111162880);
    }

    #[test]
    fn test_extract_dmidecode_value() {
        let output = "System Information\n\tManufacturer: Dell Inc.\n\tProduct Name: PowerEdge R740\n\tVersion: Not Specified";
        assert_eq!(
            extract_dmidecode_value(output, "Manufacturer").unwrap(),
            "Dell Inc."
        );
        assert_eq!(
            extract_dmidecode_value(output, "Product Name").unwrap(),
            "PowerEdge R740"
        );
        assert!(extract_dmidecode_value(output, "Missing Key").is_err());
    }

    #[test]
    fn test_parse_key_value() {
        let (key, value) = parse_key_value("CPU Model: Intel Core i7", ':').unwrap();
        assert_eq!(key, "CPU Model");
        assert_eq!(value, "Intel Core i7");
    }

    #[test]
    fn test_parse_boolean() {
        assert!(parse_boolean("yes").unwrap());
        assert!(!parse_boolean("false").unwrap());
        assert!(parse_boolean("1").unwrap());
        assert!(!parse_boolean("disabled").unwrap());
        assert!(parse_boolean("maybe").is_err());
    }

    #[test]
    fn test_bytes_to_human_readable() {
        assert_eq!(bytes_to_human_readable(0), "0 B");
        assert_eq!(bytes_to_human_readable(512), "512 B");
        assert_eq!(bytes_to_human_readable(1024), "1.0 KB");
        assert_eq!(bytes_to_human_readable(16 * 1024 * 1024 * 1024), "16.0 GB");
    }
}
