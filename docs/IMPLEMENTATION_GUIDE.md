# Implementation Guide: Learn by Doing

> **Purpose:** Step-by-step implementation guide with LeetCode patterns and real-world connections  
> **Learning Style:** Type it yourself with detailed explanations

## Table of Contents

1. [Overview](#overview)
2. [LeetCode Patterns Used](#leetcode-patterns-used)
3. [Implementation Order](#implementation-order)
4. [Step 1: Storage Enhancements](#step-1-storage-enhancements)
5. [Step 2: CPU Enhancements](#step-2-cpu-enhancements)
6. [Step 3: GPU Enhancements](#step-3-gpu-enhancements)
7. [Step 4: Memory Enhancements](#step-4-memory-enhancements)
8. [Step 5: Network Enhancements](#step-5-network-enhancements)
9. [Step 6: Cargo.toml Updates](#step-6-cargotoml-updates)

---

## Overview

This guide walks you through implementing each enhancement with:
- **Commented code** explaining every decision
- **LeetCode pattern callouts** showing real-world applications
- **Why it matters** for CMDB/inventory systems

### How to Use This Guide

1. Read each section's explanation
2. Type the code yourself (don't copy-paste!)
3. Run `cargo check` after each change
4. Run `cargo test` to verify
5. Understand the LeetCode pattern connection

---

## LeetCode Patterns Used

This project uses several classic algorithm patterns. Here's how they map:

| Pattern | LeetCode Examples | Where Used Here |
|---------|-------------------|-----------------|
| **Chain of Responsibility** | - | Multi-method detection (try method 1, fallback to 2, etc.) |
| **Strategy Pattern** | - | Different parsers for different data sources |
| **Builder Pattern** | - | Constructing complex structs with defaults |
| **Two Pointers / Sliding Window** | LC #3, #76, #567 | Parsing delimited strings |
| **Hash Map for Lookups** | LC #1, #49, #242 | PCI vendor ID → vendor name mapping |
| **Tree/Graph Traversal** | LC #200, #547 | Walking sysfs directory tree |
| **String Parsing** | LC #8, #65, #468 | Parsing nvidia-smi output, sysfs files |
| **Merge/Combine Data** | LC #56, #88 | Merging GPU info from multiple sources |
| **Filter/Transform** | LC #283, #27 | Filtering virtual devices, transforming sizes |
| **State Machine** | LC #65, #10 | Parsing multi-line dmidecode output |
| **Adapter Pattern** | - | Platform-specific implementations behind traits |

---

## Implementation Order

Follow this exact order to avoid compilation errors:

```
1. entities.rs     - Add new types (StorageType, GpuVendor, etc.)
2. parsers/*.rs    - Add parsing functions (pure, no I/O)
3. linux.rs        - Update adapter to use new parsers
4. Cargo.toml      - Add new dependencies (if needed)
5. tests           - Verify everything works
```

**Why this order?**
- Entities are dependencies for everything else
- Parsers depend only on entities (pure functions)
- Adapters depend on both entities and parsers
- Tests depend on all of the above

---

## Step 1: Storage Enhancements

### 1.1 Add StorageType Enum to entities.rs

**File:** `src/domain/entities.rs`

**Where:** Add after line 205 (after MemoryModule), before StorageInfo

**LeetCode Pattern:** This is similar to **categorization problems** like LC #49 (Group Anagrams) 
where you classify items into buckets. Here we classify storage devices by type.

```rust
// =============================================================================
// STORAGE TYPE ENUM
// =============================================================================
// 
// WHY: We need to categorize storage devices so CMDB consumers can:
//   1. Filter by type (show only SSDs)
//   2. Calculate capacity by category
//   3. Apply different monitoring thresholds
//
// LEETCODE CONNECTION: This is the "categorization" pattern seen in:
//   - LC #49 Group Anagrams: group strings by sorted chars
//   - LC #347 Top K Frequent: group by frequency
//   - Here: group storage by technology type
//
// PATTERN: Enum with associated functions for classification
// =============================================================================

/// Storage device type classification.
///
/// Classifies storage devices by their underlying technology and interface.
/// This enables filtering, capacity planning, and performance expectations.
///
/// # Detection Logic
///
/// Type is determined by examining (in order):
/// 1. Device name prefix (`nvme*` → NVMe, `mmcblk*` → eMMC)
/// 2. sysfs rotational flag (`0` = solid state, `1` = spinning)
/// 3. Interface type from sysfs
///
/// # Example
///
/// ```rust
/// use hardware_report::StorageType;
///
/// // Classify based on device name and rotational flag
/// let nvme = StorageType::from_device("nvme0n1", false);
/// assert_eq!(nvme, StorageType::Nvme);
///
/// let hdd = StorageType::from_device("sda", true);  // rotational=1
/// assert_eq!(hdd, StorageType::Hdd);
///
/// let ssd = StorageType::from_device("sda", false); // rotational=0
/// assert_eq!(ssd, StorageType::Ssd);
/// ```
///
/// # References
///
/// - [Linux Block Devices](https://www.kernel.org/doc/html/latest/block/index.html)
/// - [NVMe Specification](https://nvmexpress.org/specifications/)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StorageType {
    /// NVMe solid-state drive (PCIe interface).
    ///
    /// Highest performance storage. Detected by `nvme*` device name prefix.
    /// Uses PCIe lanes directly, bypassing SATA/SAS bottlenecks.
    Nvme,
    
    /// SATA/SAS solid-state drive.
    ///
    /// Detected by `rotational=0` on `sd*` devices.
    /// Limited by SATA (6 Gbps) or SAS (12/24 Gbps) interface.
    Ssd,
    
    /// Hard disk drive (rotational/spinning media).
    ///
    /// Detected by `rotational=1` on `sd*` devices.
    /// Mechanical seek time limits random I/O performance.
    Hdd,
    
    /// Embedded MMC storage.
    ///
    /// Common on ARM platforms (Raspberry Pi, embedded systems).
    /// Detected by `mmcblk*` device name prefix.
    Emmc,
    
    /// Virtual or memory-backed device.
    ///
    /// Includes loop devices, RAM disks, device-mapper.
    /// Usually filtered out for hardware inventory.
    Virtual,
    
    /// Unknown or unclassified storage type.
    Unknown,
}

// =============================================================================
// IMPLEMENTATION: StorageType classification logic
// =============================================================================
//
// LEETCODE CONNECTION: This classification logic is similar to:
//   - LC #68 Text Justification: pattern matching on input
//   - LC #722 Remove Comments: state-based string analysis
//
// The pattern here is: examine input characteristics → map to category
// =============================================================================

impl StorageType {
    /// Determine storage type from device name and rotational flag.
    ///
    /// This implements a decision tree:
    /// ```text
    ///                    device_name
    ///                         │
    ///         ┌───────────────┼───────────────┐
    ///         ▼               ▼               ▼
    ///     nvme*           mmcblk*          other
    ///       │                │               │
    ///       ▼                ▼               ▼
    ///     Nvme            Emmc         is_rotational?
    ///                                       │
    ///                              ┌────────┴────────┐
    ///                              ▼                 ▼
    ///                            true             false
    ///                              │                 │
    ///                              ▼                 ▼
    ///                            Hdd               Ssd
    /// ```
    ///
    /// # Arguments
    ///
    /// * `device_name` - Block device name (e.g., "nvme0n1", "sda", "mmcblk0")
    /// * `is_rotational` - Whether device uses rotational media (from sysfs)
    ///
    /// # Why This Order Matters
    ///
    /// We check name prefixes FIRST because:
    /// 1. NVMe devices always report rotational=0, but we want specific type
    /// 2. eMMC devices may not have rotational flag
    /// 3. Name-based detection is most reliable
    pub fn from_device(device_name: &str, is_rotational: bool) -> Self {
        // STEP 1: Check for NVMe (highest priority, most specific)
        // NVMe devices are named nvme{controller}n{namespace}
        // Example: nvme0n1, nvme1n1
        if device_name.starts_with("nvme") {
            return StorageType::Nvme;
        }
        
        // STEP 2: Check for eMMC (common on ARM)
        // eMMC devices are named mmcblk{N}
        // Example: mmcblk0, mmcblk1
        if device_name.starts_with("mmcblk") {
            return StorageType::Emmc;
        }
        
        // STEP 3: Check for virtual devices (filter these out usually)
        // These are not physical hardware
        if device_name.starts_with("loop")      // Loop devices (ISO mounts, etc.)
            || device_name.starts_with("ram")   // RAM disks
            || device_name.starts_with("dm-")   // Device mapper (LVM, LUKS)
            || device_name.starts_with("zram")  // Compressed RAM swap
            || device_name.starts_with("nbd")   // Network block device
        {
            return StorageType::Virtual;
        }
        
        // STEP 4: For sd* and vd* devices, use rotational flag
        // sd* = SCSI/SATA/SAS devices
        // vd* = VirtIO devices (VMs)
        if is_rotational {
            StorageType::Hdd
        } else if device_name.starts_with("sd") || device_name.starts_with("vd") {
            StorageType::Ssd
        } else {
            StorageType::Unknown
        }
    }
    
    /// Get human-readable display name.
    ///
    /// Useful for CLI output and logging.
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
    
    /// Check if this is a solid-state device (no moving parts).
    ///
    /// Useful for performance expectations and wear-leveling considerations.
    pub fn is_solid_state(&self) -> bool {
        matches!(self, StorageType::Nvme | StorageType::Ssd | StorageType::Emmc)
    }
}

// Implement Display trait for easy printing
// This allows: println!("{}", storage_type);
impl std::fmt::Display for StorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
```

### 1.2 Update StorageDevice Struct

**File:** `src/domain/entities.rs`

**Where:** Replace the existing `StorageDevice` struct (around line 214-225)

**LeetCode Pattern:** This uses the **Builder Pattern** concept where we have 
many optional fields with sensible defaults. Similar to how LC #146 LRU Cache 
needs to track multiple pieces of state for each entry.

```rust
// =============================================================================
// STORAGE DEVICE STRUCT
// =============================================================================
//
// WHY: The old struct had:
//   - type_: String  → Hard to filter/compare
//   - size: String   → "500 GB" can't be summed or compared
//   - No serial/firmware for asset tracking
//
// NEW: We add:
//   - device_type: StorageType enum → Easy filtering
//   - size_bytes: u64 → Math works!
//   - serial_number, firmware_version → Asset tracking
//
// LEETCODE CONNECTION: This is like the "design" problems:
//   - LC #146 LRU Cache: track multiple attributes per entry
//   - LC #380 Insert Delete GetRandom: need efficient lookups
//   - Here: need efficient queries by type, size, serial
// =============================================================================

/// Storage device information.
///
/// Represents a block storage device with comprehensive metadata for
/// CMDB inventory, capacity planning, and asset tracking.
///
/// # Detection Methods
///
/// Storage devices are detected using multiple methods (Chain of Responsibility):
/// 1. **sysfs** `/sys/block` - Primary, most reliable on Linux
/// 2. **lsblk** - Structured command output
/// 3. **nvme-cli** - NVMe-specific details
/// 4. **sysinfo** - Cross-platform fallback
/// 5. **smartctl** - SMART data enrichment
///
/// # Size Fields
///
/// Size is provided in multiple formats for convenience:
/// - `size_bytes` - Raw bytes (use for calculations)
/// - `size_gb` - Gigabytes as float (use for display)
/// - `size_tb` - Terabytes as float (use for large arrays)
///
/// # Example
///
/// ```rust
/// use hardware_report::{StorageDevice, StorageType};
///
/// let device = StorageDevice {
///     name: "nvme0n1".to_string(),
///     device_type: StorageType::Nvme,
///     size_bytes: 2_000_398_934_016, // ~2TB
///     ..Default::default()
/// };
///
/// // Calculate total across devices
/// let devices = vec![device];
/// let total_tb: f64 = devices.iter()
///     .map(|d| d.size_bytes as f64)
///     .sum::<f64>() / (1024.0_f64.powi(4));
/// ```
///
/// # References
///
/// - [Linux sysfs block](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-block)
/// - [NVMe CLI](https://github.com/linux-nvme/nvme-cli)
/// - [smartmontools](https://www.smartmontools.org/)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageDevice {
    // =========================================================================
    // IDENTIFICATION FIELDS
    // =========================================================================
    
    /// Block device name without /dev/ prefix.
    ///
    /// Examples: "nvme0n1", "sda", "mmcblk0"
    ///
    /// This is the kernel's name for the device, found in /sys/block/
    pub name: String,
    
    /// Full device path.
    ///
    /// Example: "/dev/nvme0n1"
    ///
    /// Use this when you need to open/read the device.
    #[serde(default)]
    pub device_path: String,
    
    // =========================================================================
    // TYPE AND CLASSIFICATION
    // =========================================================================
    
    /// Storage device type classification.
    ///
    /// Use this for filtering and categorization.
    /// This is the NEW preferred field.
    #[serde(default)]
    pub device_type: StorageType,
    
    /// Legacy type field as string.
    ///
    /// DEPRECATED: Use `device_type` instead.
    /// Kept for backward compatibility with existing consumers.
    #[serde(rename = "type")]
    pub type_: String,
    
    // =========================================================================
    // SIZE FIELDS
    // =========================================================================
    //
    // LEETCODE CONNECTION: Having multiple representations is like
    // LC #273 Integer to English Words - same data, different formats
    // =========================================================================
    
    /// Device size in bytes.
    ///
    /// PRIMARY SIZE FIELD - use this for calculations.
    ///
    /// Calculated from sysfs: sectors × 512 (sector size)
    #[serde(default)]
    pub size_bytes: u64,
    
    /// Device size in gigabytes (binary, 1 GB = 1024³ bytes).
    ///
    /// Convenience field for display. Pre-calculated from size_bytes.
    #[serde(default)]
    pub size_gb: f64,
    
    /// Device size in terabytes (binary, 1 TB = 1024⁴ bytes).
    ///
    /// Convenience field for large storage arrays.
    #[serde(default)]
    pub size_tb: f64,
    
    /// Legacy size as human-readable string.
    ///
    /// DEPRECATED: Use `size_bytes` for calculations.
    /// Example: "2 TB", "500 GB"
    pub size: String,
    
    // =========================================================================
    // HARDWARE IDENTIFICATION
    // =========================================================================
    
    /// Device model name.
    ///
    /// From sysfs `/sys/block/{dev}/device/model`
    /// May have trailing whitespace (hardware quirk).
    ///
    /// Example: "Samsung SSD 980 PRO 2TB"
    pub model: String,
    
    /// Device serial number.
    ///
    /// IMPORTANT for asset tracking and warranty.
    ///
    /// May require elevated privileges to read.
    /// Sources:
    /// - sysfs: `/sys/block/{dev}/device/serial`
    /// - NVMe: `/sys/class/nvme/{ctrl}/serial`
    /// - smartctl: `smartctl -i /dev/{dev}`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
    
    /// Device firmware version.
    ///
    /// IMPORTANT for compliance and update tracking.
    ///
    /// Sources:
    /// - sysfs: `/sys/block/{dev}/device/firmware_rev`
    /// - NVMe: `/sys/class/nvme/{ctrl}/firmware_rev`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<String>,
    
    // =========================================================================
    // INTERFACE AND TRANSPORT
    // =========================================================================
    
    /// Interface type.
    ///
    /// Examples: "NVMe", "SATA", "SAS", "USB", "eMMC", "virtio"
    #[serde(default)]
    pub interface: String,
    
    /// Whether device uses rotational media.
    ///
    /// - `true` = HDD (spinning platters, mechanical seek)
    /// - `false` = SSD/NVMe (solid state, no moving parts)
    ///
    /// From sysfs: `/sys/block/{dev}/queue/rotational`
    #[serde(default)]
    pub is_rotational: bool,
    
    /// World Wide Name (globally unique identifier).
    ///
    /// More persistent than serial in some cases.
    /// Format varies by protocol (NAA, EUI-64, NGUID).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wwn: Option<String>,
    
    // =========================================================================
    // NVME-SPECIFIC FIELDS
    // =========================================================================
    
    /// NVMe namespace ID (NVMe devices only).
    ///
    /// Identifies the namespace within the NVMe controller.
    /// Most consumer drives have a single namespace (1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nvme_namespace: Option<u32>,
    
    // =========================================================================
    // HEALTH AND MONITORING
    // =========================================================================
    
    /// SMART health status.
    ///
    /// Values: "PASSED", "FAILED", or None if unavailable.
    /// Requires smartctl or NVMe health query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smart_status: Option<String>,
    
    // =========================================================================
    // BLOCK SIZE INFORMATION
    // =========================================================================
    //
    // LEETCODE CONNECTION: Block sizes matter for alignment, similar to
    // LC #68 Text Justification where you need proper boundaries
    // =========================================================================
    
    /// Logical block size in bytes.
    ///
    /// Typically 512 or 4096. Affects I/O alignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logical_block_size: Option<u32>,
    
    /// Physical block size in bytes.
    ///
    /// May differ from logical (512e drives report 512 logical, 4096 physical).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_block_size: Option<u32>,
    
    // =========================================================================
    // METADATA
    // =========================================================================
    
    /// Detection method that discovered this device.
    ///
    /// Values: "sysfs", "lsblk", "nvme-cli", "sysinfo", "smartctl"
    ///
    /// Useful for debugging and understanding data quality.
    #[serde(default)]
    pub detection_method: String,
}

// =============================================================================
// DEFAULT IMPLEMENTATION
// =============================================================================
//
// LEETCODE CONNECTION: Default/Builder pattern is used in many design problems
// LC #146 LRU Cache, LC #355 Design Twitter - initialize with sensible defaults
// =============================================================================

impl Default for StorageType {
    fn default() -> Self {
        StorageType::Unknown
    }
}

impl Default for StorageDevice {
    fn default() -> Self {
        Self {
            name: String::new(),
            device_path: String::new(),
            device_type: StorageType::Unknown,
            type_: String::new(),
            size_bytes: 0,
            size_gb: 0.0,
            size_tb: 0.0,
            size: String::new(),
            model: String::new(),
            serial_number: None,
            firmware_version: None,
            interface: "Unknown".to_string(),
            is_rotational: false,
            wwn: None,
            nvme_namespace: None,
            smart_status: None,
            logical_block_size: None,
            physical_block_size: None,
            detection_method: String::new(),
        }
    }
}

impl StorageDevice {
    /// Calculate size_gb and size_tb from size_bytes.
    ///
    /// Call this after setting size_bytes to populate convenience fields.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut device = StorageDevice::default();
    /// device.size_bytes = 1_000_000_000_000; // 1 TB in bytes
    /// device.calculate_size_fields();
    /// assert!((device.size_gb - 931.32).abs() < 0.01); // Binary GB
    /// ```
    pub fn calculate_size_fields(&mut self) {
        // Use binary units (1024-based) as is standard for storage
        const KB: f64 = 1024.0;
        const GB: f64 = KB * KB * KB;           // 1,073,741,824
        const TB: f64 = KB * KB * KB * KB;      // 1,099,511,627,776
        
        self.size_gb = self.size_bytes as f64 / GB;
        self.size_tb = self.size_bytes as f64 / TB;
        
        // Also set the legacy string field
        if self.size_tb >= 1.0 {
            self.size = format!("{:.2} TB", self.size_tb);
        } else if self.size_gb >= 1.0 {
            self.size = format!("{:.2} GB", self.size_gb);
        } else {
            self.size = format!("{} bytes", self.size_bytes);
        }
    }
    
    /// Create device path from name.
    ///
    /// Convenience method to set device_path from name.
    pub fn set_device_path(&mut self) {
        if !self.name.is_empty() && self.device_path.is_empty() {
            self.device_path = format!("/dev/{}", self.name);
        }
    }
}
```

### 1.3 Add Storage Parser Functions

**File:** `src/domain/parsers/storage.rs`

**Where:** Add these functions to the existing file

**LeetCode Pattern:** String parsing here is like LC #8 (String to Integer), 
LC #468 (Validate IP Address), and LC #65 (Valid Number) - parsing structured 
text with edge cases.

```rust
// =============================================================================
// STORAGE PARSER FUNCTIONS
// =============================================================================
//
// These are PURE FUNCTIONS - they take strings in, return parsed data out.
// No I/O, no side effects. This makes them easy to test.
//
// ARCHITECTURE: These live in the DOMAIN layer (ports and adapters pattern)
// The ADAPTER layer (linux.rs) calls these after reading from sysfs/commands.
// =============================================================================

use crate::domain::{StorageDevice, StorageType};

// =============================================================================
// SYSFS SIZE PARSING
// =============================================================================
//
// LEETCODE CONNECTION: This is classic string-to-number parsing like:
//   - LC #8 String to Integer (atoi)
//   - LC #7 Reverse Integer
// 
// Key insight: sysfs reports sizes in 512-byte SECTORS, not bytes!
// =============================================================================

/// Parse sysfs size file to bytes.
///
/// The Linux kernel reports block device sizes in 512-byte sectors,
/// regardless of the actual hardware sector size.
///
/// # Arguments
///
/// * `content` - Content of `/sys/block/{dev}/size` file
///
/// # Returns
///
/// Size in bytes as u64.
///
/// # Formula
///
/// ```text
/// size_bytes = sectors × 512
/// ```
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::storage::parse_sysfs_size;
///
/// // A 2TB drive has approximately 3.9 billion sectors
/// let size = parse_sysfs_size("3907029168").unwrap();
/// assert_eq!(size, 3907029168 * 512); // ~2TB
///
/// // Handle whitespace (sysfs files often have trailing newline)
/// let size = parse_sysfs_size("1000000\n").unwrap();
/// assert_eq!(size, 1000000 * 512);
/// ```
///
/// # Errors
///
/// Returns error if content is not a valid integer.
///
/// # References
///
/// - [sysfs block size](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-block)
pub fn parse_sysfs_size(content: &str) -> Result<u64, String> {
    // STEP 1: Trim whitespace (sysfs files have trailing newlines)
    let trimmed = content.trim();
    
    // STEP 2: Parse as u64
    // Using parse::<u64>() which handles the conversion
    let sectors: u64 = trimmed
        .parse()
        .map_err(|e| format!("Failed to parse sector count '{}': {}", trimmed, e))?;
    
    // STEP 3: Convert sectors to bytes
    // Kernel ALWAYS uses 512-byte sectors for this file
    const SECTOR_SIZE: u64 = 512;
    Ok(sectors * SECTOR_SIZE)
}

// =============================================================================
// ROTATIONAL FLAG PARSING
// =============================================================================
//
// LEETCODE CONNECTION: Simple boolean parsing, but demonstrates
// defensive programming - handle unexpected inputs gracefully.
// =============================================================================

/// Parse sysfs rotational flag.
///
/// # Arguments
///
/// * `content` - Content of `/sys/block/{dev}/queue/rotational`
///
/// # Returns
///
/// - `true` if device is rotational (HDD)
/// - `false` if solid-state (SSD, NVMe)
///
/// # Why This Matters
///
/// Rotational devices have:
/// - Mechanical seek latency (milliseconds vs microseconds)
/// - Sequential access is much faster than random
/// - Different SMART attributes
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::storage::parse_sysfs_rotational;
///
/// assert!(parse_sysfs_rotational("1"));      // HDD
/// assert!(!parse_sysfs_rotational("0"));     // SSD
/// assert!(!parse_sysfs_rotational("0\n"));   // With newline
/// assert!(!parse_sysfs_rotational(""));      // Empty = assume SSD
/// ```
pub fn parse_sysfs_rotational(content: &str) -> bool {
    // Only "1" means rotational; anything else (0, empty, error) = non-rotational
    content.trim() == "1"
}

// =============================================================================
// LSBLK JSON PARSING
// =============================================================================
//
// LEETCODE CONNECTION: JSON parsing is like tree traversal (LC #94, #144)
// We navigate a nested structure to extract values.
//
// Also similar to LC #1 Two Sum - we're doing key lookups in a map.
// =============================================================================

/// Parse lsblk JSON output into storage devices.
///
/// # Arguments
///
/// * `output` - JSON output from `lsblk -J -o NAME,SIZE,TYPE,MODEL,SERIAL,ROTA,TRAN,WWN -b`
///
/// # Expected Format
///
/// ```json
/// {
///    "blockdevices": [
///       {
///          "name": "nvme0n1",
///          "size": 2000398934016,
///          "type": "disk",
///          "model": "Samsung SSD 980 PRO 2TB",
///          "serial": "S5GXNF0N123456",
///          "rota": false,
///          "tran": "nvme",
///          "wwn": "eui.0025385b21404321"
///       }
///    ]
/// }
/// ```
///
/// # Notes
///
/// - Use `-b` flag to get size in bytes (not human-readable)
/// - `rota` is boolean (false = SSD, true = HDD)
/// - `tran` is transport type (nvme, sata, usb, etc.)
///
/// # References
///
/// - [lsblk man page](https://man7.org/linux/man-pages/man8/lsblk.8.html)
pub fn parse_lsblk_json(output: &str) -> Result<Vec<StorageDevice>, String> {
    // STEP 1: Parse JSON
    // Using serde_json which is already a dependency
    let json: serde_json::Value = serde_json::from_str(output)
        .map_err(|e| format!("Failed to parse lsblk JSON: {}", e))?;
    
    // STEP 2: Navigate to blockdevices array
    // This is like tree traversal - we're finding a specific node
    let devices_array = json
        .get("blockdevices")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Missing 'blockdevices' array in lsblk output".to_string())?;
    
    // STEP 3: Transform each JSON object into StorageDevice
    // LEETCODE CONNECTION: This is the "transform" pattern seen in many problems
    // Like LC #2 Add Two Numbers - transform input format to output format
    let mut devices = Vec::new();
    
    for device_json in devices_array {
        // Skip non-disk entries (partitions, etc.)
        let device_type_str = device_json
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        if device_type_str != "disk" {
            continue;
        }
        
        // Extract fields with defaults for missing values
        let name = device_json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        // Skip virtual devices
        if is_virtual_device(&name) {
            continue;
        }
        
        let size_bytes = device_json
            .get("size")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let model = device_json
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()  // Models often have trailing whitespace
            .to_string();
        
        let serial = device_json
            .get("serial")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string());
        
        let is_rotational = device_json
            .get("rota")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let transport = device_json
            .get("tran")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let wwn = device_json
            .get("wwn")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Determine storage type
        let device_type = StorageType::from_device(&name, is_rotational);
        
        // Determine interface from transport
        let interface = match transport.as_str() {
            "nvme" => "NVMe".to_string(),
            "sata" => "SATA".to_string(),
            "sas" => "SAS".to_string(),
            "usb" => "USB".to_string(),
            "" => device_type.display_name().to_string(),
            other => other.to_uppercase(),
        };
        
        // Build the device struct
        let mut device = StorageDevice {
            name: name.clone(),
            device_path: format!("/dev/{}", name),
            device_type,
            type_: device_type.display_name().to_string(),
            size_bytes,
            model,
            serial_number: serial,
            interface,
            is_rotational,
            wwn,
            detection_method: "lsblk".to_string(),
            ..Default::default()
        };
        
        // Calculate convenience fields
        device.calculate_size_fields();
        
        devices.push(device);
    }
    
    Ok(devices)
}

// =============================================================================
// VIRTUAL DEVICE DETECTION
// =============================================================================
//
// LEETCODE CONNECTION: This is pattern matching, similar to:
//   - LC #10 Regular Expression Matching
//   - LC #44 Wildcard Matching
//
// We're checking if a string matches any of several patterns.
// =============================================================================

/// Check if device name indicates a virtual device.
///
/// Virtual devices are not physical hardware and should usually be
/// filtered out of hardware inventory.
///
/// # Arguments
///
/// * `name` - Block device name
///
/// # Returns
///
/// `true` if device is virtual (loop, ram, dm-*, etc.)
///
/// # Virtual Device Types
///
/// | Prefix | Description |
/// |--------|-------------|
/// | loop | Loop devices (mounted ISO files, etc.) |
/// | ram | RAM disks |
/// | dm- | Device mapper (LVM, LUKS encryption) |
/// | zram | Compressed RAM for swap |
/// | nbd | Network block device |
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::storage::is_virtual_device;
///
/// assert!(is_virtual_device("loop0"));
/// assert!(is_virtual_device("dm-0"));
/// assert!(!is_virtual_device("sda"));
/// assert!(!is_virtual_device("nvme0n1"));
/// ```
pub fn is_virtual_device(name: &str) -> bool {
    // Check prefixes that indicate virtual devices
    // Order doesn't matter for correctness, but put common ones first for efficiency
    name.starts_with("loop")
        || name.starts_with("dm-")
        || name.starts_with("ram")
        || name.starts_with("zram")
        || name.starts_with("nbd")
        || name.starts_with("sr")  // CD/DVD drives (virtual in VMs)
}

// =============================================================================
// HUMAN-READABLE SIZE PARSING
// =============================================================================
//
// LEETCODE CONNECTION: This is like LC #8 (atoi) but with unit suffixes.
// We need to handle: "500 GB", "2 TB", "1.5 TB", etc.
//
// Pattern: Parse number + parse unit + multiply
// =============================================================================

/// Parse human-readable size string to bytes.
///
/// Handles common size formats from various tools.
///
/// # Supported Formats
///
/// - "500 GB", "500GB", "500G"
/// - "2 TB", "2TB", "2T"
/// - "1.5 TB"
/// - "1000000000" (raw bytes)
///
/// # Units (Binary)
///
/// - K/KB = 1024
/// - M/MB = 1024²
/// - G/GB = 1024³
/// - T/TB = 1024⁴
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::storage::parse_size_string;
///
/// assert_eq!(parse_size_string("500 GB"), Some(500 * 1024_u64.pow(3)));
/// assert_eq!(parse_size_string("2 TB"), Some(2 * 1024_u64.pow(4)));
/// assert_eq!(parse_size_string("1.5 TB"), Some((1.5 * 1024_f64.powi(4)) as u64));
/// ```
pub fn parse_size_string(size_str: &str) -> Option<u64> {
    let s = size_str.trim().to_uppercase();
    
    // Handle "No Module Installed" or similar
    if s.contains("NO ") || s.contains("UNKNOWN") || s.is_empty() {
        return None;
    }
    
    // Try to parse as raw number first
    if let Ok(bytes) = s.parse::<u64>() {
        return Some(bytes);
    }
    
    // PATTERN: Split into number and unit
    // "500 GB" -> ["500", "GB"]
    // "500GB" -> need to find where number ends
    
    // Find where the number part ends
    let num_end = s
        .chars()
        .position(|c| !c.is_ascii_digit() && c != '.')
        .unwrap_or(s.len());
    
    if num_end == 0 {
        return None;
    }
    
    let num_str = &s[..num_end];
    let unit_str = s[num_end..].trim();
    
    // Parse the number (could be float like "1.5")
    let num: f64 = num_str.parse().ok()?;
    
    // Determine multiplier based on unit
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    let multiplier = match unit_str {
        "K" | "KB" | "KIB" => KB,
        "M" | "MB" | "MIB" => MB,
        "G" | "GB" | "GIB" => GB,
        "T" | "TB" | "TIB" => TB,
        "B" | "" => 1,
        _ => return None,
    };
    
    Some((num * multiplier as f64) as u64)
}

// =============================================================================
// UNIT TESTS
// =============================================================================
//
// IMPORTANT: Always test pure functions! They're easy to test
// because they have no dependencies.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sysfs_size() {
        // 2TB drive (approximately 3.9 billion sectors)
        assert_eq!(
            parse_sysfs_size("3907029168").unwrap(),
            3907029168 * 512
        );
        
        // With whitespace
        assert_eq!(
            parse_sysfs_size("  1000000\n").unwrap(),
            1000000 * 512
        );
        
        // Error case
        assert!(parse_sysfs_size("not a number").is_err());
        assert!(parse_sysfs_size("").is_err());
    }

    #[test]
    fn test_parse_sysfs_rotational() {
        assert!(parse_sysfs_rotational("1"));
        assert!(!parse_sysfs_rotational("0"));
        assert!(!parse_sysfs_rotational("0\n"));
        assert!(!parse_sysfs_rotational(""));
        assert!(!parse_sysfs_rotational("garbage"));
    }

    #[test]
    fn test_is_virtual_device() {
        // Virtual devices
        assert!(is_virtual_device("loop0"));
        assert!(is_virtual_device("loop1"));
        assert!(is_virtual_device("dm-0"));
        assert!(is_virtual_device("dm-1"));
        assert!(is_virtual_device("ram0"));
        assert!(is_virtual_device("zram0"));
        assert!(is_virtual_device("nbd0"));
        
        // Physical devices
        assert!(!is_virtual_device("sda"));
        assert!(!is_virtual_device("sdb"));
        assert!(!is_virtual_device("nvme0n1"));
        assert!(!is_virtual_device("mmcblk0"));
    }

    #[test]
    fn test_storage_type_from_device() {
        // NVMe
        assert_eq!(StorageType::from_device("nvme0n1", false), StorageType::Nvme);
        assert_eq!(StorageType::from_device("nvme1n1", false), StorageType::Nvme);
        
        // eMMC
        assert_eq!(StorageType::from_device("mmcblk0", false), StorageType::Emmc);
        
        // Virtual
        assert_eq!(StorageType::from_device("loop0", false), StorageType::Virtual);
        assert_eq!(StorageType::from_device("dm-0", false), StorageType::Virtual);
        
        // SSD vs HDD (based on rotational flag)
        assert_eq!(StorageType::from_device("sda", false), StorageType::Ssd);
        assert_eq!(StorageType::from_device("sda", true), StorageType::Hdd);
    }

    #[test]
    fn test_parse_size_string() {
        // GB
        assert_eq!(parse_size_string("500 GB"), Some(500 * 1024_u64.pow(3)));
        assert_eq!(parse_size_string("500GB"), Some(500 * 1024_u64.pow(3)));
        
        // TB
        assert_eq!(parse_size_string("2 TB"), Some(2 * 1024_u64.pow(4)));
        
        // Raw bytes
        assert_eq!(parse_size_string("1073741824"), Some(1073741824));
        
        // Invalid
        assert_eq!(parse_size_string("Unknown"), None);
        assert_eq!(parse_size_string(""), None);
    }
}
```

### 1.4 Update Linux Adapter for Storage

**File:** `src/adapters/secondary/system/linux.rs`

**Where:** Replace/update the `get_storage_info` method

**LeetCode Pattern:** This implements **Chain of Responsibility** - we try multiple 
detection methods in sequence until one succeeds. Similar to how you might try 
multiple algorithms for optimization.

```rust
// =============================================================================
// STORAGE DETECTION IN LINUX ADAPTER
// =============================================================================
//
// ARCHITECTURE: This is the ADAPTER layer implementation of SystemInfoProvider.
// It implements the PORT (trait) using Linux-specific mechanisms.
//
// PATTERN: Chain of Responsibility
// - Try sysfs first (most reliable)
// - Fall back to lsblk if sysfs fails
// - Use sysinfo as last resort
//
// LEETCODE CONNECTION: This pattern is used when you have multiple approaches:
//   - LC #70 Climbing Stairs: try 1 step, try 2 steps
//   - LC #322 Coin Change: try each coin denomination
//   - Here: try each detection method
// =============================================================================

// Add these imports at the top of linux.rs
use crate::domain::parsers::storage::{
    parse_sysfs_size, parse_sysfs_rotational, parse_lsblk_json, is_virtual_device
};
use crate::domain::{StorageDevice, StorageInfo, StorageType};
use std::fs;
use std::path::Path;

impl SystemInfoProvider for LinuxSystemInfoProvider {
    // ... other methods ...
    
    /// Detect storage devices using multiple methods.
    ///
    /// # Detection Chain
    ///
    /// ```text
    /// ┌─────────────────────────────────────────────────────────┐
    /// │ 1. sysfs /sys/block (PRIMARY)                          │
    /// │    - Most reliable                                      │
    /// │    - Works on all Linux (x86, ARM)                     │
    /// │    - Direct kernel interface                           │
    /// └───────────────────────┬─────────────────────────────────┘
    ///                         │ enrich with
    ///                         ▼
    /// ┌─────────────────────────────────────────────────────────┐
    /// │ 2. lsblk JSON (ENRICHMENT)                             │
    /// │    - Additional fields (WWN, transport)                │
    /// │    - Serial number (may be available)                  │
    /// └───────────────────────┬─────────────────────────────────┘
    ///                         │ if empty, fallback
    ///                         ▼
    /// ┌─────────────────────────────────────────────────────────┐
    /// │ 3. sysinfo crate (FALLBACK)                            │
    /// │    - Cross-platform                                     │
    /// │    - Limited metadata                                   │
    /// └─────────────────────────────────────────────────────────┘
    /// ```
    async fn get_storage_info(&self) -> Result<StorageInfo, SystemError> {
        let mut devices = Vec::new();
        
        // =====================================================================
        // METHOD 1: sysfs (Primary - most reliable)
        // =====================================================================
        // 
        // WHY SYSFS FIRST?
        // - Direct kernel interface, always available on Linux
        // - Doesn't require external tools (lsblk might not be installed)
        // - Works identically on x86 and ARM
        // =====================================================================
        
        match self.detect_storage_sysfs().await {
            Ok(sysfs_devices) => {
                log::debug!("sysfs detected {} storage devices", sysfs_devices.len());
                devices = sysfs_devices;
            }
            Err(e) => {
                log::warn!("sysfs storage detection failed: {}", e);
            }
        }
        
        // =====================================================================
        // METHOD 2: lsblk enrichment
        // =====================================================================
        //
        // Even if sysfs worked, lsblk might have additional data (WWN, etc.)
        // We MERGE the results rather than replace.
        //
        // LEETCODE CONNECTION: Merging data is like:
        //   - LC #88 Merge Sorted Array
        //   - LC #21 Merge Two Sorted Lists
        // Key insight: match by device name, then combine fields
        // =====================================================================
        
        if let Ok(lsblk_devices) = self.detect_storage_lsblk().await {
            log::debug!("lsblk detected {} devices for enrichment", lsblk_devices.len());
            merge_storage_info(&mut devices, lsblk_devices);
        }
        
        // =====================================================================
        // METHOD 3: sysinfo fallback
        // =====================================================================
        //
        // If we still have no devices, try sysinfo as last resort.
        // This can happen in containers or unusual environments.
        // =====================================================================
        
        if devices.is_empty() {
            log::warn!("No devices from sysfs/lsblk, trying sysinfo fallback");
            if let Ok(sysinfo_devices) = self.detect_storage_sysinfo().await {
                devices = sysinfo_devices;
            }
        }
        
        // =====================================================================
        // POST-PROCESSING
        // =====================================================================
        
        // Filter out virtual devices (they're not physical hardware)
        devices.retain(|d| d.device_type != StorageType::Virtual);
        
        // Ensure all devices have calculated size fields
        for device in &mut devices {
            if device.size_gb == 0.0 && device.size_bytes > 0 {
                device.calculate_size_fields();
            }
            device.set_device_path();
        }
        
        // Sort by name for consistent output
        devices.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(StorageInfo { devices })
    }
}

// =============================================================================
// HELPER METHODS FOR STORAGE DETECTION
// =============================================================================

impl LinuxSystemInfoProvider {
    /// Detect storage devices via sysfs.
    ///
    /// Reads directly from `/sys/block` which is the kernel's view of
    /// block devices.
    ///
    /// # sysfs Structure
    ///
    /// ```text
    /// /sys/block/{device}/
    /// ├── size                    # Size in 512-byte sectors
    /// ├── queue/
    /// │   └── rotational          # 0=SSD, 1=HDD
    /// └── device/
    ///     ├── model               # Device model name
    ///     ├── serial              # Serial number (may need root)
    ///     └── firmware_rev        # Firmware version
    /// ```
    ///
    /// # Returns
    ///
    /// Vector of storage devices found in sysfs.
    async fn detect_storage_sysfs(&self) -> Result<Vec<StorageDevice>, SystemError> {
        let mut devices = Vec::new();
        
        // Path to block devices in sysfs
        let sys_block = Path::new("/sys/block");
        
        if !sys_block.exists() {
            return Err(SystemError::NotAvailable {
                resource: "/sys/block".to_string(),
            });
        }
        
        // LEETCODE CONNECTION: This is directory traversal, similar to:
        //   - LC #200 Number of Islands (grid traversal)
        //   - LC #130 Surrounded Regions
        // We're walking a filesystem tree
        
        let entries = fs::read_dir(sys_block).map_err(|e| SystemError::IoError {
            path: "/sys/block".to_string(),
            message: e.to_string(),
        })?;
        
        for entry in entries.flatten() {
            let device_name = entry.file_name().to_string_lossy().to_string();
            
            // Skip virtual devices early (no need to read their attributes)
            if is_virtual_device(&device_name) {
                continue;
            }
            
            let device_path = entry.path();
            
            // Read size (required - skip device if we can't get size)
            let size_path = device_path.join("size");
            let size_bytes = match fs::read_to_string(&size_path) {
                Ok(content) => match parse_sysfs_size(&content) {
                    Ok(size) => size,
                    Err(_) => continue, // Skip devices we can't parse
                },
                Err(_) => continue,
            };
            
            // Skip tiny devices (< 1GB, probably not real storage)
            if size_bytes < 1_000_000_000 {
                continue;
            }
            
            // Read rotational flag
            let rotational_path = device_path.join("queue/rotational");
            let is_rotational = fs::read_to_string(&rotational_path)
                .map(|content| parse_sysfs_rotational(&content))
                .unwrap_or(false);
            
            // Determine device type
            let device_type = StorageType::from_device(&device_name, is_rotational);
            
            // Read model (in device subdirectory)
            let model = self.read_sysfs_string(&device_path.join("device/model"))
                .unwrap_or_default()
                .trim()
                .to_string();
            
            // Read serial (may require root)
            let serial_number = self.read_sysfs_string(&device_path.join("device/serial"))
                .map(|s| s.trim().to_string())
                .ok();
            
            // Read firmware version
            let firmware_version = self.read_sysfs_string(&device_path.join("device/firmware_rev"))
                .map(|s| s.trim().to_string())
                .ok();
            
            // Determine interface based on device type
            let interface = match &device_type {
                StorageType::Nvme => "NVMe".to_string(),
                StorageType::Emmc => "eMMC".to_string(),
                StorageType::Hdd | StorageType::Ssd => {
                    // Could check for SAS vs SATA here
                    "SATA".to_string()
                }
                _ => "Unknown".to_string(),
            };
            
            // Build the device
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
    
    /// Detect storage via lsblk command.
    ///
    /// Uses JSON output for reliable parsing.
    async fn detect_storage_lsblk(&self) -> Result<Vec<StorageDevice>, SystemError> {
        let cmd = SystemCommand::new("lsblk")
            .args(&[
                "-J",                    // JSON output
                "-b",                    // Size in bytes
                "-o", "NAME,SIZE,TYPE,MODEL,SERIAL,ROTA,TRAN,WWN",
            ])
            .timeout(Duration::from_secs(10));
        
        let output = self.command_executor.execute(&cmd).await.map_err(|e| {
            SystemError::CommandFailed {
                command: "lsblk".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            }
        })?;
        
        if !output.success {
            return Err(SystemError::CommandFailed {
                command: "lsblk".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr.clone(),
            });
        }
        
        parse_lsblk_json(&output.stdout).map_err(SystemError::ParseError)
    }
    
    /// Detect storage via sysinfo crate (cross-platform fallback).
    async fn detect_storage_sysinfo(&self) -> Result<Vec<StorageDevice>, SystemError> {
        use sysinfo::Disks;
        
        let disks = Disks::new_with_refreshed_list();
        let mut devices = Vec::new();
        
        for disk in disks.iter() {
            let name = disk.name().to_string_lossy().to_string();
            let size_bytes = disk.total_space();
            
            // Skip small/virtual
            if size_bytes < 1_000_000_000 {
                continue;
            }
            
            let mut device = StorageDevice {
                name: if name.is_empty() {
                    disk.mount_point().to_string_lossy().to_string()
                } else {
                    name
                },
                size_bytes,
                detection_method: "sysinfo".to_string(),
                ..Default::default()
            };
            
            device.calculate_size_fields();
            devices.push(device);
        }
        
        Ok(devices)
    }
    
    /// Helper to read a sysfs file as string.
    fn read_sysfs_string(&self, path: &Path) -> Result<String, std::io::Error> {
        fs::read_to_string(path)
    }
}

// =============================================================================
// MERGE FUNCTION
// =============================================================================
//
// LEETCODE CONNECTION: This is the merge pattern from:
//   - LC #88 Merge Sorted Array
//   - LC #56 Merge Intervals
//
// Key insight: We match by device name, then update fields that are missing
// in the primary source but present in the secondary.
// =============================================================================

/// Merge storage info from secondary source into primary.
///
/// Matches devices by name and fills in missing fields.
///
/// # Why Merge?
///
/// Different detection methods provide different data:
/// - sysfs: reliable size, rotational flag
/// - lsblk: WWN, transport type
/// - smartctl: serial, SMART status
///
/// By merging, we get the best of all sources.
fn merge_storage_info(primary: &mut Vec<StorageDevice>, secondary: Vec<StorageDevice>) {
    // LEETCODE CONNECTION: This is O(n*m) where n = primary.len(), m = secondary.len()
    // Could optimize with HashMap for O(n+m) if lists are large
    //
    // For small lists (typically < 20 devices), linear search is fine
    
    for sec_device in secondary {
        // Find matching device in primary by name
        if let Some(pri_device) = primary.iter_mut().find(|d| d.name == sec_device.name) {
            // Fill in missing fields from secondary
            // Only update if primary field is empty/None
            
            if pri_device.serial_number.is_none() {
                pri_device.serial_number = sec_device.serial_number;
            }
            
            if pri_device.firmware_version.is_none() {
                pri_device.firmware_version = sec_device.firmware_version;
            }
            
            if pri_device.wwn.is_none() {
                pri_device.wwn = sec_device.wwn;
            }
            
            if pri_device.model.is_empty() && !sec_device.model.is_empty() {
                pri_device.model = sec_device.model;
            }
        } else {
            // Device not in primary - add it
            // This handles cases where sysfs missed a device but lsblk found it
            primary.push(sec_device);
        }
    }
}
```

---

## Step 2: CPU Enhancements

### 2.1 Update CpuInfo Struct

**File:** `src/domain/entities.rs`

**Where:** Replace the existing `CpuInfo` struct (around line 163-175)

**LeetCode Pattern:** The cache hierarchy (L1/L2/L3) is a tree structure. Understanding 
cache levels is similar to tree level traversal (LC #102, #107).

```rust
// =============================================================================
// CPU CACHE INFO STRUCT
// =============================================================================
//
// WHY: CPU caches are hierarchical (L1 → L2 → L3), each with different
// characteristics. Understanding this is like understanding tree levels.
//
// LEETCODE CONNECTION: Cache hierarchy is like tree levels:
//   - LC #102 Binary Tree Level Order Traversal
//   - LC #107 Binary Tree Level Order Traversal II
//   - L1 = leaf level (fastest, smallest)
//   - L3 = root level (slowest, largest)
// =============================================================================

/// CPU cache level information.
///
/// Represents a single cache level (L1d, L1i, L2, L3).
/// Each core has its own L1/L2, while L3 is typically shared.
///
/// # Cache Hierarchy
///
/// ```text
///                     ┌─────────────────────┐
///                     │       L3 Cache      │ ← Shared across cores
///                     │    (8-256 MB)       │   Slowest but largest
///                     └──────────┬──────────┘
///                                │
///            ┌───────────────────┼───────────────────┐
///            │                   │                   │
///     ┌──────┴──────┐     ┌──────┴──────┐     ┌──────┴──────┐
///     │  L2 Cache   │     │  L2 Cache   │     │  L2 Cache   │
///     │ (256KB-1MB) │     │ (per core)  │     │             │
///     └──────┬──────┘     └──────┬──────┘     └──────┬──────┘
///            │                   │                   │
///     ┌──────┴──────┐     ┌──────┴──────┐     ┌──────┴──────┐
///     │ L1d │ L1i   │     │ L1d │ L1i   │     │ L1d │ L1i   │
///     │(32KB each)  │     │ (per core)  │     │             │
///     └─────────────┘     └─────────────┘     └─────────────┘
///         Core 0              Core 1              Core N
/// ```
///
/// # References
///
/// - [CPU Cache Wikipedia](https://en.wikipedia.org/wiki/CPU_cache)
/// - [Linux cache sysfs](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-system-cpu)
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CpuCacheInfo {
    /// Cache level (1, 2, or 3).
    pub level: u8,
    
    /// Cache type.
    /// 
    /// Values: "Data" (L1d), "Instruction" (L1i), "Unified" (L2, L3)
    pub cache_type: String,
    
    /// Cache size in kilobytes.
    pub size_kb: u32,
    
    /// Number of ways of associativity.
    ///
    /// Higher = more flexible but complex.
    /// Common values: 4, 8, 12, 16
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ways_of_associativity: Option<u32>,
    
    /// Cache line size in bytes.
    ///
    /// Typically 64 bytes on modern CPUs.
    /// Important for avoiding false sharing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_size_bytes: Option<u32>,
    
    /// Number of sets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sets: Option<u32>,
    
    /// Whether this cache is shared across cores.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared: Option<bool>,
}

// =============================================================================
// CPU INFO STRUCT
// =============================================================================
//
// WHY: The old struct had:
//   - speed: String → Can't sort/compare CPUs by frequency
//   - No cache info → Missing important performance data
//   - No architecture → Can't distinguish x86 from ARM
//
// LEETCODE CONNECTION: CPU topology is a tree:
//   - System → Sockets → Cores → Threads
//   - Similar to LC #429 N-ary Tree Level Order Traversal
// =============================================================================

/// CPU information with extended details.
///
/// Provides comprehensive CPU information including frequency,
/// cache hierarchy, and feature flags.
///
/// # Detection Methods
///
/// Information is gathered from multiple sources (Chain of Responsibility):
/// 1. **sysfs** `/sys/devices/system/cpu` - Frequency, cache
/// 2. **raw-cpuid** - CPUID instruction (x86 only)
/// 3. **/proc/cpuinfo** - Model, vendor, flags
/// 4. **lscpu** - Topology
/// 5. **dmidecode** - SMBIOS data
/// 6. **sysinfo** - Cross-platform fallback
///
/// # Topology
///
/// ```text
/// System
/// └── Socket 0 (physical CPU package)
///     ├── Core 0
///     │   ├── Thread 0 (logical CPU 0)
///     │   └── Thread 1 (logical CPU 1, if SMT/HT enabled)
///     └── Core 1
///         ├── Thread 0 (logical CPU 2)
///         └── Thread 1 (logical CPU 3)
/// └── Socket 1 (if multi-socket)
///     └── ...
/// ```
///
/// # Example
///
/// ```rust
/// use hardware_report::CpuInfo;
///
/// // Check if CPU has AVX-512 for vectorized workloads
/// let has_avx512 = cpu.flags.iter().any(|f| f.starts_with("avx512"));
///
/// // Calculate total compute units
/// let total_threads = cpu.sockets * cpu.cores * cpu.threads;
/// ```
///
/// # References
///
/// - [Linux CPU sysfs](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-system-cpu)
/// - [Intel CPUID](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
/// - [ARM CPU ID](https://developer.arm.com/documentation/ddi0487/latest)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuInfo {
    // =========================================================================
    // IDENTIFICATION
    // =========================================================================
    
    /// CPU model name.
    ///
    /// Examples:
    /// - "AMD EPYC 7763 64-Core Processor"
    /// - "Intel(R) Xeon(R) Platinum 8380 CPU @ 2.30GHz"
    /// - "Neoverse-N1" (ARM)
    pub model: String,
    
    /// CPU vendor identifier.
    ///
    /// Values:
    /// - "GenuineIntel" (Intel)
    /// - "AuthenticAMD" (AMD)
    /// - "ARM" (ARM-based)
    #[serde(default)]
    pub vendor: String,
    
    // =========================================================================
    // TOPOLOGY
    // =========================================================================
    //
    // LEETCODE CONNECTION: Understanding topology is like tree traversal
    // total_threads = sockets × cores × threads_per_core
    // =========================================================================
    
    /// Physical cores per socket.
    pub cores: u32,
    
    /// Threads per core (SMT/Hyperthreading).
    ///
    /// Usually 1 (no SMT) or 2 (SMT enabled).
    pub threads: u32,
    
    /// Number of CPU sockets.
    ///
    /// Desktop: 1, Server: 1-8
    pub sockets: u32,
    
    /// Total physical cores (cores × sockets).
    #[serde(default)]
    pub total_cores: u32,
    
    /// Total logical CPUs (cores × threads × sockets).
    #[serde(default)]
    pub total_threads: u32,
    
    // =========================================================================
    // FREQUENCY
    // =========================================================================
    //
    // WHY MULTIPLE FREQUENCIES?
    // - base = guaranteed frequency
    // - max = turbo/boost frequency (brief bursts)
    // - min = power-saving frequency
    // =========================================================================
    
    /// CPU frequency in MHz.
    ///
    /// This is the PRIMARY frequency field (current or max).
    /// Use for CMDB inventory and general reporting.
    #[serde(default)]
    pub frequency_mhz: u32,
    
    /// Legacy speed field as string.
    ///
    /// DEPRECATED: Use `frequency_mhz` instead.
    pub speed: String,
    
    /// Minimum scaling frequency in MHz.
    ///
    /// From cpufreq scaling_min_freq (power saving).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_min_mhz: Option<u32>,
    
    /// Maximum scaling frequency in MHz.
    ///
    /// From cpufreq scaling_max_freq (turbo/boost).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_max_mhz: Option<u32>,
    
    /// Base (non-turbo) frequency in MHz.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_base_mhz: Option<u32>,
    
    // =========================================================================
    // ARCHITECTURE
    // =========================================================================
    
    /// CPU architecture.
    ///
    /// Values: "x86_64", "aarch64", "armv7l"
    #[serde(default)]
    pub architecture: String,
    
    /// CPU microarchitecture name.
    ///
    /// Examples:
    /// - Intel: "Ice Lake", "Sapphire Rapids"
    /// - AMD: "Zen3", "Zen4"
    /// - ARM: "Neoverse N1", "Neoverse V2"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub microarchitecture: Option<String>,
    
    // =========================================================================
    // CACHE SIZES
    // =========================================================================
    //
    // WHY SEPARATE L1d AND L1i?
    // - L1d = data cache (for variables, arrays)
    // - L1i = instruction cache (for code)
    // - They're accessed differently, may have different sizes
    // =========================================================================
    
    /// L1 data cache size in KB (per core).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_l1d_kb: Option<u32>,
    
    /// L1 instruction cache size in KB (per core).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_l1i_kb: Option<u32>,
    
    /// L2 cache size in KB (usually per core).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_l2_kb: Option<u32>,
    
    /// L3 cache size in KB (usually shared).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_l3_kb: Option<u32>,
    
    /// Detailed cache information.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub caches: Vec<CpuCacheInfo>,
    
    // =========================================================================
    // FEATURES AND FLAGS
    // =========================================================================
    //
    // LEETCODE CONNECTION: Checking flags is like set membership (LC #217)
    // "Does this CPU support AVX-512?" = "Is avx512f in the set?"
    // =========================================================================
    
    /// CPU feature flags.
    ///
    /// x86 examples: "avx", "avx2", "avx512f", "aes", "sse4_2"
    /// ARM examples: "fp", "asimd", "sve", "sve2"
    ///
    /// # Usage
    ///
    /// ```rust
    /// // Check for AVX-512 support
    /// let has_avx512 = cpu.flags.iter().any(|f| f.starts_with("avx512"));
    ///
    /// // Check for AES-NI (hardware encryption)
    /// let has_aes = cpu.flags.contains(&"aes".to_string());
    /// ```
    #[serde(default)]
    pub flags: Vec<String>,
    
    // =========================================================================
    // ADDITIONAL METADATA
    // =========================================================================
    
    /// Microcode/firmware version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub microcode_version: Option<String>,
    
    /// CPU stepping (silicon revision).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stepping: Option<u32>,
    
    /// CPU family number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<u32>,
    
    /// CPU model number (not the name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_number: Option<u32>,
    
    /// Virtualization technology.
    ///
    /// Values: "VT-x" (Intel), "AMD-V" (AMD), "none"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtualization: Option<String>,
    
    /// Number of NUMA nodes.
    #[serde(default)]
    pub numa_nodes: u32,
    
    /// Detection methods used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detection_methods: Vec<String>,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            model: String::new(),
            vendor: String::new(),
            cores: 0,
            threads: 1,
            sockets: 1,
            total_cores: 0,
            total_threads: 0,
            frequency_mhz: 0,
            speed: String::new(),
            frequency_min_mhz: None,
            frequency_max_mhz: None,
            frequency_base_mhz: None,
            architecture: std::env::consts::ARCH.to_string(),
            microarchitecture: None,
            cache_l1d_kb: None,
            cache_l1i_kb: None,
            cache_l2_kb: None,
            cache_l3_kb: None,
            caches: Vec::new(),
            flags: Vec::new(),
            microcode_version: None,
            stepping: None,
            family: None,
            model_number: None,
            virtualization: None,
            numa_nodes: 1,
            detection_methods: Vec::new(),
        }
    }
}

impl CpuInfo {
    /// Calculate total_cores and total_threads from topology.
    pub fn calculate_totals(&mut self) {
        self.total_cores = self.sockets * self.cores;
        self.total_threads = self.total_cores * self.threads;
    }
    
    /// Set legacy speed field from frequency_mhz.
    pub fn set_speed_string(&mut self) {
        if self.frequency_mhz > 0 {
            if self.frequency_mhz >= 1000 {
                self.speed = format!("{:.2} GHz", self.frequency_mhz as f64 / 1000.0);
            } else {
                self.speed = format!("{} MHz", self.frequency_mhz);
            }
        }
    }
}
```

### 2.2 Add CPU Parser Functions

**File:** `src/domain/parsers/cpu.rs`

**LeetCode Pattern:** Parsing /proc/cpuinfo is a **State Machine** problem similar to 
LC #65 (Valid Number) - we track state while processing each line.

```rust
// =============================================================================
// CPU PARSER FUNCTIONS
// =============================================================================
//
// Architecture: DOMAIN layer - pure functions, no I/O
// =============================================================================

use crate::domain::{CpuInfo, CpuCacheInfo};

// =============================================================================
// SYSFS FREQUENCY PARSING
// =============================================================================
//
// LEETCODE CONNECTION: This is number parsing like LC #8 (atoi)
// but with unit conversion (kHz to MHz).
// =============================================================================

/// Parse sysfs frequency file (kHz) to MHz.
///
/// The kernel reports CPU frequencies in kHz in sysfs.
///
/// # Arguments
///
/// * `content` - Content of cpufreq file (in kHz)
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::cpu::parse_sysfs_freq_khz;
///
/// // 3.5 GHz in kHz
/// assert_eq!(parse_sysfs_freq_khz("3500000").unwrap(), 3500);
/// assert_eq!(parse_sysfs_freq_khz("2100000\n").unwrap(), 2100);
/// ```
pub fn parse_sysfs_freq_khz(content: &str) -> Result<u32, String> {
    let khz: u32 = content
        .trim()
        .parse()
        .map_err(|e| format!("Invalid frequency '{}': {}", content.trim(), e))?;
    
    // Convert kHz to MHz
    Ok(khz / 1000)
}

// =============================================================================
// SYSFS CACHE SIZE PARSING
// =============================================================================
//
// LEETCODE CONNECTION: Similar to LC #8 but with unit suffixes (K, M, G)
// Need to handle: "32K", "1M", "256K", "16M"
// =============================================================================

/// Parse sysfs cache size (e.g., "32K", "1M") to KB.
///
/// # Arguments
///
/// * `content` - Content of cache size file
///
/// # Supported Units
///
/// - K = kilobytes (multiply by 1)
/// - M = megabytes (multiply by 1024)
/// - G = gigabytes (multiply by 1024²)
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::cpu::parse_sysfs_cache_size;
///
/// assert_eq!(parse_sysfs_cache_size("32K").unwrap(), 32);
/// assert_eq!(parse_sysfs_cache_size("1M").unwrap(), 1024);
/// assert_eq!(parse_sysfs_cache_size("256K").unwrap(), 256);
/// ```
pub fn parse_sysfs_cache_size(content: &str) -> Result<u32, String> {
    let s = content.trim().to_uppercase();
    
    // Handle common formats: "32K", "1M", "32768K"
    if s.ends_with('K') {
        let num_str = &s[..s.len()-1];
        num_str.parse::<u32>()
            .map_err(|e| format!("Invalid cache size '{}': {}", s, e))
    } else if s.ends_with('M') {
        let num_str = &s[..s.len()-1];
        num_str.parse::<u32>()
            .map(|v| v * 1024)
            .map_err(|e| format!("Invalid cache size '{}': {}", s, e))
    } else if s.ends_with('G') {
        let num_str = &s[..s.len()-1];
        num_str.parse::<u32>()
            .map(|v| v * 1024 * 1024)
            .map_err(|e| format!("Invalid cache size '{}': {}", s, e))
    } else {
        // Assume raw KB value
        s.parse::<u32>()
            .map_err(|e| format!("Invalid cache size '{}': {}", s, e))
    }
}

// =============================================================================
// /proc/cpuinfo PARSING
// =============================================================================
//
// LEETCODE CONNECTION: This is a STATE MACHINE problem like:
//   - LC #65 Valid Number
//   - LC #10 Regular Expression Matching
//
// We process line by line, extracting key:value pairs.
// Different architectures (x86 vs ARM) have different keys!
// =============================================================================

/// Parse /proc/cpuinfo into CpuInfo.
///
/// # Format Differences
///
/// **x86/x86_64:**
/// ```text
/// model name  : Intel(R) Xeon(R) Platinum 8380 CPU @ 2.30GHz
/// vendor_id   : GenuineIntel
/// flags       : fpu vme de pse avx avx2 avx512f ...
/// ```
///
/// **ARM/aarch64:**
/// ```text
/// CPU implementer : 0x41
/// CPU part        : 0xd0c
/// Features        : fp asimd evtstrm aes ...
/// ```
///
/// # LeetCode Pattern
///
/// This is similar to parsing problems:
/// - Split by delimiter (`:`)
/// - Handle whitespace
/// - Accumulate results
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::cpu::parse_proc_cpuinfo;
///
/// let cpuinfo = "model name\t: Intel Xeon\nflags\t: avx avx2\n";
/// let info = parse_proc_cpuinfo(cpuinfo).unwrap();
/// assert!(info.flags.contains(&"avx".to_string()));
/// ```
pub fn parse_proc_cpuinfo(content: &str) -> Result<CpuInfo, String> {
    let mut info = CpuInfo::default();
    let mut processor_count = 0;
    
    // Process each line
    // PATTERN: Key-value parsing with colon delimiter
    for line in content.lines() {
        // Split on first colon
        // "model name\t: Intel Xeon" -> ["model name\t", " Intel Xeon"]
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        
        if parts.len() != 2 {
            continue;
        }
        
        let key = parts[0].trim().to_lowercase();
        let value = parts[1].trim();
        
        // Match on key (different for x86 vs ARM)
        match key.as_str() {
            // x86 keys
            "model name" => {
                if info.model.is_empty() {
                    info.model = value.to_string();
                }
            }
            "vendor_id" => {
                if info.vendor.is_empty() {
                    info.vendor = value.to_string();
                }
            }
            "cpu family" => {
                info.family = value.parse().ok();
            }
            "model" => {
                // Note: "model" is the number, "model name" is the string
                info.model_number = value.parse().ok();
            }
            "stepping" => {
                info.stepping = value.parse().ok();
            }
            "microcode" => {
                info.microcode_version = Some(value.to_string());
            }
            "cpu mhz" => {
                // Parse frequency from cpuinfo (may be floating point)
                if let Ok(mhz) = value.parse::<f64>() {
                    info.frequency_mhz = mhz as u32;
                }
            }
            "flags" => {
                // x86 feature flags (space-separated)
                info.flags = value.split_whitespace()
                    .map(String::from)
                    .collect();
            }
            
            // ARM keys
            "features" => {
                // ARM feature flags (like x86 "flags")
                info.flags = value.split_whitespace()
                    .map(String::from)
                    .collect();
            }
            "cpu implementer" => {
                // ARM: indicates vendor
                if info.vendor.is_empty() {
                    info.vendor = "ARM".to_string();
                }
            }
            "cpu part" => {
                // ARM: CPU part number -> map to microarchitecture
                if let Some(arch_name) = arm_cpu_part_to_name(value) {
                    info.microarchitecture = Some(arch_name.to_string());
                }
            }
            
            // Count processors
            "processor" => {
                processor_count += 1;
            }
            
            _ => {}
        }
    }
    
    // Set total_threads from processor count
    if processor_count > 0 {
        info.total_threads = processor_count;
    }
    
    info.detection_methods.push("proc_cpuinfo".to_string());
    
    Ok(info)
}

// =============================================================================
// ARM CPU PART MAPPING
// =============================================================================
//
// LEETCODE CONNECTION: This is a HASH MAP lookup problem like:
//   - LC #1 Two Sum (lookup in map)
//   - LC #49 Group Anagrams (categorization)
//
// ARM CPUs are identified by a part number. We map to human-readable names.
// =============================================================================

/// Map ARM CPU part ID to microarchitecture name.
///
/// ARM CPUs report a "CPU part" number in /proc/cpuinfo.
/// This function maps it to a human-readable name.
///
/// # Arguments
///
/// * `part` - CPU part from /proc/cpuinfo (e.g., "0xd0c")
///
/// # Returns
///
/// Human-readable microarchitecture name, or None if unknown.
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::cpu::arm_cpu_part_to_name;
///
/// assert_eq!(arm_cpu_part_to_name("0xd0c"), Some("Neoverse N1"));
/// assert_eq!(arm_cpu_part_to_name("0xd49"), Some("Neoverse N2"));
/// assert_eq!(arm_cpu_part_to_name("0xffff"), None);
/// ```
///
/// # References
///
/// - [ARM CPU Part Numbers](https://developer.arm.com/documentation/ddi0487/latest)
/// - [Kernel ARM CPU table](https://github.com/torvalds/linux/blob/master/arch/arm64/kernel/cpuinfo.c)
pub fn arm_cpu_part_to_name(part: &str) -> Option<&'static str> {
    // Normalize: remove "0x" prefix, convert to lowercase
    let normalized = part.trim().to_lowercase();
    let part_id = normalized.strip_prefix("0x").unwrap_or(&normalized);
    
    // LEETCODE CONNECTION: This is essentially a hash map lookup
    // In LeetCode terms: O(1) lookup after building the map
    // We use match here for compile-time optimization
    
    match part_id {
        // ARM Cortex-A series (mobile/embedded)
        "d03" => Some("Cortex-A53"),
        "d04" => Some("Cortex-A35"),
        "d05" => Some("Cortex-A55"),
        "d06" => Some("Cortex-A65"),
        "d07" => Some("Cortex-A57"),
        "d08" => Some("Cortex-A72"),
        "d09" => Some("Cortex-A73"),
        "d0a" => Some("Cortex-A75"),
        "d0b" => Some("Cortex-A76"),
        "d0c" => Some("Neoverse N1"),      // Server (AWS Graviton2)
        "d0d" => Some("Cortex-A77"),
        "d0e" => Some("Cortex-A76AE"),
        
        // ARM Neoverse (server/cloud)
        "d40" => Some("Neoverse V1"),      // Server
        "d41" => Some("Cortex-A78"),
        "d42" => Some("Cortex-A78AE"),
        "d43" => Some("Cortex-A65AE"),
        "d44" => Some("Cortex-X1"),
        "d46" => Some("Cortex-A510"),
        "d47" => Some("Cortex-A710"),
        "d48" => Some("Cortex-X2"),
        "d49" => Some("Neoverse N2"),      // Server (AWS Graviton3)
        "d4a" => Some("Neoverse E1"),
        "d4b" => Some("Cortex-A78C"),
        "d4c" => Some("Cortex-X1C"),
        "d4d" => Some("Cortex-A715"),
        "d4e" => Some("Cortex-X3"),
        "d4f" => Some("Neoverse V2"),      // Server
        
        // Newer cores
        "d80" => Some("Cortex-A520"),
        "d81" => Some("Cortex-A720"),
        "d82" => Some("Cortex-X4"),
        
        // NVIDIA (based on ARM)
        "004" => Some("NVIDIA Denver"),
        "003" => Some("NVIDIA Carmel"),
        
        _ => None,
    }
}

// =============================================================================
// UNIT TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sysfs_freq_khz() {
        assert_eq!(parse_sysfs_freq_khz("3500000").unwrap(), 3500);
        assert_eq!(parse_sysfs_freq_khz("2100000\n").unwrap(), 2100);
        assert_eq!(parse_sysfs_freq_khz("  1000000  ").unwrap(), 1000);
        assert!(parse_sysfs_freq_khz("invalid").is_err());
    }

    #[test]
    fn test_parse_sysfs_cache_size() {
        assert_eq!(parse_sysfs_cache_size("32K").unwrap(), 32);
        assert_eq!(parse_sysfs_cache_size("512K").unwrap(), 512);
        assert_eq!(parse_sysfs_cache_size("1M").unwrap(), 1024);
        assert_eq!(parse_sysfs_cache_size("32M").unwrap(), 32768);
        assert_eq!(parse_sysfs_cache_size("32768K").unwrap(), 32768);
    }

    #[test]
    fn test_arm_cpu_part_mapping() {
        assert_eq!(arm_cpu_part_to_name("0xd0c"), Some("Neoverse N1"));
        assert_eq!(arm_cpu_part_to_name("0xd49"), Some("Neoverse N2"));
        assert_eq!(arm_cpu_part_to_name("d0c"), Some("Neoverse N1"));  // Without 0x
        assert_eq!(arm_cpu_part_to_name("0xD0C"), Some("Neoverse N1")); // Uppercase
        assert_eq!(arm_cpu_part_to_name("0xffff"), None);
    }

    #[test]
    fn test_parse_proc_cpuinfo_x86() {
        let content = r#"
processor	: 0
vendor_id	: GenuineIntel
cpu family	: 6
model		: 106
model name	: Intel(R) Xeon(R) Platinum 8380 CPU @ 2.30GHz
stepping	: 6
microcode	: 0xd0003a5
cpu MHz		: 2300.000
flags		: fpu vme avx avx2 avx512f
"#;
        
        let info = parse_proc_cpuinfo(content).unwrap();
        
        assert_eq!(info.vendor, "GenuineIntel");
        assert!(info.model.contains("Xeon"));
        assert_eq!(info.family, Some(6));
        assert_eq!(info.model_number, Some(106));
        assert_eq!(info.stepping, Some(6));
        assert!(info.flags.contains(&"avx512f".to_string()));
    }

    #[test]
    fn test_parse_proc_cpuinfo_arm() {
        let content = r#"
processor	: 0
BogoMIPS	: 50.00
Features	: fp asimd evtstrm aes pmull sha1 sha2 crc32
CPU implementer	: 0x41
CPU architecture: 8
CPU variant	: 0x3
CPU part	: 0xd0c
"#;
        
        let info = parse_proc_cpuinfo(content).unwrap();
        
        assert_eq!(info.vendor, "ARM");
        assert_eq!(info.microarchitecture, Some("Neoverse N1".to_string()));
        assert!(info.flags.contains(&"asimd".to_string()));
    }
}
```

---

## Step 3: GPU Enhancements

### 3.1 Add GpuVendor Enum

**File:** `src/domain/entities.rs`

**Where:** Add before the GpuDevice struct

**LeetCode Pattern:** PCI vendor ID lookup is a **Hash Map** problem (LC #1 Two Sum).
We're mapping a key (vendor ID) to a value (vendor enum).

```rust
// =============================================================================
// GPU VENDOR ENUM
// =============================================================================
//
// WHY: Different GPU vendors have different detection methods:
//   - NVIDIA: NVML, nvidia-smi
//   - AMD: ROCm, rocm-smi
//   - Intel: sysfs
//
// LEETCODE CONNECTION: This is lookup/categorization like:
//   - LC #1 Two Sum: lookup by key
//   - LC #49 Group Anagrams: group by category
// =============================================================================

/// GPU vendor classification.
///
/// Used to determine which detection method to use and
/// which vendor-specific features are available.
///
/// # PCI Vendor IDs
///
/// | Vendor | PCI ID |
/// |--------|--------|
/// | NVIDIA | 0x10de |
/// | AMD | 0x1002 |
/// | Intel | 0x8086 |
///
/// # References
///
/// - [PCI Vendor IDs](https://pci-ids.ucw.cz/)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum GpuVendor {
    /// NVIDIA Corporation (PCI vendor 0x10de).
    ///
    /// Detection: NVML, nvidia-smi
    /// Features: CUDA, compute capability
    Nvidia,
    
    /// Advanced Micro Devices (PCI vendor 0x1002).
    ///
    /// Detection: ROCm SMI, sysfs
    /// Features: ROCm, HIP
    Amd,
    
    /// Intel Corporation (PCI vendor 0x8086).
    ///
    /// Detection: sysfs, Intel GPU tools
    /// Features: OpenCL, Level Zero
    Intel,
    
    /// Apple Inc. (integrated GPUs on Apple Silicon).
    ///
    /// Detection: system_profiler
    /// Features: Metal
    Apple,
    
    /// Unknown or unrecognized vendor.
    Unknown,
}

impl Default for GpuVendor {
    fn default() -> Self {
        GpuVendor::Unknown
    }
}

impl GpuVendor {
    /// Create GpuVendor from PCI vendor ID.
    ///
    /// # Arguments
    ///
    /// * `vendor_id` - PCI vendor ID (e.g., "10de", "0x10de")
    ///
    /// # Example
    ///
    /// ```rust
    /// use hardware_report::GpuVendor;
    ///
    /// assert_eq!(GpuVendor::from_pci_vendor("10de"), GpuVendor::Nvidia);
    /// assert_eq!(GpuVendor::from_pci_vendor("0x1002"), GpuVendor::Amd);
    /// assert_eq!(GpuVendor::from_pci_vendor("8086"), GpuVendor::Intel);
    /// ```
    ///
    /// # LeetCode Pattern
    ///
    /// This is a simple hash lookup - O(1) time.
    /// Similar to LC #1 Two Sum where you look up complement in a map.
    pub fn from_pci_vendor(vendor_id: &str) -> Self {
        // Normalize: remove "0x" prefix, convert to lowercase
        let normalized = vendor_id.trim().to_lowercase();
        let id = normalized.strip_prefix("0x").unwrap_or(&normalized);
        
        match id {
            "10de" => GpuVendor::Nvidia,
            "1002" => GpuVendor::Amd,
            "8086" => GpuVendor::Intel,
            _ => GpuVendor::Unknown,
        }
    }
    
    /// Get the vendor name as string.
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

impl std::fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
```

### 3.2 Update GpuDevice Struct

**File:** `src/domain/entities.rs`

**Where:** Replace the existing `GpuDevice` struct

```rust
// =============================================================================
// GPU DEVICE STRUCT
// =============================================================================
//
// WHY THE CHANGES:
//   OLD: memory: String ("80 GB") - can't parse!
//   NEW: memory_total_mb: u64 (81920) - math works!
//
// DETECTION METHODS (Chain of Responsibility):
//   1. NVML (native library) - most accurate
//   2. nvidia-smi (command) - fallback for NVIDIA
//   3. rocm-smi (command) - AMD GPUs
//   4. sysfs /sys/class/drm - universal Linux
//   5. lspci - basic enumeration
//   6. sysinfo - cross-platform fallback
// =============================================================================

/// GPU device information.
///
/// Represents a discrete or integrated GPU with comprehensive metadata.
///
/// # Memory Format Change (v0.2.0)
///
/// **BREAKING CHANGE**: Memory is now numeric!
///
/// ```rust
/// // OLD (v0.1.x) - String that couldn't be parsed
/// let memory: &str = &gpu.memory; // "80 GB"
/// let mb: u64 = memory.parse().unwrap(); // FAILS!
///
/// // NEW (v0.2.0) - Numeric, just works
/// let memory_mb: u64 = gpu.memory_total_mb; // 81920
/// let memory_gb: f64 = memory_mb as f64 / 1024.0; // 80.0
/// ```
///
/// # Detection Methods
///
/// GPUs are detected using multiple methods:
///
/// | Priority | Method | Vendor | Memory | Driver |
/// |----------|--------|--------|--------|--------|
/// | 1 | NVML | NVIDIA | Yes | Yes |
/// | 2 | nvidia-smi | NVIDIA | Yes | Yes |
/// | 3 | rocm-smi | AMD | Yes | Yes |
/// | 4 | sysfs DRM | All | Varies | No |
/// | 5 | lspci | All | No | No |
///
/// # Example
///
/// ```rust
/// use hardware_report::{GpuDevice, GpuVendor};
///
/// // Calculate total GPU memory across all GPUs
/// let gpus: Vec<GpuDevice> = get_gpus();
/// let total_memory_gb: f64 = gpus.iter()
///     .map(|g| g.memory_total_mb as f64 / 1024.0)
///     .sum();
///
/// // Filter NVIDIA GPUs
/// let nvidia_gpus: Vec<_> = gpus.iter()
///     .filter(|g| g.vendor == GpuVendor::Nvidia)
///     .collect();
/// ```
///
/// # References
///
/// - [NVIDIA NVML](https://docs.nvidia.com/deploy/nvml-api/)
/// - [AMD ROCm SMI](https://rocm.docs.amd.com/projects/rocm_smi_lib/en/latest/)
/// - [Linux DRM](https://www.kernel.org/doc/html/latest/gpu/drm-uapi.html)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuDevice {
    // =========================================================================
    // IDENTIFICATION
    // =========================================================================
    
    /// GPU index (0-based, unique per system).
    pub index: u32,
    
    /// GPU product name.
    ///
    /// Examples:
    /// - "NVIDIA H100 80GB HBM3"
    /// - "AMD Instinct MI250X"
    /// - "Intel Arc A770"
    pub name: String,
    
    /// GPU UUID (globally unique identifier).
    ///
    /// NVIDIA format: "GPU-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
    pub uuid: String,
    
    // =========================================================================
    // MEMORY (THE BIG FIX!)
    // =========================================================================
    //
    // LEETCODE CONNECTION: Having numeric types enables all the math:
    //   - LC #1 Two Sum: can now sum GPU memory
    //   - LC #215 Kth Largest: can sort by memory
    // =========================================================================
    
    /// Total GPU memory in megabytes.
    ///
    /// **PRIMARY FIELD** - use this for calculations!
    ///
    /// Examples:
    /// - H100 80GB: 81920 MB
    /// - A100 40GB: 40960 MB
    /// - RTX 4090: 24576 MB
    #[serde(default)]
    pub memory_total_mb: u64,
    
    /// Free GPU memory in megabytes (runtime value).
    ///
    /// Returns None if not queryable (e.g., lspci-only detection).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_free_mb: Option<u64>,
    
    /// Used GPU memory in megabytes.
    ///
    /// Calculated as: total - free
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_used_mb: Option<u64>,
    
    /// Legacy memory as string (DEPRECATED).
    ///
    /// Kept for backward compatibility. Use `memory_total_mb` instead.
    #[deprecated(since = "0.2.0", note = "Use memory_total_mb instead")]
    pub memory: String,
    
    // =========================================================================
    // PCI INFORMATION
    // =========================================================================
    
    /// PCI vendor:device ID (e.g., "10de:2330").
    ///
    /// Format: `{vendor_id}:{device_id}` in lowercase hex.
    pub pci_id: String,
    
    /// PCI bus address (e.g., "0000:01:00.0").
    ///
    /// Format: `{domain}:{bus}:{device}.{function}`
    /// Useful for NUMA correlation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pci_bus_id: Option<String>,
    
    // =========================================================================
    // VENDOR INFORMATION
    // =========================================================================
    
    /// GPU vendor enum.
    ///
    /// Use for programmatic comparisons.
    #[serde(default)]
    pub vendor: GpuVendor,
    
    /// Vendor name as string.
    ///
    /// For display and backward compatibility.
    #[serde(default)]
    pub vendor_name: String,
    
    // =========================================================================
    // DRIVER AND CAPABILITIES
    // =========================================================================
    
    /// GPU driver version.
    ///
    /// NVIDIA example: "535.129.03"
    /// AMD example: "6.3.6"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driver_version: Option<String>,
    
    /// CUDA compute capability (NVIDIA only).
    ///
    /// Format: "major.minor"
    /// Examples: "9.0" (Hopper), "8.9" (Ada), "8.0" (Ampere)
    ///
    /// # References
    ///
    /// - [CUDA Compute Capability](https://developer.nvidia.com/cuda-gpus)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute_capability: Option<String>,
    
    /// GPU architecture name.
    ///
    /// Examples:
    /// - NVIDIA: "Hopper", "Ada Lovelace", "Ampere"
    /// - AMD: "CDNA2", "RDNA3"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    
    // =========================================================================
    // TOPOLOGY
    // =========================================================================
    
    /// NUMA node affinity.
    ///
    /// Which NUMA node this GPU is attached to.
    /// Important for optimal CPU-GPU data transfer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numa_node: Option<i32>,
    
    // =========================================================================
    // RUNTIME METRICS (Optional)
    // =========================================================================
    
    /// Current temperature in Celsius.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature_celsius: Option<u32>,
    
    /// Power limit in watts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power_limit_watts: Option<u32>,
    
    /// Current power usage in watts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power_usage_watts: Option<u32>,
    
    /// GPU utilization percentage (0-100).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utilization_percent: Option<u32>,
    
    // =========================================================================
    // METADATA
    // =========================================================================
    
    /// Detection method that discovered this GPU.
    ///
    /// Values: "nvml", "nvidia-smi", "rocm-smi", "sysfs", "lspci", "sysinfo"
    #[serde(default)]
    pub detection_method: String,
}

impl Default for GpuDevice {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::new(),
            uuid: String::new(),
            memory_total_mb: 0,
            memory_free_mb: None,
            memory_used_mb: None,
            #[allow(deprecated)]
            memory: String::new(),
            pci_id: String::new(),
            pci_bus_id: None,
            vendor: GpuVendor::Unknown,
            vendor_name: "Unknown".to_string(),
            driver_version: None,
            compute_capability: None,
            architecture: None,
            numa_node: None,
            temperature_celsius: None,
            power_limit_watts: None,
            power_usage_watts: None,
            utilization_percent: None,
            detection_method: String::new(),
        }
    }
}

impl GpuDevice {
    /// Set the legacy memory string from memory_total_mb.
    #[allow(deprecated)]
    pub fn set_memory_string(&mut self) {
        if self.memory_total_mb > 0 {
            let gb = self.memory_total_mb as f64 / 1024.0;
            if gb >= 1.0 {
                self.memory = format!("{:.0} GB", gb);
            } else {
                self.memory = format!("{} MB", self.memory_total_mb);
            }
        }
    }
    
    /// Calculate memory_used_mb from total and free.
    pub fn calculate_memory_used(&mut self) {
        if let Some(free) = self.memory_free_mb {
            if self.memory_total_mb >= free {
                self.memory_used_mb = Some(self.memory_total_mb - free);
            }
        }
    }
}
```

### 3.3 Create GPU Parser Module

**File:** `src/domain/parsers/gpu.rs` (NEW FILE)

```rust
// =============================================================================
// GPU PARSING MODULE
// =============================================================================
//
// This module contains PURE FUNCTIONS for parsing GPU information
// from various sources.
//
// ARCHITECTURE: Domain layer - no I/O, no side effects
// =============================================================================

//! GPU information parsing functions.
//!
//! Pure parsing functions for GPU data from nvidia-smi, rocm-smi, lspci, etc.
//!
//! # Supported Formats
//!
//! - nvidia-smi CSV output
//! - rocm-smi JSON output
//! - lspci text output
//!
//! # References
//!
//! - [nvidia-smi](https://developer.nvidia.com/nvidia-system-management-interface)
//! - [rocm-smi](https://rocm.docs.amd.com/projects/rocm_smi_lib/en/latest/)

use crate::domain::{GpuDevice, GpuVendor};

// =============================================================================
// NVIDIA-SMI PARSING
// =============================================================================
//
// LEETCODE CONNECTION: CSV parsing is like:
//   - LC #722 Remove Comments: process structured text
//   - LC #468 Validate IP Address: parse delimited fields
//
// Pattern: Split by delimiter, extract fields by position
// =============================================================================

/// Parse nvidia-smi CSV output into GPU devices.
///
/// # Command
///
/// ```bash
/// nvidia-smi --query-gpu=index,name,uuid,memory.total,memory.free,pci.bus_id,driver_version,compute_cap \
///   --format=csv,noheader,nounits
/// ```
///
/// # Expected Format
///
/// ```text
/// 0, NVIDIA H100 80GB HBM3, GPU-xxxx, 81920, 81000, 00000000:01:00.0, 535.129.03, 9.0
/// 1, NVIDIA H100 80GB HBM3, GPU-yyyy, 81920, 80500, 00000000:02:00.0, 535.129.03, 9.0
/// ```
///
/// # Fields
///
/// 0. index
/// 1. name
/// 2. uuid
/// 3. memory.total (MiB, without units)
/// 4. memory.free (MiB)
/// 5. pci.bus_id
/// 6. driver_version
/// 7. compute_cap
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::gpu::parse_nvidia_smi_output;
///
/// let output = "0, NVIDIA H100, GPU-xxx, 81920, 81000, 00:01:00.0, 535.129.03, 9.0";
/// let gpus = parse_nvidia_smi_output(output).unwrap();
/// assert_eq!(gpus[0].memory_total_mb, 81920);
/// ```
pub fn parse_nvidia_smi_output(output: &str) -> Result<Vec<GpuDevice>, String> {
    let mut devices = Vec::new();
    
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Split by comma
        // LEETCODE: This is like parsing CSV - split and extract
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        // Need at least 4 fields (index, name, uuid, memory)
        if parts.len() < 4 {
            continue;
        }
        
        // Parse index
        let index: u32 = parts[0].parse().unwrap_or(devices.len() as u32);
        
        // Parse memory (nvidia-smi with nounits gives MiB directly)
        let memory_total_mb: u64 = parts.get(3)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        
        let memory_free_mb: Option<u64> = parts.get(4)
            .and_then(|s| s.parse().ok());
        
        let mut device = GpuDevice {
            index,
            name: parts.get(1).unwrap_or(&"").to_string(),
            uuid: parts.get(2).unwrap_or(&"").to_string(),
            memory_total_mb,
            memory_free_mb,
            pci_bus_id: parts.get(5).map(|s| s.to_string()),
            driver_version: parts.get(6).map(|s| s.to_string()),
            compute_capability: parts.get(7).map(|s| s.to_string()),
            vendor: GpuVendor::Nvidia,
            vendor_name: "NVIDIA".to_string(),
            detection_method: "nvidia-smi".to_string(),
            ..Default::default()
        };
        
        // Set legacy fields
        device.set_memory_string();
        device.calculate_memory_used();
        
        // Build PCI ID from bus ID if possible
        // Bus ID format: 00000000:01:00.0
        // We'd need device ID from somewhere else for full pci_id
        
        devices.push(device);
    }
    
    Ok(devices)
}

// =============================================================================
// LSPCI PARSING
// =============================================================================
//
// LEETCODE CONNECTION: This is pattern matching in strings:
//   - LC #28 Find Index of First Occurrence
//   - LC #10 Regular Expression Matching
//
// We scan for GPU-related PCI class codes
// =============================================================================

/// Parse lspci output for GPU devices.
///
/// # Command
///
/// ```bash
/// lspci -nn
/// ```
///
/// # Expected Format
///
/// ```text
/// 01:00.0 3D controller [0302]: NVIDIA Corporation GH100 [H100] [10de:2330] (rev a1)
/// 02:00.0 VGA compatible controller [0300]: Advanced Micro Devices [1002:73bf]
/// ```
///
/// # PCI Class Codes
///
/// - 0300: VGA compatible controller
/// - 0302: 3D controller (NVIDIA compute GPUs)
/// - 0380: Display controller
///
/// # Limitations
///
/// lspci does NOT provide:
/// - GPU memory (returns 0)
/// - Driver version
/// - UUID
///
/// Use this as fallback enumeration only.
pub fn parse_lspci_gpu_output(output: &str) -> Result<Vec<GpuDevice>, String> {
    let mut devices = Vec::new();
    let mut gpu_index = 0;
    
    for line in output.lines() {
        let line_lower = line.to_lowercase();
        
        // Check for GPU-related PCI classes
        // [0300] = VGA controller
        // [0302] = 3D controller
        // [0380] = Display controller
        let is_gpu = line_lower.contains("[0300]")
            || line_lower.contains("[0302]")
            || line_lower.contains("[0380]")
            || line_lower.contains("vga compatible")
            || line_lower.contains("3d controller")
            || line_lower.contains("display controller");
        
        if !is_gpu {
            continue;
        }
        
        // Extract PCI bus ID (first field)
        // Format: "01:00.0 3D controller..."
        let pci_bus_id = line.split_whitespace().next().map(String::from);
        
        // Extract vendor:device ID
        // Look for pattern [xxxx:yyyy]
        let pci_id = extract_pci_id(line);
        
        // Determine vendor from PCI ID
        let vendor = pci_id.as_ref()
            .map(|id| {
                let vendor_id = id.split(':').next().unwrap_or("");
                GpuVendor::from_pci_vendor(vendor_id)
            })
            .unwrap_or(GpuVendor::Unknown);
        
        // Extract name (everything between class and PCI ID)
        let name = extract_gpu_name_from_lspci(line);
        
        let device = GpuDevice {
            index: gpu_index,
            name,
            uuid: format!("pci-{}", pci_bus_id.as_deref().unwrap_or("unknown")),
            pci_id: pci_id.unwrap_or_default(),
            pci_bus_id,
            vendor: vendor.clone(),
            vendor_name: vendor.name().to_string(),
            detection_method: "lspci".to_string(),
            // NOTE: lspci cannot determine memory!
            memory_total_mb: 0,
            ..Default::default()
        };
        
        devices.push(device);
        gpu_index += 1;
    }
    
    Ok(devices)
}

/// Extract PCI vendor:device ID from lspci line.
///
/// Looks for pattern `[xxxx:yyyy]` at end of line.
fn extract_pci_id(line: &str) -> Option<String> {
    // Find the last occurrence of [xxxx:yyyy]
    // LEETCODE: This is like LC #28 - finding a pattern
    
    let mut result = None;
    let mut remaining = line;
    
    while let Some(start) = remaining.find('[') {
        if let Some(end) = remaining[start..].find(']') {
            let bracket_content = &remaining[start+1..start+end];
            
            // Check if it looks like a PCI ID (xxxx:yyyy)
            if bracket_content.len() == 9 && bracket_content.chars().nth(4) == Some(':') {
                // Verify it's hex
                let parts: Vec<&str> = bracket_content.split(':').collect();
                if parts.len() == 2 
                    && parts[0].chars().all(|c| c.is_ascii_hexdigit())
                    && parts[1].chars().all(|c| c.is_ascii_hexdigit()) 
                {
                    result = Some(bracket_content.to_lowercase());
                }
            }
            
            remaining = &remaining[start+end+1..];
        } else {
            break;
        }
    }
    
    result
}

/// Extract GPU name from lspci line.
fn extract_gpu_name_from_lspci(line: &str) -> String {
    // Try to find the name between the class description and PCI IDs
    // "01:00.0 3D controller [0302]: NVIDIA Corporation GH100 [H100] [10de:2330]"
    //                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    
    if let Some(colon_pos) = line.find("]:") {
        let after_class = &line[colon_pos + 2..].trim();
        
        // Find where PCI IDs start (last [xxxx:yyyy])
        if let Some(pci_start) = after_class.rfind('[') {
            let name = after_class[..pci_start].trim();
            // Remove trailing [device name] brackets too
            if let Some(name_end) = name.rfind('[') {
                return name[..name_end].trim().to_string();
            }
            return name.to_string();
        }
        return after_class.to_string();
    }
    
    line.to_string()
}

// =============================================================================
// PCI VENDOR LOOKUP
// =============================================================================

/// Parse PCI vendor ID to GpuVendor.
///
/// # Arguments
///
/// * `vendor_id` - Hex string (e.g., "10de", "0x10de")
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::gpu::parse_pci_vendor;
/// use hardware_report::GpuVendor;
///
/// assert_eq!(parse_pci_vendor("10de"), GpuVendor::Nvidia);
/// assert_eq!(parse_pci_vendor("0x1002"), GpuVendor::Amd);
/// ```
pub fn parse_pci_vendor(vendor_id: &str) -> GpuVendor {
    GpuVendor::from_pci_vendor(vendor_id)
}

// =============================================================================
// UNIT TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nvidia_smi_output() {
        let output = r#"0, NVIDIA H100 80GB HBM3, GPU-12345678-1234-1234-1234-123456789abc, 81920, 81000, 00000000:01:00.0, 535.129.03, 9.0
1, NVIDIA H100 80GB HBM3, GPU-87654321-4321-4321-4321-cba987654321, 81920, 80500, 00000000:02:00.0, 535.129.03, 9.0"#;
        
        let gpus = parse_nvidia_smi_output(output).unwrap();
        
        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].index, 0);
        assert_eq!(gpus[0].name, "NVIDIA H100 80GB HBM3");
        assert_eq!(gpus[0].memory_total_mb, 81920);
        assert_eq!(gpus[0].memory_free_mb, Some(81000));
        assert_eq!(gpus[0].driver_version, Some("535.129.03".to_string()));
        assert_eq!(gpus[0].compute_capability, Some("9.0".to_string()));
        assert_eq!(gpus[0].vendor, GpuVendor::Nvidia);
    }

    #[test]
    fn test_parse_nvidia_smi_empty() {
        let output = "";
        let gpus = parse_nvidia_smi_output(output).unwrap();
        assert!(gpus.is_empty());
    }

    #[test]
    fn test_parse_lspci_gpu_output() {
        let output = r#"
00:02.0 VGA compatible controller [0300]: Intel Corporation Device [8086:9a49] (rev 01)
01:00.0 3D controller [0302]: NVIDIA Corporation GH100 [H100 SXM5 80GB] [10de:2330] (rev a1)
02:00.0 3D controller [0302]: NVIDIA Corporation GH100 [H100 SXM5 80GB] [10de:2330] (rev a1)
"#;
        
        let gpus = parse_lspci_gpu_output(output).unwrap();
        
        assert_eq!(gpus.len(), 3);
        
        // Intel GPU
        assert_eq!(gpus[0].vendor, GpuVendor::Intel);
        assert_eq!(gpus[0].pci_id, "8086:9a49");
        
        // NVIDIA GPUs
        assert_eq!(gpus[1].vendor, GpuVendor::Nvidia);
        assert_eq!(gpus[1].pci_id, "10de:2330");
        assert_eq!(gpus[1].pci_bus_id, Some("01:00.0".to_string()));
    }

    #[test]
    fn test_extract_pci_id() {
        assert_eq!(
            extract_pci_id("...controller [0302]: NVIDIA [10de:2330] (rev a1)"),
            Some("10de:2330".to_string())
        );
        assert_eq!(
            extract_pci_id("...controller [0300]: Intel [8086:9a49]"),
            Some("8086:9a49".to_string())
        );
        assert_eq!(extract_pci_id("no pci id here"), None);
    }

    #[test]
    fn test_parse_pci_vendor() {
        assert_eq!(parse_pci_vendor("10de"), GpuVendor::Nvidia);
        assert_eq!(parse_pci_vendor("0x10de"), GpuVendor::Nvidia);
        assert_eq!(parse_pci_vendor("1002"), GpuVendor::Amd);
        assert_eq!(parse_pci_vendor("8086"), GpuVendor::Intel);
        assert_eq!(parse_pci_vendor("unknown"), GpuVendor::Unknown);
    }
}
```

---

## Step 4: Memory Enhancements

### 4.1 Update MemoryModule Struct

**File:** `src/domain/entities.rs`

**Where:** Update the existing `MemoryModule` struct

```rust
// =============================================================================
// MEMORY MODULE STRUCT
// =============================================================================
//
// KEY ADDITION: part_number field for asset tracking!
//
// CMDB use case: "Which exact memory do we need to order for replacement?"
// Answer: Look up the part_number and order that exact module.
// =============================================================================

/// Memory technology type.
///
/// # References
///
/// - [JEDEC Standards](https://www.jedec.org/)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub enum MemoryType {
    Ddr3,
    Ddr4,
    Ddr5,
    Lpddr4,
    Lpddr5,
    Hbm2,
    Hbm3,
    #[default]
    Unknown,
}

impl MemoryType {
    /// Parse memory type from string.
    pub fn from_string(type_str: &str) -> Self {
        match type_str.to_uppercase().as_str() {
            "DDR3" => MemoryType::Ddr3,
            "DDR4" => MemoryType::Ddr4,
            "DDR5" => MemoryType::Ddr5,
            "LPDDR4" | "LPDDR4X" => MemoryType::Lpddr4,
            "LPDDR5" | "LPDDR5X" => MemoryType::Lpddr5,
            "HBM2" | "HBM2E" => MemoryType::Hbm2,
            "HBM3" | "HBM3E" => MemoryType::Hbm3,
            _ => MemoryType::Unknown,
        }
    }
}

/// Individual memory module (DIMM).
///
/// # New Fields (v0.2.0)
///
/// - `part_number` - Manufacturer part number for ordering
/// - `size_bytes` - Numeric size for calculations
/// - `speed_mhz` - Numeric speed for comparisons
///
/// # Example
///
/// ```rust
/// use hardware_report::MemoryModule;
///
/// // Calculate total memory
/// let total_gb: f64 = modules.iter()
///     .map(|m| m.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
///     .sum();
///
/// // Find part number for ordering replacement
/// let part = modules[0].part_number.as_deref().unwrap_or("Unknown");
/// println!("Order part: {}", part);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryModule {
    /// Physical slot location (e.g., "DIMM_A1", "ChannelA-DIMM0").
    pub location: String,
    
    /// Bank locator (e.g., "BANK 0", "P0 CHANNEL A").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bank_locator: Option<String>,
    
    /// Module size in bytes.
    ///
    /// PRIMARY SIZE FIELD - use for calculations.
    #[serde(default)]
    pub size_bytes: u64,
    
    /// Module size as string (e.g., "32 GB").
    ///
    /// For display and backward compatibility.
    pub size: String,
    
    /// Memory type enum.
    #[serde(default)]
    pub memory_type: MemoryType,
    
    /// Memory type as string (e.g., "DDR4", "DDR5").
    #[serde(rename = "type")]
    pub type_: String,
    
    /// Speed in MT/s (megatransfers per second).
    ///
    /// DDR4-3200 = 3200 MT/s
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed_mts: Option<u32>,
    
    /// Configured clock speed in MHz.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed_mhz: Option<u32>,
    
    /// Speed as string (e.g., "3200 MT/s").
    pub speed: String,
    
    /// Manufacturer name (e.g., "Samsung", "Micron", "SK Hynix").
    pub manufacturer: String,
    
    /// Module serial number.
    pub serial: String,
    
    /// Manufacturer part number.
    ///
    /// **IMPORTANT** for procurement and warranty!
    ///
    /// Examples:
    /// - "M393A4K40EB3-CWE" (Samsung 32GB DDR4-3200)
    /// - "MTA36ASF8G72PZ-3G2E1" (Micron 64GB DDR4-3200)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_number: Option<String>,
    
    /// Number of memory ranks (1, 2, 4, or 8).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    
    /// Data width in bits (64, 72 for ECC).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_width_bits: Option<u32>,
    
    /// Whether ECC is supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ecc: Option<bool>,
    
    /// Configured voltage in volts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voltage: Option<f32>,
}

impl Default for MemoryModule {
    fn default() -> Self {
        Self {
            location: String::new(),
            bank_locator: None,
            size_bytes: 0,
            size: String::new(),
            memory_type: MemoryType::Unknown,
            type_: String::new(),
            speed_mts: None,
            speed_mhz: None,
            speed: String::new(),
            manufacturer: String::new(),
            serial: String::new(),
            part_number: None,
            rank: None,
            data_width_bits: None,
            ecc: None,
            voltage: None,
        }
    }
}
```

---

## Step 5: Network Enhancements

### 5.1 Update NetworkInterface Struct

**File:** `src/domain/entities.rs`

```rust
// =============================================================================
// NETWORK INTERFACE TYPE ENUM
// =============================================================================

/// Network interface type classification.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub enum NetworkInterfaceType {
    Ethernet,
    Wireless,
    Loopback,
    Bridge,
    Vlan,
    Bond,
    Veth,
    TunTap,
    Infiniband,
    #[default]
    Unknown,
}

/// Network interface information.
///
/// # New Fields (v0.2.0)
///
/// - `driver` / `driver_version` - For compatibility tracking
/// - `speed_mbps` - Numeric speed
/// - `mtu` - Maximum transmission unit
/// - `is_up` / `is_virtual` - State flags
///
/// # Example
///
/// ```rust
/// // Find all 10G+ physical interfaces that are up
/// let fast_nics: Vec<_> = interfaces.iter()
///     .filter(|i| i.is_up && !i.is_virtual && i.speed_mbps.unwrap_or(0) >= 10000)
///     .collect();
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInterface {
    /// Interface name (e.g., "eth0", "ens192").
    pub name: String,
    
    /// MAC address (e.g., "00:11:22:33:44:55").
    pub mac: String,
    
    /// Primary IPv4 address.
    pub ip: String,
    
    /// Network prefix length (e.g., "24").
    pub prefix: String,
    
    /// Link speed in Mbps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed_mbps: Option<u32>,
    
    /// Link speed as string.
    pub speed: Option<String>,
    
    /// Interface type enum.
    #[serde(default)]
    pub interface_type: NetworkInterfaceType,
    
    /// Interface type as string.
    #[serde(rename = "type")]
    pub type_: String,
    
    /// Hardware vendor name.
    pub vendor: String,
    
    /// Hardware model.
    pub model: String,
    
    /// PCI vendor:device ID.
    pub pci_id: String,
    
    /// NUMA node affinity.
    pub numa_node: Option<i32>,
    
    /// Kernel driver in use (e.g., "igb", "mlx5_core").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    
    /// Driver version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driver_version: Option<String>,
    
    /// Firmware version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<String>,
    
    /// Maximum Transmission Unit in bytes.
    #[serde(default)]
    pub mtu: u32,
    
    /// Whether interface is operationally up.
    #[serde(default)]
    pub is_up: bool,
    
    /// Whether this is a virtual interface.
    #[serde(default)]
    pub is_virtual: bool,
    
    /// Link detected (carrier present).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carrier: Option<bool>,
    
    /// Duplex mode ("full", "half").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duplex: Option<String>,
}

impl Default for NetworkInterface {
    fn default() -> Self {
        Self {
            name: String::new(),
            mac: String::new(),
            ip: String::new(),
            prefix: String::new(),
            speed_mbps: None,
            speed: None,
            interface_type: NetworkInterfaceType::Unknown,
            type_: String::new(),
            vendor: String::new(),
            model: String::new(),
            pci_id: String::new(),
            numa_node: None,
            driver: None,
            driver_version: None,
            firmware_version: None,
            mtu: 1500,
            is_up: false,
            is_virtual: false,
            carrier: None,
            duplex: None,
        }
    }
}
```

---

## Step 6: Update Cargo.toml

**File:** `Cargo.toml`

Add feature flags for optional dependencies:

```toml
[package]
name = "hardware_report"
version = "0.2.0"  # Bump version for breaking changes
edition = "2021"
authors = ["Kenny Sheridan"]
description = "A tool for generating hardware information reports"

[dependencies]
# Existing dependencies...
lazy_static = "1.4"
tonic = "0.10"
reqwest = { version = "0.11", features = ["json"] }
structopt = "0.3"
sysinfo = "0.32.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.4", features = ["derive"] }
thiserror = "1.0"
log = "0.4"
env_logger = "0.11.5"
regex = "1.11.1"
toml = "0.8.19"
libc = "0.2.161"
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# NEW: Optional NVIDIA GPU support via NVML
# Requires NVIDIA driver at runtime
nvml-wrapper = { version = "0.9", optional = true }

# x86-specific CPU detection (only on x86/x86_64)
[target.'cfg(any(target_arch = "x86", target_arch = "x86_64"))'.dependencies]
raw-cpuid = { version = "11", optional = true }

[features]
default = []
nvidia = ["nvml-wrapper"]
x86-cpu = ["raw-cpuid"]
full = ["nvidia", "x86-cpu"]

[dev-dependencies]
tempfile = "3.8"
assert_fs = "1.0"
predicates = "3.0"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[lib]
name = "hardware_report"
path = "src/lib.rs"

[[bin]]
name = "hardware_report"
path = "src/bin/hardware_report.rs"
```

---

## Summary: Implementation Checklist

Use this checklist as you implement:

### Entities (`src/domain/entities.rs`)

- [ ] Add `StorageType` enum (after line 205)
- [ ] Update `StorageDevice` struct with new fields
- [ ] Add `Default` impl for `StorageType`
- [ ] Add `Default` impl for `StorageDevice`
- [ ] Add `CpuCacheInfo` struct
- [ ] Update `CpuInfo` struct with new fields
- [ ] Add `GpuVendor` enum
- [ ] Update `GpuDevice` struct with numeric memory
- [ ] Update `MemoryModule` with `part_number`
- [ ] Add `NetworkInterfaceType` enum
- [ ] Update `NetworkInterface` with driver fields

### Parsers (`src/domain/parsers/`)

- [ ] Update `storage.rs` with new functions
- [ ] Update `cpu.rs` with sysfs/cpuinfo parsing
- [ ] Create `gpu.rs` module (new file)
- [ ] Update `mod.rs` to export `gpu`

### Adapters (`src/adapters/secondary/system/linux.rs`)

- [ ] Update `get_storage_info` with sysfs detection
- [ ] Update `get_cpu_info` with frequency/cache
- [ ] Update `get_gpu_info` with multi-method detection
- [ ] Add helper methods for sysfs reading

### Configuration

- [ ] Update `Cargo.toml` with features
- [ ] Update version to `0.2.0`

### Testing

- [ ] Run `cargo check` after each change
- [ ] Run `cargo test` to verify parsers
- [ ] Test on ARM hardware if available

---

## LeetCode Practice Problems

To solidify your understanding of the patterns used, practice these:

| Pattern | LeetCode Problem | Used In |
|---------|------------------|---------|
| Hash Map Lookup | LC #1 Two Sum | PCI vendor ID → vendor name |
| String Parsing | LC #8 atoi | sysfs frequency parsing |
| String Parsing | LC #468 Validate IP Address | MAC address validation |
| Tree Traversal | LC #102 Level Order | Cache hierarchy, sysfs directory walking |
| Merge | LC #88 Merge Sorted Array | Merging GPU info from multiple sources |
| Filter | LC #283 Move Zeroes | Filter virtual devices |
| Categorization | LC #49 Group Anagrams | StorageType, GpuVendor classification |
| State Machine | LC #65 Valid Number | Parsing multi-line /proc/cpuinfo |
| Pattern Matching | LC #28 Find Index | Extract PCI ID from lspci |

Good luck with your implementation! Type it in yourself to learn the patterns.

<function_calls>
<invoke name="todowrite">
<parameter name="todos">[{"id": "learn-1", "content": "Update ENHANCEMENTS.md with implementation guide overview", "status": "completed", "priority": "high"}, {"id": "learn-2", "content": "Update STORAGE_DETECTION.md with full implementation code", "status": "completed", "priority": "high"}, {"id": "learn-3", "content": "Update GPU_DETECTION.md with full implementation code", "status": "in_progress", "priority": "high"}, {"id": "learn-4", "content": "Update CPU_ENHANCEMENTS.md with full implementation code", "status": "pending", "priority": "high"}, {"id": "learn-5", "content": "Update MEMORY_ENHANCEMENTS.md with full implementation code", "status": "pending", "priority": "medium"}, {"id": "learn-6", "content": "Update NETWORK_ENHANCEMENTS.md with full implementation code", "status": "pending", "priority": "medium"}, {"id": "learn-7", "content": "Create step-by-step implementation checklist", "status": "pending", "priority": "high"}]