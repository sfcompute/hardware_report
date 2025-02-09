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

/*
This Rust program collects various hardware and network information from the local server
and serializes it into a TOML configuration file.

It gathers information such as:
- Hostname
- IP addresses of network interfaces
- BMC (Baseboard Management Controller) IP and MAC addresses
- CPU, memory, storage, and GPU details
- Network interface details, including Infiniband if present

The collected data is written to `server_config.toml`.
*/

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
        let required_packages = vec![
            ("numactl", "NUMA topology information"),
            ("lspci", "PCI device information"),
            ("ethtool", "Network interface information"),
            ("dmidecode", "System hardware information"),
        ];

        let mut missing_packages = Vec::new();

        // Check which packages are missing
        for (package, purpose) in &required_packages {
            let status = Command::new("which").arg(package).output()?;

            if !status.status.success() {
                missing_packages.push(*package);
                eprintln!("Missing {}: required for {}", package, purpose);
            }
        }

        if !missing_packages.is_empty() {
            Self::install_missing_packages()?;
        }

        Ok(missing_packages)
    }
    /// Gets motherboard information using dmidecode
    fn get_motherboard_info() -> Result<MotherboardInfo, Box<dyn Error>> {
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
        let size_str = size.replace(" ", "");
        let re = Regex::new(r"(\d+(?:\.\d+)?)(B|K|M|G|T)")?;

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

            Ok((value * multiplier as f64) as u64)
        } else {
            Err("Invalid storage size format".into())
        }
    }

    fn install_missing_packages() -> Result<(), Box<dyn Error>> {
        let required_packages = vec![
            ("numactl", "NUMA topology information"),
            ("lspci", "PCI device information"),
            ("ethtool", "Network interface information"),
            ("dmidecode", "System hardware information"),
        ];

        let mut missing_packages = Vec::new();

        // Check which packages are missing
        for (package, purpose) in &required_packages {
            let status = Command::new("which").arg(package).output()?;

            if !status.status.success() {
                missing_packages.push(*package);
                eprintln!("Missing {}: required for {}", package, purpose);
            }
        }

        if !missing_packages.is_empty() {
            eprintln!("\nSome required packages are missing. Attempting to install them...");

            // Detect package manager
            let mut package_manager = "apt-get";
            let status = Command::new("which").arg("dnf").output();
            if let Ok(output) = status {
                if output.status.success() {
                    package_manager = "dnf";
                }
            }

            // Install missing packages
            let install_command = Command::new("sudo")
                .arg(package_manager)
                .arg("install")
                .arg("-y")
                .args(&missing_packages)
                .status()?;

            if !install_command.success() {
                return Err("Failed to install required system packages. Please install them manually and try again.".into());
            }
        }

        Ok(())
    }

    /// Gets hostname of the server
    fn get_hostname() -> Result<String, Box<dyn Error>> {
        let output = Command::new("hostname").args(&["-f"]).output()?;
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Gets PCI information for a device
    fn get_pci_info(pci_addr: &str) -> Result<(String, String, String), Box<dyn Error>> {
        // Run lspci with verbose output and machine-readable format
        let output = Command::new("lspci")
            .args(&["-vmm", "-s", pci_addr])
            .output()?;

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
        let id_output = Command::new("lspci")
            .args(&["-n", "-s", pci_addr])
            .output()?;

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

        let pci_id = format!("{}:{}", vendor_id, device_id);
        Ok((vendor, device, pci_id))
    }

    /// Gets NUMA node for a PCI device
    fn get_numa_node(pci_addr: &str) -> Option<i32> {
        if let Ok(path) = std::fs::read_link(format!("/sys/bus/pci/devices/{}/numa_node", pci_addr))
        {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(node) = content.trim().parse() {
                    return Some(node);
                }
            }
        }
        None
    }

    fn collect_numa_topology() -> Result<HashMap<String, NumaNode>, Box<dyn Error>> {
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
        let output = Command::new("lscpu").args(&["-p=cpu,node"]).output()?;

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
        let output = Command::new("ip").args(&["-j", "addr"]).output()?;
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

    /// Gets system UUID and serial from dmidecode
    fn get_system_info() -> Result<SystemInfo, Box<dyn Error>> {
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
        // Check dependencies first
        let missing_packages = Self::check_dependencies()?;

        // If any essential packages are missing, return an error
        if !missing_packages.is_empty() {
            return Err(
                "Missing required system packages. Please install them and try again.".into(),
            );
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
        let hardware = Self::collect_hardware_info()?;
        let network = Self::collect_network_info()?;
        let system_info = Self::get_system_info()?;
        let (bmc_ip, bmc_mac) = Self::collect_ipmi_info()?;
        let os_ip = Self::collect_ip_addresses()?;

        let summary = Self::generate_summary(&hardware, &network, &system_info)?;

        Ok(ServerInfo {
            summary,
            hostname,
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
        let output = Command::new("df")
            .args(["-h", "--output=source,fstype,size,used,avail,target"])
            .output()?;

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

    /// Gets BIOS information using dmidecode with minimal output
    fn get_bios_info() -> Result<BiosInfo, Box<dyn Error>> {
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

    /// Gets chassis information using dmidecode with minimal output
    fn get_chassis_info() -> Result<ChassisInfo, Box<dyn Error>> {
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

        Err(format!("Could not find key: {}", key).into())
    }

    /// Gets detailed CPU topology information
    fn get_cpu_topology() -> Result<CpuTopology, Box<dyn Error>> {
        let output = Command::new("lscpu").args(&["-J"]).output()?;

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

    /// Collects CPU information by parsing 'lscpu' output.
    fn collect_cpu_info() -> Result<CpuInfo, Box<dyn Error>> {
        // Use 'lscpu -J' for JSON output to ensure reliable parsing.
        let output = Command::new("lscpu").args(&["-J"]).output()?;
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
            speed: format!("{} MHz", speed),
        })
    }

    /// Collects memory information by parsing 'dmidecode' output.
    fn collect_memory_info() -> Result<MemoryInfo, Box<dyn Error>> {
        let output = Command::new("dmidecode").args(&["-t", "memory"]).output()?;
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

    /// Retrieves the total memory size using 'free -h'.
    fn get_total_memory() -> Result<String, Box<dyn Error>> {
        let output = Command::new("free").arg("-h").output()?;
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

        Ok(MemoryModule {
            size,
            type_,
            speed,
            location,
        })
    }

    /// Collects storage information by parsing 'lsblk' output.
    fn collect_storage_info() -> Result<StorageInfo, Box<dyn Error>> {
        let output = Command::new("lsblk")
            .args(&["-J", "-o", "NAME,TYPE,SIZE,MODEL"])
            .output()?;

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

    /// Collects GPU information by parsing 'nvidia-smi' output.
    fn collect_gpu_info() -> Result<GpuInfo, Box<dyn Error>> {
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
                        memory: format!("{}", parts[3].trim()),
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
        let mut interfaces = Vec::new();
        let output = Command::new("ip").args(&["-j", "addr", "show"]).output()?;
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

                    // Get IP address
                    if let Some(addr_info) = iface["addr_info"].as_array() {
                        for addr in addr_info {
                            if addr["family"].as_str() == Some("inet") {
                                ip = addr["local"].as_str().unwrap_or("").to_string();
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
                        std::fs::read_link(format!("/sys/class/net/{}/device", name))
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

                    // Get speed using ethtool
                    let speed = match Command::new("ethtool").arg(name).output() {
                        Ok(output) => {
                            let output_str = String::from_utf8(output.stdout)?;
                            NETWORK_SPEED_RE
                                .captures(&output_str)
                                .map(|cap| cap[1].to_string())
                        }
                        Err(_) => None,
                    };

                    interfaces.push(NetworkInterface {
                        name: name.to_string(),
                        mac,
                        ip,
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
