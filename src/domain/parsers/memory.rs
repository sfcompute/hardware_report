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

//! Memory information parsing functions

use super::common::{bytes_to_human_readable, clean_value, parse_size_to_bytes};
use crate::domain::{MemoryInfo, MemoryModule};

/// Parse memory information from Linux free command output
///
/// # Arguments
/// * `free_output` - Raw output from free command
///
/// # Returns
/// * `Ok(String)` - Total memory size as string
/// * `Err(String)` - Parse error description
pub fn parse_free_output(free_output: &str) -> Result<String, String> {
    for line in free_output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(total_kb) = parts[1].parse::<u64>() {
                    let total_bytes = total_kb * 1024;
                    return Ok(bytes_to_human_readable(total_bytes));
                }
            }
        }
    }
    Err("Could not find memory information in free output".to_string())
}

/// Parse memory modules from dmidecode memory output
///
/// # Arguments
/// * `dmidecode_output` - Raw output from dmidecode -t memory
///
/// # Returns
/// * `Ok(Vec<MemoryModule>)` - List of memory modules
/// * `Err(String)` - Parse error description
pub fn parse_dmidecode_memory(dmidecode_output: &str) -> Result<Vec<MemoryModule>, String> {
    let mut modules = Vec::new();
    let mut current_module: Option<MemoryModule> = None;
    let mut in_memory_device = false;

    for line in dmidecode_output.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Memory Device") {
            // Save previous module if it exists
            if let Some(module) = current_module.take() {
                if module.size != "No Module Installed" && module.size != "Unknown" {
                    modules.push(module);
                }
            }

            // Start new module
            current_module = Some(MemoryModule {
                size: "Unknown".to_string(),
                type_: "Unknown".to_string(),
                speed: "Unknown".to_string(),
                location: "Unknown".to_string(),
                manufacturer: "Unknown".to_string(),
                serial: "Unknown".to_string(),
            });
            in_memory_device = true;
            continue;
        }

        if !in_memory_device {
            continue;
        }

        if let Some(ref mut module) = current_module {
            if trimmed.starts_with("Size:") {
                let size = trimmed.split(':').nth(1).unwrap_or("Unknown").trim();
                if size != "No Module Installed" {
                    module.size = clean_value(size);
                }
            } else if trimmed.starts_with("Type:") {
                module.type_ = clean_value(trimmed.split(':').nth(1).unwrap_or("Unknown").trim());
            } else if trimmed.starts_with("Speed:") {
                module.speed = clean_value(trimmed.split(':').nth(1).unwrap_or("Unknown").trim());
            } else if trimmed.starts_with("Locator:") {
                module.location =
                    clean_value(trimmed.split(':').nth(1).unwrap_or("Unknown").trim());
            } else if trimmed.starts_with("Manufacturer:") {
                module.manufacturer =
                    clean_value(trimmed.split(':').nth(1).unwrap_or("Unknown").trim());
            } else if trimmed.starts_with("Serial Number:") {
                module.serial = clean_value(trimmed.split(':').nth(1).unwrap_or("Unknown").trim());
            }
        }

        // End of section
        if trimmed.is_empty() {
            in_memory_device = false;
        }
    }

    // Save last module
    if let Some(module) = current_module {
        if module.size != "No Module Installed" && module.size != "Unknown" {
            modules.push(module);
        }
    }

    Ok(modules)
}

/// Parse memory information from macOS system_profiler output
///
/// # Arguments
/// * `system_profiler_output` - Raw output from system_profiler SPMemoryDataType
///
/// # Returns
/// * `Ok(MemoryInfo)` - Parsed memory information
/// * `Err(String)` - Parse error description
pub fn parse_macos_memory_info(system_profiler_output: &str) -> Result<MemoryInfo, String> {
    let mut total = "Unknown".to_string();
    let mut type_ = "Unknown".to_string();
    let mut manufacturer = "Unknown".to_string();
    let mut modules = Vec::new();

    for line in system_profiler_output.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Memory:") {
            total = trimmed
                .split(':')
                .nth(1)
                .unwrap_or("Unknown")
                .trim()
                .to_string();
        } else if trimmed.starts_with("Type:") {
            type_ = clean_value(trimmed.split(':').nth(1).unwrap_or("Unknown").trim());
        } else if trimmed.starts_with("Manufacturer:") {
            manufacturer = clean_value(trimmed.split(':').nth(1).unwrap_or("Unknown").trim());
        }
    }

    // For Apple Silicon, create a synthetic module entry
    if type_ != "Unknown" || manufacturer != "Unknown" {
        modules.push(MemoryModule {
            size: total.clone(),
            type_: type_.clone(),
            speed: "Integrated".to_string(),
            location: "System Memory".to_string(),
            manufacturer: manufacturer.clone(),
            serial: "N/A".to_string(),
        });
    }

    let speed = if type_.contains("LPDDR") {
        "Integrated".to_string()
    } else {
        "Unknown".to_string()
    };

    Ok(MemoryInfo {
        total,
        type_,
        speed,
        modules,
    })
}

/// Create memory configuration string
///
/// # Arguments
/// * `memory_info` - Memory information
///
/// # Returns
/// * Memory configuration string (e.g., "DDR4 @ 3200 MHz")
pub fn create_memory_config_string(memory_info: &MemoryInfo) -> String {
    format!("{} @ {}", memory_info.type_, memory_info.speed)
}

/// Calculate total memory from modules
///
/// # Arguments
/// * `modules` - List of memory modules
///
/// # Returns
/// * Total memory as human-readable string
pub fn calculate_total_memory_from_modules(modules: &[MemoryModule]) -> String {
    let total_bytes: u64 = modules
        .iter()
        .map(|module| parse_size_to_bytes(&module.size).unwrap_or(0))
        .sum();

    if total_bytes > 0 {
        bytes_to_human_readable(total_bytes)
    } else {
        "Unknown".to_string()
    }
}

/// Determine common memory type from modules
///
/// # Arguments
/// * `modules` - List of memory modules
///
/// # Returns
/// * Common memory type or "Mixed" if different types
pub fn determine_memory_type(modules: &[MemoryModule]) -> String {
    if modules.is_empty() {
        return "Unknown".to_string();
    }

    let first_type = &modules[0].type_;
    if modules.iter().all(|m| m.type_ == *first_type) {
        first_type.clone()
    } else {
        "Mixed".to_string()
    }
}

/// Determine common memory speed from modules
///
/// # Arguments
/// * `modules` - List of memory modules
///
/// # Returns
/// * Common memory speed or "Mixed" if different speeds
pub fn determine_memory_speed(modules: &[MemoryModule]) -> String {
    if modules.is_empty() {
        return "Unknown".to_string();
    }

    let first_speed = &modules[0].speed;
    if modules.iter().all(|m| m.speed == *first_speed) {
        first_speed.clone()
    } else {
        "Mixed".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_free_output() {
        let free_output = r#"               total        used        free      shared  buff/cache   available
Mem:        16777216     8388608     4194304           0     4194304     8388608
Swap:        2097152           0     2097152"#;

        let total_memory = parse_free_output(free_output).unwrap();
        assert_eq!(total_memory, "16.0 GB");
    }

    #[test]
    fn test_parse_dmidecode_memory() {
        let dmidecode_output = r#"Memory Device
	Array Handle: 0x003C
	Error Information Handle: Not Provided
	Total Width: 64 bits
	Data Width: 64 bits
	Size: 16 GB
	Form Factor: SODIMM
	Set: None
	Locator: ChannelA-DIMM0
	Bank Locator: BANK 0
	Type: DDR4
	Type Detail: Synchronous
	Speed: 3200 MT/s
	Manufacturer: Samsung
	Serial Number: 12345678
	Asset Tag: 9876543210
	Part Number: M471A2K43EB1-CWE"#;

        let modules = parse_dmidecode_memory(dmidecode_output).unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].size, "16 GB");
        assert_eq!(modules[0].type_, "DDR4");
        assert_eq!(modules[0].speed, "3200 MT/s");
        assert_eq!(modules[0].manufacturer, "Samsung");
        assert_eq!(modules[0].location, "ChannelA-DIMM0");
    }

    #[test]
    fn test_create_memory_config_string() {
        let memory_info = MemoryInfo {
            total: "32 GB".to_string(),
            type_: "DDR4".to_string(),
            speed: "3200 MT/s".to_string(),
            modules: vec![],
        };

        let config = create_memory_config_string(&memory_info);
        assert_eq!(config, "DDR4 @ 3200 MT/s");
    }

    #[test]
    fn test_determine_memory_type() {
        let modules = vec![
            MemoryModule {
                size: "16 GB".to_string(),
                type_: "DDR4".to_string(),
                speed: "3200 MT/s".to_string(),
                location: "DIMM0".to_string(),
                manufacturer: "Samsung".to_string(),
                serial: "123".to_string(),
            },
            MemoryModule {
                size: "16 GB".to_string(),
                type_: "DDR4".to_string(),
                speed: "3200 MT/s".to_string(),
                location: "DIMM1".to_string(),
                manufacturer: "Samsung".to_string(),
                serial: "456".to_string(),
            },
        ];

        assert_eq!(determine_memory_type(&modules), "DDR4");
        assert_eq!(determine_memory_speed(&modules), "3200 MT/s");
    }
}
