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

/// Represents the overall server information.
#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    /// Hostname of the server.
    hostname: String,
    /// Mapping from OS network interface names to their IP addresses.
    os_ip: HashMap<String, String>, // Interface name -> IP
    /// IP address of the BMC (if available).
    bmc_ip: Option<String>,
    /// MAC address of the BMC (if available).
    bmc_mac: Option<String>,
    /// Detailed hardware information.
    hardware: HardwareInfo,
    /// Network interfaces and Infiniband information.
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
    /// GPU index.
    index: u32,
    /// GPU name.
    name: String,
    /// GPU UUID.
    uuid: String,
    /// Total GPU memory.
    memory: String,
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

impl ServerInfo {
    /// Collects all server information by calling various system commands.
    fn collect() -> Result<Self, Box<dyn Error>> {
        let hostname = Self::get_hostname()?;
        let network = Self::collect_network_info()?;
        let hardware = Self::collect_hardware_info()?;
        let (bmc_ip, bmc_mac) = Self::collect_ipmi_info()?;

        Ok(ServerInfo {
            hostname,
            os_ip: Self::collect_ip_addresses()?,
            bmc_ip,
            bmc_mac,
            hardware,
            network,
        })
    }

    /// Retrieves the hostname of the server.
    fn get_hostname() -> Result<String, Box<dyn Error>> {
        let output = Command::new("hostname").output()?;
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
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

    /// Extracts a value from 'dmidecode' output given a specific key.
    fn extract_dmidecode_value(text: &str, key: &str) -> Result<String, Box<dyn Error>> {
        let re = Regex::new(&format!(r"\t{}: (.*)", regex::escape(key)))?;
        if let Some(cap) = re.captures(text) {
            Ok(cap[1].trim().to_string())
        } else {
            Err(format!("Could not find key: {}", key).into())
        }
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
                "--query-gpu=index,name,uuid,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .output();

        match output {
            Ok(output) => {
                let output_str = String::from_utf8(output.stdout)?;
                let mut devices = Vec::new();

                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 4 {
                        devices.push(GpuDevice {
                            index: parts[0].trim().parse()?,
                            name: parts[1].trim().to_string(),
                            uuid: parts[2].trim().to_string(),
                            memory: format!("{} MB", parts[3].trim()),
                        });
                    }
                }

                Ok(GpuInfo { devices })
            }
            Err(_) => {
                // nvidia-smi command not found or failed.
                Ok(GpuInfo {
                    devices: Vec::new(),
                })
            }
        }
    }

    /// Collects network information, including Infiniband if available.
    fn collect_network_info() -> Result<NetworkInfo, Box<dyn Error>> {
        let mut interfaces = Vec::new();
        let output = Command::new("ip").args(&["-j", "addr", "show"]).output()?;

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

        if let Some(ifaces) = json.as_array() {
            for iface in ifaces {
                if let Some(name) = iface["ifname"].as_str() {
                    let mac = iface["address"].as_str().unwrap_or("").to_string();
                    let mut ip = String::new();
                    if let Some(addr_info) = iface["addr_info"].as_array() {
                        for addr in addr_info {
                            if addr["family"].as_str() == Some("inet") {
                                ip = addr["local"].as_str().unwrap_or("").to_string();
                                break;
                            }
                        }
                    }

                    // Retrieve interface speed using 'ethtool'.
                    let speed_output = Command::new("ethtool").arg(name).output();

                    let speed = match speed_output {
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

                    // Determine interface type.
                    let type_ = iface["link_type"].as_str().unwrap_or("").to_string();

                    interfaces.push(NetworkInterface {
                        name: name.to_string(),
                        mac,
                        ip,
                        speed,
                        type_,
                    });
                }
            }
        }

        let infiniband = Self::collect_infiniband_info()?;

        Ok(NetworkInfo {
            interfaces,
            infiniband,
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
    // Collect server information.
    let server_info = ServerInfo::collect()?;

    // Convert to TOML.
    let toml_string = toml::to_string_pretty(&server_info)?;

    // Write to file.
    std::fs::write("server_config.toml", toml_string)?;

    println!("Configuration has been written to server_config.toml");

    Ok(())
}
