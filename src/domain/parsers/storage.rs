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

//! Storage information parsing functions

use crate::domain::StorageDevice;
use super::common::{parse_size_to_bytes, clean_value};

/// Parse storage devices from lsblk output
pub fn parse_lsblk_output(lsblk_output: &str) -> Result<Vec<StorageDevice>, String> {
    let mut devices = Vec::new();
    
    for line in lsblk_output.lines().skip(1) { // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let name = parts[0].to_string();
            let size = parts[3].to_string();
            let type_ = if name.contains("nvme") { "ssd" } else { "disk" };
            
            devices.push(StorageDevice {
                name: clean_value(&name),
                type_: type_.to_string(),
                size: clean_value(&size),
                model: name.clone(),
            });
        }
    }
    
    Ok(devices)
}

/// Parse storage devices from macOS system_profiler output
pub fn parse_macos_storage_info(system_profiler_output: &str) -> Result<Vec<StorageDevice>, String> {
    let mut devices = Vec::new();
    
    // Simplified implementation - in real code this would be more comprehensive
    for line in system_profiler_output.lines() {
        if line.contains("APPLE SSD") {
            devices.push(StorageDevice {
                name: "APPLE SSD".to_string(),
                type_: "ssd".to_string(),
                size: "Unknown".to_string(),
                model: "Apple SSD".to_string(),
            });
            break;
        }
    }
    
    Ok(devices)
}

/// Calculate total storage size from devices
pub fn calculate_total_storage_size(devices: &[StorageDevice]) -> f64 {
    devices.iter()
        .map(|device| parse_size_to_bytes(&device.size).unwrap_or(0))
        .sum::<u64>() as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0) // Convert to TB
}