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
    /// CPU speed
    pub speed: String,
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

/// Storage device information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageDevice {
    /// Device name
    pub name: String,
    /// Device type (e.g., ssd, hdd)
    pub type_: String,
    /// Device size
    pub size: String,
    /// Device model
    pub model: String,
}

/// GPU information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuInfo {
    /// List of GPU devices
    pub devices: Vec<GpuDevice>,
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
    /// Total GPU memory
    pub memory: String,
    /// PCI ID (vendor:device) or Apple Fabric for Apple Silicon
    pub pci_id: String,
    /// Vendor name
    pub vendor: String,
    /// NUMA node
    pub numa_node: Option<i32>,
}

/// Network information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInfo {
    /// List of network interfaces
    pub interfaces: Vec<NetworkInterface>,
    /// Infiniband information, if available
    pub infiniband: Option<InfinibandInfo>,
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
    /// Interface speed
    pub speed: Option<String>,
    /// Interface type
    pub type_: String,
    /// Vendor
    pub vendor: String,
    /// Model
    pub model: String,
    /// PCI ID or Apple Fabric for Apple Silicon
    pub pci_id: String,
    /// NUMA node
    pub numa_node: Option<i32>,
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
