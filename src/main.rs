/*!
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

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::process::Command;
use toml;

/// CPU topology information
#[derive(Debug, Serialize, Deserialize)]
struct CpuTopology {
    total_cores: u32,
    total_threads: u32,
    sockets: u32,
    cores_per_socket: u32,
    threads_per_core: u32,
    numa_nodes: u32,
    cpu_model: String,
}

/// Summary of key system components
#[derive(Debug, Serialize, Deserialize)]
struct SystemSummary {
    /// Total system memory capacity
    total_memory: String,
    /// Memory speed and type
    memory_config: String,
    /// Total storage capacity
    total_storage: String,
    /// Available filesystems
    filesystems: Vec<String>,
    /// BIOS information
    bios: BiosInfo,
    /// System chassis information
    chassis: ChassisInfo,
    /// Total number of GPUs
    total_gpus: usize,
    /// Total number of network interfaces
    total_nics: usize,
    /// NUMA topology information
    numa_topology: HashMap<String, NumaNode>, 
}

/// BIOS information
#[derive(Debug, Serialize, Deserialize)]
struct BiosInfo {
    vendor: String,
    version: String,
    release_date: String,
    firmware_version: String,
}

/// Chassis information
#[derive(Debug, Serialize, Deserialize)]
struct ChassisInfo {
    manufacturer: String,
    type_: String,
    serial: String,
}

/// Represents the overall server information
#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    /// System summary
    summary: SystemSummary,
    /// Other fields remain the same
    hostname: String,
    os_ip: HashMap<String, String>,
    bmc_ip: Option<String>,
    bmc_mac: Option<String>,
    hardware: HardwareInfo,
    network: NetworkInfo,
}

/// Contains detailed hardware information.
#[derive(Debug, Serialize, Deserialize)]
struct HardwareInfo {
    /// CPU information.
    cpu: CpuInfo,
    /// Memory information.
    memory: MemoryInfo,
    /// Storage information.
    storage: StorageInfo,
    /// GPU information.
    gpus: GpuInfo,
}

/// Represents CPU information.
#[derive(Debug, Serialize, Deserialize)]
struct CpuInfo {
    /// CPU model name.
    model: String,
    /// Number of cores per socket.
    cores: u32,
    /// Number of threads per core.
    threads: u32,
    /// Number of sockets.
    sockets: u32,
    /// CPU speed in MHz.
    speed: String,
}

/// Represents memory information.
#[derive(Debug, Serialize, Deserialize)]
struct MemoryInfo {
    /// Total memory size.
    total: String,
    /// Memory type (e.g., DDR4).
    type_: String,
    /// Memory speed.
    speed: String,
    /// Individual memory modules.
    modules: Vec<MemoryModule>,
}

/// Represents a memory module.
#[derive(Debug, Serialize, Deserialize)]
struct MemoryModule {
    /// Size of the memory module.
    size: String,
    /// Type of the memory module.
    type_: String,
    /// Speed of the memory module.
    speed: String,
    /// Physical location of the memory module.
    location: String,
}

/// Represents storage information.
#[derive(Debug, Serialize, Deserialize)]
struct StorageInfo {
    /// List of storage devices.
    devices: Vec<StorageDevice>,
}

/// Represents a storage device.
#[derive(Debug, Serialize, Deserialize)]
struct StorageDevice {
    /// Device name.
    name: String,
    /// Device type (e.g., disk).
    type_: String,
    /// Device size.
    size: String,
    /// Device model.
    model: String,
}

/// Represents GPU information.
#[derive(Debug, Serialize, Deserialize)]
struct GpuInfo {
    /// List of GPU devices.
    devices: Vec<GpuDevice>,
}

/// Represents a GPU device.
#[derive(Debug, Serialize, Deserialize)]
struct GpuDevice {
    /// GPU index
    index: u32,
    /// GPU name
    name: String,
    /// GPU UUID
    uuid: String,
    /// Total GPU memory
    memory: String,
    /// PCI ID (vendor:device)
    pci_id: String,
    /// Vendor name
    vendor: String,
    /// NUMA node
    numa_node: Option<i32>, 
}

/// Represents a NUMA node
#[derive(Debug, Serialize, Deserialize)]
struct NumaNode {
    /// Node ID
    id: i32,
    /// CPU list
    cpus: Vec<u32>,
    /// Memory size
    memory: String,
    /// Devices attached to this node
    devices: Vec<NumaDevice>,
    /// distances to other nodse (node_id _> distance)
    distances: HashMap<String, u32>,
}

/// Represents a device attached to a NUMA node
#[derive(Debug, Serialize, Deserialize)]
struct NumaDevice {
    /// Device type (GPU, NIC, etc.)
    type_: String,
    /// PCI ID
    pci_id: String,
    /// Device name
    name: String,
}


/// Represents network information.
#[derive(Debug, Serialize, Deserialize)]
struct NetworkInfo {
    /// List of network interfaces.
    interfaces: Vec<NetworkInterface>,
    /// Infiniband information, if available.
    infiniband: Option<InfinibandInfo>,
}

/// Represents a network interface.
#[derive(Debug, Serialize, Deserialize)]
struct NetworkInterface {
    /// Interface name.
    name: String,
    /// MAC address.
    mac: String,
    /// IP address.
    ip: String,
    /// Interface speed.
    speed: Option<String>,
    /// Interface type.
    type_: String,
    vendor: String,
    model: String,
    pci_id: String,
    numa_node: Option<i32>,
    
}

/// Represents Infiniband information.
#[derive(Debug, Serialize, Deserialize)]
struct InfinibandInfo {
    /// List of Infiniband interfaces.
    interfaces: Vec<IbInterface>,
}

/// Represents an Infiniband interface.
#[derive(Debug, Serialize, Deserialize)]
struct IbInterface {
    /// Interface name.
    name: String,
    /// Port number.
    port: u32,
    /// Interface state.
    state: String,
    /// Interface rate.
    rate: String,
}

struct NumaInfo {
    nodes: Vec<NumaNode>
}

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
            let status = Command::new("which")
                .arg(package)
                .output()?;

            if !status.status.success() {
                missing_packages.push(*package);
                eprintln!("Missing {}: required for {}", package, purpose);
            }
        }

        if !missing_packages.is_empty() {
            eprintln!("\nSome required packages are missing. Please install them using your package manager:");
            eprintln!("\nFor Debian/Ubuntu:");
            eprintln!("  apt-get install {}", missing_packages.join(" "));
            eprintln!("\nFor RHEL/CentOS/Fedora:");
            eprintln!("  dnf install {}", missing_packages.join(" "));
            eprintln!("\nThen run this program again.\n");
        }

        Ok(missing_packages)
    }
    /// Gets hostname of the server
    fn get_hostname() -> Result<String, Box<dyn Error>> {
        let output = Command::new("hostname").output()?;
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
                    "SVendor" => if vendor.is_empty() { vendor = value.to_string() },
                    "SDevice" => if device.is_empty() { device = value.to_string() },
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
        if let Ok(path) = std::fs::read_link(format!("/sys/bus/pci/devices/{}/numa_node", pci_addr)) {
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
        let output = Command::new("numactl")
            .args(&["--hardware"])
            .output()?;

        let output_str = String::from_utf8(output.stdout)?;

        for line in output_str.lines() {
            if line.starts_with("node ") && line.contains("size:") {
                // Parse node and memory information
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(id) = parts[1].parse::<i32>() {
                        let memory = format!("{} {}", parts[3], parts[4]);

                        // Create new node entry
                        nodes.insert(id.to_string(), NumaNode {
                            id,
                            memory,
                            cpus: Vec::new(),
                            distances: HashMap::new(),
                            devices: Vec::new(),
                        });
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
        let output = Command::new("lscpu")
            .args(&["-p=cpu,node"])
            .output()?;

        let output_str = String::from_utf8(output.stdout)?;
        for line in output_str.lines() {
            if line.starts_with('#') { continue; }
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

    /// Collects NUMA topology information
    fn collect_numa_info() -> Result<NumaInfo, Box<dyn Error>> {
        let mut nodes = Vec::new();

        // Read NUMA node information using numactl
        let output = Command::new("numactl")
            .args(&["--hardware"])
            .output()?;

        let output_str = String::from_utf8(output.stdout)?;
        let mut current_node: Option<NumaNode> = None;

        for line in output_str.lines() {
            if line.starts_with("node ") && line.contains("size:") {
                // Parse node ID and memory size
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(id) = parts[1].parse::<i32>() {
                        let memory = format!("{} {}", parts[3], parts[4]);
                        if let Some(node) = current_node {
                            nodes.push(node);
                        }
                        current_node = Some(NumaNode {
                            id,
                            memory,
                            cpus: Vec::new(),
                            devices: Vec::new(),
                            distances: HashMap::new(),
                        });
                    }
                }
            } else if line.contains("node distances:") {
                if let Some(node) = current_node {
                    nodes.push(node);
                    break;
                }
            }
        }

        // Collect CPU information for each node
        for node in &mut nodes {
            let output = Command::new("lscpu")
                .args(&["-p=cpu,node"])
                .output()?;

            let output_str = String::from_utf8(output.stdout)?;
            for line in output_str.lines() {
                if line.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    if let (Ok(cpu), Ok(numa_node)) = (parts[0].parse::<u32>(), parts[1].parse::<i32>()) {
                        if numa_node == node.id {
                            node.cpus.push(cpu);
                        }
                    }
                }
            }
        }

        Ok(NumaInfo { nodes })
    }


    /// Collects all server information
    fn collect() -> Result<Self, Box<dyn Error>> {
        // Check dependencies first
        let missing_packages = Self::check_dependencies()?;

        // If any essential packages are missing, return an error
        if !missing_packages.is_empty() {
            return Err("Missing required system packages. Please install them and try again.".into());
        }

        // Check if running as root
        let euid = unsafe { libc::geteuid() };
        if euid != 0 {
            eprintln!("\nWarning: This program requires root privileges to access all hardware information.");
            eprintln!("Please run it with: sudo {}", std::env::args().next().unwrap_or_default());
            eprintln!("Continuing with limited functionality...\n");
        }

        let hostname = Self::get_hostname()?;
        let hardware = Self::collect_hardware_info()?;
        let network = Self::collect_network_info()?;
        let (bmc_ip, bmc_mac) = Self::collect_ipmi_info()?;
        let os_ip = Self::collect_ip_addresses()?;

        let summary = Self::generate_summary(&hardware, &network)?;

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

    /// Calculates total storage capacity
    fn calculate_total_storage(storage: &StorageInfo) -> Result<String, Box<dyn Error>> {
        let mut total_bytes: u64 = 0;

        for device in &storage.devices {
            let size_str = device.size.replace(" ", "");
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
            .args(&["-h", "--output=source,fstype,size,used,avail,target"])
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
    fn extract_dmidecode_value(text: &str, key: &str) -> Result<String, Box<dyn Error>> {
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
        let output = Command::new("lscpu")
            .args(&["-J"])
            .output()?;

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut info_map = HashMap::new();

        if let Some(entries) = json["lscpu"].as_array() {
            for entry in entries {
                if let (Some(field), Some(data)) = (entry["field"].as_str(), entry["data"].as_str()) {
                    let key = field.trim_end_matches(':');
                    info_map.insert(key.to_string(), data.to_string());
                }
            }
        }

        Ok(CpuTopology {
            total_cores: info_map.get("Core(s) per socket")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0) * info_map.get("Socket(s)")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            total_threads: info_map.get("CPU(s)")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            sockets: info_map.get("Socket(s)")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            cores_per_socket: info_map.get("Core(s) per socket")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            threads_per_core: info_map.get("Thread(s) per core")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            numa_nodes: info_map.get("NUMA node(s)")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            cpu_model: info_map.get("Model name")
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string()),
        })
    }

    /// Generates system summary with better error handling
    /// Enhanced summary generation with NUMA topology
    fn generate_summary(
        hardware: &HardwareInfo,
        network: &NetworkInfo,
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

        Ok(SystemSummary {
            total_memory: hardware.memory.total.clone(),
            memory_config: format!("{} @ {}", hardware.memory.type_, hardware.memory.speed),
            total_storage: Self::calculate_total_storage(&hardware.storage)?,
            filesystems: Self::get_filesystems().unwrap_or_default(),
            bios,
            chassis,
            total_gpus: hardware.gpus.devices.len(),
            total_nics: network.interfaces.len(),
            numa_topology: Self::collect_numa_topology()?,
        })
    }
    /// Collects IP addresses for all network interfaces.
    fn collect_ip_addresses() -> Result<HashMap<String, String>, Box<dyn Error>> {
        // Run 'ip -j addr show' to get JSON output of network interfaces.
        let output = Command::new("ip").args(&["-j", "addr", "show"]).output()?;

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut addresses = HashMap::new();

        if let Some(interfaces) = json.as_array() {
            for interface in interfaces {
                if let (Some(name), Some(addr_info)) = (
                    interface["ifname"].as_str(),
                    interface["addr_info"].as_array(),
                ) {
                    for addr in addr_info {
                        if let Some(ip) = addr["local"].as_str() {
                            if addr["family"].as_str() == Some("inet") {
                                // Map interface name to IPv4 address.
                                addresses.insert(name.to_string(), ip.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(addresses)
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
                    let (vendor, _, pci_id) = Self::get_pci_info(pci_addr)
                        .unwrap_or(("NVIDIA".to_string(), "Unknown".to_string(), "Unknown".to_string()));

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

    /// Collects NUMA topology information
    fn get_numa_topology(
        hardware: &HardwareInfo,
        network: &NetworkInfo,
    ) -> Result<HashMap<i32, NumaNode>, Box<dyn Error>> {
        let mut nodes = HashMap::new();

        // Get basic NUMA information using numactl
        let output = Command::new("numactl")
            .args(&["--hardware"])
            .output()?;
        let output_str = String::from_utf8(output.stdout)?;

        // Parse node information
        let mut current_node_id = None;
        for line in output_str.lines() {
            if line.starts_with("node ") && line.contains("size:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(id) = parts[1].parse::<i32>() {
                        current_node_id = Some(id);
                        let memory = format!("{} {}", parts[3], parts[4]);
                        nodes.insert(id, NumaNode {
                            id,
                            memory,
                            cpus: Vec::new(),
                            devices: Vec::new(),
                            distances: HashMap::new(),
                        });
                    }
                }
            } else if line.contains("node distances:") {
                break;
            }
        }

        // Get CPU to node mapping
        let output = Command::new("lscpu")
            .args(&["-p=cpu,node"])
            .output()?;
        let output_str = String::from_utf8(output.stdout)?;

        for line in output_str.lines() {
            if line.starts_with('#') { continue; }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let (Ok(cpu), Ok(node)) = (parts[0].parse::<u32>(), parts[1].parse::<i32>()) {
                    if let Some(numa_node) = nodes.get_mut(&node) {
                        numa_node.cpus.push(cpu);
                    }
                }
            }
        }

        // Get device to node mapping
        // First, collect GPUs
        for gpu in &hardware.gpus.devices {
            if let Some(node) = gpu.numa_node {
                if let Some(numa_node) = nodes.get_mut(&node) {
                    numa_node.devices.push(NumaDevice {
                        type_: "GPU".to_string(),
                        name: gpu.name.clone(),
                        pci_id: gpu.pci_id.clone(),
                    });
                }
            }
        }

        // Then, collect NICs
        for nic in &network.interfaces {
            if let Some(node) = nic.numa_node {
                if let Some(numa_node) = nodes.get_mut(&node) {
                    numa_node.devices.push(NumaDevice {
                        type_: "NIC".to_string(),
                        name: nic.name.clone(),
                        pci_id: nic.pci_id.clone(),
                    });
                }
            }
        }

        // Get node distances
        let output = Command::new("numactl")
            .args(&["--hardware"])
            .output()?;
        let output_str = String::from_utf8(output.stdout)?;
        let mut reading_distances = false;

        for line in output_str.lines() {
            if line.contains("node distances:") {
                reading_distances = true;
                continue;
            }
            if reading_distances && line.starts_with("node") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    if let Ok(from_node) = parts[1].parse::<i32>() {
                        for (i, distance_str) in parts[2..].iter().enumerate() {
                            if let Ok(distance) = distance_str.parse::<u32>() {
                                if let Some(node) = nodes.get_mut(&from_node) {
                                    node.distances.insert((i as i32).to_string(), distance.to_string().parse().unwrap());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(nodes)
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

                    if let Ok(pci_addr) = std::fs::read_link(format!("/sys/class/net/{}/device", name)) {
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
                            let re_speed = Regex::new(r"Speed:\s+(\S+)")?;
                            if let Some(cap) = re_speed.captures(&output_str) {
                                Some(cap[1].to_string())
                            } else {
                                None
                            }
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

fn main() -> Result<(), Box<dyn Error>> {
    // Collect server information
    let server_info = ServerInfo::collect()?;

    // Generate summary output for console
    println!("System Summary:");
    println!("==============");
    println!(
        "Memory: {} ({})",
        server_info.hardware.memory.total,
        format!(
            "{} @ {}",
            server_info.hardware.memory.type_, server_info.hardware.memory.speed
        )
    );

    // Calculate total storage
    let total_storage = server_info
        .hardware
        .storage
        .devices
        .iter()
        .map(|device| device.size.clone())
        .collect::<Vec<String>>()
        .join(" + ");
    println!("Storage: {}", total_storage);

    // Get BIOS information from dmidecode
    let output = Command::new("dmidecode").args(&["-t", "bios"]).output()?;
    let bios_str = String::from_utf8(output.stdout)?;
    println!(
        "BIOS: {} {} ({})",
        ServerInfo::extract_dmidecode_value(&bios_str, "Vendor")?,
        ServerInfo::extract_dmidecode_value(&bios_str, "Version")?,
        ServerInfo::extract_dmidecode_value(&bios_str, "Release Date")?
    );

    // Get chassis information from dmidecode
    let output = Command::new("dmidecode")
        .args(&["-t", "chassis"])
        .output()?;
    let chassis_str = String::from_utf8(output.stdout)?;
    println!(
        "Chassis: {} {} (S/N: {})",
        ServerInfo::extract_dmidecode_value(&chassis_str, "Manufacturer")?,
        ServerInfo::extract_dmidecode_value(&chassis_str, "Type")?,
        ServerInfo::extract_dmidecode_value(&chassis_str, "Serial Number")?
    );

    println!("\nNetwork Interfaces:");
    for nic in &server_info.network.interfaces {
        println!("  {} - {} {} ({}) [Speed: {}] [NUMA: {}]",
                 nic.name,
                 nic.vendor,
                 nic.model,
                 nic.pci_id,
                 nic.speed.as_deref().unwrap_or("Unknown"),
                 nic.numa_node.map_or("Unknown".to_string(), |n| n.to_string())
        );
    }

    println!("\nGPUs:");
    for gpu in &server_info.hardware.gpus.devices {
        println!("  {} - {} ({}) [NUMA: {}]",
                 gpu.name,
                 gpu.vendor,
                 gpu.pci_id,
                 gpu.numa_node.map_or("Unknown".to_string(), |n| n.to_string())
        );
    }

    println!("\nNUMA Topology:");
    for (node_id, node) in &server_info.summary.numa_topology {
        println!("  Node {}:", node_id);
        println!("    Memory: {}", node.memory);
        println!("    CPUs: {:?}", node.cpus);

        if !node.devices.is_empty() {
            println!("    Devices:");
            for device in &node.devices {
                println!("      {} - {} (PCI ID: {})",
                         device.type_,
                         device.name,
                         device.pci_id
                );
            }
        }

        println!("    Distances:");
        let mut distances: Vec<_> = node.distances.iter().collect();
        distances.sort_by_key(|&(k, _)| k);
        for (to_node, distance) in distances {
            println!("      To Node {}: {}", to_node, distance);
        }
    }    

    // Get filesystem information
    println!("\nFilesystems:");
    let output = Command::new("df")
        .args(&["-h", "--output=source,fstype,size,used,avail,target"])
        .output()?;
    let fs_str = String::from_utf8(output.stdout)?;
    for line in fs_str.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 6 {
            println!(
                "  {} ({}) - {} total, {} used, {} available, mounted on {}",
                fields[0], fields[1], fields[2], fields[3], fields[4], fields[5]
            );
        }
    }

    println!("\nFull configuration being written to server_config.toml...");

    // Convert to TOML
    let toml_string = toml::to_string_pretty(&server_info)?;

    // Write to file
    std::fs::write("server_config.toml", toml_string)?;

    println!("Configuration has been written to server_config.toml");

    Ok(())
}
