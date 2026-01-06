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

//! GPU information parsing functions

use crate::domain::{GpuDevice, GpuVendor};

/// Parse nvidia-smi CSV output
///
/// Expected format from command:
/// `nvidia-smi --query-gpu=index,name,uuid,memory.total,memory.free,pci.bus_id,driver_version,compute_cap --format=csv,noheader,nounits`
///
/// # Arguments
///
/// * `output` - CSV output from nvidia-smi
///
/// # Returns
///
/// List of GPU devices.
pub fn parse_nvidia_smi_output(output: &str) -> Result<Vec<GpuDevice>, String> {
    let mut devices = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 4 {
            continue;
        }

        let index: u32 = parts[0].parse().unwrap_or(devices.len() as u32);
        let name = parts[1].to_string();
        let uuid = parts[2].to_string();
        let memory_total_mb: u64 = parts[3].parse().unwrap_or(0);

        let memory_free_mb = if parts.len() > 4 {
            parts[4].parse().ok()
        } else {
            None
        };

        let pci_bus_id = if parts.len() > 5 {
            Some(parts[5].to_string())
        } else {
            None
        };

        let driver_version = if parts.len() > 6 && !parts[6].is_empty() {
            Some(parts[6].to_string())
        } else {
            None
        };

        let compute_capability = if parts.len() > 7 && !parts[7].is_empty() {
            Some(parts[7].to_string())
        } else {
            None
        };

        let mut device = GpuDevice {
            index,
            name,
            uuid,
            memory_total_mb,
            memory_free_mb,
            pci_bus_id,
            vendor: "NVIDIA".to_string(),
            vendor_enum: GpuVendor::Nvidia,
            driver_version,
            compute_capability,
            detection_method: "nvidia-smi".to_string(),
            ..Default::default()
        };

        device.set_memory_string();
        devices.push(device);
    }

    Ok(devices)
}

/// Parse lspci output for GPU devices
///
/// Expected command: `lspci -nn`
///
/// # Arguments
///
/// * `output` - Output from lspci -nn
pub fn parse_lspci_gpu_output(output: &str) -> Result<Vec<GpuDevice>, String> {
    let mut devices = Vec::new();
    let mut gpu_index = 0;

    for line in output.lines() {
        let line_lower = line.to_lowercase();
        
        // Look for VGA compatible or 3D controller
        if !line_lower.contains("vga") && !line_lower.contains("3d") {
            continue;
        }

        // Extract PCI ID from brackets like [10de:2204]
        let pci_id = extract_pci_id(line);
        
        // Determine vendor from PCI ID
        let (vendor_enum, vendor_name) = if let Some(ref pci) = pci_id {
            let vendor_id = pci.split(':').next().unwrap_or("");
            let vendor = GpuVendor::from_pci_vendor(vendor_id);
            (vendor.clone(), vendor.name().to_string())
        } else {
            (GpuVendor::Unknown, "Unknown".to_string())
        };

        // Extract name (everything after the colon and space)
        let name = line
            .split_once(':')
            .map(|(_, rest)| rest.trim())
            .unwrap_or(line)
            .to_string();

        let device = GpuDevice {
            index: gpu_index,
            name,
            uuid: format!("lspci-gpu-{}", gpu_index),
            pci_id: pci_id.clone().unwrap_or_default(),
            vendor: vendor_name,
            vendor_enum,
            detection_method: "lspci".to_string(),
            ..Default::default()
        };

        devices.push(device);
        gpu_index += 1;
    }

    Ok(devices)
}

/// Extract PCI vendor:device ID from lspci output line
///
/// Looks for pattern like [10de:2204] - must be 4 hex chars : 4 hex chars
fn extract_pci_id(line: &str) -> Option<String> {
    // Find all bracket patterns and look for PCI ID format [xxxx:yyyy]
    let mut search_start = 0;
    while let Some(start) = line[search_start..].find('[') {
        let abs_start = search_start + start;
        if let Some(end) = line[abs_start..].find(']') {
            let bracket_content = &line[abs_start + 1..abs_start + end];
            
            // Check if it looks like a PCI ID (4 hex chars : 4 hex chars)
            if bracket_content.len() == 9 && bracket_content.chars().nth(4) == Some(':') {
                // Verify it's all hex chars
                let parts: Vec<&str> = bracket_content.split(':').collect();
                if parts.len() == 2 
                    && parts[0].len() == 4 
                    && parts[1].len() == 4
                    && parts[0].chars().all(|c| c.is_ascii_hexdigit())
                    && parts[1].chars().all(|c| c.is_ascii_hexdigit())
                {
                    return Some(bracket_content.to_string());
                }
            }
            search_start = abs_start + end + 1;
        } else {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nvidia_smi_output() {
        let output = "0, NVIDIA GeForce RTX 3090, GPU-12345678-1234-1234-1234-123456789012, 24576, 24000, 00000000:01:00.0, 535.129.03, 8.6";
        let devices = parse_nvidia_smi_output(output).unwrap();
        
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "NVIDIA GeForce RTX 3090");
        assert_eq!(devices[0].memory_total_mb, 24576);
        assert_eq!(devices[0].vendor, "NVIDIA");
    }

    #[test]
    fn test_parse_lspci_gpu_output() {
        let output = r#"01:00.0 VGA compatible controller [0300]: NVIDIA Corporation GA102 [GeForce RTX 3090] [10de:2204] (rev a1)
00:02.0 VGA compatible controller [0300]: Intel Corporation Device [8086:9a49] (rev 01)"#;
        
        let devices = parse_lspci_gpu_output(output).unwrap();
        
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].vendor, "NVIDIA");
        assert_eq!(devices[1].vendor, "Intel");
    }

    #[test]
    fn test_extract_pci_id() {
        assert_eq!(extract_pci_id("[10de:2204]"), Some("10de:2204".to_string()));
        assert_eq!(extract_pci_id("NVIDIA [10de:2204] (rev a1)"), Some("10de:2204".to_string()));
        assert_eq!(extract_pci_id("No PCI ID here"), None);
    }
}
