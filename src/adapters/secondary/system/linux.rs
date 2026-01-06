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
//!
//! Implements `SystemInfoProvider` for Linux systems using:
//! - sysfs (`/sys`) for direct kernel data
//! - procfs (`/proc`) for process/system info  
//! - Command execution for tools like lsblk, nvidia-smi
//!
//! # Platform Support
//!
//! - x86_64: Full support including raw-cpuid
//! - aarch64: Full support via sysfs (ARM servers, DGX Spark)
//!
//! # Detection Strategy
//!
//! Each hardware type uses multiple detection methods:
//! 1. Primary: sysfs (most reliable, always available)
//! 2. Secondary: Command output (lsblk, nvidia-smi, etc.)
//! 3. Fallback: sysinfo crate (cross-platform)

use crate::domain::{
    combine_cpu_info, determine_memory_speed, determine_memory_type, parse_dmidecode_bios_info,
    parse_dmidecode_chassis_info, parse_dmidecode_cpu, parse_dmidecode_memory,
    parse_dmidecode_system_info, parse_free_output, parse_hostname_output, parse_ip_output,
    parse_lscpu_output, BiosInfo, ChassisInfo, CpuInfo, GpuDevice, GpuInfo, GpuVendor, MemoryInfo,
    MotherboardInfo, NetworkInfo, NetworkInterface, NetworkInterfaceType, NumaNode, StorageDevice,
    StorageInfo, StorageType, SystemError, SystemInfo,
};

use crate::domain::parsers::storage::{
    is_virtual_device, parse_lsblk_json, parse_sysfs_rotational, parse_sysfs_size,
};

use crate::ports::{CommandExecutor, SystemCommand, SystemInfoProvider};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use std::fs;
use std::path::{Path, PathBuf};

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

    /// Read a sysfs file and return contents as String
    fn read_sysfs_file(&self, path: &Path) -> Result<String, std::io::Error> {
        fs::read_to_string(path)
    }

    /// Detect storage devices via sysfs /sys/block.
    async fn detect_storage_sysfs(&self) -> Result<Vec<StorageDevice>, SystemError> {
        let mut devices = Vec::new();

        let sys_block = Path::new("/sys/block");

        if !sys_block.exists() {
            return Err(SystemError::NotAvailable {
                resource: "/sys/block".to_string(),
            });
        }

        let entries = fs::read_dir(sys_block).map_err(|e| SystemError::IoErrorWithPath {
            path: "/sys/block".to_string(),
            message: e.to_string(),
        })?;

        for entry in entries.flatten() {
            let device_name = entry.file_name().to_string_lossy().to_string();

            // Filter early to avoid needless I/O
            if is_virtual_device(&device_name) {
                log::trace!("Skipping virtual device: {}", device_name);
                continue;
            }

            let device_path = entry.path();

            // If we can't get the size, skip the device
            let size_path = device_path.join("size");
            let Ok(content) = self.read_sysfs_file(&size_path) else {
                continue;
            };
            let Ok(size_bytes) = parse_sysfs_size(&content) else {
                log::trace!("Cannot parse size for {}: invalid format", device_name);
                continue;
            };

            // Skip tiny devices (< 1GB)
            const MIN_SIZE: u64 = 1_000_000_000;
            if size_bytes < MIN_SIZE {
                log::trace!(
                    "Skipping small device {}: {} bytes",
                    device_name,
                    size_bytes
                );
                continue;
            }

            // Rotational flag
            let rotational_path = device_path.join("queue/rotational");
            let is_rotational = self
                .read_sysfs_file(&rotational_path)
                .map(|content| parse_sysfs_rotational(&content))
                .unwrap_or(false);

            // Determine device type
            let device_type = StorageType::from_device(&device_name, is_rotational);

            // Read optional fields
            let model = self
                .read_sysfs_file(&device_path.join("device/model"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let serial_number = self
                .read_sysfs_file(&device_path.join("device/serial"))
                .map(|s| s.trim().to_string())
                .ok()
                .filter(|s| !s.is_empty());

            let firmware_version = self
                .read_sysfs_file(&device_path.join("device/firmware_rev"))
                .map(|s| s.trim().to_string())
                .ok()
                .filter(|s| !s.is_empty());

            // Alternate paths for NVMe
            let (serial_number, firmware_version) = if device_type == StorageType::Nvme {
                self.read_nvme_sysfs_attrs(&device_name, serial_number, firmware_version)
            } else {
                (serial_number, firmware_version)
            };

            let interface = match &device_type {
                StorageType::Nvme => "NVMe".to_string(),
                StorageType::Emmc => "eMMC".to_string(),
                StorageType::Hdd | StorageType::Ssd => "SATA".to_string(),
                _ => "Unknown".to_string(),
            };

            let mut device = StorageDevice {
                name: device_name.clone(),
                device_path: format!("/dev/{}", device_name),
                device_type: device_type.clone(),
                type_: device_type.display_name().to_string(),
                size_bytes,
                model,
                serial_number,
                firmware_version,
                interface,
                is_rotational,
                detection_method: "sysfs".to_string(),
                ..Default::default()
            };

            device.calculate_size_fields();
            devices.push(device);
        }

        Ok(devices)
    }

    /// Read NVMe-specific sysfs attributes
    fn read_nvme_sysfs_attrs(
        &self,
        device_name: &str,
        existing_serial: Option<String>,
        existing_firmware: Option<String>,
    ) -> (Option<String>, Option<String>) {
        // Extract controller name: "nvme0n1" -> "nvme0"
        let controller = if let Some(stripped) = device_name.strip_prefix("nvme") {
            if let Some(pos) = stripped.find('n') {
                &device_name[..4 + pos]
            } else {
                device_name
            }
        } else {
            device_name
        };

        let nvme_path = PathBuf::from("/sys/class/nvme").join(controller);

        let serial = existing_serial.or_else(|| {
            self.read_sysfs_file(&nvme_path.join("serial"))
                .map(|s| s.trim().to_string())
                .ok()
                .filter(|s| !s.is_empty())
        });

        let firmware = existing_firmware.or_else(|| {
            self.read_sysfs_file(&nvme_path.join("firmware_rev"))
                .map(|s| s.trim().to_string())
                .ok()
                .filter(|s| !s.is_empty())
        });

        (serial, firmware)
    }

    /// Detect storage via lsblk command (JSON output)
    async fn detect_storage_lsblk(&self) -> Result<Vec<StorageDevice>, SystemError> {
        let cmd = SystemCommand::new("lsblk")
            .args(&[
                "-J",
                "-b",
                "-d",
                "-o",
                "NAME,SIZE,TYPE,MODEL,SERIAL,ROTA,TRAN,WWN",
            ])
            .timeout(Duration::from_secs(10));

        let output =
            self.command_executor
                .execute(&cmd)
                .await
                .map_err(|e| SystemError::CommandFailed {
                    command: "lsblk".to_string(),
                    exit_code: None,
                    stderr: e.to_string(),
                })?;

        if !output.success {
            return Err(SystemError::CommandFailed {
                command: "lsblk".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr,
            });
        }

        parse_lsblk_json(&output.stdout).map_err(SystemError::ParseError)
    }

    /// Detect storage via sysinfo crate (cross-platform fallback)
    fn detect_storage_sysinfo(&self) -> Result<Vec<StorageDevice>, SystemError> {
        use sysinfo::Disks;

        let disks = Disks::new_with_refreshed_list();
        let mut devices = Vec::new();

        for disk in disks.iter() {
            let size_bytes = disk.total_space();

            if size_bytes < 1_000_000_000 {
                continue;
            }

            let name = disk.name().to_string_lossy().to_string();
            let name = if name.is_empty() {
                disk.mount_point().to_string_lossy().to_string()
            } else {
                name
            };

            let mut device = StorageDevice {
                name,
                size_bytes,
                detection_method: "sysinfo".to_string(),
                ..Default::default()
            };

            device.calculate_size_fields();
            devices.push(device);
        }
        Ok(devices)
    }

    /// Merge storage info from secondary source into primary
    fn merge_storage_info(&self, primary: &mut Vec<StorageDevice>, secondary: Vec<StorageDevice>) {
        for sec_device in secondary {
            if let Some(pri_device) = primary.iter_mut().find(|d| d.name == sec_device.name) {
                // Fill in missing fields from secondary
                pri_device.serial_number =
                    pri_device.serial_number.take().or(sec_device.serial_number);
                pri_device.firmware_version = pri_device
                    .firmware_version
                    .take()
                    .or(sec_device.firmware_version);
                pri_device.wwn = pri_device.wwn.take().or(sec_device.wwn);

                if pri_device.model.is_empty() && !sec_device.model.is_empty() {
                    pri_device.model = sec_device.model;
                }
            } else {
                // Device not in primary - add it
                primary.push(sec_device);
            }
        }
    }

    /// Enrich network interface with sysfs data
    fn enrich_network_interface_sysfs(&self, iface: &mut NetworkInterface) {
        let iface_path = PathBuf::from("/sys/class/net").join(&iface.name);

        if !iface_path.exists() {
            return;
        }

        // Operational state
        iface.is_up = self
            .read_sysfs_file(&iface_path.join("operstate"))
            .map(|s| s.trim().to_lowercase() == "up")
            .unwrap_or(false);

        // Speed (may be -1 if link is down)
        if let Ok(speed_str) = self.read_sysfs_file(&iface_path.join("speed")) {
            if let Ok(speed) = speed_str.trim().parse::<i32>() {
                if speed > 0 {
                    iface.speed_mbps = Some(speed as u32);
                    iface.speed = Some(format!("{} Mbps", speed));
                }
            }
        }

        // MTU
        if let Ok(mtu_str) = self.read_sysfs_file(&iface_path.join("mtu")) {
            if let Ok(mtu) = mtu_str.trim().parse::<u32>() {
                iface.mtu = mtu;
            }
        }

        // Carrier (link detected)
        iface.carrier = self
            .read_sysfs_file(&iface_path.join("carrier"))
            .map(|s| s.trim() == "1")
            .ok();

        // Virtual interface detection
        let device_path = iface_path.join("device");
        iface.is_virtual = !device_path.exists()
            || iface.name.starts_with("lo")
            || iface.name.starts_with("veth")
            || iface.name.starts_with("br")
            || iface.name.starts_with("docker")
            || iface.name.starts_with("virbr");

        // Driver information (only for physical interfaces)
        if !iface.is_virtual {
            let driver_link = device_path.join("driver");
            if let Ok(driver_path) = fs::read_link(&driver_link) {
                if let Some(driver_name) = driver_path.file_name() {
                    let driver_str = driver_name.to_string_lossy().to_string();
                    iface.driver = Some(driver_str.clone());

                    // Driver version
                    let version_path = PathBuf::from("/sys/module")
                        .join(&driver_str)
                        .join("version");
                    if let Ok(version) = self.read_sysfs_file(&version_path) {
                        iface.driver_version = Some(version.trim().to_string());
                    }
                }
            }
        }

        // Interface type
        iface.interface_type = if iface.name == "lo" {
            NetworkInterfaceType::Loopback
        } else if iface.name.starts_with("br") || iface.name.starts_with("virbr") {
            NetworkInterfaceType::Bridge
        } else if iface.name.starts_with("veth") {
            NetworkInterfaceType::Veth
        } else if iface.name.starts_with("bond") {
            NetworkInterfaceType::Bond
        } else if iface.name.contains('.') {
            NetworkInterfaceType::Vlan
        } else if iface.name.starts_with("wl") {
            NetworkInterfaceType::Wireless
        } else if iface.name.starts_with("ib") {
            NetworkInterfaceType::Infiniband
        } else {
            NetworkInterfaceType::Ethernet
        };
    }
}

#[async_trait]
impl SystemInfoProvider for LinuxSystemInfoProvider {
    async fn get_cpu_info(&self) -> Result<CpuInfo, SystemError> {
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
            _ => Ok(lscpu_info),
        }
    }

    async fn get_memory_info(&self) -> Result<MemoryInfo, SystemError> {
        let free_cmd = SystemCommand::new("free")
            .args(&["-b"])
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
        let mut devices = Vec::new();

        // Try sysfs first
        if let Ok(sysfs_devices) = self.detect_storage_sysfs().await {
            log::debug!("sysfs detected {} storage devices", sysfs_devices.len());
            devices = sysfs_devices;
        } else {
            log::warn!("sysfs storage detection failed, trying next method");
        }

        // Enrich with lsblk
        if let Ok(lsblk_devices) = self.detect_storage_lsblk().await {
            log::debug!(
                "lsblk found {} devices for additional info",
                lsblk_devices.len()
            );
            self.merge_storage_info(&mut devices, lsblk_devices);
        }

        // Fallback to sysinfo
        if devices.is_empty() {
            log::warn!("No devices from sysfs/lsblk, trying sysinfo as fallback");
            if let Ok(sysinfo_devices) = self.detect_storage_sysinfo() {
                devices = sysinfo_devices;
            }
        }

        // Filter virtual devices and sort
        devices.retain(|d| d.device_type != StorageType::Virtual);

        for device in &mut devices {
            if device.size_gb == 0.0 && device.size_bytes > 0 {
                device.calculate_size_fields();
            }
            device.set_device_path();
        }

        devices.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(StorageInfo { devices })
    }

    async fn get_gpu_info(&self) -> Result<GpuInfo, SystemError> {
        let nvidia_cmd = SystemCommand::new("nvidia-smi")
            .args(&[
                "--query-gpu=index,name,uuid,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .timeout(Duration::from_secs(10));

        let mut devices = Vec::new();

        if let Ok(nvidia_output) = self.command_executor.execute(&nvidia_cmd).await {
            if nvidia_output.success {
                for (index, line) in nvidia_output.stdout.lines().enumerate() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 4 {
                        let memory_mb: u64 = parts[3].parse().unwrap_or(0);
                        devices.push(GpuDevice {
                            index: index as u32,
                            name: parts[1].to_string(),
                            uuid: parts[2].to_string(),
                            memory: format!("{} MB", parts[3]),
                            memory_total_mb: memory_mb,
                            pci_id: String::new(),
                            vendor: "NVIDIA".to_string(),
                            vendor_enum: GpuVendor::Nvidia,
                            numa_node: None,
                            detection_method: "nvidia-smi".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Fallback to lspci
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
                            devices.push(GpuDevice {
                                index: gpu_index,
                                name: line.to_string(),
                                uuid: format!("pci-gpu-{gpu_index}"),
                                memory: "Unknown".to_string(),
                                pci_id: String::new(),
                                vendor: "Unknown".to_string(),
                                vendor_enum: GpuVendor::Unknown,
                                numa_node: None,
                                detection_method: "lspci".to_string(),
                                ..Default::default()
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

        let mut interfaces = parse_ip_output(&ip_output.stdout).map_err(SystemError::ParseError)?;

        // Enrich with sysfs data
        for iface in &mut interfaces {
            self.enrich_network_interface_sysfs(iface);
        }

        Ok(NetworkInfo {
            interfaces,
            infiniband: None,
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
