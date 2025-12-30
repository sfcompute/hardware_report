# Network Interface Enhancement Plan

> **Category:** Data Gap  
> **Target Platforms:** Linux (x86_64, aarch64)  
> **Priority:** Medium - Missing driver version and operational state

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Current Implementation](#current-implementation)
3. [Entity Changes](#entity-changes)
4. [Detection Method Details](#detection-method-details)
5. [Adapter Implementation](#adapter-implementation)
6. [Parser Implementation](#parser-implementation)
7. [Testing Requirements](#testing-requirements)
8. [References](#references)

---

## Problem Statement

### Current Issue

The `NetworkInterface` structure lacks driver information and operational state:

```rust
// Current struct - missing fields
pub struct NetworkInterface {
    pub name: String,
    pub mac: String,
    pub ip: String,
    pub prefix: String,
    pub speed: Option<String>,  // String, not numeric
    pub type_: String,
    pub vendor: String,
    pub model: String,
    pub pci_id: String,
    pub numa_node: Option<i32>,
    // Missing: driver, driver_version, mtu, is_up, is_virtual
}
```

### Impact

- Cannot track NIC driver versions for compatibility
- No MTU information for network configuration validation
- Cannot determine interface operational state
- Cannot distinguish physical vs virtual interfaces

### Requirements

1. **Driver information** - driver name and version
2. **Numeric speed** - `speed_mbps: Option<u32>`
3. **Operational state** - `is_up: bool`
4. **MTU** - `mtu: u32`
5. **Virtual interface detection** - `is_virtual: bool`

---

## Current Implementation

### Location

- **Entity:** `src/domain/entities.rs:263-285`
- **Adapter:** `src/adapters/secondary/system/linux.rs:234-252`
- **Parser:** `src/domain/parsers/network.rs`

### Current Detection

```
┌────────────────────────────────────────────┐
│ LinuxSystemInfoProvider::get_network_info()│
└────────────────────────────────────────────┘
                    │
                    ▼
         ┌──────────────────┐
         │ ip addr show     │
         └──────────────────┘
                    │
                    ▼
         Parse interface list
```

---

## Entity Changes

### New NetworkInterface Structure

```rust
// src/domain/entities.rs

/// Network interface type classification
///
/// # References
///
/// - [Linux Networking](https://www.kernel.org/doc/html/latest/networking/index.html)
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

/// Network interface information
///
/// Represents a network interface with comprehensive metadata for
/// CMDB inventory and network configuration.
///
/// # Detection Methods
///
/// Network information is gathered from multiple sources:
/// 1. **sysfs /sys/class/net** - Primary source for most fields
/// 2. **ip command** - Address and routing information
/// 3. **ethtool** - Speed, driver, firmware (requires privileges)
///
/// # Driver Information
///
/// The `driver` and `driver_version` fields are essential for:
/// - Compatibility tracking
/// - Firmware update planning
/// - Troubleshooting network issues
///
/// # Example
///
/// ```
/// use hardware_report::NetworkInterface;
///
/// // Check if interface is usable
/// if iface.is_up && !iface.is_virtual && iface.speed_mbps.unwrap_or(0) >= 10000 {
///     println!("{} is a 10G+ physical interface", iface.name);
/// }
/// ```
///
/// # References
///
/// - [Linux sysfs net](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-net)
/// - [ethtool](https://man7.org/linux/man-pages/man8/ethtool.8.html)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInterface {
    /// Interface name (e.g., "eth0", "ens192", "enp0s3")
    pub name: String,
    
    /// MAC address in colon-separated format
    ///
    /// Example: "00:11:22:33:44:55"
    pub mac: String,
    
    /// Permanent MAC address (if different from current)
    ///
    /// Some NICs allow changing the MAC address.
    pub permanent_mac: Option<String>,
    
    /// Primary IPv4 address
    pub ip: String,
    
    /// IPv4 addresses with prefix length
    pub ipv4_addresses: Vec<String>,
    
    /// IPv6 addresses with prefix length
    pub ipv6_addresses: Vec<String>,
    
    /// Network prefix length (e.g., "24" for /24)
    pub prefix: String,
    
    /// Link speed in Mbps
    ///
    /// Common values: 1000 (1G), 10000 (10G), 25000 (25G), 100000 (100G)
    pub speed_mbps: Option<u32>,
    
    /// Link speed as string (e.g., "10000 Mbps")
    pub speed: Option<String>,
    
    /// Interface type classification
    pub interface_type: NetworkInterfaceType,
    
    /// Interface type as string (for backward compatibility)
    pub type_: String,
    
    /// Hardware vendor name
    pub vendor: String,
    
    /// Hardware model/description
    pub model: String,
    
    /// PCI vendor:device ID (e.g., "8086:1521")
    pub pci_id: String,
    
    /// PCI bus address (e.g., "0000:01:00.0")
    pub pci_bus_id: Option<String>,
    
    /// NUMA node affinity
    pub numa_node: Option<i32>,
    
    /// Kernel driver in use
    ///
    /// Examples: "igb", "i40e", "mlx5_core", "bnxt_en"
    pub driver: Option<String>,
    
    /// Driver version
    ///
    /// From `/sys/module/{driver}/version` or ethtool.
    pub driver_version: Option<String>,
    
    /// Firmware version
    ///
    /// From ethtool -i.
    pub firmware_version: Option<String>,
    
    /// Maximum Transmission Unit in bytes
    ///
    /// Standard: 1500, Jumbo frames: 9000
    pub mtu: u32,
    
    /// Whether the interface is operationally up
    ///
    /// From `/sys/class/net/{iface}/operstate`.
    pub is_up: bool,
    
    /// Whether this is a virtual interface
    ///
    /// Virtual interfaces include: bridges, VLANs, bonds, veths, tun/tap.
    pub is_virtual: bool,
    
    /// Whether this interface is a loopback
    pub is_loopback: bool,
    
    /// Link detected (carrier present)
    pub carrier: Option<bool>,
    
    /// Duplex mode: "full", "half", or None if not applicable
    pub duplex: Option<String>,
    
    /// Auto-negotiation status
    pub autoneg: Option<bool>,
    
    /// Wake-on-LAN support
    pub wake_on_lan: Option<String>,
    
    /// Transmit queue length
    pub tx_queue_len: Option<u32>,
    
    /// Number of RX queues
    pub rx_queues: Option<u32>,
    
    /// Number of TX queues
    pub tx_queues: Option<u32>,
    
    /// SR-IOV Virtual Functions enabled
    pub sriov_numvfs: Option<u32>,
    
    /// Maximum SR-IOV Virtual Functions
    pub sriov_totalvfs: Option<u32>,
}

impl Default for NetworkInterface {
    fn default() -> Self {
        Self {
            name: String::new(),
            mac: String::new(),
            permanent_mac: None,
            ip: String::new(),
            ipv4_addresses: Vec::new(),
            ipv6_addresses: Vec::new(),
            prefix: String::new(),
            speed_mbps: None,
            speed: None,
            interface_type: NetworkInterfaceType::Unknown,
            type_: String::new(),
            vendor: String::new(),
            model: String::new(),
            pci_id: String::new(),
            pci_bus_id: None,
            numa_node: None,
            driver: None,
            driver_version: None,
            firmware_version: None,
            mtu: 1500,
            is_up: false,
            is_virtual: false,
            is_loopback: false,
            carrier: None,
            duplex: None,
            autoneg: None,
            wake_on_lan: None,
            tx_queue_len: None,
            rx_queues: None,
            tx_queues: None,
            sriov_numvfs: None,
            sriov_totalvfs: None,
        }
    }
}
```

---

## Detection Method Details

### Method 1: sysfs /sys/class/net (Primary)

**sysfs paths:**

```
/sys/class/net/{iface}/
├── address             # MAC address
├── addr_len            # Address length
├── mtu                 # MTU
├── operstate           # up/down/unknown
├── carrier             # 1=link, 0=no link
├── speed               # Speed in Mbps (may be -1)
├── duplex              # full/half
├── tx_queue_len        # TX queue length
├── type                # Interface type (ARPHRD_*)
├── device/             # -> PCI device (if physical)
│   ├── vendor          # PCI vendor ID
│   ├── device          # PCI device ID
│   ├── numa_node       # NUMA affinity
│   ├── driver/         # -> driver symlink
│   │   └── module/
│   │       └── version # Driver version
│   └── net_dev/queues/
│       ├── rx-*/       # RX queues
│       └── tx-*/       # TX queues
├── queues/
│   ├── rx-*/           # RX queues
│   └── tx-*/           # TX queues
└── statistics/         # Interface statistics
    ├── rx_bytes
    ├── tx_bytes
    ├── rx_packets
    └── tx_packets
```

**Virtual interface detection:**
```rust
fn is_virtual_interface(name: &str, sysfs_path: &Path) -> bool {
    // Virtual interfaces don't have a /device symlink
    !sysfs_path.join("device").exists()
        || name.starts_with("veth")
        || name.starts_with("br")
        || name.starts_with("virbr")
        || name.starts_with("docker")
        || name.starts_with("vlan")
        || name.contains("bond")
        || name.starts_with("tun")
        || name.starts_with("tap")
}
```

**References:**
- [sysfs-class-net](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-net)

---

### Method 2: ethtool

**When:** For driver/firmware info, detailed link settings

**Commands:**
```bash
# Driver info
ethtool -i eth0

# Link settings
ethtool eth0

# Firmware/EEPROM info
ethtool -e eth0
```

**ethtool -i output:**
```
driver: igb
version: 5.4.0-k
firmware-version: 1.67, 0x80000d38
bus-info: 0000:01:00.0
```

**References:**
- [ethtool man page](https://man7.org/linux/man-pages/man8/ethtool.8.html)

---

### Method 3: ip command

**When:** For IP addresses

**Command:**
```bash
ip -j addr show  # JSON output
```

**References:**
- [ip command](https://man7.org/linux/man-pages/man8/ip.8.html)

---

## Parser Implementation

### File: `src/domain/parsers/network.rs`

```rust
//! Network interface parsing functions
//!
//! # References
//!
//! - [sysfs-class-net](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-net)
//! - [ethtool](https://man7.org/linux/man-pages/man8/ethtool.8.html)

use crate::domain::{NetworkInterface, NetworkInterfaceType};

/// Parse sysfs operstate to boolean
///
/// # Arguments
///
/// * `content` - Content of `/sys/class/net/{iface}/operstate`
///
/// # Returns
///
/// `true` if interface is up.
///
/// # Example
///
/// ```
/// use hardware_report::domain::parsers::network::parse_operstate;
///
/// assert!(parse_operstate("up"));
/// assert!(!parse_operstate("down"));
/// ```
pub fn parse_operstate(content: &str) -> bool {
    content.trim().to_lowercase() == "up"
}

/// Parse sysfs speed to Mbps
///
/// # Arguments
///
/// * `content` - Content of `/sys/class/net/{iface}/speed`
///
/// # Returns
///
/// Speed in Mbps, or None if invalid/unknown.
pub fn parse_sysfs_speed(content: &str) -> Option<u32> {
    let speed: i32 = content.trim().parse().ok()?;
    if speed > 0 {
        Some(speed as u32)
    } else {
        None // -1 means unknown
    }
}

/// Parse ethtool -i output for driver info
///
/// # Arguments
///
/// * `output` - Output from `ethtool -i {iface}`
///
/// # Returns
///
/// Tuple of (driver, version, firmware_version, bus_info).
///
/// # References
///
/// - [ethtool](https://man7.org/linux/man-pages/man8/ethtool.8.html)
pub fn parse_ethtool_driver_info(output: &str) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let mut driver = None;
    let mut version = None;
    let mut firmware = None;
    let mut bus_info = None;
    
    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        
        let key = parts[0].trim();
        let value = parts[1].trim();
        
        match key {
            "driver" => driver = Some(value.to_string()),
            "version" => version = Some(value.to_string()),
            "firmware-version" => firmware = Some(value.to_string()),
            "bus-info" => bus_info = Some(value.to_string()),
            _ => {}
        }
    }
    
    (driver, version, firmware, bus_info)
}

/// Parse ip -j addr output
///
/// # Arguments
///
/// * `output` - JSON output from `ip -j addr show`
///
/// # References
///
/// - [ip-address](https://man7.org/linux/man-pages/man8/ip-address.8.html)
pub fn parse_ip_json(output: &str) -> Result<Vec<NetworkInterface>, String> {
    todo!()
}

/// Determine interface type from name and sysfs
///
/// # Arguments
///
/// * `name` - Interface name
/// * `sysfs_type` - Content of `/sys/class/net/{name}/type`
pub fn determine_interface_type(name: &str, sysfs_type: Option<&str>) -> NetworkInterfaceType {
    // Check name patterns first
    if name == "lo" {
        return NetworkInterfaceType::Loopback;
    }
    if name.starts_with("br") || name.starts_with("virbr") {
        return NetworkInterfaceType::Bridge;
    }
    if name.starts_with("bond") {
        return NetworkInterfaceType::Bond;
    }
    if name.starts_with("veth") {
        return NetworkInterfaceType::Veth;
    }
    if name.contains(".") || name.starts_with("vlan") {
        return NetworkInterfaceType::Vlan;
    }
    if name.starts_with("tun") || name.starts_with("tap") {
        return NetworkInterfaceType::TunTap;
    }
    if name.starts_with("ib") {
        return NetworkInterfaceType::Infiniband;
    }
    if name.starts_with("wl") || name.starts_with("wlan") {
        return NetworkInterfaceType::Wireless;
    }
    
    // Check sysfs type (ARPHRD_* values)
    if let Some(type_str) = sysfs_type {
        if let Ok(type_num) = type_str.trim().parse::<u32>() {
            match type_num {
                1 => return NetworkInterfaceType::Ethernet, // ARPHRD_ETHER
                772 => return NetworkInterfaceType::Loopback, // ARPHRD_LOOPBACK
                32 => return NetworkInterfaceType::Infiniband, // ARPHRD_INFINIBAND
                _ => {}
            }
        }
    }
    
    // Default to Ethernet for physical interfaces
    if name.starts_with("eth") || name.starts_with("en") {
        NetworkInterfaceType::Ethernet
    } else {
        NetworkInterfaceType::Unknown
    }
}
```

---

## Testing Requirements

### Unit Tests

| Test | Description |
|------|-------------|
| `test_parse_operstate` | Parse up/down states |
| `test_parse_sysfs_speed` | Parse speed values |
| `test_parse_ethtool_driver` | Parse ethtool -i output |
| `test_interface_type_detection` | Name to type mapping |
| `test_virtual_interface_detection` | Virtual interface check |

### Integration Tests

| Test | Platform | Description |
|------|----------|-------------|
| `test_network_detection` | Linux | Full network detection |
| `test_sysfs_network` | Linux | sysfs parsing |
| `test_ethtool_info` | Linux | ethtool integration |

---

## References

### Official Documentation

| Resource | URL |
|----------|-----|
| sysfs-class-net | https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-net |
| ethtool | https://man7.org/linux/man-pages/man8/ethtool.8.html |
| ip command | https://man7.org/linux/man-pages/man8/ip.8.html |
| Linux ARPHRD | https://github.com/torvalds/linux/blob/master/include/uapi/linux/if_arp.h |

---

## Changelog

| Date | Changes |
|------|---------|
| 2024-12-29 | Initial specification |
