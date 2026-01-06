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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the overall hardware report (root aggregate)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareReport {
    /// System summary information
    pub summary: SystemSummary,
    /// System hostname
    pub hostname: String,
    /// Fully qualified domain name
    pub fqdn: String,
    /// Operating system IP addresses
    pub os_ip: Vec<InterfaceIPs>,
    /// BMC IP address
    pub bmc_ip: Option<String>,
    /// BMC MAC address
    pub bmc_mac: Option<String>,
    /// Detailed hardware information
    pub hardware: HardwareInfo,
    /// Network information
    pub network: NetworkInfo,
}

/// Summary of key system components
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSummary {
    /// Basic system information
    pub system_info: SystemInfo,
    /// Total system memory capacity
    pub total_memory: String,
    /// Memory speed and type configuration
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

/// System identification information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    /// System UUID
    pub uuid: String,
    /// System serial number
    pub serial: String,
    /// Product name
    pub product_name: String,
    /// Product manufacturer
    pub product_manufacturer: String,
}

/// BIOS/Firmware information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiosInfo {
    /// BIOS vendor
    pub vendor: String,
    /// BIOS version
    pub version: String,
    /// BIOS release date
    pub release_date: String,
    /// Firmware version
    pub firmware_version: String,
}

/// System chassis information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChassisInfo {
    /// Chassis manufacturer
    pub manufacturer: String,
    /// Chassis type
    pub type_: String,
    /// Chassis serial number
    pub serial: String,
}

/// Motherboard information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MotherboardInfo {
    /// Motherboard manufacturer
    pub manufacturer: String,
    /// Product name
    pub product_name: String,
    /// Version
    pub version: String,
    /// Serial number
    pub serial: String,
    /// Features
    pub features: String,
    /// Physical location
    pub location: String,
    /// Motherboard type
    pub type_: String,
}

/// CPU topology information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuTopology {
    /// Total number of cores
    pub total_cores: u32,
    /// Total number of threads
    pub total_threads: u32,
    /// Number of sockets
    pub sockets: u32,
    /// Cores per socket
    pub cores_per_socket: u32,
    /// Threads per core
    pub threads_per_core: u32,
    /// Number of NUMA nodes
    pub numa_nodes: u32,
    /// CPU model name
    pub cpu_model: String,
}

/// Contains detailed hardware information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareInfo {
    /// CPU information
    pub cpu: CpuInfo,
    /// Memory information
    pub memory: MemoryInfo,
    /// Storage information
    pub storage: StorageInfo,
    /// GPU information
    pub gpus: GpuInfo,
}

/// CPU information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuInfo {
    /// CPU model name
    pub model: String,
    /// Number of cores per socket
    pub cores: u32,
    /// Number of threads per core
    pub threads: u32,
    /// Number of sockets
    pub sockets: u32,
    /// CPU speed as string
    pub speed: String,
    /// CPU vendor (e.g., "GenuineIntel", "AuthenticAMD")
    #[serde(default)]
    pub vendor: String,
    /// CPU architecture
    #[serde(default)]
    pub architecture: String,
    /// CPU frequency in MHz
    #[serde(default)]
    pub frequency_mhz: u32,
    /// Minimum frequency in MHz
    #[serde(default)]
    pub frequency_min_mhz: Option<u32>,
    /// Maximum frequency in MHz  
    #[serde(default)]
    pub frequency_max_mhz: Option<u32>,
    /// L1 data cache size in KB
    #[serde(default)]
    pub cache_l1d_kb: Option<u32>,
    /// L1 instruction cache size in KB
    #[serde(default)]
    pub cache_l1i_kb: Option<u32>,
    /// L2 cache size in KB
    #[serde(default)]
    pub cache_l2_kb: Option<u32>,
    /// L3 cache size in KB
    #[serde(default)]
    pub cache_l3_kb: Option<u32>,
    /// CPU flags/features
    #[serde(default)]
    pub flags: Vec<String>,
    /// Microarchitecture (e.g., "Zen 3", "Ice Lake")
    #[serde(default)]
    pub microarchitecture: Option<String>,
    /// Detailed cache information
    #[serde(default)]
    pub caches: Vec<CpuCacheInfo>,
    /// Detection methods used
    #[serde(default)]
    pub detection_methods: Vec<String>,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            model: String::new(),
            cores: 0,
            threads: 0,
            sockets: 0,
            speed: String::new(),
            vendor: String::new(),
            architecture: String::new(),
            frequency_mhz: 0,
            frequency_min_mhz: None,
            frequency_max_mhz: None,
            cache_l1d_kb: None,
            cache_l1i_kb: None,
            cache_l2_kb: None,
            cache_l3_kb: None,
            flags: Vec::new(),
            microarchitecture: None,
            caches: Vec::new(),
            detection_methods: Vec::new(),
        }
    }
}

impl CpuInfo {
    /// Set speed string from frequency_mhz
    pub fn set_speed_string(&mut self) {
        if self.frequency_mhz > 0 {
            if self.frequency_mhz >= 1000 {
                self.speed = format!("{:.2} GHz", self.frequency_mhz as f64 / 1000.0);
            } else {
                self.speed = format!("{} MHz", self.frequency_mhz);
            }
        }
    }

    /// Calculate total cores and threads
    pub fn calculate_totals(&mut self) {
        // These are typically already set correctly from parsing
    }
}

/// CPU cache information
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CpuCacheInfo {
    /// Cache level (1, 2, 3, etc.)
    pub level: u8,
    /// Cache type ("Data", "Instruction", "Unified")
    pub cache_type: String,
    /// Size in KB
    pub size_kb: u32,
    /// Ways of associativity
    pub ways_of_associativity: Option<u32>,
    /// Cache line size in bytes
    pub line_size_bytes: Option<u32>,
    /// Number of sets
    pub sets: Option<u32>,
    /// Whether this cache is shared between cores
    pub shared: Option<bool>,
}

/// Memory information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryInfo {
    /// Total memory size
    pub total: String,
    /// Memory type (e.g., DDR4, LPDDR5)
    pub type_: String,
    /// Memory speed
    pub speed: String,
    /// Individual memory modules
    pub modules: Vec<MemoryModule>,
}

/// Individual memory module
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryModule {
    /// Size of the memory module
    pub size: String,
    /// Type of the memory module
    pub type_: String,
    /// Speed of the memory module
    pub speed: String,
    /// Physical location
    pub location: String,
    /// Manufacturer
    pub manufacturer: String,
    /// Serial number
    pub serial: String,
}

/// Storage information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageInfo {
    /// List of storage devices
    pub devices: Vec<StorageDevice>,
}

/// Storage type classification
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StorageType {
    /// NVMe SSD
    Nvme,
    /// SATA SSD
    Ssd,
    /// Hard Disk Drive
    Hdd,
    /// eMMC storage (common on ARM)
    Emmc,
    /// Virtual device (should be filtered)
    Virtual,
    /// Unknown type
    Unknown,
}

impl StorageType {
    /// Determine storage type from device name and rotational flag
    pub fn from_device(name: &str, is_rotational: bool) -> Self {
        if name.starts_with("nvme") {
            StorageType::Nvme
        } else if name.starts_with("mmcblk") {
            StorageType::Emmc
        } else if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("dm-") {
            StorageType::Virtual
        } else if is_rotational {
            StorageType::Hdd
        } else {
            StorageType::Ssd
        }
    }

    /// Get display name for the storage type
    pub fn display_name(&self) -> &'static str {
        match self {
            StorageType::Nvme => "NVMe SSD",
            StorageType::Ssd => "SSD",
            StorageType::Hdd => "HDD",
            StorageType::Emmc => "eMMC",
            StorageType::Virtual => "Virtual",
            StorageType::Unknown => "Unknown",
        }
    }
}

impl Default for StorageType {
    fn default() -> Self {
        StorageType::Unknown
    }
}

/// Storage device information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageDevice {
    /// Device name (e.g., "sda", "nvme0n1")
    pub name: String,
    /// Device path (e.g., "/dev/sda")
    #[serde(default)]
    pub device_path: String,
    /// Device type enum
    #[serde(default)]
    pub device_type: StorageType,
    /// Device type as string (for backward compatibility)
    pub type_: String,
    /// Device size as human-readable string
    pub size: String,
    /// Device size in bytes
    #[serde(default)]
    pub size_bytes: u64,
    /// Device size in GB
    #[serde(default)]
    pub size_gb: f64,
    /// Device model
    pub model: String,
    /// Serial number
    #[serde(default)]
    pub serial_number: Option<String>,
    /// Firmware version
    #[serde(default)]
    pub firmware_version: Option<String>,
    /// World Wide Name
    #[serde(default)]
    pub wwn: Option<String>,
    /// Interface type (e.g., "NVMe", "SATA", "SAS")
    #[serde(default)]
    pub interface: String,
    /// Whether this is a rotational device (HDD)
    #[serde(default)]
    pub is_rotational: bool,
    /// Detection method used
    #[serde(default)]
    pub detection_method: String,
}

impl Default for StorageDevice {
    fn default() -> Self {
        Self {
            name: String::new(),
            device_path: String::new(),
            device_type: StorageType::Unknown,
            type_: String::new(),
            size: String::new(),
            size_bytes: 0,
            size_gb: 0.0,
            model: String::new(),
            serial_number: None,
            firmware_version: None,
            wwn: None,
            interface: String::new(),
            is_rotational: false,
            detection_method: String::new(),
        }
    }
}

impl StorageDevice {
    /// Calculate size_gb and size string from size_bytes
    pub fn calculate_size_fields(&mut self) {
        if self.size_bytes > 0 {
            self.size_gb = self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            if self.size_gb >= 1000.0 {
                self.size = format!("{:.2} TB", self.size_gb / 1024.0);
            } else {
                self.size = format!("{:.2} GB", self.size_gb);
            }
        }
    }

    /// Set device path from name if not already set
    pub fn set_device_path(&mut self) {
        if self.device_path.is_empty() && !self.name.is_empty() {
            self.device_path = format!("/dev/{}", self.name);
        }
    }
}

/// GPU information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuInfo {
    /// List of GPU devices
    pub devices: Vec<GpuDevice>,
}

/// GPU vendor classification
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum GpuVendor {
    /// NVIDIA GPU
    Nvidia,
    /// AMD GPU
    Amd,
    /// Intel GPU
    Intel,
    /// Apple GPU (Apple Silicon)
    Apple,
    /// Unknown vendor
    Unknown,
}

impl GpuVendor {
    /// Determine vendor from PCI vendor ID
    pub fn from_pci_vendor(vendor_id: &str) -> Self {
        match vendor_id.to_lowercase().as_str() {
            "10de" => GpuVendor::Nvidia,
            "1002" => GpuVendor::Amd,
            "8086" => GpuVendor::Intel,
            _ => GpuVendor::Unknown,
        }
    }

    /// Get vendor name string
    pub fn name(&self) -> &'static str {
        match self {
            GpuVendor::Nvidia => "NVIDIA",
            GpuVendor::Amd => "AMD",
            GpuVendor::Intel => "Intel",
            GpuVendor::Apple => "Apple",
            GpuVendor::Unknown => "Unknown",
        }
    }
}

impl Default for GpuVendor {
    fn default() -> Self {
        GpuVendor::Unknown
    }
}

/// GPU device information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuDevice {
    /// GPU index
    pub index: u32,
    /// GPU name
    pub name: String,
    /// GPU UUID
    pub uuid: String,
    /// Total GPU memory as string (for backward compatibility)
    pub memory: String,
    /// Total GPU memory in MB
    #[serde(default)]
    pub memory_total_mb: u64,
    /// Free GPU memory in MB
    #[serde(default)]
    pub memory_free_mb: Option<u64>,
    /// PCI ID (vendor:device) or Apple Fabric for Apple Silicon
    pub pci_id: String,
    /// PCI bus ID (e.g., "0000:01:00.0")
    #[serde(default)]
    pub pci_bus_id: Option<String>,
    /// Vendor name (for backward compatibility)
    pub vendor: String,
    /// Vendor classification enum
    #[serde(default)]
    pub vendor_enum: GpuVendor,
    /// NUMA node
    pub numa_node: Option<i32>,
    /// Driver version
    #[serde(default)]
    pub driver_version: Option<String>,
    /// Compute capability (NVIDIA specific)
    #[serde(default)]
    pub compute_capability: Option<String>,
    /// Detection method used
    #[serde(default)]
    pub detection_method: String,
}

impl Default for GpuDevice {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::new(),
            uuid: String::new(),
            memory: String::new(),
            memory_total_mb: 0,
            memory_free_mb: None,
            pci_id: String::new(),
            pci_bus_id: None,
            vendor: String::new(),
            vendor_enum: GpuVendor::Unknown,
            numa_node: None,
            driver_version: None,
            compute_capability: None,
            detection_method: String::new(),
        }
    }
}

impl GpuDevice {
    /// Set memory string from memory_total_mb
    pub fn set_memory_string(&mut self) {
        if self.memory_total_mb > 0 {
            if self.memory_total_mb >= 1024 {
                self.memory = format!("{:.1} GB", self.memory_total_mb as f64 / 1024.0);
            } else {
                self.memory = format!("{} MB", self.memory_total_mb);
            }
        }
    }
}

/// Network information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInfo {
    /// List of network interfaces
    pub interfaces: Vec<NetworkInterface>,
    /// Infiniband information, if available
    pub infiniband: Option<InfinibandInfo>,
}

/// Network interface type classification
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum NetworkInterfaceType {
    /// Physical Ethernet interface
    Ethernet,
    /// Wireless interface
    Wireless,
    /// Loopback interface
    Loopback,
    /// Bridge interface
    Bridge,
    /// VLAN interface
    Vlan,
    /// Bond/LAG interface
    Bond,
    /// Virtual Ethernet (veth pair)
    Veth,
    /// TUN/TAP interface
    TunTap,
    /// InfiniBand interface
    Infiniband,
    /// Macvlan interface
    Macvlan,
    /// Unknown type
    Unknown,
}

impl Default for NetworkInterfaceType {
    fn default() -> Self {
        NetworkInterfaceType::Unknown
    }
}

/// Network interface information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    /// MAC address
    pub mac: String,
    /// IP address
    pub ip: String,
    /// IP prefix
    pub prefix: String,
    /// Interface speed as string (e.g., "1000 Mbps")
    pub speed: Option<String>,
    /// Interface speed in Mbps (numeric)
    #[serde(default)]
    pub speed_mbps: Option<u32>,
    /// Interface type as string (for backward compatibility)
    pub type_: String,
    /// Interface type classification
    #[serde(default)]
    pub interface_type: NetworkInterfaceType,
    /// Vendor
    pub vendor: String,
    /// Model
    pub model: String,
    /// PCI ID or Apple Fabric for Apple Silicon
    pub pci_id: String,
    /// NUMA node
    pub numa_node: Option<i32>,
    /// Kernel driver in use
    #[serde(default)]
    pub driver: Option<String>,
    /// Driver version
    #[serde(default)]
    pub driver_version: Option<String>,
    /// Maximum Transmission Unit in bytes
    #[serde(default = "default_mtu")]
    pub mtu: u32,
    /// Whether the interface is operationally up
    #[serde(default)]
    pub is_up: bool,
    /// Whether this is a virtual interface
    #[serde(default)]
    pub is_virtual: bool,
    /// Link detected (carrier present)
    #[serde(default)]
    pub carrier: Option<bool>,
}

fn default_mtu() -> u32 {
    1500
}

impl Default for NetworkInterface {
    fn default() -> Self {
        Self {
            name: String::new(),
            mac: String::new(),
            ip: String::new(),
            prefix: String::new(),
            speed: None,
            speed_mbps: None,
            type_: String::new(),
            interface_type: NetworkInterfaceType::Unknown,
            vendor: String::new(),
            model: String::new(),
            pci_id: String::new(),
            numa_node: None,
            driver: None,
            driver_version: None,
            mtu: 1500,
            is_up: false,
            is_virtual: false,
            carrier: None,
        }
    }
}

/// Infiniband information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfinibandInfo {
    /// List of Infiniband interfaces
    pub interfaces: Vec<IbInterface>,
}

/// Infiniband interface
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IbInterface {
    /// Interface name
    pub name: String,
    /// Port number
    pub port: u32,
    /// Interface state
    pub state: String,
    /// Interface rate
    pub rate: String,
}

/// NUMA node information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NumaNode {
    /// Node ID
    pub id: i32,
    /// CPU list
    pub cpus: Vec<u32>,
    /// Memory size
    pub memory: String,
    /// Devices attached to this node
    pub devices: Vec<NumaDevice>,
    /// Distances to other nodes (node_id -> distance)
    pub distances: HashMap<String, u32>,
}

/// Device attached to a NUMA node
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NumaDevice {
    /// Device type (GPU, NIC, etc.)
    pub type_: String,
    /// PCI ID
    pub pci_id: String,
    /// Device name
    pub name: String,
}

/// Interface IP addresses
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InterfaceIPs {
    /// Interface name
    pub interface: String,
    /// IP addresses assigned to this interface
    pub ip_addresses: Vec<String>,
}

/// Configuration for hardware report generation
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Include sensitive information
    pub include_sensitive: bool,
    /// Skip privilege escalation
    pub skip_sudo: bool,
    /// Timeout for commands in seconds
    pub command_timeout: u64,
    /// Enable verbose output
    pub verbose: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_sensitive: false,
            skip_sudo: false,
            command_timeout: 30,
            verbose: false,
        }
    }
}

/// Configuration for publishing reports
#[derive(Debug, Clone)]
pub struct PublishConfig {
    /// Endpoint URL
    pub endpoint: String,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Skip TLS verification
    pub skip_tls_verify: bool,
    /// Additional labels/metadata
    pub labels: HashMap<String, String>,
}
