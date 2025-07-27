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

//! Linux system information provider

use crate::domain::{
    combine_cpu_info, determine_memory_speed, determine_memory_type, parse_dmidecode_bios_info,
    parse_dmidecode_chassis_info, parse_dmidecode_cpu, parse_dmidecode_memory,
    parse_dmidecode_system_info, parse_free_output, parse_hostname_output, parse_ip_output,
    parse_lsblk_output, parse_lscpu_output, BiosInfo, ChassisInfo, CpuInfo, GpuInfo, MemoryInfo,
    MotherboardInfo, NetworkInfo, NumaNode, StorageInfo, SystemError, SystemInfo,
};
use crate::ports::{CommandExecutor, SystemCommand, SystemInfoProvider};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Linux system information provider using standard system commands
pub struct LinuxSystemInfoProvider {
    command_executor: Arc<dyn CommandExecutor>,
}

impl LinuxSystemInfoProvider {
    /// Create a new Linux system information provider
    pub fn new(command_executor: Arc<dyn CommandExecutor>) -> Self {
        Self { command_executor }
    }

    /// Check if required commands are available
    pub async fn check_required_commands(&self) -> Vec<String> {
        let required_commands = [
            "lscpu",
            "dmidecode",
            "free",
            "lsblk",
            "ip",
            "hostname",
            "df",
        ];

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
impl SystemInfoProvider for LinuxSystemInfoProvider {
    async fn get_cpu_info(&self) -> Result<CpuInfo, SystemError> {
        // Get CPU info from lscpu
        let lscpu_cmd = SystemCommand::new("lscpu").timeout(Duration::from_secs(10));
        let lscpu_output = self
            .command_executor
            .execute(&lscpu_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "lscpu".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let lscpu_info =
            parse_lscpu_output(&lscpu_output.stdout).map_err(SystemError::ParseError)?;

        // Try to get additional info from dmidecode (may require sudo)
        let dmidecode_cmd = SystemCommand::new("dmidecode")
            .args(&["-t", "processor"])
            .timeout(Duration::from_secs(10));

        match self
            .command_executor
            .execute_with_privileges(&dmidecode_cmd)
            .await
        {
            Ok(dmidecode_output) if dmidecode_output.success => {
                if let Ok(dmidecode_info) = parse_dmidecode_cpu(&dmidecode_output.stdout) {
                    Ok(combine_cpu_info(lscpu_info, dmidecode_info))
                } else {
                    Ok(lscpu_info)
                }
            }
            _ => Ok(lscpu_info), // Fall back to lscpu info
        }
    }

    async fn get_memory_info(&self) -> Result<MemoryInfo, SystemError> {
        // Get total memory from free command
        let free_cmd = SystemCommand::new("free")
            .args(&["-b"]) // Show in bytes
            .timeout(Duration::from_secs(5));
        let free_output = self
            .command_executor
            .execute(&free_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "free".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let total_memory =
            parse_free_output(&free_output.stdout).map_err(SystemError::ParseError)?;

        // Try to get detailed memory info from dmidecode
        let dmidecode_cmd = SystemCommand::new("dmidecode")
            .args(&["-t", "memory"])
            .timeout(Duration::from_secs(10));

        let (modules, type_, speed) = match self
            .command_executor
            .execute_with_privileges(&dmidecode_cmd)
            .await
        {
            Ok(dmidecode_output) if dmidecode_output.success => {
                match parse_dmidecode_memory(&dmidecode_output.stdout) {
                    Ok(modules) if !modules.is_empty() => {
                        let type_ = determine_memory_type(&modules);
                        let speed = determine_memory_speed(&modules);
                        (modules, type_, speed)
                    }
                    _ => (Vec::new(), "Unknown".to_string(), "Unknown".to_string()),
                }
            }
            _ => (Vec::new(), "Unknown".to_string(), "Unknown".to_string()),
        };

        Ok(MemoryInfo {
            total: total_memory,
            type_,
            speed,
            modules,
        })
    }

    async fn get_storage_info(&self) -> Result<StorageInfo, SystemError> {
        let lsblk_cmd = SystemCommand::new("lsblk")
            .args(&["-d", "-o", "NAME,SIZE,TYPE"])
            .timeout(Duration::from_secs(10));
        let lsblk_output = self
            .command_executor
            .execute(&lsblk_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "lsblk".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let devices = parse_lsblk_output(&lsblk_output.stdout).map_err(SystemError::ParseError)?;

        Ok(StorageInfo { devices })
    }

    async fn get_gpu_info(&self) -> Result<GpuInfo, SystemError> {
        // Try nvidia-smi first
        let nvidia_cmd = SystemCommand::new("nvidia-smi")
            .args(&[
                "--query-gpu=index,name,uuid,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .timeout(Duration::from_secs(10));

        let mut devices = Vec::new();

        if let Ok(nvidia_output) = self.command_executor.execute(&nvidia_cmd).await {
            if nvidia_output.success {
                // Parse NVIDIA GPU info
                for (index, line) in nvidia_output.stdout.lines().enumerate() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 4 {
                        devices.push(crate::domain::GpuDevice {
                            index: index as u32,
                            name: parts[1].to_string(),
                            uuid: parts[2].to_string(),
                            memory: format!("{} MB", parts[3]),
                            pci_id: "Unknown".to_string(),
                            vendor: "NVIDIA".to_string(),
                            numa_node: None,
                        });
                    }
                }
            }
        }

        // Fall back to lspci for basic GPU detection
        if devices.is_empty() {
            let lspci_cmd = SystemCommand::new("lspci")
                .args(&["-nn"])
                .timeout(Duration::from_secs(5));

            if let Ok(lspci_output) = self.command_executor.execute(&lspci_cmd).await {
                if lspci_output.success {
                    let mut gpu_index = 0;
                    for line in lspci_output.stdout.lines() {
                        if line.to_lowercase().contains("vga") || line.to_lowercase().contains("3d")
                        {
                            devices.push(crate::domain::GpuDevice {
                                index: gpu_index,
                                name: line.to_string(),
                                uuid: format!("pci-gpu-{gpu_index}"),
                                memory: "Unknown".to_string(),
                                pci_id: "Unknown".to_string(),
                                vendor: "Unknown".to_string(),
                                numa_node: None,
                            });
                            gpu_index += 1;
                        }
                    }
                }
            }
        }

        Ok(GpuInfo { devices })
    }

    async fn get_network_info(&self) -> Result<NetworkInfo, SystemError> {
        let ip_cmd = SystemCommand::new("ip")
            .args(&["addr", "show"])
            .timeout(Duration::from_secs(5));
        let ip_output = self.command_executor.execute(&ip_cmd).await.map_err(|e| {
            SystemError::CommandFailed {
                command: "ip".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            }
        })?;

        let interfaces = parse_ip_output(&ip_output.stdout).map_err(SystemError::ParseError)?;

        Ok(NetworkInfo {
            interfaces,
            infiniband: None, // TODO: Add infiniband detection
        })
    }

    async fn get_bios_info(&self) -> Result<BiosInfo, SystemError> {
        let dmidecode_cmd = SystemCommand::new("dmidecode")
            .args(&["-t", "bios"])
            .timeout(Duration::from_secs(10));
        let dmidecode_output = self
            .command_executor
            .execute_with_privileges(&dmidecode_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "dmidecode".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        parse_dmidecode_bios_info(&dmidecode_output.stdout).map_err(SystemError::ParseError)
    }

    async fn get_chassis_info(&self) -> Result<ChassisInfo, SystemError> {
        let dmidecode_cmd = SystemCommand::new("dmidecode")
            .args(&["-t", "chassis"])
            .timeout(Duration::from_secs(10));
        let dmidecode_output = self
            .command_executor
            .execute_with_privileges(&dmidecode_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "dmidecode".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        parse_dmidecode_chassis_info(&dmidecode_output.stdout).map_err(SystemError::ParseError)
    }

    async fn get_motherboard_info(&self) -> Result<MotherboardInfo, SystemError> {
        let dmidecode_cmd = SystemCommand::new("dmidecode")
            .args(&["-t", "2"])
            .timeout(Duration::from_secs(10));
        let _dmidecode_output = self
            .command_executor
            .execute_with_privileges(&dmidecode_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "dmidecode".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        // Parse motherboard info (simplified)
        // TODO: Parse _dmidecode_output.stdout to extract actual values
        Ok(MotherboardInfo {
            manufacturer: "Unknown".to_string(),
            product_name: "Unknown".to_string(),
            version: "Unknown".to_string(),
            serial: "Unknown".to_string(),
            features: "Unknown".to_string(),
            location: "Unknown".to_string(),
            type_: "Motherboard".to_string(),
        })
    }

    async fn get_system_info(&self) -> Result<SystemInfo, SystemError> {
        let dmidecode_cmd = SystemCommand::new("dmidecode")
            .args(&["-t", "system"])
            .timeout(Duration::from_secs(10));
        let dmidecode_output = self
            .command_executor
            .execute_with_privileges(&dmidecode_cmd)
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "dmidecode".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        parse_dmidecode_system_info(&dmidecode_output.stdout).map_err(SystemError::ParseError)
    }

    async fn get_numa_topology(&self) -> Result<HashMap<String, NumaNode>, SystemError> {
        // Simplified NUMA topology - in real implementation this would be more comprehensive
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
        let hostname_cmd = SystemCommand::new("hostname")
            .args(&["-f"])
            .timeout(Duration::from_secs(5));
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
            .args(&["-h", "--output=source,fstype,size,used,avail,target"])
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
            if !line.trim().is_empty() {
                filesystems.push(line.to_string());
            }
        }

        Ok(filesystems)
    }

    async fn has_required_privileges(&self) -> Result<bool, SystemError> {
        self.command_executor
            .has_elevated_privileges()
            .await
            .map_err(|e| SystemError::CommandFailed {
                command: "privilege_check".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            })
    }

    async fn get_missing_dependencies(&self) -> Result<Vec<String>, SystemError> {
        Ok(self.check_required_commands().await)
    }
}
