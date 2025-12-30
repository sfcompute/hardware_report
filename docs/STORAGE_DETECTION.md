# Storage Detection Enhancement Plan

> **Category:** Critical Issue  
> **Target Platforms:** Linux (x86_64, aarch64)  
> **Priority:** Critical - Storage returns empty on ARM platforms

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Current Implementation](#current-implementation)
3. [Multi-Method Detection Strategy](#multi-method-detection-strategy)
4. [Entity Changes](#entity-changes)
5. [Detection Method Details](#detection-method-details)
6. [Adapter Implementation](#adapter-implementation)
7. [Parser Implementation](#parser-implementation)
8. [ARM/aarch64 Considerations](#armaarch64-considerations)
9. [Testing Requirements](#testing-requirements)
10. [References](#references)

---

## Problem Statement

### Current Issue

The `hardware_report` crate returns an empty storage array on ARM/aarch64 platforms:

```rust
// Current output on ARM
StorageInfo {
    devices: [],  // Empty!
}
```

Additionally, the current `StorageDevice` structure lacks critical fields for CMDB:

```rust
// Current struct - missing fields
pub struct StorageDevice {
    pub name: String,
    pub type_: String,   // String, not enum
    pub size: String,    // String, not numeric
    pub model: String,
    // Missing: serial_number, firmware_version, interface, etc.
}
```

### Impact

- No storage inventory on ARM platforms (DGX Spark, Graviton, etc.)
- CMDB cannot track storage serial numbers for asset management
- No firmware version for compliance tracking
- Size as string breaks automated capacity calculations

### Requirements

1. **Reliable detection on ARM/aarch64** - Primary target platform
2. **Numeric size fields** - `size_bytes: u64` and `size_gb: f64`
3. **Serial number extraction** - For asset tracking (may require privileges)
4. **Firmware version** - For compliance and update tracking
5. **Multi-method fallback** - sysfs primary, lsblk secondary, sysinfo tertiary

---

## Current Implementation

### Location

- **Entity:** `src/domain/entities.rs:207-225`
- **Adapter:** `src/adapters/secondary/system/linux.rs:153-170`
- **Parser:** `src/domain/parsers/storage.rs`

### Current Detection Flow

```
┌─────────────────────────────────────────────┐
│ LinuxSystemInfoProvider::get_storage_info() │
└─────────────────────────────────────────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │ lsblk -d -o          │
         │ NAME,SIZE,TYPE       │
         └──────────────────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │ Parse text output    │
         │ (whitespace split)   │
         └──────────────────────┘
                    │
                    ▼
              Return devices
              (may be empty!)
```

### Why It Fails on ARM

1. **lsblk output format differs** - Column ordering/presence varies
2. **No fallback** - If lsblk fails, no alternative tried
3. **Parsing assumes columns** - `parts[3]` for size fails if fewer columns
4. **No sysfs fallback** - Most reliable source not used

---

## Multi-Method Detection Strategy

### Detection Priority Chain

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      STORAGE DETECTION CHAIN                                 │
│                                                                              │
│  Priority 1: sysfs /sys/block (Linux)                                       │
│  ├── Most reliable across architectures                                     │
│  ├── Direct kernel interface                                                │
│  ├── Works on x86_64 and aarch64                                           │
│  ├── Serial/firmware may require elevated privileges                        │
│  └── Paths:                                                                 │
│      ├── /sys/block/{dev}/size                                             │
│      ├── /sys/block/{dev}/device/model                                     │
│      ├── /sys/block/{dev}/device/serial                                    │
│      ├── /sys/block/{dev}/device/firmware_rev                              │
│      └── /sys/block/{dev}/queue/rotational                                 │
│                          │                                                   │
│                          ▼ (enrich with additional data)                    │
│  Priority 2: lsblk JSON output                                              │
│  ├── Structured output format                                               │
│  ├── Additional fields (FSTYPE, MOUNTPOINT)                                │
│  └── Command: lsblk -J -o NAME,SIZE,TYPE,MODEL,SERIAL,ROTA                 │
│                          │                                                   │
│                          ▼ (if lsblk unavailable)                           │
│  Priority 3: NVMe CLI (for NVMe devices)                                    │
│  ├── Detailed NVMe information                                              │
│  ├── Firmware version                                                       │
│  └── Command: nvme list -o json                                            │
│                          │                                                   │
│                          ▼ (cross-platform fallback)                        │
│  Priority 4: sysinfo crate                                                  │
│  ├── Cross-platform disk enumeration                                        │
│  ├── Limited metadata                                                       │
│  └── Good for basic size/mount info                                        │
│                          │                                                   │
│                          ▼ (for serial numbers if other methods fail)       │
│  Priority 5: smartctl (SMART data)                                          │
│  ├── Serial number                                                          │
│  ├── Firmware version                                                       │
│  ├── Health status                                                          │
│  └── Requires smartmontools package                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Method Capabilities Matrix

| Method | Size | Model | Serial | Firmware | Type | Rotational | NVMe-specific |
|--------|------|-------|--------|----------|------|------------|---------------|
| sysfs | Yes | Yes | Maybe* | Maybe* | Yes | Yes | Partial |
| lsblk | Yes | Yes | Maybe* | No | Yes | Yes | No |
| nvme-cli | Yes | Yes | Yes | Yes | NVMe only | No | Yes |
| sysinfo | Yes | No | No | No | Limited | No | No |
| smartctl | Yes | Yes | Yes | Yes | Yes | Yes | Yes |

*Requires elevated privileges or specific kernel configuration

---

## Entity Changes

### New StorageType Enum

```rust
// src/domain/entities.rs

/// Storage device type classification
///
/// Classifies storage devices by their underlying technology and interface.
///
/// # Detection
///
/// Type is determined by:
/// 1. Device name prefix (nvme*, sd*, mmcblk*)
/// 2. sysfs rotational flag
/// 3. Interface type from sysfs
///
/// # References
///
/// - [Linux Block Devices](https://www.kernel.org/doc/html/latest/block/index.html)
/// - [NVMe Specification](https://nvmexpress.org/specifications/)
/// - [SATA Specification](https://sata-io.org/developers/sata-revision-3-5-specification)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StorageType {
    /// NVMe solid-state drive
    ///
    /// Detected by device name starting with "nvme" or interface type.
    /// Typically provides highest performance.
    Nvme,
    
    /// SATA/SAS solid-state drive
    ///
    /// Detected by rotational=0 on sd* devices.
    Ssd,
    
    /// Hard disk drive (rotational media)
    ///
    /// Detected by rotational=1 on sd* devices.
    Hdd,
    
    /// Embedded MMC storage
    ///
    /// Common on ARM platforms (eMMC). Detected by mmcblk* device name.
    Emmc,
    
    /// Virtual or memory-backed device
    ///
    /// Includes RAM disks, loop devices, and device-mapper devices.
    Virtual,
    
    /// Unknown or unclassified storage type
    Unknown,
}

impl StorageType {
    /// Determine storage type from device name and rotational flag
    ///
    /// # Arguments
    ///
    /// * `device_name` - Block device name (e.g., "nvme0n1", "sda", "mmcblk0")
    /// * `is_rotational` - Whether the device uses rotational media
    ///
    /// # Example
    ///
    /// ```
    /// use hardware_report::StorageType;
    ///
    /// assert_eq!(StorageType::from_device("nvme0n1", false), StorageType::Nvme);
    /// assert_eq!(StorageType::from_device("sda", false), StorageType::Ssd);
    /// assert_eq!(StorageType::from_device("sda", true), StorageType::Hdd);
    /// assert_eq!(StorageType::from_device("mmcblk0", false), StorageType::Emmc);
    /// ```
    pub fn from_device(device_name: &str, is_rotational: bool) -> Self {
        if device_name.starts_with("nvme") {
            StorageType::Nvme
        } else if device_name.starts_with("mmcblk") {
            StorageType::Emmc
        } else if device_name.starts_with("loop") 
            || device_name.starts_with("ram") 
            || device_name.starts_with("dm-") 
        {
            StorageType::Virtual
        } else if is_rotational {
            StorageType::Hdd
        } else if device_name.starts_with("sd") || device_name.starts_with("vd") {
            StorageType::Ssd
        } else {
            StorageType::Unknown
        }
    }
    
    /// Get human-readable display name
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

impl std::fmt::Display for StorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
```

### New StorageDevice Structure

```rust
// src/domain/entities.rs

/// Storage device information
///
/// Represents a block storage device detected in the system. Provides both
/// numeric and string representations of size for flexibility.
///
/// # Detection Methods
///
/// Storage devices are detected using multiple methods in priority order:
/// 1. **sysfs** - `/sys/block` interface (most reliable on Linux)
/// 2. **lsblk** - Block device listing command
/// 3. **nvme-cli** - NVMe-specific tooling
/// 4. **sysinfo** - Cross-platform crate fallback
/// 5. **smartctl** - SMART data for enrichment
///
/// # Filtering
///
/// Virtual devices (loop, ram, dm-*) are excluded by default. Use the
/// `include_virtual` configuration option to include them.
///
/// # Privileges
///
/// Some fields (serial_number, firmware_version) may require elevated
/// privileges (root/sudo) to read. These will be `None` if inaccessible.
///
/// # Example
///
/// ```
/// use hardware_report::StorageDevice;
///
/// // Size is available in multiple formats
/// let size_tb = device.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0);
/// let size_gb = device.size_gb;  // Pre-calculated convenience field
/// ```
///
/// # References
///
/// - [Linux sysfs block ABI](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-block)
/// - [NVMe CLI](https://github.com/linux-nvme/nvme-cli)
/// - [smartmontools](https://www.smartmontools.org/)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageDevice {
    /// Block device name (e.g., "nvme0n1", "sda")
    ///
    /// This is the kernel device name without the `/dev/` prefix.
    pub name: String,
    
    /// Full device path (e.g., "/dev/nvme0n1")
    pub device_path: String,
    
    /// Storage device type classification
    pub device_type: StorageType,
    
    /// Device size in bytes
    ///
    /// Calculated from sysfs `size` (512-byte sectors) or other sources.
    /// This is the raw capacity, not the formatted/usable capacity.
    pub size_bytes: u64,
    
    /// Device size in gigabytes (convenience field)
    ///
    /// Calculated as `size_bytes / (1024^3)`. Note this uses binary GB (GiB).
    pub size_gb: f64,
    
    /// Device size in terabytes (convenience field)
    ///
    /// Calculated as `size_bytes / (1024^4)`. Note this uses binary TB (TiB).
    pub size_tb: f64,
    
    /// Device model name
    ///
    /// From sysfs `/sys/block/{dev}/device/model` or equivalent.
    /// May include trailing whitespace from hardware.
    pub model: String,
    
    /// Device serial number
    ///
    /// Important for asset tracking and CMDB inventory.
    ///
    /// # Note
    ///
    /// May require elevated privileges to read. Returns `None` if
    /// inaccessible or not available.
    ///
    /// # Sources
    ///
    /// - sysfs: `/sys/block/{dev}/device/serial`
    /// - NVMe: `/sys/class/nvme/{ctrl}/serial`
    /// - smartctl: `smartctl -i /dev/{dev}`
    pub serial_number: Option<String>,
    
    /// Device firmware version
    ///
    /// Important for compliance tracking and identifying devices
    /// that need firmware updates.
    ///
    /// # Sources
    ///
    /// - sysfs: `/sys/block/{dev}/device/firmware_rev`
    /// - NVMe: `/sys/class/nvme/{ctrl}/firmware_rev`
    /// - smartctl: `smartctl -i /dev/{dev}`
    pub firmware_version: Option<String>,
    
    /// Interface type
    ///
    /// Examples: "NVMe", "SATA", "SAS", "USB", "eMMC", "virtio"
    pub interface: String,
    
    /// Whether the device uses rotational media
    ///
    /// - `true` = HDD (spinning platters)
    /// - `false` = SSD/NVMe/eMMC (solid state)
    ///
    /// Read from sysfs `/sys/block/{dev}/queue/rotational`.
    pub is_rotational: bool,
    
    /// World Wide Name (WWN) if available
    ///
    /// A globally unique identifier for the device. Format varies:
    /// - SATA: NAA format (e.g., "0x5000c5004567890a")
    /// - NVMe: EUI-64 or NGUID
    ///
    /// # Sources
    ///
    /// - sysfs: `/sys/block/{dev}/device/wwid`
    /// - lsblk: WWN column
    pub wwn: Option<String>,
    
    /// NVMe Namespace ID (NVMe devices only)
    ///
    /// For NVMe devices, this identifies the namespace within the controller.
    /// Typically 1 for single-namespace devices.
    pub nvme_namespace: Option<u32>,
    
    /// SMART health status
    ///
    /// Indicates overall device health based on SMART data.
    /// Values: "PASSED", "FAILED", or `None` if unavailable.
    pub smart_status: Option<String>,
    
    /// Transport protocol
    ///
    /// More specific than `interface`. Examples:
    /// - "PCIe 4.0 x4" (NVMe)
    /// - "SATA 6Gb/s"
    /// - "SAS 12Gb/s"
    pub transport: Option<String>,
    
    /// Logical block size in bytes
    ///
    /// Typically 512 or 4096. Affects alignment requirements.
    pub block_size: Option<u32>,
    
    /// Physical block size in bytes
    ///
    /// May differ from logical block size (e.g., 4Kn drives).
    pub physical_block_size: Option<u32>,
    
    /// Detection method that discovered this device
    ///
    /// One of: "sysfs", "lsblk", "nvme-cli", "sysinfo", "smartctl"
    pub detection_method: String,
}

impl Default for StorageDevice {
    fn default() -> Self {
        Self {
            name: String::new(),
            device_path: String::new(),
            device_type: StorageType::Unknown,
            size_bytes: 0,
            size_gb: 0.0,
            size_tb: 0.0,
            model: String::new(),
            serial_number: None,
            firmware_version: None,
            interface: "Unknown".to_string(),
            is_rotational: false,
            wwn: None,
            nvme_namespace: None,
            smart_status: None,
            transport: None,
            block_size: None,
            physical_block_size: None,
            detection_method: String::new(),
        }
    }
}

impl StorageDevice {
    /// Calculate size fields from bytes
    ///
    /// Updates `size_gb` and `size_tb` based on `size_bytes`.
    pub fn calculate_size_fields(&mut self) {
        const GB: f64 = 1024.0 * 1024.0 * 1024.0;
        const TB: f64 = GB * 1024.0;
        self.size_gb = self.size_bytes as f64 / GB;
        self.size_tb = self.size_bytes as f64 / TB;
    }
}
```

---

## Detection Method Details

### Method 1: sysfs /sys/block (Primary)

**When:** Linux systems (always attempted first)

**sysfs paths for each device:**

```
/sys/block/{device}/
├── size                    # Size in 512-byte sectors
├── queue/
│   ├── rotational          # 0=SSD, 1=HDD
│   ├── logical_block_size  # Logical block size
│   └── physical_block_size # Physical block size
├── device/
│   ├── model               # Device model (may have trailing spaces)
│   ├── vendor              # Device vendor
│   ├── serial              # Serial number (may need root)
│   ├── firmware_rev        # Firmware version
│   └── wwid                # World Wide Name
└── ... (other attributes)
```

**NVMe-specific paths:**

```
/sys/class/nvme/{controller}/
├── serial                  # Controller serial number
├── model                   # Controller model
├── firmware_rev            # Firmware revision
└── transport               # Transport type (pcie, tcp, rdma)

/sys/class/nvme/{controller}/nvme{X}n{Y}/
├── size                    # Namespace size
├── wwid                    # Namespace WWID
└── ...
```

**Size calculation:**

```rust
// sysfs reports size in 512-byte sectors
let sectors: u64 = read_sysfs_file("/sys/block/sda/size")?.parse()?;
let size_bytes = sectors * 512;
```

**References:**
- [sysfs-block ABI](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-block)
- [sysfs-class-nvme](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-nvme)

---

### Method 2: lsblk JSON Output

**When:** Enrichment after sysfs, or if sysfs incomplete

**Command:**
```bash
lsblk -J -o NAME,SIZE,TYPE,MODEL,SERIAL,ROTA,TRAN,WWN,FSTYPE,MOUNTPOINT -b
```

**Output format:**
```json
{
   "blockdevices": [
      {
         "name": "nvme0n1",
         "size": 2000398934016,
         "type": "disk",
         "model": "Samsung SSD 980 PRO 2TB",
         "serial": "S5GXNF0N123456",
         "rota": false,
         "tran": "nvme",
         "wwn": "eui.0025385b21404321"
      }
   ]
}
```

**Note:** The `-b` flag outputs size in bytes, avoiding parsing human-readable formats.

**References:**
- [lsblk man page](https://man7.org/linux/man-pages/man8/lsblk.8.html)
- [util-linux source](https://github.com/util-linux/util-linux)

---

### Method 3: NVMe CLI

**When:** NVMe devices detected, nvme-cli available

**Command:**
```bash
nvme list -o json
```

**Output format:**
```json
{
  "Devices": [
    {
      "DevicePath": "/dev/nvme0n1",
      "Firmware": "1B2QGXA7",
      "ModelNumber": "Samsung SSD 980 PRO 2TB",
      "SerialNumber": "S5GXNF0N123456",
      "PhysicalSize": 2000398934016,
      "UsedBytes": 1500000000000
    }
  ]
}
```

**References:**
- [nvme-cli GitHub](https://github.com/linux-nvme/nvme-cli)
- [NVMe Specification](https://nvmexpress.org/specifications/)

---

### Method 4: sysinfo Crate

**When:** Cross-platform fallback, or other methods unavailable

**Usage:**
```rust
use sysinfo::Disks;

let disks = Disks::new_with_refreshed_list();
for disk in disks.iter() {
    let name = disk.name().to_string_lossy();
    let size = disk.total_space();
    let fs_type = disk.file_system().to_string_lossy();
    let mount_point = disk.mount_point();
}
```

**Limitations:**
- Reports mounted filesystems, not raw block devices
- No serial number or firmware version
- Limited device type detection

**References:**
- [sysinfo crate](https://docs.rs/sysinfo)

---

### Method 5: smartctl

**When:** Serial/firmware needed and not available from sysfs

**Command:**
```bash
smartctl -i /dev/sda --json
```

**Output format:**
```json
{
  "model_name": "Samsung SSD 870 EVO 2TB",
  "serial_number": "S5XXNX0N123456",
  "firmware_version": "SVT01B6Q",
  "smart_status": {
    "passed": true
  }
}
```

**Note:** Requires `smartmontools` package and often root privileges.

**References:**
- [smartmontools](https://www.smartmontools.org/)
- [smartctl man page](https://linux.die.net/man/8/smartctl)

---

## Adapter Implementation

### File: `src/adapters/secondary/system/linux.rs`

```rust
// Pseudocode for new implementation

impl SystemInfoProvider for LinuxSystemInfoProvider {
    async fn get_storage_info(&self) -> Result<StorageInfo, SystemError> {
        let mut devices = Vec::new();
        
        // Method 1: sysfs (primary)
        match self.detect_storage_sysfs().await {
            Ok(sysfs_devices) => {
                log::debug!("Found {} devices via sysfs", sysfs_devices.len());
                devices = sysfs_devices;
            }
            Err(e) => {
                log::warn!("sysfs storage detection failed: {}", e);
            }
        }
        
        // Method 2: lsblk enrichment
        if let Ok(lsblk_devices) = self.detect_storage_lsblk().await {
            self.merge_storage_info(&mut devices, lsblk_devices);
        }
        
        // Method 3: NVMe CLI enrichment (for NVMe devices)
        if devices.iter().any(|d| d.device_type == StorageType::Nvme) {
            if let Ok(nvme_devices) = self.detect_storage_nvme_cli().await {
                self.merge_storage_info(&mut devices, nvme_devices);
            }
        }
        
        // Method 4: sysinfo fallback (if no devices found)
        if devices.is_empty() {
            if let Ok(sysinfo_devices) = self.detect_storage_sysinfo().await {
                devices = sysinfo_devices;
            }
        }
        
        // Method 5: smartctl enrichment (for missing serial/firmware)
        for device in &mut devices {
            if device.serial_number.is_none() || device.firmware_version.is_none() {
                if let Ok(smart_info) = self.get_smart_info(&device.name).await {
                    self.merge_smart_info(device, smart_info);
                }
            }
        }
        
        // Filter out virtual devices (configurable)
        devices.retain(|d| d.device_type != StorageType::Virtual);
        
        // Calculate convenience fields
        for device in &mut devices {
            device.calculate_size_fields();
        }
        
        Ok(StorageInfo { devices })
    }
}
```

### Helper Methods

```rust
impl LinuxSystemInfoProvider {
    /// Detect storage devices via sysfs
    ///
    /// Primary detection method for Linux. Reads directly from
    /// `/sys/block` kernel interface.
    ///
    /// # References
    ///
    /// - [sysfs-block ABI](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-block)
    async fn detect_storage_sysfs(&self) -> Result<Vec<StorageDevice>, SystemError> {
        // Read /sys/block directory
        // For each entry, read attributes
        // Build StorageDevice
        todo!()
    }
    
    /// Detect storage devices via lsblk command
    ///
    /// # Requirements
    ///
    /// - `lsblk` must be in PATH (util-linux package)
    async fn detect_storage_lsblk(&self) -> Result<Vec<StorageDevice>, SystemError> {
        let cmd = SystemCommand::new("lsblk")
            .args(&["-J", "-o", "NAME,SIZE,TYPE,MODEL,SERIAL,ROTA,TRAN,WWN", "-b"])
            .timeout(Duration::from_secs(10));
        
        let output = self.command_executor.execute(&cmd).await?;
        parse_lsblk_json(&output.stdout).map_err(SystemError::ParseError)
    }
    
    /// Detect NVMe devices via nvme-cli
    ///
    /// # Requirements
    ///
    /// - `nvme` must be in PATH (nvme-cli package)
    async fn detect_storage_nvme_cli(&self) -> Result<Vec<StorageDevice>, SystemError> {
        let cmd = SystemCommand::new("nvme")
            .args(&["list", "-o", "json"])
            .timeout(Duration::from_secs(10));
        
        let output = self.command_executor.execute(&cmd).await?;
        parse_nvme_list_json(&output.stdout).map_err(SystemError::ParseError)
    }
    
    /// Detect storage via sysinfo crate
    ///
    /// Cross-platform fallback with limited information.
    async fn detect_storage_sysinfo(&self) -> Result<Vec<StorageDevice>, SystemError> {
        use sysinfo::Disks;
        
        let disks = Disks::new_with_refreshed_list();
        let mut devices = Vec::new();
        
        for disk in disks.iter() {
            // Convert sysinfo disk to StorageDevice
            // ...
        }
        
        Ok(devices)
    }
    
    /// Get SMART information for a device
    ///
    /// # Requirements
    ///
    /// - `smartctl` must be in PATH (smartmontools package)
    /// - Often requires root privileges
    async fn get_smart_info(&self, device_name: &str) -> Result<SmartInfo, SystemError> {
        let cmd = SystemCommand::new("smartctl")
            .args(&["-i", "--json", &format!("/dev/{}", device_name)])
            .timeout(Duration::from_secs(10));
        
        let output = self.command_executor.execute_with_privileges(&cmd).await?;
        parse_smartctl_json(&output.stdout).map_err(SystemError::ParseError)
    }
    
    /// Merge storage info from secondary source
    ///
    /// Matches devices by name and fills in missing fields.
    fn merge_storage_info(&self, primary: &mut Vec<StorageDevice>, secondary: Vec<StorageDevice>) {
        for sec_dev in secondary {
            if let Some(pri_dev) = primary.iter_mut().find(|d| d.name == sec_dev.name) {
                // Fill in missing fields
                if pri_dev.serial_number.is_none() {
                    pri_dev.serial_number = sec_dev.serial_number;
                }
                if pri_dev.firmware_version.is_none() {
                    pri_dev.firmware_version = sec_dev.firmware_version;
                }
                // ... other fields
            } else {
                // Device not in primary, add it
                primary.push(sec_dev);
            }
        }
    }
}
```

---

## Parser Implementation

### File: `src/domain/parsers/storage.rs`

```rust
//! Storage information parsing functions
//!
//! This module provides pure parsing functions for storage device information
//! from various sources. All functions take string input and return parsed
//! results without performing I/O.
//!
//! # Supported Formats
//!
//! - sysfs file contents
//! - lsblk JSON output
//! - nvme-cli JSON output
//! - smartctl JSON output
//!
//! # References
//!
//! - [Linux sysfs block](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-block)
//! - [lsblk JSON format](https://github.com/util-linux/util-linux)

use crate::domain::{StorageDevice, StorageType};

/// Parse sysfs size file to bytes
///
/// # Arguments
///
/// * `content` - Content of `/sys/block/{dev}/size` file
///
/// # Returns
///
/// Size in bytes. sysfs reports size in 512-byte sectors.
///
/// # Example
///
/// ```
/// use hardware_report::domain::parsers::storage::parse_sysfs_size;
///
/// let size_bytes = parse_sysfs_size("3907029168").unwrap();
/// assert_eq!(size_bytes, 3907029168 * 512); // ~2TB
/// ```
pub fn parse_sysfs_size(content: &str) -> Result<u64, String> {
    let sectors: u64 = content
        .trim()
        .parse()
        .map_err(|e| format!("Failed to parse size: {}", e))?;
    Ok(sectors * 512)
}

/// Parse sysfs rotational flag
///
/// # Arguments
///
/// * `content` - Content of `/sys/block/{dev}/queue/rotational` file
///
/// # Returns
///
/// `true` if device is rotational (HDD), `false` for SSD/NVMe.
///
/// # Example
///
/// ```
/// use hardware_report::domain::parsers::storage::parse_sysfs_rotational;
///
/// assert_eq!(parse_sysfs_rotational("1"), true);  // HDD
/// assert_eq!(parse_sysfs_rotational("0"), false); // SSD
/// ```
pub fn parse_sysfs_rotational(content: &str) -> bool {
    content.trim() == "1"
}

/// Parse lsblk JSON output
///
/// # Arguments
///
/// * `output` - JSON output from `lsblk -J -o NAME,SIZE,TYPE,MODEL,SERIAL,ROTA,TRAN,WWN -b`
///
/// # Returns
///
/// Vector of storage devices parsed from lsblk output.
///
/// # Expected Format
///
/// ```json
/// {
///    "blockdevices": [
///       {"name": "sda", "size": 1000204886016, "type": "disk", ...}
///    ]
/// }
/// ```
///
/// # References
///
/// - [lsblk man page](https://man7.org/linux/man-pages/man8/lsblk.8.html)
pub fn parse_lsblk_json(output: &str) -> Result<Vec<StorageDevice>, String> {
    todo!()
}

/// Parse nvme-cli list JSON output
///
/// # Arguments
///
/// * `output` - JSON output from `nvme list -o json`
///
/// # Returns
///
/// Vector of NVMe storage devices.
///
/// # Expected Format
///
/// ```json
/// {
///   "Devices": [
///     {"DevicePath": "/dev/nvme0n1", "SerialNumber": "...", ...}
///   ]
/// }
/// ```
///
/// # References
///
/// - [nvme-cli](https://github.com/linux-nvme/nvme-cli)
pub fn parse_nvme_list_json(output: &str) -> Result<Vec<StorageDevice>, String> {
    todo!()
}

/// Parse smartctl JSON output
///
/// # Arguments
///
/// * `output` - JSON output from `smartctl -i --json /dev/{device}`
///
/// # Returns
///
/// Partial storage device with SMART information.
///
/// # References
///
/// - [smartmontools](https://www.smartmontools.org/)
pub fn parse_smartctl_json(output: &str) -> Result<StorageDevice, String> {
    todo!()
}

/// Check if device name is a virtual device
///
/// # Arguments
///
/// * `name` - Block device name (e.g., "sda", "loop0", "dm-0")
///
/// # Returns
///
/// `true` if the device is virtual (loop, ram, dm-*, etc.)
pub fn is_virtual_device(name: &str) -> bool {
    name.starts_with("loop")
        || name.starts_with("ram")
        || name.starts_with("dm-")
        || name.starts_with("zram")
        || name.starts_with("nbd")
}
```

---

## ARM/aarch64 Considerations

### Known ARM Platforms

| Platform | Storage Type | Notes |
|----------|--------------|-------|
| NVIDIA DGX Spark | NVMe | Grace Hopper, ARM Neoverse |
| AWS Graviton | NVMe, EBS | Various instance storage |
| Ampere Altra | NVMe | Server-class ARM |
| Raspberry Pi | SD/eMMC | mmcblk* devices |
| Apple Silicon | NVMe | Not Linux target |

### ARM-Specific sysfs Paths

Some ARM platforms use slightly different sysfs layouts:

```
# Standard path
/sys/block/nvme0n1/device/serial

# Some ARM platforms
/sys/class/nvme/nvme0/serial

# eMMC on ARM
/sys/block/mmcblk0/device/cid  # Contains serial in CID register
```

### eMMC CID Parsing

eMMC devices encode serial number in the CID (Card Identification) register:

```rust
/// Parse eMMC CID to extract serial number
///
/// # Arguments
///
/// * `cid` - Content of `/sys/block/mmcblk*/device/cid` (32 hex chars)
///
/// # References
///
/// - [JEDEC eMMC Standard](https://www.jedec.org/)
pub fn parse_emmc_cid_serial(cid: &str) -> Option<String> {
    // CID format: MID(1) + OID(2) + PNM(6) + PRV(1) + PSN(4) + MDT(2) + CRC(1)
    // PSN (Product Serial Number) is bytes 10-13
    if cid.len() < 32 {
        return None;
    }
    let serial_hex = &cid[20..28]; // PSN bytes
    Some(serial_hex.to_uppercase())
}
```

### Testing on ARM

```bash
# Test sysfs availability
ls -la /sys/block/
cat /sys/block/*/size

# Check for NVMe
ls -la /sys/class/nvme/

# Check for eMMC
ls -la /sys/block/mmcblk*
```

---

## Testing Requirements

### Unit Tests

| Test | Description |
|------|-------------|
| `test_parse_sysfs_size` | Parse sector count to bytes |
| `test_parse_sysfs_rotational` | Parse rotational flag |
| `test_parse_lsblk_json` | Parse lsblk JSON output |
| `test_parse_nvme_list_json` | Parse nvme-cli JSON |
| `test_parse_smartctl_json` | Parse smartctl JSON |
| `test_storage_type_from_device` | Device name to type mapping |
| `test_is_virtual_device` | Virtual device detection |
| `test_parse_emmc_cid` | eMMC CID serial extraction |

### Integration Tests

| Test | Platform | Description |
|------|----------|-------------|
| `test_sysfs_detection` | Linux | Full sysfs detection |
| `test_lsblk_detection` | Linux | lsblk fallback |
| `test_nvme_detection` | Linux + NVMe | NVMe-specific detection |
| `test_arm_detection` | aarch64 | ARM platform detection |
| `test_emmc_detection` | aarch64 | eMMC device detection |

### Test Hardware Matrix

| Platform | Storage | Test Type |
|----------|---------|-----------|
| x86_64 Linux | NVMe | CI + Manual |
| x86_64 Linux | SATA SSD | Manual |
| x86_64 Linux | HDD | Manual |
| aarch64 Linux (DGX Spark) | NVMe | Manual |
| aarch64 Linux (Graviton) | NVMe | CI |
| aarch64 Linux (RPi) | eMMC | Manual |

---

## References

### Official Documentation

| Resource | URL |
|----------|-----|
| Linux sysfs block ABI | https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-block |
| Linux sysfs nvme ABI | https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-nvme |
| NVMe Specification | https://nvmexpress.org/specifications/ |
| JEDEC eMMC Standard | https://www.jedec.org/ |
| smartmontools | https://www.smartmontools.org/ |
| nvme-cli | https://github.com/linux-nvme/nvme-cli |
| lsblk (util-linux) | https://github.com/util-linux/util-linux |

### Crate Documentation

| Crate | URL |
|-------|-----|
| sysinfo | https://docs.rs/sysinfo |
| serde_json | https://docs.rs/serde_json |

### Kernel Documentation

| Path | Description |
|------|-------------|
| `/sys/block/` | Block device sysfs |
| `/sys/class/nvme/` | NVMe controller class |
| `/proc/partitions` | Partition information |
| `/dev/disk/by-id/` | Persistent device naming |

---

## Changelog

| Date | Changes |
|------|---------|
| 2024-12-29 | Initial specification |
