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

//! macOS system information provider

use crate::domain::{
    parse_hostname_output, parse_macos_cpu_info, parse_macos_memory_info, parse_macos_network_info,
    parse_macos_storage_info, BiosInfo, ChassisInfo, CpuInfo, GpuInfo, MemoryInfo, MotherboardInfo,
    NetworkInfo, NumaNode, StorageInfo, SystemError, SystemInfo,
};
use crate::ports::{CommandExecutor, SystemCommand, SystemInfoProvider};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// macOS system information provider using system_profiler and other macOS tools
pub struct MacOSSystemInfoProvider {
    command_executor: Arc<dyn CommandExecutor>,
}

impl MacOSSystemInfoProvider {
    /// Create a new macOS system information provider
    pub fn new(command_executor: Arc<dyn CommandExecutor>) -> Self {
        Self { command_executor }
    }

    /// Check if required commands are available
    pub async fn check_required_commands(&self) -> Vec<String> {
        let required_commands = ["system_profiler", "sysctl", "ioreg", "hostname", "df"];

        let mut missing = Vec::new();
        for cmd in &required_commands {
            if let Ok(false) = self.command_executor.is_command_available(cmd).await {
                missing.push(cmd.to_string());
            }
        }
        missing
    }
}

#[async_trait]
impl SystemInfoProvider for MacOSSystemInfoProvider {
    async fn get_cpu_info(&self) -> Result<CpuInfo, SystemError> {
        let system_profiler_cmd = SystemCommand::new("system_profiler")
            .args(&["SPHardwareDataType"])
            .timeout(Duration::from_secs(15));
        let output = self
            .command_executor
            .execute(&system_profiler_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "system_profiler".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        parse_macos_cpu_info(&output.stdout).map_err(SystemError::ParseError)
    }

    async fn get_memory_info(&self) -> Result<MemoryInfo, SystemError> {
        let system_profiler_cmd = SystemCommand::new("system_profiler")
            .args(&["SPMemoryDataType"])
            .timeout(Duration::from_secs(15));
        let output = self
            .command_executor
            .execute(&system_profiler_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "system_profiler".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        parse_macos_memory_info(&output.stdout).map_err(SystemError::ParseError)
    }

    async fn get_storage_info(&self) -> Result<StorageInfo, SystemError> {
        let system_profiler_cmd = SystemCommand::new("system_profiler")
            .args(&["SPStorageDataType", "-detailLevel", "full"])
            .timeout(Duration::from_secs(15));
        let output = self
            .command_executor
            .execute(&system_profiler_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "system_profiler".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let devices = parse_macos_storage_info(&output.stdout).map_err(SystemError::ParseError)?;

        Ok(StorageInfo { devices })
    }

    async fn get_gpu_info(&self) -> Result<GpuInfo, SystemError> {
        let system_profiler_cmd = SystemCommand::new("system_profiler")
            .args(&["SPDisplaysDataType"])
            .timeout(Duration::from_secs(15));
        let output = self
            .command_executor
            .execute(&system_profiler_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "system_profiler".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let mut devices = Vec::new();
        let mut gpu_index = 0;

        // Parse macOS GPU/display info
        for line in output.stdout.lines() {
            let trimmed = line.trim();
            if (trimmed.contains("M1")
                || trimmed.contains("M2")
                || trimmed.contains("M3")
                || trimmed.contains("M4"))
                && (trimmed.contains("Max") || trimmed.contains("Pro") || trimmed.contains("Ultra"))
            {
                let memory_cores = if trimmed.contains("M4 Max") {
                    "40 cores"
                } else if trimmed.contains("M4 Pro") {
                    "20 cores"
                } else if trimmed.contains("M3 Max") {
                    "40 cores"
                } else if trimmed.contains("M3 Pro") {
                    "18 cores"
                } else if trimmed.contains("M2 Max") {
                    "38 cores"
                } else if trimmed.contains("M2 Pro") {
                    "19 cores"
                } else if trimmed.contains("M1 Max") {
                    "32 cores"
                } else if trimmed.contains("M1 Pro") {
                    "16 cores"
                } else {
                    "Unknown"
                };

                devices.push(crate::domain::GpuDevice {
                    index: gpu_index,
                    name: format!("Apple {trimmed} (Metal 3)"),
                    uuid: format!("macOS-GPU-{gpu_index}"),
                    memory: format!("Unified Memory ({memory_cores} GPU cores)"),
                    pci_id: "Apple Fabric (Integrated)".to_string(),
                    vendor: "Apple".to_string(),
                    numa_node: None,
                    ..Default::default()
                });
                gpu_index += 1;
            }
        }

        // If no Apple Silicon GPU found, add a generic entry
        if devices.is_empty() {
            devices.push(crate::domain::GpuDevice {
                index: 0,
                name: "Integrated Graphics".to_string(),
                uuid: "macOS-GPU-0".to_string(),
                memory: "Unknown".to_string(),
                pci_id: "Apple Fabric (Integrated)".to_string(),
                vendor: "Apple".to_string(),
                numa_node: None,
                ..Default::default()
            });
        }

        Ok(GpuInfo { devices })
    }

    async fn get_network_info(&self) -> Result<NetworkInfo, SystemError> {
        let ifconfig_cmd = SystemCommand::new("ifconfig").timeout(Duration::from_secs(10));
        let output = self
            .command_executor
            .execute(&ifconfig_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "ifconfig".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let interfaces =
            parse_macos_network_info(&output.stdout).map_err(SystemError::ParseError)?;

        Ok(NetworkInfo {
            interfaces,
            infiniband: None, // macOS doesn't typically have InfiniBand
        })
    }

    async fn get_bios_info(&self) -> Result<BiosInfo, SystemError> {
        // For Apple Silicon, get firmware info
        let system_profiler_cmd = SystemCommand::new("system_profiler")
            .args(&["SPHardwareDataType"])
            .timeout(Duration::from_secs(15));
        let output = self
            .command_executor
            .execute(&system_profiler_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "system_profiler".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let vendor = "Apple Inc.".to_string();
        let mut version = "Unknown".to_string();
        let mut release_date = "Unknown".to_string();

        for line in output.stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("System Firmware Version:") {
                version = trimmed
                    .split(':')
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
            } else if trimmed.starts_with("OS Loader Version:") {
                release_date = trimmed
                    .split(':')
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
            }
        }

        Ok(BiosInfo {
            vendor,
            version: version.clone(),
            release_date,
            firmware_version: version,
        })
    }

    async fn get_chassis_info(&self) -> Result<ChassisInfo, SystemError> {
        let system_profiler_cmd = SystemCommand::new("system_profiler")
            .args(&["SPHardwareDataType"])
            .timeout(Duration::from_secs(15));
        let output = self
            .command_executor
            .execute(&system_profiler_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "system_profiler".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let manufacturer = "Apple Inc.".to_string();
        let mut type_ = "Laptop".to_string();
        let mut serial = "Unknown".to_string();

        for line in output.stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Model Name:") {
                let model = trimmed.split(':').nth(1).unwrap_or("").trim();
                if model.contains("iMac")
                    || model.contains("Mac Pro")
                    || model.contains("Mac Studio")
                {
                    type_ = "Desktop".to_string();
                }
            } else if trimmed.starts_with("Serial Number (system):") {
                serial = trimmed
                    .split(':')
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
            }
        }

        Ok(ChassisInfo {
            manufacturer,
            type_,
            serial,
        })
    }

    async fn get_motherboard_info(&self) -> Result<MotherboardInfo, SystemError> {
        let system_profiler_cmd = SystemCommand::new("system_profiler")
            .args(&["SPHardwareDataType"])
            .timeout(Duration::from_secs(15));
        let output = self
            .command_executor
            .execute(&system_profiler_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "system_profiler".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let manufacturer = "Apple Inc.".to_string();
        let mut product_name = "Unknown Product".to_string();
        let mut version = "Unknown Version".to_string();
        let mut serial = "Unknown S/N".to_string();

        for line in output.stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Model Identifier:") {
                product_name = trimmed
                    .split(':')
                    .nth(1)
                    .unwrap_or("Unknown Product")
                    .trim()
                    .to_string();
            } else if trimmed.starts_with("System Firmware Version:") {
                version = trimmed
                    .split(':')
                    .nth(1)
                    .unwrap_or("Unknown Version")
                    .trim()
                    .to_string();
            } else if trimmed.starts_with("Serial Number (system):") {
                serial = trimmed
                    .split(':')
                    .nth(1)
                    .unwrap_or("Unknown S/N")
                    .trim()
                    .to_string();
            }
        }

        Ok(MotherboardInfo {
            manufacturer,
            product_name,
            version,
            serial,
            features: "Integrated".to_string(),
            location: "System Board".to_string(),
            type_: "Motherboard".to_string(),
        })
    }

    async fn get_system_info(&self) -> Result<SystemInfo, SystemError> {
        let system_profiler_cmd = SystemCommand::new("system_profiler")
            .args(&["SPHardwareDataType"])
            .timeout(Duration::from_secs(15));
        let output = self
            .command_executor
            .execute(&system_profiler_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "system_profiler".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let mut uuid = "Unknown".to_string();
        let mut serial = "Unknown".to_string();
        let mut product_name = "Unknown".to_string();
        let manufacturer = "Apple Inc.".to_string();

        for line in output.stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Hardware UUID:") {
                uuid = trimmed
                    .split(':')
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
            } else if trimmed.starts_with("Serial Number (system):") {
                serial = trimmed
                    .split(':')
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
            } else if trimmed.starts_with("Model Name:") {
                let model = trimmed.split(':').nth(1).unwrap_or("Unknown").trim();
                let chip = output
                    .stdout
                    .lines()
                    .find(|line| line.trim().starts_with("Chip:"))
                    .and_then(|line| line.split(':').nth(1))
                    .unwrap_or("")
                    .trim();
                product_name = if !chip.is_empty() {
                    format!("{model} ({chip})")
                } else {
                    model.to_string()
                };
            }
        }

        Ok(SystemInfo {
            uuid,
            serial,
            product_name,
            product_manufacturer: manufacturer,
        })
    }

    async fn get_numa_topology(&self) -> Result<HashMap<String, NumaNode>, SystemError> {
        // macOS doesn't expose NUMA topology in the same way as Linux
        Ok(HashMap::new())
    }

    async fn get_hostname(&self) -> Result<String, SystemError> {
        let hostname_cmd = SystemCommand::new("hostname").timeout(Duration::from_secs(5));
        let hostname_output = self
            .command_executor
            .execute(&hostname_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "hostname".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        parse_hostname_output(&hostname_output.stdout).map_err(SystemError::ParseError)
    }

    async fn get_fqdn(&self) -> Result<String, SystemError> {
        // On macOS, hostname -f might not work, so fall back to regular hostname
        let hostname_cmd = SystemCommand::new("hostname").timeout(Duration::from_secs(5));
        let hostname_output = self
            .command_executor
            .execute(&hostname_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "hostname".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        parse_hostname_output(&hostname_output.stdout).map_err(SystemError::ParseError)
    }

    async fn get_filesystems(&self) -> Result<Vec<String>, SystemError> {
        let df_cmd = SystemCommand::new("df")
            .args(&["-h"])
            .timeout(Duration::from_secs(5));
        let df_output = self.command_executor.execute(&df_cmd).await.map_err(|e| {
            SystemError::CommandFailed {
                command: "df".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            }
        })?;

        let mut filesystems = Vec::new();
        for line in df_output.stdout.lines().skip(1) {
            // Skip header
            if !line.trim().is_empty() && !line.starts_with("map ") {
                filesystems.push(line.to_string());
            }
        }

        Ok(filesystems)
    }

    async fn has_required_privileges(&self) -> Result<bool, SystemError> {
        // Most macOS system_profiler commands don't require sudo
        Ok(true)
    }

    async fn get_missing_dependencies(&self) -> Result<Vec<String>, SystemError> {
        Ok(self.check_required_commands().await)
    }
}
