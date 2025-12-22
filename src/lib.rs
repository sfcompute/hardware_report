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

//! Hardware Report Library
//!
//! This library provides hardware information collection capabilities using a
//! Ports and Adapters (Hexagonal) architecture for maintainability and testability.
//!
//! # Architecture
//!
//! - **Domain**: Core business logic and entities
//! - **Ports**: Interfaces for external interactions
//! - **Adapters**: Platform-specific implementations
//!
//! # Usage
//!
//! ## As a Library
//!
//! ```rust,no_run
//! use hardware_report::{HardwareReportingService, ReportConfig};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create service with platform-specific adapters
//!     let service = hardware_report::create_service(None).await?;
//!     
//!     // Generate hardware report
//!     let config = ReportConfig::default();
//!     let report = service.generate_report(config).await?;
//!     
//!     println!("System: {} ({})", report.hostname, report.summary.system_info.product_name);
//!     Ok(())
//! }
//! ```
//!
//! ## Legacy Compatibility
//!
//! The library maintains backward compatibility with the original `ServerInfo` API:
//!
//! ```rust,no_run
//! use hardware_report::ServerInfo;
//!
//! fn legacy_example() -> Result<(), Box<dyn std::error::Error>> {
//!     let server_info = ServerInfo::collect()?;
//!     println!("Hostname: {}", server_info.hostname);
//!     Ok(())
//! }
//! ```

// New Ports and Adapters Architecture
pub mod adapters;
pub mod container;
pub mod domain;
pub mod ports;

// Re-export public API - specific exports to avoid conflicts with legacy types
// Only export new types that don't conflict with legacy compatibility layer
pub use adapters::{
    FileDataPublisher, FileSystemRepository, HttpDataPublisher, LinuxSystemInfoProvider,
    MacOSSystemInfoProvider, UnixCommandExecutor,
};
pub use container::{ContainerConfig, ContainerConfigBuilder, ServiceContainer};
pub use domain::{PublishConfig, PublishError, ReportConfig, ReportError};
pub use ports::{
    CommandExecutor, ConfigurationProvider, DataPublisher, FileRepository,
    HardwareReportingService, OutputFormat, SystemInfoProvider,
};

// Re-export domain entities under a namespace to avoid conflicts
pub use domain::HardwareReport as NewHardwareReport;
pub mod new_domain {
    pub use crate::domain::*;
}

// Legacy compatibility - keep original types and implementations
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::process::Command;

lazy_static! {
    static ref STORAGE_SIZE_RE: Regex = Regex::new(r"(\d+(?:\.\d+)?)(B|K|M|G|T)").unwrap();
    static ref NETWORK_SPEED_RE: Regex = Regex::new(r"Speed:\s+(\S+)").unwrap();
}

/// CPU topology information
#[derive(Debug, Serialize, Deserialize)]
pub struct CpuTopology {
    pub total_cores: u32,
    pub total_threads: u32,
    pub sockets: u32,
    pub cores_per_socket: u32,
    pub threads_per_core: u32,
    pub numa_nodes: u32,
    pub cpu_model: String,
}

/// Motherboard information
#[derive(Debug, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product_name: String,
    pub version: String,
    pub serial: String,
    pub features: String,
    pub location: String,
    pub type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub uuid: String,
    pub serial: String,
    pub product_name: String,
    pub product_manufacturer: String,
}

/// Summary of key system components
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSummary {
    /// System information
    pub system_info: SystemInfo,
    /// Total system memory capacity
    pub total_memory: String,
    /// Memory speed and type
    pub memory_config: String,
    /// Total storage capacity
    pub total_storage: String,
    /// Total storage capacity in TB
    pub total_storage_tb: f64,
    /// Available filesystems
    pub filesystems: Vec<String>,
    /// BIOS information
    pub bios: BiosInfo,
    /// System chassis information
    pub chassis: ChassisInfo,
    /// Motherboard information
    pub motherboard: MotherboardInfo,
    /// Total number of GPUs
    pub total_gpus: usize,
    /// Total number of network interfaces
    pub total_nics: usize,
    /// NUMA topology information
    pub numa_topology: HashMap<String, NumaNode>,
    /// CPU topology information
    pub cpu_topology: CpuTopology,
    /// CPU configuration summary
    pub cpu_summary: String,
}

/// BIOS information
#[derive(Debug, Serialize, Deserialize)]
pub struct BiosInfo {
    pub vendor: String,
    pub version: String,
    pub release_date: String,
    pub firmware_version: String,
}

/// Chassis information
#[derive(Debug, Serialize, Deserialize)]
pub struct ChassisInfo {
    pub manufacturer: String,
    pub type_: String,
    pub serial: String,
}

/// Represents the overall server information
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    /// System summary
    pub summary: SystemSummary,
    /// Other fields remain the same
    pub hostname: String,
    pub fqdn: String,
    pub os_ip: Vec<InterfaceIPs>,
    pub bmc_ip: Option<String>,
    pub bmc_mac: Option<String>,
    pub hardware: HardwareInfo,
    pub network: NetworkInfo,
}

/// Contains detailed hardware information
#[derive(Debug, Serialize, Deserialize)]
pub struct HardwareInfo {
    /// CPU information.
    pub cpu: CpuInfo,
    /// Memory information.
    pub memory: MemoryInfo,
    /// Storage information.
    pub storage: StorageInfo,
    /// GPU information.
    pub gpus: GpuInfo,
}

/// Represents CPU information.
#[derive(Debug, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU model name.
    pub model: String,
    /// Number of cores per socket.
    pub cores: u32,
    /// Number of threads per core.
    pub threads: u32,
    /// Number of sockets.
    pub sockets: u32,
    /// CPU speed in MHz.
    pub speed: String,
}

/// Represents memory information.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total memory size.
    pub total: String,
    /// Memory type (e.g., DDR4).
    pub type_: String,
    /// Memory speed.
    pub speed: String,
    /// Individual memory modules.
    pub modules: Vec<MemoryModule>,
}

/// Represents a memory module.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryModule {
    /// Size of the memory module.
    pub size: String,
    /// Type of the memory module.
    pub type_: String,
    /// Speed of the memory module.
    pub speed: String,
    /// Physical location of the memory module.
    pub location: String,
    /// Manufacturer of the memory module.
    pub manufacturer: String,
    /// Serial number of the memory module.
    pub serial: String,
}

/// Represents storage information.
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageInfo {
    /// List of storage devices.
    pub devices: Vec<StorageDevice>,
}

/// Represents a storage device.
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageDevice {
    /// Device name.
    pub name: String,
    /// Device type (e.g., disk).
    pub type_: String,
    /// Device size.
    pub size: String,
    /// Device model.
    pub model: String,
}

/// Represents GPU information.
#[derive(Debug, Serialize, Deserialize)]
pub struct GpuInfo {
    /// List of GPU devices.
    pub devices: Vec<GpuDevice>,
}

/// Represents a GPU device.
#[derive(Debug, Serialize, Deserialize)]
pub struct GpuDevice {
    /// GPU index
    pub index: u32,
    /// GPU name
    pub name: String,
    /// GPU UUID
    pub uuid: String,
    /// Total GPU memory
    pub memory: String,
    /// PCI ID (vendor:device)
    pub pci_id: String,
    /// Vendor name
    pub vendor: String,
    /// NUMA node
    pub numa_node: Option<i32>,
}

/// Represents a NUMA node
#[derive(Debug, Serialize, Deserialize)]
pub struct NumaNode {
    /// Node ID
    pub id: i32,
    /// CPU list
    pub cpus: Vec<u32>,
    /// Memory size
    pub memory: String,
    /// Devices attached to this node
    pub devices: Vec<NumaDevice>,
    /// distances to other nodse (node_id _> distance)
    pub distances: HashMap<String, u32>,
}

/// Represents a device attached to a NUMA node
#[derive(Debug, Serialize, Deserialize)]
pub struct NumaDevice {
    /// Device type (GPU, NIC, etc.)
    pub type_: String,
    /// PCI ID
    pub pci_id: String,
    /// Device name
    pub name: String,
}

/// Represents network information.
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// List of network interfaces.
    pub interfaces: Vec<NetworkInterface>,
    /// Infiniband information, if available.
    pub infiniband: Option<InfinibandInfo>,
}

/// Represents a network interface.
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name.
    pub name: String,
    /// MAC address.
    pub mac: String,
    /// IP address.
    pub ip: String,
    /// IP prefix.
    pub prefix: String,
    /// Interface speed.
    pub speed: Option<String>,
    /// Interface type.
    pub type_: String,
    pub vendor: String,
    pub model: String,
    pub pci_id: String,
    pub numa_node: Option<i32>,
}

/// Represents Infiniband information.
#[derive(Debug, Serialize, Deserialize)]
pub struct InfinibandInfo {
    /// List of Infiniband interfaces.
    pub interfaces: Vec<IbInterface>,
}

/// Represents an Infiniband interface.
#[derive(Debug, Serialize, Deserialize)]
pub struct IbInterface {
    /// Interface name.
    pub name: String,
    /// Port number.
    pub port: u32,
    /// Interface state.
    pub state: String,
    /// Interface rate.
    pub rate: String,
}

#[allow(dead_code)]
pub struct NumaInfo {
    pub nodes: Vec<NumaNode>,
}

pub mod posting;

#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceIPs {
    pub interface: String,
    pub ip_addresses: Vec<String>,
}

#[allow(unused_variables)]
#[allow(unused_assignments)]
#[allow(clippy::useless_format)]
#[allow(clippy::manual_map)]
#[allow(clippy::format_in_format_args)]
#[allow(clippy::needless_borrows_for_generic_args)]
impl ServerInfo {
    /// Checks for required system dependencies and returns any missing ones
    fn check_dependencies() -> Result<Vec<&'static str>, Box<dyn Error>> {
        let required_packages = if cfg!(target_os = "macos") {
            vec![
                ("system_profiler", "System hardware information"),
                ("sysctl", "System configuration information"),
                ("ioreg", "Hardware registry information"),
                ("hostname", "System hostname"),
                ("df", "Filesystem information"),
            ]
        } else {
            vec![
                ("numactl", "NUMA topology information"),
                ("lspci", "PCI device information"),
                ("ethtool", "Network interface information"),
                ("dmidecode", "System hardware information"),
                ("lscpu", "CPU information"),
                ("ip", "Network interface details"),
                ("lsblk", "Storage device information"),
                ("hostname", "System hostname"),
                ("free", "Memory usage information"),
                ("df", "Filesystem information"),
            ]
        };

        let mut missing_packages = Vec::new();
        let mut missing_info = Vec::new();

        // Check which packages are missing
        for (package, purpose) in &required_packages {
            let status = Command::new("which").arg(package).output()?;

            if !status.status.success() {
                missing_packages.push(*package);
                missing_info.push(format!("  - {package}: {purpose}"));
            }
        }

        if !missing_packages.is_empty() {
            eprintln!("\nWarning: Some system utilities are not installed.");
            eprintln!("Missing utilities:");
            for info in &missing_info {
                eprintln!("{info}");
            }
            eprintln!("\nSome hardware information may be incomplete or unavailable.");

            // Separate packages that are typically pre-installed vs specialized tools
            let core_utils = ["hostname", "ip", "lscpu", "free", "df", "lsblk"];
            let specialized_tools = ["numactl", "lspci", "ethtool", "dmidecode"];

            let missing_core: Vec<&str> = missing_packages
                .iter()
                .filter(|&&pkg| core_utils.contains(&pkg))
                .copied()
                .collect();
            let missing_specialized: Vec<&str> = missing_packages
                .iter()
                .filter(|&&pkg| specialized_tools.contains(&pkg))
                .copied()
                .collect();

            if !missing_core.is_empty() {
                eprintln!("\nCore utilities missing (usually pre-installed):");
                eprintln!(
                    "  Ubuntu/Debian: sudo apt install {}",
                    missing_core
                        .iter()
                        .map(|&pkg| match pkg {
                            "ip" => "iproute2",
                            "lscpu" | "lsblk" => "util-linux",
                            "free" | "hostname" => "procps",
                            "df" => "coreutils",
                            _ => pkg,
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                eprintln!(
                    "  RHEL/Fedora: sudo dnf install {}",
                    missing_core
                        .iter()
                        .map(|&pkg| match pkg {
                            "ip" => "iproute",
                            "lscpu" | "lsblk" => "util-linux",
                            "free" => "procps-ng",
                            "hostname" | "df" => "coreutils",
                            _ => pkg,
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }

            if !missing_specialized.is_empty() {
                eprintln!("\nSpecialized tools missing:");
                eprintln!(
                    "  Ubuntu/Debian: sudo apt install {}",
                    missing_specialized.join(" ")
                );
                eprintln!(
                    "  RHEL/Fedora: sudo dnf install {}",
                    missing_specialized.join(" ")
                );
            }
            eprintln!();
        }

        Ok(missing_packages)
    }
    /// Gets motherboard information using dmidecode
    fn get_motherboard_info() -> Result<MotherboardInfo, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::get_motherboard_info_macos()
        } else {
            Self::get_motherboard_info_linux()
        }
    }

    fn get_motherboard_info_macos() -> Result<MotherboardInfo, Box<dyn Error>> {
        let manufacturer = "Apple Inc.".to_string();
        let mut product_name = "Unknown Product".to_string();
        let mut version = "Unknown Version".to_string();
        let mut serial = "Unknown S/N".to_string();

        // Get hardware information from system_profiler
        if let Ok(output) = Command::new("system_profiler")
            .args(&["SPHardwareDataType"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);

            for line in output_str.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("Model Identifier:") {
                    product_name = trimmed
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown Product")
                        .trim()
                        .to_string();
                } else if trimmed.starts_with("System Firmware Version:") {
                    version = trimmed
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown Version")
                        .trim()
                        .to_string();
                } else if trimmed.starts_with("Serial Number (system):") {
                    serial = trimmed
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown S/N")
                        .trim()
                        .to_string();
                }
            }
        }

        // Try to get MLB (Main Logic Board) serial from ioreg if system serial not found
        if serial == "Unknown S/N" {
            if let Ok(output) = Command::new("ioreg")
                .args(&["-c", "IOPlatformExpertDevice", "-d", "2"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("\"serial-number\"") {
                        if let Some(start) = line.find('<') {
                            if let Some(end) = line.find('>') {
                                let hex_bytes = &line[start + 1..end];
                                // Convert hex bytes to string if possible
                                serial = hex_bytes.replace(" ", "");
                                break;
                            }
                        }
                    }
                }
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

    fn get_motherboard_info_linux() -> Result<MotherboardInfo, Box<dyn Error>> {
        let output = match Command::new("dmidecode").args(&["-t", "2"]).output() {
            Ok(out) => {
                if !out.status.success() {
                    Command::new("sudo")
                        .args(&["dmidecode", "-t", "2"])
                        .output()?
                } else {
                    out
                }
            }
            Err(_) => Command::new("sudo")
                .args(&["dmidecode", "-t", "2"])
                .output()?,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        if !output.status.success() || stdout.trim().is_empty() {
            return Ok(MotherboardInfo {
                manufacturer: "Unknown Manufacturer".to_string(),
                product_name: "Unknown Product".to_string(),
                version: "Unknown Version".to_string(),
                serial: "Unknown S/N".to_string(),
                features: "Unknown".to_string(),
                location: "Unknown".to_string(),
                type_: "Unknown".to_string(),
            });
        }

        Ok(MotherboardInfo {
            manufacturer: Self::extract_dmidecode_value(&stdout, "Manufacturer")
                .unwrap_or_else(|_| "Unknown Manufacturer".to_string()),
            product_name: Self::extract_dmidecode_value(&stdout, "Product Name")
                .unwrap_or_else(|_| "Unknown Product".to_string()),
            version: Self::extract_dmidecode_value(&stdout, "Version")
                .unwrap_or_else(|_| "Unknown Version".to_string()),
            serial: Self::extract_dmidecode_value(&stdout, "Serial Number")
                .unwrap_or_else(|_| "Unknown S/N".to_string()),
            features: Self::extract_dmidecode_value(&stdout, "Features")
                .unwrap_or_else(|_| "Unknown".to_string()),
            location: Self::extract_dmidecode_value(&stdout, "Location In Chassis")
                .unwrap_or_else(|_| "Unknown".to_string()),
            type_: Self::extract_dmidecode_value(&stdout, "Type")
                .unwrap_or_else(|_| "Unknown".to_string()),
        })
    }

    /// Converts storage size string to bytes
    fn parse_storage_size(size: &str) -> Result<u64, Box<dyn Error>> {
        if size == "Unknown" || size.is_empty() {
            return Ok(0);
        }

        // First try to extract exact byte count from parentheses (macOS format)
        // Example: "2.0 TB (2001111162880 Bytes)" or "2 TB (1,995,218,165,760 bytes)"
        if let Some(start) = size.find('(') {
            // Try both uppercase and lowercase "bytes"
            let end_pos = size.find(" Bytes)").or_else(|| size.find(" bytes)"));
            if let Some(end) = end_pos {
                let bytes_str = &size[start + 1..end];
                if let Ok(bytes) = bytes_str.replace(",", "").parse::<u64>() {
                    return Ok(bytes);
                }
            }
        }

        let size_str = size.replace(" ", "").to_uppercase();

        // Handle both Linux format (123G) and macOS format (123 GB)
        let re = Regex::new(r"(\d+(?:\.\d+)?)\s*(B|KB?|MB?|GB?|TB?|BYTES?)$")?;

        if let Some(caps) = re.captures(&size_str) {
            let value: f64 = caps[1].parse()?;
            let unit = &caps[2];

            let multiplier = match unit {
                "B" | "BYTES" => 1_u64,
                "K" | "KB" => 1024_u64,
                "M" | "MB" => 1024_u64 * 1024,
                "G" | "GB" => 1024_u64 * 1024 * 1024,
                "T" | "TB" => 1024_u64 * 1024 * 1024 * 1024,
                _ => return Err(format!("Unknown storage unit: {unit}").into()),
            };

            Ok((value * multiplier as f64) as u64)
        } else {
            // Try to handle other formats gracefully
            if size_str.contains("BYTES") || size_str.contains("B") {
                Ok(0) // Return 0 for unparseable sizes instead of erroring
            } else {
                Err(format!("Invalid storage size format: {size}").into())
            }
        }
    }

    /// Automatically installs numactl if not present
    fn auto_install_numactl() -> Result<bool, Box<dyn Error>> {
        // Check if we have sudo/root privileges
        let euid = unsafe { libc::geteuid() };
        let use_sudo = euid != 0;

        // Detect the package manager
        let pkg_managers = vec![
            ("apt-get", vec!["update"], vec!["install", "-y", "numactl"]),
            ("apt", vec!["update"], vec!["install", "-y", "numactl"]),
            ("dnf", vec![], vec!["install", "-y", "numactl"]),
            ("yum", vec![], vec!["install", "-y", "numactl"]),
            ("zypper", vec!["refresh"], vec!["install", "-y", "numactl"]),
        ];

        for (manager, update_args, install_args) in pkg_managers {
            // Check if the package manager exists
            if Command::new("which")
                .arg(manager)
                .output()?
                .status
                .success()
            {
                // Run update command if needed
                if !update_args.is_empty() {
                    let mut update_cmd = if use_sudo {
                        let mut cmd = Command::new("sudo");
                        cmd.arg(manager);
                        cmd
                    } else {
                        Command::new(manager)
                    };

                    update_cmd.args(&update_args);
                    let _ = update_cmd.output(); // Ignore update errors
                }

                // Run install command
                let mut install_cmd = if use_sudo {
                    let mut cmd = Command::new("sudo");
                    cmd.arg(manager);
                    cmd
                } else {
                    Command::new(manager)
                };

                install_cmd.args(&install_args);
                let output = install_cmd.output()?;

                if output.status.success() {
                    // Verify numactl was installed
                    if Command::new("which")
                        .arg("numactl")
                        .output()?
                        .status
                        .success()
                    {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    // Remove automatic package installation
    #[allow(dead_code)]
    fn suggest_package_installation(missing_packages: &[&str]) {
        if !missing_packages.is_empty() {
            eprintln!(
                "\nTo get complete hardware information, please install the missing utilities:"
            );
            eprintln!("\nFor Ubuntu/Debian:");
            eprintln!("  sudo apt install {}", missing_packages.join(" "));
            eprintln!("\nFor RHEL/Fedora:");
            eprintln!("  sudo dnf install {}", missing_packages.join(" "));
        }
    }

    /// Gets hostname of the server
    fn get_hostname() -> Result<String, Box<dyn Error>> {
        match Command::new("hostname").output() {
            Ok(output) => Ok(String::from_utf8(output.stdout)?.trim().to_string()),
            Err(_) => {
                // Fallback to reading /etc/hostname or use system name
                if let Ok(contents) = std::fs::read_to_string("/etc/hostname") {
                    Ok(contents.trim().to_string())
                } else {
                    Ok("unknown".to_string())
                }
            }
        }
    }

    fn get_fqdn() -> Result<String, Box<dyn Error>> {
        match Command::new("hostname").args(&["-f"]).output() {
            Ok(output) => Ok(String::from_utf8(output.stdout)?.trim().to_string()),
            Err(_) => {
                // Fallback to hostname if FQDN lookup fails
                Self::get_hostname()
            }
        }
    }

    /// Gets PCI information for a device
    fn get_pci_info(pci_addr: &str) -> Result<(String, String, String), Box<dyn Error>> {
        // Run lspci with verbose output and machine-readable format
        let output = match Command::new("lspci")
            .args(&["-vmm", "-s", pci_addr])
            .output()
        {
            Ok(output) => output,
            Err(_) => {
                // lspci not available, return unknown values
                return Ok((
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                ));
            }
        };

        let output_str = String::from_utf8(output.stdout)?;
        let mut vendor = String::new();
        let mut device = String::new();
        let mut vendor_id = String::new();
        let mut device_id = String::new();

        for line in output_str.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let value = parts[1].trim();
                match parts[0].trim() {
                    "Vendor" => vendor = value.to_string(),
                    "Device" => device = value.to_string(),
                    "SVendor" => {
                        if vendor.is_empty() {
                            vendor = value.to_string()
                        }
                    }
                    "SDevice" => {
                        if device.is_empty() {
                            device = value.to_string()
                        }
                    }
                    _ => {}
                }
            }
        }

        // Get vendor and device IDs using -n flag
        let id_output = match Command::new("lspci").args(&["-n", "-s", pci_addr]).output() {
            Ok(output) => output,
            Err(_) => {
                // Return early if lspci is not available
                return Ok((vendor, device, "Unknown".to_string()));
            }
        };

        let id_str = String::from_utf8(id_output.stdout)?;
        if let Some(line) = id_str.lines().next() {
            if let Some(ids) = line.split_whitespace().nth(2) {
                let parts: Vec<&str> = ids.split(':').collect();
                if parts.len() >= 2 {
                    vendor_id = parts[0].to_string();
                    device_id = parts[1].to_string();
                }
            }
        }

        let pci_id = format!("{vendor_id}:{device_id}");
        Ok((vendor, device, pci_id))
    }

    /// Gets NUMA node for a PCI device
    fn get_numa_node(pci_addr: &str) -> Option<i32> {
        if let Ok(path) = std::fs::read_link(format!("/sys/bus/pci/devices/{pci_addr}/numa_node")) {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(node) = content.trim().parse() {
                    return Some(node);
                }
            }
        }
        None
    }

    fn collect_numa_topology() -> Result<HashMap<String, NumaNode>, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            // NUMA topology is not applicable on macOS in the same way
            // Return empty HashMap for macOS
            return Ok(HashMap::new());
        }

        let mut nodes = HashMap::new();
        let mut collecting_distances = false;

        // Get NUMA information using numactl
        let output = Command::new("numactl").args(&["--hardware"]).output()?;

        let output_str = String::from_utf8(output.stdout)?;

        for line in output_str.lines() {
            if line.starts_with("node ") && line.contains("size:") {
                // Parse node and memory information
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(id) = parts[1].parse::<i32>() {
                        let memory = format!("{} {}", parts[3], parts[4]);

                        // Create new node entry
                        nodes.insert(
                            id.to_string(),
                            NumaNode {
                                id,
                                memory,
                                cpus: Vec::new(),
                                distances: HashMap::new(),
                                devices: Vec::new(),
                            },
                        );
                    }
                }
            } else if line.contains("node distances:") {
                collecting_distances = true;
                continue;
            } else if collecting_distances && line.trim().starts_with("node") {
                // Parse distance information
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 2 {
                    if let Ok(from_node) = parts[1].parse::<i32>() {
                        for (i, dist_str) in parts[2..].iter().enumerate() {
                            if let Ok(distance) = dist_str.parse::<u32>() {
                                if let Some(node) = nodes.get_mut(&from_node.to_string()) {
                                    node.distances.insert(i.to_string(), distance);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Get CPU to node mapping
        let output = match Command::new("lscpu").args(&["-p=cpu,node"]).output() {
            Ok(output) => output,
            Err(_) => {
                // lscpu not available, skip CPU to node mapping
                return Ok(nodes);
            }
        };

        let output_str = String::from_utf8(output.stdout)?;
        for line in output_str.lines() {
            if line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let (Ok(cpu), Ok(node)) = (parts[0].parse::<u32>(), parts[1].parse::<i32>()) {
                    if let Some(numa_node) = nodes.get_mut(&node.to_string()) {
                        numa_node.cpus.push(cpu);
                    }
                }
            }
        }

        // Sort CPUs within each node
        for node in nodes.values_mut() {
            node.cpus.sort();
        }

        Ok(nodes)
    }

    fn collect_ip_addresses() -> Result<Vec<InterfaceIPs>, Box<dyn Error>> {
        let output = match Command::new("ip").args(&["-j", "addr"]).output() {
            Ok(output) => output,
            Err(_) => {
                // ip command not available, return empty list
                return Ok(Vec::new());
            }
        };
        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

        let mut interfaces = Vec::new();

        if let Some(ifaces) = json.as_array() {
            for iface in ifaces {
                if let Some(name) = iface["ifname"].as_str() {
                    if name == "lo" {
                        continue;
                    } // Skip loopback

                    let mut ip_addresses = Vec::new();

                    if let Some(addr_info) = iface["addr_info"].as_array() {
                        for addr in addr_info {
                            if addr["family"].as_str() == Some("inet") {
                                if let Some(ip) = addr["local"].as_str() {
                                    ip_addresses.push(ip.to_string());
                                }
                            }
                        }
                    }

                    if !ip_addresses.is_empty() {
                        interfaces.push(InterfaceIPs {
                            interface: name.to_string(),
                            ip_addresses,
                        });
                    }
                }
            }
        }

        Ok(interfaces)
    }

    /// Gets system UUID and serial using platform-specific commands
    fn get_system_info() -> Result<SystemInfo, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::get_system_info_macos()
        } else {
            Self::get_system_info_linux()
        }
    }

    /// Gets system UUID and serial on macOS using system_profiler
    fn get_system_info_macos() -> Result<SystemInfo, Box<dyn Error>> {
        let output = match Command::new("system_profiler")
            .args(&["SPHardwareDataType", "-detailLevel", "basic"])
            .output()
        {
            Ok(output) => output,
            Err(_) => {
                return Ok(SystemInfo {
                    uuid: "Unknown".to_string(),
                    serial: "Unknown".to_string(),
                    product_name: "Mac".to_string(),
                    product_manufacturer: "Apple Inc.".to_string(),
                });
            }
        };

        let output_str = String::from_utf8(output.stdout)?;
        let mut uuid = "Unknown".to_string();
        let mut serial = "Unknown".to_string();
        let mut model = "Mac".to_string();

        for line in output_str.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Hardware UUID:") {
                uuid = trimmed
                    .split(":")
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
            } else if trimmed.starts_with("Serial Number (system):") {
                serial = trimmed
                    .split(":")
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
            } else if trimmed.starts_with("Model Name:") {
                model = trimmed
                    .split(":")
                    .nth(1)
                    .unwrap_or("Mac")
                    .trim()
                    .to_string();
            } else if trimmed.starts_with("Chip:") {
                // Also extract chip info for newer Macs that don't show "Processor Name:"
                let chip_name = trimmed
                    .split(":")
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
                if !chip_name.is_empty() && chip_name != "Unknown" {
                    model = format!("{model} ({chip_name})");
                }
            }
        }

        Ok(SystemInfo {
            uuid,
            serial,
            product_name: model,
            product_manufacturer: "Apple Inc.".to_string(),
        })
    }

    /// Gets system UUID and serial from dmidecode on Linux
    fn get_system_info_linux() -> Result<SystemInfo, Box<dyn Error>> {
        let output = match Command::new("dmidecode").args(&["-t", "system"]).output() {
            Ok(out) => {
                if !out.status.success() {
                    Command::new("sudo")
                        .args(&["dmidecode", "-t", "system"])
                        .output()?
                } else {
                    out
                }
            }
            Err(_) => Command::new("sudo")
                .args(&["dmidecode", "-t", "system"])
                .output()?,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        if !output.status.success() || stdout.trim().is_empty() {
            return Ok(SystemInfo {
                uuid: "Unknown".to_string(),
                serial: "Unknown".to_string(),
                product_name: "Unknown".to_string(),
                product_manufacturer: "Unknown".to_string(),
            });
        }

        let uuid = Self::extract_dmidecode_value(&stdout, "UUID")
            .unwrap_or_else(|_| "Unknown".to_string());
        let serial = Self::extract_dmidecode_value(&stdout, "Serial Number")
            .unwrap_or_else(|_| "Unknown".to_string());
        let product_name = Self::extract_dmidecode_value(&stdout, "Product Name")
            .unwrap_or_else(|_| "Unknown".to_string());
        let product_manufacturer = Self::extract_dmidecode_value(&stdout, "Manufacturer")
            .unwrap_or_else(|_| "Unknown".to_string());

        Ok(SystemInfo {
            uuid,
            serial,
            product_name,
            product_manufacturer,
        })
    }

    /// Collects all server information
    pub fn collect() -> Result<Self, Box<dyn Error>> {
        // Check dependencies first and warn about missing packages
        let missing_packages = Self::check_dependencies()?;

        // Automatically install numactl if it's missing (Linux only)
        if !cfg!(target_os = "macos") && missing_packages.contains(&"numactl") {
            eprintln!("numactl is not installed. Attempting automatic installation...");

            // Try to detect the package manager and install numactl
            if Self::auto_install_numactl()? {
                eprintln!("Successfully installed numactl.");
            } else {
                eprintln!("Warning: Could not automatically install numactl. NUMA information may be incomplete.");
            }
        }

        // Check if running as root
        let euid = unsafe { libc::geteuid() };
        if euid != 0 {
            eprintln!(
            "\nWarning: This program requires root privileges to access all hardware information."
        );
            eprintln!(
                "Please run it with: sudo {}",
                std::env::args().next().unwrap_or_default()
            );
            eprintln!("Continuing with limited functionality...\n");
        }

        let hostname = Self::get_hostname()?;
        let fqdn = Self::get_fqdn()?;
        let hardware = Self::collect_hardware_info()?;
        let network = Self::collect_network_info()?;
        let system_info = Self::get_system_info()?;
        let (bmc_ip, bmc_mac) = Self::collect_ipmi_info()?;
        let os_ip = Self::collect_ip_addresses()?;

        let summary = Self::generate_summary(&hardware, &network, &system_info)?;

        Ok(ServerInfo {
            summary,
            hostname,
            fqdn,
            os_ip,
            bmc_ip,
            bmc_mac,
            hardware,
            network,
        })
    }

    /// Calculates total storage in terabytes
    fn calculate_total_storage_tb(storage: &StorageInfo) -> Result<f64, Box<dyn Error>> {
        let mut total_bytes: u64 = 0;

        for device in &storage.devices {
            total_bytes += Self::parse_storage_size(&device.size)?;
        }

        Ok(total_bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
    }

    /// Calculates total storage capacity
    fn calculate_total_storage(storage: &StorageInfo) -> Result<String, Box<dyn Error>> {
        let mut total_bytes: u64 = 0;
        let re = Regex::new(r"(\d+(?:\.\d+)?)(B|K|M|G|T)")?;

        for device in &storage.devices {
            let size_str = device.size.replace(" ", "");

            if let Some(caps) = re.captures(&size_str) {
                let value: f64 = caps[1].parse()?;
                let unit = &caps[2];

                let multiplier = match unit {
                    "B" => 1_u64,
                    "K" => 1024_u64,
                    "M" => 1024_u64 * 1024,
                    "G" => 1024_u64 * 1024 * 1024,
                    "T" => 1024_u64 * 1024 * 1024 * 1024,
                    _ => 0_u64,
                };

                total_bytes += (value * multiplier as f64) as u64;
            }
        }

        if total_bytes >= 1024 * 1024 * 1024 * 1024 {
            Ok(format!(
                "{:.1} TB",
                total_bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0)
            ))
        } else {
            Ok(format!(
                "{:.1} GB",
                total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            ))
        }
    }

    /// Gets filesystem information
    fn get_filesystems() -> Result<Vec<String>, Box<dyn Error>> {
        let output = match Command::new("df")
            .args(["-h", "--output=source,fstype,size,used,avail,target"])
            .output()
        {
            Ok(output) => output,
            Err(_) => {
                // df not available, return empty list
                return Ok(Vec::new());
            }
        };

        let output_str = String::from_utf8(output.stdout)?;
        let mut filesystems = Vec::new();

        for line in output_str.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 6 {
                filesystems.push(format!(
                    "{} ({}) - {} total, {} used, {} available, mounted on {}",
                    fields[0], fields[1], fields[2], fields[3], fields[4], fields[5]
                ));
            }
        }

        Ok(filesystems)
    }

    /// Gets BIOS information using platform-specific commands
    fn get_bios_info() -> Result<BiosInfo, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::get_bios_info_macos()
        } else {
            Self::get_bios_info_linux()
        }
    }

    /// Gets firmware information on macOS using system_profiler
    fn get_bios_info_macos() -> Result<BiosInfo, Box<dyn Error>> {
        let output = match Command::new("system_profiler")
            .args(&["SPHardwareDataType", "-detailLevel", "basic"])
            .output()
        {
            Ok(output) => output,
            Err(_) => {
                return Ok(BiosInfo {
                    vendor: "Apple Inc.".to_string(),
                    version: "Unknown Version".to_string(),
                    release_date: "Unknown Date".to_string(),
                    firmware_version: "N/A".to_string(),
                });
            }
        };

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut firmware_version = "N/A".to_string();

        for line in output_str.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("System Firmware Version:") {
                firmware_version = trimmed
                    .split(":")
                    .nth(1)
                    .unwrap_or("N/A")
                    .trim()
                    .to_string();
                break;
            }
        }

        Ok(BiosInfo {
            vendor: "Apple Inc.".to_string(),
            version: firmware_version.clone(),
            release_date: "N/A".to_string(),
            firmware_version,
        })
    }

    /// Gets BIOS information using dmidecode on Linux
    fn get_bios_info_linux() -> Result<BiosInfo, Box<dyn Error>> {
        // Try without sudo first, then with sudo if needed
        let output = match Command::new("dmidecode").args(&["-t", "0"]).output() {
            Ok(out) => {
                if !out.status.success() {
                    Command::new("sudo")
                        .args(&["dmidecode", "-t", "0"])
                        .output()?
                } else {
                    out
                }
            }
            Err(_) => Command::new("sudo")
                .args(&["dmidecode", "-t", "0"])
                .output()?,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        if !output.status.success() || stdout.trim().is_empty() {
            return Ok(BiosInfo {
                vendor: "Unknown Vendor".to_string(),
                version: "Unknown Version".to_string(),
                release_date: "Unknown Date".to_string(),
                firmware_version: "N/A".to_string(),
            });
        }

        Ok(BiosInfo {
            vendor: Self::extract_dmidecode_value(&stdout, "Vendor")
                .unwrap_or_else(|_| "Unknown Vendor".to_string()),
            version: Self::extract_dmidecode_value(&stdout, "Version")
                .unwrap_or_else(|_| "Unknown Version".to_string()),
            release_date: Self::extract_dmidecode_value(&stdout, "Release Date")
                .unwrap_or_else(|_| "Unknown Date".to_string()),
            firmware_version: Self::extract_dmidecode_value(&stdout, "Firmware Revision")
                .unwrap_or_else(|_| "N/A".to_string()),
        })
    }

    /// Gets chassis information using platform-specific commands
    fn get_chassis_info() -> Result<ChassisInfo, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::get_chassis_info_macos()
        } else {
            Self::get_chassis_info_linux()
        }
    }

    /// Gets chassis information on macOS using system_profiler
    fn get_chassis_info_macos() -> Result<ChassisInfo, Box<dyn Error>> {
        let output = match Command::new("system_profiler")
            .args(&["SPHardwareDataType", "-detailLevel", "basic"])
            .output()
        {
            Ok(output) => output,
            Err(_) => {
                return Ok(ChassisInfo {
                    manufacturer: "Apple Inc.".to_string(),
                    type_: "Laptop".to_string(),
                    serial: "Unknown S/N".to_string(),
                });
            }
        };

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut serial = "Unknown S/N".to_string();
        let mut chassis_type = "Laptop".to_string();

        for line in output_str.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Serial Number (system):") {
                serial = trimmed
                    .split(":")
                    .nth(1)
                    .unwrap_or("Unknown S/N")
                    .trim()
                    .to_string();
            } else if trimmed.starts_with("Model Name:") {
                let model = trimmed.split(":").nth(1).unwrap_or("").trim().to_string();
                if model.contains("Mac Pro")
                    || model.contains("Mac Studio")
                    || model.contains("iMac Pro")
                {
                    chassis_type = "Desktop".to_string();
                } else if model.contains("iMac") {
                    chassis_type = "All-in-One".to_string();
                } else if model.contains("MacBook") {
                    chassis_type = "Laptop".to_string();
                } else if model.contains("Mac mini") {
                    chassis_type = "Mini PC".to_string();
                }
            }
        }

        Ok(ChassisInfo {
            manufacturer: "Apple Inc.".to_string(),
            type_: chassis_type,
            serial,
        })
    }

    /// Gets chassis information using dmidecode on Linux
    fn get_chassis_info_linux() -> Result<ChassisInfo, Box<dyn Error>> {
        let output = match Command::new("dmidecode").args(&["-t", "3"]).output() {
            Ok(out) => {
                if !out.status.success() {
                    Command::new("sudo")
                        .args(&["dmidecode", "-t", "3"])
                        .output()?
                } else {
                    out
                }
            }
            Err(_) => Command::new("sudo")
                .args(&["dmidecode", "-t", "3"])
                .output()?,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        if !output.status.success() || stdout.trim().is_empty() {
            return Ok(ChassisInfo {
                manufacturer: "Unknown Manufacturer".to_string(),
                type_: "Unknown Type".to_string(),
                serial: "Unknown S/N".to_string(),
            });
        }

        Ok(ChassisInfo {
            manufacturer: Self::extract_dmidecode_value(&stdout, "Manufacturer")
                .unwrap_or_else(|_| "Unknown Manufacturer".to_string()),
            type_: Self::extract_dmidecode_value(&stdout, "Type")
                .unwrap_or_else(|_| "Unknown Type".to_string()),
            serial: Self::extract_dmidecode_value(&stdout, "Serial Number")
                .unwrap_or_else(|_| "Unknown S/N".to_string()),
        })
    }

    /// Extracts a value from 'dmidecode' output without debug output
    pub fn extract_dmidecode_value(text: &str, key: &str) -> Result<String, Box<dyn Error>> {
        let patterns = [
            format!(r"(?im)^\s*{}: (.*)$", regex::escape(key)),
            format!(r"(?im)^\s*{}\s+(.*)$", regex::escape(key)),
            format!(r"(?im)^{}: (.*)$", regex::escape(key)),
        ];

        for pattern in patterns.iter() {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(cap) = re.captures(text) {
                    if let Some(value) = cap.get(1) {
                        let result = value.as_str().trim().to_string();
                        if !result.is_empty() {
                            return Ok(result);
                        }
                    }
                }
            }
        }

        Err(format!("Could not find key: {key}").into())
    }

    /// Gets detailed CPU topology information
    fn get_cpu_topology() -> Result<CpuTopology, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::get_cpu_topology_macos()
        } else {
            Self::get_cpu_topology_linux()
        }
    }

    /// Gets CPU topology information on macOS
    fn get_cpu_topology_macos() -> Result<CpuTopology, Box<dyn Error>> {
        let physical_cores = Self::get_macos_cpu_cores().unwrap_or(0);
        let logical_cores = Self::get_macos_logical_cpu_cores().unwrap_or(physical_cores);
        let threads_per_core = if physical_cores > 0 {
            logical_cores / physical_cores
        } else {
            1
        };

        // Get CPU model from system_profiler
        let mut cpu_model = "Unknown".to_string();
        if let Ok(output) = Command::new("system_profiler")
            .args(&["SPHardwareDataType", "-detailLevel", "basic"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.trim().starts_with("Processor Name:") || line.trim().starts_with("Chip:") {
                    cpu_model = line
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown")
                        .trim()
                        .to_string();
                    break;
                }
            }
        }

        Ok(CpuTopology {
            total_cores: physical_cores,
            total_threads: logical_cores,
            sockets: 1, // Most Macs have 1 socket
            cores_per_socket: physical_cores,
            threads_per_core,
            numa_nodes: 1, // macOS typically has 1 NUMA node
            cpu_model,
        })
    }

    /// Gets CPU topology information on Linux using lscpu
    fn get_cpu_topology_linux() -> Result<CpuTopology, Box<dyn Error>> {
        let output = match Command::new("lscpu").args(&["-J"]).output() {
            Ok(output) => output,
            Err(_) => {
                // lscpu not available, return default topology
                return Ok(CpuTopology {
                    total_cores: 0,
                    total_threads: 0,
                    sockets: 0,
                    cores_per_socket: 0,
                    threads_per_core: 0,
                    numa_nodes: 0,
                    cpu_model: "Unknown".to_string(),
                });
            }
        };

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut info_map = HashMap::new();

        if let Some(entries) = json["lscpu"].as_array() {
            for entry in entries {
                if let (Some(field), Some(data)) = (entry["field"].as_str(), entry["data"].as_str())
                {
                    let key = field.trim_end_matches(':');
                    info_map.insert(key.to_string(), data.to_string());
                }
            }
        }

        Ok(CpuTopology {
            total_cores: info_map
                .get("Core(s) per socket")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
                * info_map
                    .get("Socket(s)")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
            total_threads: info_map
                .get("CPU(s)")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            sockets: info_map
                .get("Socket(s)")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            cores_per_socket: info_map
                .get("Core(s) per socket")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            threads_per_core: info_map
                .get("Thread(s) per core")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            numa_nodes: info_map
                .get("NUMA node(s)")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            cpu_model: info_map
                .get("Model name")
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string()),
        })
    }

    /// Generates system summary with better error handling
    /// Enhanced summary generation with NUMA topology
    fn generate_summary(
        hardware: &HardwareInfo,
        network: &NetworkInfo,
        system_info: &SystemInfo,
    ) -> Result<SystemSummary, Box<dyn Error>> {
        let bios = Self::get_bios_info().unwrap_or_else(|_| BiosInfo {
            vendor: "Unknown Vendor".to_string(),
            version: "Unknown Version".to_string(),
            release_date: "Unknown Date".to_string(),
            firmware_version: "N/A".to_string(),
        });

        let chassis = Self::get_chassis_info().unwrap_or_else(|_| ChassisInfo {
            manufacturer: "Unknown Manufacturer".to_string(),
            type_: "Unknown Type".to_string(),
            serial: "Unknown S/N".to_string(),
        });
        let motherboard = Self::get_motherboard_info().unwrap_or_else(|_| MotherboardInfo {
            manufacturer: "Unknown Manufacturer".to_string(),
            product_name: "Unknown Product".to_string(),
            version: "Unknown Version".to_string(),
            serial: "Unknown S/N".to_string(),
            features: "Unknown".to_string(),
            location: "Unknown".to_string(),
            type_: "Unknown".to_string(),
        });

        let cpu_topology = Self::get_cpu_topology()?;

        // Create a detailed CPU summary string
        let cpu_summary = format!(
            "{} ({} Socket{}, {} Core{}/Socket, {} Thread{}/Core, {} NUMA Node{})",
            cpu_topology.cpu_model,
            cpu_topology.sockets,
            if cpu_topology.sockets > 1 { "s" } else { "" },
            cpu_topology.cores_per_socket,
            if cpu_topology.cores_per_socket > 1 {
                "s"
            } else {
                ""
            },
            cpu_topology.threads_per_core,
            if cpu_topology.threads_per_core > 1 {
                "s"
            } else {
                ""
            },
            cpu_topology.numa_nodes,
            if cpu_topology.numa_nodes > 1 { "s" } else { "" }
        );

        let total_storage_tb = Self::calculate_total_storage_tb(&hardware.storage)?;

        Ok(SystemSummary {
            system_info: SystemInfo {
                uuid: system_info.uuid.clone(),
                serial: system_info.serial.clone(),
                product_name: system_info.product_name.clone(),
                product_manufacturer: system_info.product_manufacturer.clone(),
            },
            total_memory: hardware.memory.total.clone(),
            memory_config: format!("{} @ {}", hardware.memory.type_, hardware.memory.speed),
            total_storage_tb,
            total_storage: Self::calculate_total_storage(&hardware.storage)?,
            filesystems: Self::get_filesystems().unwrap_or_default(),
            bios,
            chassis,
            motherboard,
            total_gpus: hardware.gpus.devices.len(),
            total_nics: network.interfaces.len(),
            numa_topology: Self::collect_numa_topology()?,
            cpu_topology,
            cpu_summary,
        })
    }

    /// Collects detailed hardware information.
    fn collect_hardware_info() -> Result<HardwareInfo, Box<dyn Error>> {
        Ok(HardwareInfo {
            cpu: Self::collect_cpu_info()?,
            memory: Self::collect_memory_info()?,
            storage: Self::collect_storage_info()?,
            gpus: Self::collect_gpu_info()?,
        })
    }

    /// Collects CPU information by parsing platform-specific commands.
    fn collect_cpu_info() -> Result<CpuInfo, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::collect_cpu_info_macos()
        } else {
            Self::collect_cpu_info_linux()
        }
    }

    /// Collects CPU information on Linux using lscpu
    fn collect_cpu_info_linux() -> Result<CpuInfo, Box<dyn Error>> {
        // Use 'lscpu -J' for JSON output to ensure reliable parsing.
        let output = match Command::new("lscpu").args(&["-J"]).output() {
            Ok(output) => output,
            Err(_) => {
                // lscpu not available, return basic CPU info
                return Ok(CpuInfo {
                    model: "Unknown".to_string(),
                    cores: 0,
                    threads: 0,
                    sockets: 0,
                    speed: "Unknown".to_string(),
                });
            }
        };
        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

        // Build a map of lscpu key-value pairs.
        let mut cpu_info_map = HashMap::new();
        if let Some(entries) = json["lscpu"].as_array() {
            for entry in entries {
                if let (Some(field), Some(data)) = (entry["field"].as_str(), entry["data"].as_str())
                {
                    let key = field.trim_end_matches(':');
                    cpu_info_map.insert(key.to_string(), data.to_string());
                }
            }
        }

        // Extract required CPU information.
        let model = cpu_info_map.get("Model name").cloned().unwrap_or_default();
        let cores: u32 = cpu_info_map
            .get("Core(s) per socket")
            .unwrap_or(&"0".to_string())
            .parse()?;
        let threads: u32 = cpu_info_map
            .get("Thread(s) per core")
            .unwrap_or(&"0".to_string())
            .parse()?;
        let sockets: u32 = cpu_info_map
            .get("Socket(s)")
            .unwrap_or(&"0".to_string())
            .parse()?;
        let speed = cpu_info_map.get("CPU MHz").cloned().unwrap_or_default();

        Ok(CpuInfo {
            model,
            cores,
            threads,
            sockets,
            speed: format!("{speed} MHz"),
        })
    }

    /// Collects CPU information on macOS using system_profiler and sysctl
    fn collect_cpu_info_macos() -> Result<CpuInfo, Box<dyn Error>> {
        let cores = Self::get_macos_cpu_cores().unwrap_or(0);
        let logical_cores = Self::get_macos_logical_cpu_cores().unwrap_or(cores);
        let threads = if logical_cores > cores {
            logical_cores / cores
        } else {
            1
        };
        let speed = Self::get_macos_cpu_speed().unwrap_or("Unknown".to_string());

        // Get CPU model using system_profiler
        let model = match Command::new("system_profiler")
            .args(&["SPHardwareDataType", "-detailLevel", "basic"])
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8(output.stdout)?;
                // Extract processor name from system_profiler output
                for line in output_str.lines() {
                    if line.trim().starts_with("Processor Name:")
                        || line.trim().starts_with("Chip:")
                    {
                        let cpu_name = line
                            .split(":")
                            .nth(1)
                            .unwrap_or("Unknown")
                            .trim()
                            .to_string();
                        return Ok(CpuInfo {
                            model: cpu_name,
                            cores,
                            threads,
                            sockets: 1, // Most Macs have 1 socket
                            speed,
                        });
                    }
                }
                "Unknown".to_string()
            }
            Err(_) => "Unknown".to_string(),
        };

        Ok(CpuInfo {
            model,
            cores,
            threads,
            sockets: 1,
            speed,
        })
    }

    fn get_macos_cpu_cores() -> Result<u32, Box<dyn Error>> {
        let output = Command::new("sysctl")
            .args(&["-n", "hw.physicalcpu"])
            .output()?;
        let cores_str = String::from_utf8(output.stdout)?;
        Ok(cores_str.trim().parse().unwrap_or(0))
    }

    fn get_macos_logical_cpu_cores() -> Result<u32, Box<dyn Error>> {
        let output = Command::new("sysctl")
            .args(&["-n", "hw.logicalcpu"])
            .output()?;
        let cores_str = String::from_utf8(output.stdout)?;
        Ok(cores_str.trim().parse().unwrap_or(0))
    }

    fn get_macos_cpu_speed() -> Result<String, Box<dyn Error>> {
        // Try different sysctl keys for CPU frequency
        let freq_keys = [
            "hw.cpufrequency_max",
            "hw.cpufrequency",
            "machdep.cpu.max_basic",
        ];

        for key in &freq_keys {
            if let Ok(output) = Command::new("sysctl").args(&["-n", key]).output() {
                let freq_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(freq_hz) = freq_str.trim().parse::<u64>() {
                    let freq_mhz = freq_hz / 1_000_000;
                    return Ok(format!("{freq_mhz} MHz"));
                }
            }
        }

        // Fallback: try to get from system_profiler
        if let Ok(output) = Command::new("system_profiler")
            .args(&["SPHardwareDataType", "-detailLevel", "basic"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.trim().starts_with("Processor Speed:") {
                    return Ok(line
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown")
                        .trim()
                        .to_string());
                }
            }
        }

        Ok("Unknown".to_string())
    }

    /// Collects memory information by parsing platform-specific commands.
    fn collect_memory_info() -> Result<MemoryInfo, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::collect_memory_info_macos()
        } else {
            Self::collect_memory_info_linux()
        }
    }

    /// Collects memory information on Linux using dmidecode
    fn collect_memory_info_linux() -> Result<MemoryInfo, Box<dyn Error>> {
        let output = match Command::new("dmidecode").args(&["-t", "memory"]).output() {
            Ok(output) => output,
            Err(_) => {
                // dmidecode not available, try to get basic info from /proc/meminfo
                let total = Self::get_total_memory()?;
                return Ok(MemoryInfo {
                    total,
                    type_: "Unknown".to_string(),
                    speed: "Unknown".to_string(),
                    modules: Vec::new(),
                });
            }
        };
        let output_str = String::from_utf8(output.stdout)?;

        // Parse dmidecode output for detailed memory information.
        let mut modules = Vec::new();
        let re_module = Regex::new(r"Memory Device\n(?:\t.+\n)+")?;

        for cap in re_module.find_iter(&output_str) {
            let module_text = cap.as_str();
            if let Ok(module) = Self::parse_memory_module(module_text) {
                modules.push(module);
            }
        }

        // Determine total memory size.
        let total = Self::get_total_memory()?;

        // Determine overall memory type and speed.
        let mut type_set = HashSet::new();
        let mut speed_set = HashSet::new();
        for module in &modules {
            type_set.insert(module.type_.clone());
            speed_set.insert(module.speed.clone());
        }

        let type_ = if type_set.len() == 1 {
            type_set.into_iter().next().unwrap_or_default()
        } else {
            "Mixed".to_string()
        };

        let speed = if speed_set.len() == 1 {
            speed_set.into_iter().next().unwrap_or_default()
        } else {
            "Mixed".to_string()
        };

        Ok(MemoryInfo {
            total,
            type_,
            speed,
            modules,
        })
    }

    /// Collects memory information on macOS using system_profiler
    fn collect_memory_info_macos() -> Result<MemoryInfo, Box<dyn Error>> {
        let total = Self::get_total_memory_macos()?;
        let mut type_ = "Unknown".to_string();
        let mut manufacturer = "Unknown".to_string();

        // Use system_profiler to get memory details
        let output = match Command::new("system_profiler")
            .args(&["SPMemoryDataType", "-detailLevel", "full"])
            .output()
        {
            Ok(output) => output,
            Err(_) => {
                return Ok(MemoryInfo {
                    total,
                    type_: "Unknown".to_string(),
                    speed: "Unknown".to_string(),
                    modules: Vec::new(),
                });
            }
        };

        let output_str = String::from_utf8(output.stdout)?;

        // For Apple Silicon Macs, memory info is at the top level
        for line in output_str.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Memory:") {
                // Extract just the size, we already have the total
                continue;
            } else if trimmed.starts_with("Type:") {
                type_ = trimmed
                    .split(":")
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
            } else if trimmed.starts_with("Manufacturer:") {
                manufacturer = trimmed
                    .split(":")
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
            }
        }

        // For Apple Silicon, memory is integrated, so we create a synthetic module entry
        let modules = if type_ != "Unknown" || manufacturer != "Unknown" {
            vec![MemoryModule {
                size: total.clone(),
                type_: type_.clone(),
                speed: "Integrated".to_string(),
                location: "System Memory".to_string(),
                manufacturer: manufacturer.clone(),
                serial: "N/A".to_string(),
            }]
        } else {
            // Check for traditional DIMM slots (Intel Macs)
            let mut modules = Vec::new();
            let mut current_module = None;
            let mut current_slot = String::new();

            for line in output_str.lines() {
                let trimmed = line.trim();

                if trimmed.starts_with("DIMM") || trimmed.starts_with("BANK") {
                    // Save previous module if exists
                    if let Some(module) = current_module.take() {
                        modules.push(module);
                    }
                    current_slot = trimmed.to_string();
                    current_module = Some(MemoryModule {
                        size: "Unknown".to_string(),
                        type_: "Unknown".to_string(),
                        speed: "Unknown".to_string(),
                        location: current_slot.clone(),
                        manufacturer: "Unknown".to_string(),
                        serial: "Unknown".to_string(),
                    });
                } else if let Some(ref mut module) = current_module {
                    if trimmed.starts_with("Size:") {
                        module.size = trimmed
                            .split(":")
                            .nth(1)
                            .unwrap_or("Unknown")
                            .trim()
                            .to_string();
                    } else if trimmed.starts_with("Type:") {
                        module.type_ = trimmed
                            .split(":")
                            .nth(1)
                            .unwrap_or("Unknown")
                            .trim()
                            .to_string();
                    } else if trimmed.starts_with("Speed:") {
                        module.speed = trimmed
                            .split(":")
                            .nth(1)
                            .unwrap_or("Unknown")
                            .trim()
                            .to_string();
                    } else if trimmed.starts_with("Manufacturer:") {
                        module.manufacturer = trimmed
                            .split(":")
                            .nth(1)
                            .unwrap_or("Unknown")
                            .trim()
                            .to_string();
                    } else if trimmed.starts_with("Serial Number:") {
                        module.serial = trimmed
                            .split(":")
                            .nth(1)
                            .unwrap_or("Unknown")
                            .trim()
                            .to_string();
                    }
                }
            }

            // Save last module
            if let Some(module) = current_module {
                modules.push(module);
            }
            modules
        };

        // Determine overall memory type and speed from modules
        let mut type_set = HashSet::new();
        let mut speed_set = HashSet::new();
        for module in &modules {
            if module.type_ != "Unknown" {
                type_set.insert(module.type_.clone());
            }
            if module.speed != "Unknown" && module.speed != "Integrated" {
                speed_set.insert(module.speed.clone());
            }
        }

        let final_type = if type_set.len() == 1 {
            type_set.into_iter().next().unwrap_or(type_)
        } else if type_set.is_empty() {
            type_
        } else {
            "Mixed".to_string()
        };

        let speed = if speed_set.len() == 1 {
            speed_set
                .into_iter()
                .next()
                .unwrap_or("Integrated".to_string())
        } else if speed_set.is_empty() {
            "Integrated".to_string()
        } else {
            "Mixed".to_string()
        };

        Ok(MemoryInfo {
            total,
            type_: final_type,
            speed,
            modules,
        })
    }

    /// Retrieves the total memory size using platform-specific commands.
    fn get_total_memory() -> Result<String, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::get_total_memory_macos()
        } else {
            Self::get_total_memory_linux()
        }
    }

    /// Retrieves the total memory size on macOS using sysctl.
    fn get_total_memory_macos() -> Result<String, Box<dyn Error>> {
        let output = Command::new("sysctl")
            .args(&["-n", "hw.memsize"])
            .output()?;
        let memsize_str = String::from_utf8(output.stdout)?;
        if let Ok(bytes) = memsize_str.trim().parse::<u64>() {
            let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            Ok(format!("{gb:.1}G"))
        } else {
            Ok("Unknown".to_string())
        }
    }

    /// Retrieves the total memory size using 'free -h' on Linux.
    fn get_total_memory_linux() -> Result<String, Box<dyn Error>> {
        let output = match Command::new("free").arg("-h").output() {
            Ok(output) => output,
            Err(_) => {
                // free command not available, try reading from /proc/meminfo
                if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
                    for line in contents.lines() {
                        if line.starts_with("MemTotal:") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let kb: u64 = parts[1].parse().unwrap_or(0);
                                let gb = kb as f64 / 1024.0 / 1024.0;
                                return Ok(format!("{gb:.1}G"));
                            }
                        }
                    }
                }
                return Ok("Unknown".to_string());
            }
        };
        let output_str = String::from_utf8(output.stdout)?;
        let re = Regex::new(r"Mem:\s+(\S+)")?;

        if let Some(cap) = re.captures(&output_str) {
            Ok(cap[1].to_string())
        } else {
            Err("Could not determine total memory".into())
        }
    }

    /// Parses a memory module's information from a section of 'dmidecode' output.
    fn parse_memory_module(text: &str) -> Result<MemoryModule, Box<dyn Error>> {
        let size = Self::extract_dmidecode_value(text, "Size")?;
        if size == "No Module Installed" || size == "Not Installed" {
            // Skip slots without installed memory modules.
            return Err("Memory module not installed".into());
        }

        let type_ = Self::extract_dmidecode_value(text, "Type")?;
        let speed = Self::extract_dmidecode_value(text, "Speed")?;
        let location = Self::extract_dmidecode_value(text, "Locator")?;
        let manufacturer = Self::extract_dmidecode_value(text, "Manufacturer")?;
        let serial = Self::extract_dmidecode_value(text, "Serial Number")?;

        Ok(MemoryModule {
            size,
            type_,
            speed,
            location,
            manufacturer,
            serial,
        })
    }

    /// Collects storage information using platform-specific commands.
    fn collect_storage_info() -> Result<StorageInfo, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::collect_storage_info_macos()
        } else {
            Self::collect_storage_info_linux()
        }
    }

    /// Collects storage information on macOS using system_profiler primarily
    fn collect_storage_info_macos() -> Result<StorageInfo, Box<dyn Error>> {
        let mut devices = Vec::new();

        // First, try system_profiler (more portable and comprehensive)
        if let Ok(output) = Command::new("system_profiler")
            .args(&["SPStorageDataType", "-detailLevel", "full"])
            .output()
        {
            let output_str = String::from_utf8(output.stdout)?;
            let mut physical_drives = std::collections::HashSet::new();
            let lines: Vec<&str> = output_str.lines().collect();
            let mut i = 0;

            while i < lines.len() {
                let line = lines[i].trim();

                if line.starts_with("Physical Drive:") {
                    i += 1;
                    let mut device_name = "Unknown".to_string();
                    let mut capacity = "2 TB".to_string(); // Default for your system
                    let mut protocol = "Unknown".to_string();
                    let mut medium_type = "Unknown".to_string();

                    while i < lines.len() && lines[i].starts_with("        ") {
                        let detail_line = lines[i].trim();
                        if detail_line.starts_with("Device Name:") {
                            device_name = detail_line
                                .split(":")
                                .nth(1)
                                .unwrap_or("Unknown")
                                .trim()
                                .to_string();
                        } else if detail_line.starts_with("Medium Type:") {
                            medium_type = detail_line
                                .split(":")
                                .nth(1)
                                .unwrap_or("Unknown")
                                .trim()
                                .to_string();
                        } else if detail_line.starts_with("Protocol:") {
                            protocol = detail_line
                                .split(":")
                                .nth(1)
                                .unwrap_or("Unknown")
                                .trim()
                                .to_string();
                        }
                        i += 1;
                    }

                    // Look backwards for capacity in the parent volume
                    let start_search = i.saturating_sub(15);
                    for j in start_search..i {
                        if j < lines.len() && lines[j].trim().starts_with("Capacity:") {
                            capacity = lines[j]
                                .trim()
                                .split(":")
                                .nth(1)
                                .unwrap_or("2 TB")
                                .trim()
                                .to_string();
                            break;
                        }
                    }

                    if !physical_drives.contains(&device_name) && device_name != "Unknown" {
                        physical_drives.insert(device_name.clone());

                        devices.push(StorageDevice {
                            name: device_name.clone(),
                            type_: medium_type.to_lowercase(),
                            size: capacity,
                            model: format!("{device_name} ({protocol})"),
                        });
                    }
                } else {
                    i += 1;
                }
            }
        }

        // If no drives found via system_profiler, fall back to diskutil
        if devices.is_empty() {
            if let Ok(diskutil_output) = Command::new("diskutil").args(&["list"]).output() {
                let diskutil_str = String::from_utf8_lossy(&diskutil_output.stdout);

                for line in diskutil_str.lines() {
                    if line.contains("(internal, physical)") {
                        // Extract disk identifier (e.g., "/dev/disk0")
                        if let Some(disk_path) = line.split_whitespace().next() {
                            if let Some(disk_id) = disk_path.strip_prefix("/dev/") {
                                // Get detailed info for this physical disk
                                if let Ok(info_output) =
                                    Command::new("diskutil").args(&["info", disk_id]).output()
                                {
                                    let info_str = String::from_utf8_lossy(&info_output.stdout);
                                    let mut device_name = "Unknown".to_string();
                                    let mut total_size = "Unknown".to_string();
                                    let mut device_location = "Unknown".to_string();
                                    let mut solid_state = false;

                                    for info_line in info_str.lines() {
                                        let trimmed = info_line.trim();
                                        if trimmed.starts_with("Device / Media Name:") {
                                            device_name = trimmed
                                                .split(":")
                                                .nth(1)
                                                .unwrap_or("Unknown")
                                                .trim()
                                                .to_string();
                                        } else if trimmed.starts_with("Total Size:")
                                            || trimmed.starts_with("Disk Size:")
                                        {
                                            // Extract size (e.g., "2.0 TB (2000398934016 Bytes)")
                                            total_size = trimmed
                                                .split(":")
                                                .nth(1)
                                                .unwrap_or("Unknown")
                                                .trim()
                                                .to_string();
                                        } else if trimmed.starts_with("Device Location:") {
                                            device_location = trimmed
                                                .split(":")
                                                .nth(1)
                                                .unwrap_or("Unknown")
                                                .trim()
                                                .to_string();
                                        } else if trimmed.starts_with("Solid State:") {
                                            solid_state = trimmed.contains("Yes");
                                        }
                                    }

                                    // Create device entry
                                    devices.push(StorageDevice {
                                        name: device_name.clone(),
                                        type_: if solid_state {
                                            "ssd".to_string()
                                        } else {
                                            "hdd".to_string()
                                        },
                                        size: total_size,
                                        model: format!("{device_name} ({device_location})"),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(StorageInfo { devices })
    }

    /// Collects storage information on Linux using lsblk
    fn collect_storage_info_linux() -> Result<StorageInfo, Box<dyn Error>> {
        let output = match Command::new("lsblk")
            .args(&["-J", "-o", "NAME,TYPE,SIZE,MODEL"])
            .output()
        {
            Ok(output) => output,
            Err(_) => {
                // lsblk not available, return empty storage info
                return Ok(StorageInfo {
                    devices: Vec::new(),
                });
            }
        };

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut devices = Vec::new();

        if let Some(blockdevices) = json["blockdevices"].as_array() {
            for device in blockdevices {
                if device["type"].as_str() == Some("disk") {
                    devices.push(StorageDevice {
                        name: device["name"].as_str().unwrap_or("").to_string(),
                        type_: device["type"].as_str().unwrap_or("").to_string(),
                        size: device["size"].as_str().unwrap_or("").to_string(),
                        model: device["model"].as_str().unwrap_or("").to_string(),
                    });
                }
            }
        }

        Ok(StorageInfo { devices })
    }

    /// Collects GPU information using platform-specific commands.
    fn collect_gpu_info() -> Result<GpuInfo, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::collect_gpu_info_macos()
        } else {
            Self::collect_gpu_info_linux()
        }
    }

    /// Collects GPU information on macOS using system_profiler
    fn collect_gpu_info_macos() -> Result<GpuInfo, Box<dyn Error>> {
        let mut devices = Vec::new();

        let output = match Command::new("system_profiler")
            .args(&["SPDisplaysDataType", "-detailLevel", "full"])
            .output()
        {
            Ok(output) => output,
            Err(_) => {
                return Ok(GpuInfo { devices });
            }
        };

        let output_str = String::from_utf8(output.stdout)?;
        let mut index = 0;
        let mut current_gpu = None;

        for line in output_str.lines() {
            let trimmed = line.trim();

            // Look for actual GPU entries (indented exactly 4 spaces and ending with colon)
            // Skip display entries and other sub-sections
            if line.starts_with("    ")
                && !line.starts_with("      ")
                && trimmed.ends_with(":")
                && !trimmed.starts_with("Displays:")
                && !trimmed.starts_with("Graphics/Displays:")
                && !trimmed.contains("Display")
            {
                // Save previous GPU
                if let Some(gpu) = current_gpu.take() {
                    devices.push(gpu);
                    index += 1;
                }

                let name = trimmed.trim_end_matches(':').to_string();

                current_gpu = Some(GpuDevice {
                    index,
                    name: name.clone(),
                    uuid: format!("macOS-GPU-{index}"),
                    memory: "Unknown".to_string(),
                    pci_id: if name.contains("Apple")
                        || name.contains("M1")
                        || name.contains("M2")
                        || name.contains("M3")
                        || name.contains("M4")
                    {
                        "Apple Fabric (Integrated)".to_string()
                    } else {
                        "Unknown".to_string()
                    },
                    vendor: if name.contains("Apple")
                        || name.contains("M1")
                        || name.contains("M2")
                        || name.contains("M3")
                        || name.contains("M4")
                    {
                        "Apple".to_string()
                    } else {
                        "Unknown".to_string()
                    },
                    numa_node: None,
                });
            } else if let Some(ref mut gpu) = current_gpu {
                // Parse GPU properties
                if trimmed.starts_with("Chipset Model:") {
                    // Update the name to be more descriptive if we have chipset model
                    gpu.name = trimmed
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown")
                        .trim()
                        .to_string();
                } else if trimmed.starts_with("VRAM (Total):") || trimmed.starts_with("VRAM:") {
                    gpu.memory = trimmed
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown")
                        .trim()
                        .to_string();
                } else if trimmed.starts_with("Vendor:") {
                    let vendor_str = trimmed.split(":").nth(1).unwrap_or("Unknown").trim();
                    // Extract vendor name from format like "Apple (0x106b)"
                    gpu.vendor = vendor_str
                        .split_whitespace()
                        .next()
                        .unwrap_or(vendor_str)
                        .to_string();
                } else if trimmed.starts_with("Device ID:") {
                    gpu.pci_id = trimmed
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown")
                        .trim()
                        .to_string();
                } else if trimmed.starts_with("Total Number of Cores:") {
                    // For Apple Silicon GPUs, they don't report VRAM separately
                    let cores = trimmed.split(":").nth(1).unwrap_or("0").trim();
                    gpu.memory = format!("Unified Memory ({cores} cores)");
                } else if trimmed.starts_with("Metal Support:") {
                    // Capture Metal support version
                    let metal_version = trimmed.split(":").nth(1).unwrap_or("").trim();
                    if !metal_version.is_empty() && gpu.name.contains("Apple") {
                        gpu.name = format!("{} ({metal_version})", gpu.name);
                    }
                }
            }
        }

        // Save last GPU
        if let Some(gpu) = current_gpu {
            devices.push(gpu);
        }

        // For Apple Silicon, if we didn't find a discrete GPU, add the integrated one
        if devices.is_empty() {
            // Check if we can get chip info from hardware data
            if let Ok(hw_output) = Command::new("system_profiler")
                .args(&["SPHardwareDataType", "-detailLevel", "basic"])
                .output()
            {
                let hw_str = String::from_utf8_lossy(&hw_output.stdout);
                for line in hw_str.lines() {
                    if line.trim().starts_with("Chip:") {
                        let chip_name = line.split(":").nth(1).unwrap_or("Unknown").trim();
                        devices.push(GpuDevice {
                            index: 0,
                            name: format!("{chip_name} GPU"),
                            uuid: "macOS-integrated-GPU".to_string(),
                            memory: "Unified Memory".to_string(),
                            pci_id: "Integrated".to_string(),
                            vendor: "Apple".to_string(),
                            numa_node: None,
                        });
                        break;
                    }
                }
            }
        }

        Ok(GpuInfo { devices })
    }

    /// Collects GPU information on Linux using nvidia-smi
    fn collect_gpu_info_linux() -> Result<GpuInfo, Box<dyn Error>> {
        let output = Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=index,name,uuid,memory.total,pci.bus_id",
                "--format=csv,noheader",
            ])
            .output();

        let mut devices = Vec::new();

        if let Ok(output) = output {
            let output_str = String::from_utf8(output.stdout)?;

            for line in output_str.lines() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 5 {
                    let pci_addr = parts[4].trim();
                    let (vendor, _, pci_id) = Self::get_pci_info(pci_addr).unwrap_or((
                        "NVIDIA".to_string(),
                        "Unknown".to_string(),
                        "Unknown".to_string(),
                    ));

                    devices.push(GpuDevice {
                        index: parts[0].trim().parse()?,
                        name: parts[1].trim().to_string(),
                        uuid: parts[2].trim().to_string(),
                        memory: parts[3].trim().to_string(),
                        pci_id,
                        vendor,
                        numa_node: Self::get_numa_node(pci_addr),
                    });
                }
            }
        }

        Ok(GpuInfo { devices })
    }

    /// Collects network information, including Infiniband if available.
    fn collect_network_info() -> Result<NetworkInfo, Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            Self::collect_network_info_macos()
        } else {
            Self::collect_network_info_linux()
        }
    }

    /// Collects network information on macOS using system_profiler and ifconfig
    fn collect_network_info_macos() -> Result<NetworkInfo, Box<dyn Error>> {
        let mut interfaces = Vec::new();

        // Get ifconfig output for actual runtime interface information
        let ifconfig_output = Command::new("ifconfig").output();
        let mut ifconfig_data = std::collections::HashMap::new();

        if let Ok(output) = ifconfig_output {
            let output_str = String::from_utf8(output.stdout).unwrap_or_default();
            let mut current_if = String::new();

            for line in output_str.lines() {
                if !line.starts_with('\t') && !line.starts_with(' ') && line.contains(':') {
                    // New interface
                    current_if = line.split(':').next().unwrap_or("").to_string();
                    ifconfig_data.insert(current_if.clone(), std::collections::HashMap::new());
                } else if !current_if.is_empty() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("ether ") {
                        if let Some(mac) = trimmed.split_whitespace().nth(1) {
                            ifconfig_data
                                .get_mut(&current_if)
                                .unwrap()
                                .insert("mac".to_string(), mac.to_string());
                        }
                    } else if trimmed.starts_with("inet ") {
                        if let Some(ip) = trimmed.split_whitespace().nth(1) {
                            ifconfig_data
                                .get_mut(&current_if)
                                .unwrap()
                                .insert("ip".to_string(), ip.to_string());
                        }
                    } else if trimmed.contains("status: active") {
                        ifconfig_data
                            .get_mut(&current_if)
                            .unwrap()
                            .insert("status".to_string(), "active".to_string());
                    }
                }
            }
        }

        // Use system_profiler to get network interface details
        let output = match Command::new("system_profiler")
            .args(&["SPNetworkDataType", "-detailLevel", "full"])
            .output()
        {
            Ok(output) => output,
            Err(_) => {
                // Fallback: create interfaces from ifconfig data only
                for (name, data) in ifconfig_data {
                    let interface_type = Self::classify_macos_interface_type(&name);
                    let vendor = if name.starts_with("en") || name.starts_with("bridge") {
                        "Apple"
                    } else {
                        "Unknown"
                    };
                    let model = Self::get_macos_interface_model(&interface_type);
                    let pci_id = if vendor == "Apple" {
                        "Apple Fabric (Integrated)"
                    } else {
                        "Unknown"
                    };

                    interfaces.push(NetworkInterface {
                        name: name.clone(),
                        mac: data.get("mac").cloned().unwrap_or("Unknown".to_string()),
                        ip: data.get("ip").cloned().unwrap_or("Unknown".to_string()),
                        prefix: data.get("prefix").cloned().unwrap_or("Unknown".to_string()),
                        speed: Self::estimate_macos_interface_speed(&name, &interface_type),
                        type_: interface_type,
                        vendor: vendor.to_string(),
                        model: model.to_string(),
                        pci_id: pci_id.to_string(),
                        numa_node: None,
                    });
                }

                return Ok(NetworkInfo {
                    interfaces,
                    infiniband: None,
                });
            }
        };

        let output_str = String::from_utf8(output.stdout)?;
        let mut current_interface = None;

        for line in output_str.lines() {
            let trimmed = line.trim();

            if trimmed.ends_with(":")
                && !line.starts_with("      ")
                && !line.starts_with("        ")
            {
                // Save previous interface
                if let Some(interface) = current_interface.take() {
                    interfaces.push(interface);
                }

                let name = trimmed.trim_end_matches(':');

                // Skip the main "Network" section header
                if name == "Network" {
                    continue;
                }
                let interface_type = Self::classify_macos_interface_type(name);
                let vendor = if name.starts_with("en") || name.starts_with("bridge") {
                    "Apple"
                } else {
                    "Unknown"
                };
                let model = Self::get_macos_interface_model(&interface_type);
                let pci_id = if vendor == "Apple" {
                    "Apple Fabric (Integrated)"
                } else {
                    "Unknown"
                };

                // Get runtime data from ifconfig
                let ifconfig_info = ifconfig_data.get(name).cloned().unwrap_or_default();

                current_interface = Some(NetworkInterface {
                    name: name.to_string(),
                    mac: ifconfig_info
                        .get("mac")
                        .cloned()
                        .unwrap_or("Unknown".to_string()),
                    ip: ifconfig_info
                        .get("ip")
                        .cloned()
                        .unwrap_or("Unknown".to_string()),
                    prefix: ifconfig_info
                        .get("prefix")
                        .cloned()
                        .unwrap_or("Unknown".to_string()),
                    speed: Self::estimate_macos_interface_speed(name, &interface_type),
                    type_: interface_type,
                    vendor: vendor.to_string(),
                    model: model.to_string(),
                    pci_id: pci_id.to_string(),
                    numa_node: None,
                });
            } else if let Some(ref mut interface) = current_interface {
                if trimmed.starts_with("Type:") {
                    let sys_type = trimmed
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown")
                        .trim()
                        .to_string();
                    if sys_type != "Unknown" {
                        interface.type_ = sys_type;
                    }
                } else if trimmed.starts_with("Hardware:") {
                    let hardware = trimmed
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown")
                        .trim()
                        .to_string();
                    if hardware != "Unknown" {
                        interface.model = hardware.clone();
                    }

                    // Set vendor based on interface types - Apple is the manufacturer for built-in interfaces
                    if interface.type_.contains("AirPort") || hardware.contains("AirPort") {
                        interface.vendor = "Apple".to_string();
                        interface.model = "Wi-Fi 802.11 a/b/g/n/ac/ax".to_string();
                        interface.pci_id = "Apple Fabric (Integrated)".to_string();
                    } else if interface.type_.contains("Ethernet") || hardware.contains("Ethernet")
                    {
                        interface.vendor = "Apple".to_string();
                        interface.model = "Ethernet".to_string();
                        interface.pci_id = "Apple Fabric (Integrated)".to_string();
                    } else if interface.name.starts_with("en") && interface.vendor == "Unknown" {
                        // Apple built-in interfaces
                        interface.vendor = "Apple".to_string();
                        interface.pci_id = "Apple Fabric (Integrated)".to_string();
                        if hardware.contains("Ethernet") || interface.type_.contains("Ethernet") {
                            interface.model = "Ethernet".to_string();
                        }
                    } else if interface.name.starts_with("bridge") {
                        interface.vendor = "Apple".to_string();
                        interface.model = "Bridge".to_string();
                        interface.pci_id = "Apple Fabric (Integrated)".to_string();
                    }
                } else if trimmed.starts_with("BSD Device Name:") {
                    let bsd_name = trimmed.split(":").nth(1).unwrap_or("").trim();
                    if !bsd_name.is_empty() {
                        interface.name = bsd_name.to_string();
                    }
                } else if trimmed.starts_with("MAC Address:") {
                    interface.mac = trimmed
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown")
                        .trim()
                        .to_string();
                } else if trimmed.starts_with("IPv4 Addresses:") {
                    interface.ip = trimmed
                        .split(":")
                        .nth(1)
                        .unwrap_or("Unknown")
                        .trim()
                        .to_string();
                }
            }
        }

        // Save last interface
        if let Some(interface) = current_interface {
            interfaces.push(interface);
        }

        // Get additional IP information and speeds using ifconfig and system_profiler for active interfaces
        for interface in &mut interfaces {
            if let Ok(output) = Command::new("ifconfig").arg(&interface.name).output() {
                let ifconfig_str = String::from_utf8_lossy(&output.stdout);
                for line in ifconfig_str.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("inet ") && !trimmed.contains("127.0.0.1") {
                        if let Some(ip) = trimmed.split_whitespace().nth(1) {
                            interface.ip = ip.to_string();
                            break;
                        }
                    }
                }
            }
        }

        // Get Wi-Fi speeds from AirPort data
        if let Ok(airport_output) = Command::new("system_profiler")
            .args(&["SPAirPortDataType"])
            .output()
        {
            let airport_str = String::from_utf8_lossy(&airport_output.stdout);
            let mut current_interface = "";

            for line in airport_str.lines() {
                let trimmed = line.trim();
                if trimmed.ends_with(":") && !trimmed.starts_with(" ") {
                    current_interface = trimmed.trim_end_matches(':');
                } else if trimmed.starts_with("Transmit Rate:") && !current_interface.is_empty() {
                    if let Some(rate) = trimmed.split(":").nth(1) {
                        let rate_mbps = format!("{} Mbps", rate.trim());
                        // Find the interface and update its speed
                        for interface in &mut interfaces {
                            if interface.name == current_interface
                                || (current_interface == "en0" && interface.type_ == "AirPort")
                            {
                                interface.speed = Some(rate_mbps.clone());
                                break;
                            }
                        }
                    }
                } else if trimmed.starts_with("Supported PHY Modes:")
                    && !current_interface.is_empty()
                {
                    if let Some(modes) = trimmed.split(":").nth(1) {
                        // Update the interface model with PHY modes
                        for interface in &mut interfaces {
                            if interface.name == current_interface
                                || (current_interface == "en0" && interface.type_ == "AirPort")
                            {
                                interface.model = format!("Wi-Fi {}", modes.trim());
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Set reasonable defaults for known interface types
        for interface in &mut interfaces {
            if interface.speed.is_none() {
                interface.speed = match interface.type_.as_str() {
                    "Ethernet" => Some("1000 Mbps".to_string()), // Default Gigabit Ethernet
                    "AirPort" => Some("1200 Mbps".to_string()),  // Default Wi-Fi 6
                    _ => None,
                };
            }
        }

        Ok(NetworkInfo {
            interfaces,
            infiniband: None, // Infiniband not typically available on macOS
        })
    }

    /// Classify macOS interface type based on name
    fn classify_macos_interface_type(name: &str) -> String {
        if name.starts_with("en") && name != "en0" {
            "Ethernet".to_string()
        } else if name == "en0" {
            "AirPort".to_string() // Primary interface on macOS is usually Wi-Fi
        } else if name.starts_with("bridge") {
            "Ethernet".to_string()
        } else if name.starts_with("utun") {
            "VPN (io.tailscale.ipn.macos)".to_string()
        } else if name.starts_with("lo") {
            "Loopback".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Get macOS interface model based on type
    fn get_macos_interface_model(interface_type: &str) -> String {
        match interface_type {
            "AirPort" => "Wi-Fi 802.11 a/b/g/n/ac/ax".to_string(),
            "Ethernet" => "Ethernet".to_string(),
            "VPN (io.tailscale.ipn.macos)" => "Unknown".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Estimate macOS interface speed based on type and name
    fn estimate_macos_interface_speed(name: &str, interface_type: &str) -> Option<String> {
        match interface_type {
            "AirPort" => Some("1200 Mbps".to_string()), // Wi-Fi 6 typical
            "Ethernet" if name.starts_with("en") => Some("1000 Mbps".to_string()),
            _ => None,
        }
    }

    /// Collects network information on Linux using ip command
    fn collect_network_info_linux() -> Result<NetworkInfo, Box<dyn Error>> {
        let mut interfaces = Vec::new();
        let output = match Command::new("ip").args(&["-j", "addr", "show"]).output() {
            Ok(output) => output,
            Err(_) => {
                // ip command not available, return empty network info
                return Ok(NetworkInfo {
                    interfaces: Vec::new(),
                    infiniband: None,
                });
            }
        };
        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

        if let Some(ifaces) = json.as_array() {
            for iface in ifaces {
                if let Some(name) = iface["ifname"].as_str() {
                    // Skip loopback
                    if name == "lo" {
                        continue;
                    }

                    let mac = iface["address"].as_str().unwrap_or("").to_string();
                    let mut ip = String::new();
                    let mut prefix: String = String::new();

                    // Get IP address
                    if let Some(addr_info) = iface["addr_info"].as_array() {
                        for addr in addr_info {
                            if addr["family"].as_str() == Some("inet") {
                                ip = addr["local"].as_str().unwrap_or("").to_string();
                                prefix = addr["prefixlen"].as_str().unwrap_or("").to_string();
                                break;
                            }
                        }
                    }

                    // Get PCI information and speed
                    let mut vendor = String::new();
                    let mut model = String::new();
                    let mut pci_id = String::new();
                    let mut numa_node = None;

                    if let Ok(pci_addr) =
                        std::fs::read_link(format!("/sys/class/net/{name}/device"))
                    {
                        if let Some(addr_str) = pci_addr.file_name().and_then(|n| n.to_str()) {
                            if let Ok((v, m, p)) = Self::get_pci_info(addr_str) {
                                vendor = v;
                                model = m;
                                pci_id = p;
                                numa_node = Self::get_numa_node(addr_str);
                            }
                        }
                    }

                    // Get speed using ethtool if available
                    let speed =
                        Command::new("ethtool")
                            .arg(name)
                            .output()
                            .ok()
                            .and_then(|output| {
                                String::from_utf8(output.stdout)
                                    .ok()
                                    .and_then(|output_str| {
                                        NETWORK_SPEED_RE
                                            .captures(&output_str)
                                            .map(|cap| cap[1].to_string())
                                    })
                            });

                    interfaces.push(NetworkInterface {
                        name: name.to_string(),
                        mac,
                        ip,
                        prefix,
                        speed,
                        type_: iface["link_type"].as_str().unwrap_or("").to_string(),
                        vendor,
                        model,
                        pci_id,
                        numa_node,
                    });
                }
            }
        }

        Ok(NetworkInfo {
            interfaces,
            infiniband: Self::collect_infiniband_info()?,
        })
    }

    /// Collects Infiniband information by parsing 'ibstat' output.
    fn collect_infiniband_info() -> Result<Option<InfinibandInfo>, Box<dyn Error>> {
        let output = Command::new("ibstat").output();

        match output {
            Ok(output) => {
                let output_str = String::from_utf8(output.stdout)?;
                let mut interfaces = Vec::new();

                // Parse ibstat output.
                let re = Regex::new(
                    r"CA '([^']+)'\n(?:\s+.+\n)*?\s+Port (\d+):\n(?:\s+.+\n)*?\s+State:\s+(\S+)\s+(?:\S+)\n(?:\s+.+\n)*?\s+Rate:\s+(\S+)",
                )?;

                for cap in re.captures_iter(&output_str) {
                    interfaces.push(IbInterface {
                        name: cap[1].to_string(),
                        port: cap[2].parse()?,
                        state: cap[3].to_string(),
                        rate: cap[4].to_string(),
                    });
                }

                if interfaces.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(InfinibandInfo { interfaces }))
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Collects BMC IP and MAC addresses by parsing 'ipmitool' output.
    fn collect_ipmi_info() -> Result<(Option<String>, Option<String>), Box<dyn Error>> {
        if cfg!(target_os = "macos") {
            // IPMI is not typically available on macOS
            return Ok((None, None));
        }

        let output = Command::new("ipmitool").args(&["lan", "print"]).output();

        match output {
            Ok(output) => {
                let output_str = String::from_utf8(output.stdout)?;
                let ip_re = Regex::new(r"IP Address\s+:\s+(.+)")?;
                let mac_re = Regex::new(r"MAC Address\s+:\s+(.+)")?;

                let ip = ip_re
                    .captures(&output_str)
                    .map(|cap| cap[1].trim().to_string());
                let mac = mac_re
                    .captures(&output_str)
                    .map(|cap| cap[1].trim().to_string());

                Ok((ip, mac))
            }
            Err(_) => Ok((None, None)),
        }
    }
}

// Legacy compatibility will be handled by keeping the old ServerInfo struct
// and implementing From traits for conversion between old and new types

// Convenience factory function for creating a hardware reporting service
/// Create a hardware reporting service with platform-appropriate adapters
///
/// This function sets up the complete dependency injection container with
/// platform-specific implementations for the current operating system.
///
/// # Returns
/// * `Ok(Arc<dyn HardwareReportingService>)` - Configured service ready to use
/// * `Err(Box<dyn Error>)` - Error occurred during service creation
///
/// # Example
///
/// ```rust,no_run
/// use hardware_report::{create_service, ReportConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let service = create_service(None).await?;
///     let report = service.generate_report(ReportConfig::default()).await?;
///     println!("Generated report for {}", report.hostname);
///     Ok(())
/// }
/// ```
/// Create a hardware reporting service with platform-appropriate adapters
///
/// This function sets up the complete dependency injection container with
/// platform-specific implementations for the current operating system.
///
/// # Arguments
/// * `config` - Optional report configuration (uses defaults if None)
///
/// # Returns
/// * `Ok(Arc<dyn HardwareReportingService>)` - Configured service ready to use
/// * `Err(Box<dyn Error>)` - Error occurred during service creation
///
/// # Example
///
/// ```rust,no_run
/// use hardware_report::{create_service, ReportConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let service = create_service(None).await?;
///     let report = service.generate_report(ReportConfig::default()).await?;
///     println!("Generated report for {}", report.hostname);
///     Ok(())
/// }
/// ```
pub async fn create_service(
    config: Option<ReportConfig>,
) -> Result<std::sync::Arc<dyn HardwareReportingService>, Box<dyn Error>> {
    let container = ServiceContainer::with_defaults();
    container.create_hardware_reporting_service(config)
}

/// Create a hardware reporting service with custom container configuration
///
/// # Arguments
/// * `container_config` - Container configuration for customizing behavior
/// * `report_config` - Optional report configuration
///
/// # Returns
/// * Configured hardware reporting service
pub async fn create_service_with_config(
    container_config: ContainerConfig,
    report_config: Option<ReportConfig>,
) -> Result<std::sync::Arc<dyn HardwareReportingService>, Box<dyn Error>> {
    let container = ServiceContainer::new(container_config);
    container.create_hardware_reporting_service(report_config)
}

/// Validate system dependencies and privileges
///
/// # Returns
/// * `Ok((missing_deps, has_privileges))` - Missing dependencies and privilege status
/// * `Err(Box<dyn Error>)` - Error occurred during validation
///
/// # Example
///
/// ```rust,no_run
/// use hardware_report::validate_system;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let (missing, has_privs) = validate_system().await?;
///     
///     if !missing.is_empty() {
///         println!("Missing dependencies: {:?}", missing);
///     }
///     
///     if !has_privs {
///         println!("Warning: Running without elevated privileges");
///     }
///     
///     Ok(())
/// }
/// ```
pub async fn validate_system() -> Result<(Vec<String>, bool), Box<dyn Error>> {
    let container = ServiceContainer::with_defaults();
    let missing_deps = container.validate_dependencies().await?;
    let has_privileges = container.check_privileges().await?;
    Ok((missing_deps, has_privileges))
}
