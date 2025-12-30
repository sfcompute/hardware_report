# Memory Enhancement Plan

> **Category:** Data Gap  
> **Target Platforms:** Linux (x86_64, aarch64)  
> **Priority:** Medium - Missing DIMM part_number and numeric fields

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

The `MemoryModule` structure lacks the `part_number` field and uses string-based sizes:

```rust
// Current struct - missing fields
pub struct MemoryModule {
    pub size: String,          // String, not numeric
    pub type_: String,
    pub speed: String,         // String, not numeric
    pub location: String,
    pub manufacturer: String,
    pub serial: String,
    // Missing: part_number, rank, configured_voltage
}
```

### Impact

- Cannot track memory part numbers for procurement/warranty
- Size as string breaks capacity calculations
- No memory rank information for performance analysis
- Missing voltage data for power analysis

### Requirements

1. **Add part_number field** - For asset tracking
2. **Numeric size field** - `size_bytes: u64`
3. **Numeric speed field** - `speed_mhz: Option<u32>`
4. **Additional metadata** - rank, voltage, bank locator

---

## Current Implementation

### Location

- **Entity:** `src/domain/entities.rs:178-205`
- **Adapter:** `src/adapters/secondary/system/linux.rs:104-151`
- **Parser:** `src/domain/parsers/memory.rs`

### Current Detection Flow

```
┌──────────────────────────────────────────┐
│ LinuxSystemInfoProvider::get_memory_info()│
└──────────────────────────────────────────┘
                    │
                    ▼
         ┌──────────────────┐
         │ free -b          │──────▶ Total memory
         └──────────────────┘
                    │
                    ▼
         ┌───────────────────────┐
         │ dmidecode -t memory   │──────▶ Module details
         │ (requires privileges) │
         └───────────────────────┘
```

---

## Entity Changes

### New MemoryModule Structure

```rust
// src/domain/entities.rs

/// Memory technology type
///
/// # References
///
/// - [JEDEC Standards](https://www.jedec.org/)
/// - [SMBIOS Type 17](https://www.dmtf.org/standards/smbios)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum MemoryType {
    /// DDR4 SDRAM
    Ddr4,
    /// DDR5 SDRAM
    Ddr5,
    /// LPDDR4 (Low Power DDR4)
    Lpddr4,
    /// LPDDR5 (Low Power DDR5)
    Lpddr5,
    /// DDR3 SDRAM
    Ddr3,
    /// HBM (High Bandwidth Memory)
    Hbm,
    /// HBM2
    Hbm2,
    /// HBM3
    Hbm3,
    /// Unknown type
    Unknown,
}

impl MemoryType {
    /// Parse memory type from SMBIOS/dmidecode string
    ///
    /// # Arguments
    ///
    /// * `type_str` - Type string from dmidecode (e.g., "DDR4", "DDR5")
    ///
    /// # Example
    ///
    /// ```
    /// use hardware_report::MemoryType;
    ///
    /// assert_eq!(MemoryType::from_string("DDR4"), MemoryType::Ddr4);
    /// assert_eq!(MemoryType::from_string("LPDDR5"), MemoryType::Lpddr5);
    /// ```
    pub fn from_string(type_str: &str) -> Self {
        match type_str.to_uppercase().as_str() {
            "DDR4" => MemoryType::Ddr4,
            "DDR5" => MemoryType::Ddr5,
            "LPDDR4" | "LPDDR4X" => MemoryType::Lpddr4,
            "LPDDR5" | "LPDDR5X" => MemoryType::Lpddr5,
            "DDR3" => MemoryType::Ddr3,
            "HBM" => MemoryType::Hbm,
            "HBM2" | "HBM2E" => MemoryType::Hbm2,
            "HBM3" | "HBM3E" => MemoryType::Hbm3,
            _ => MemoryType::Unknown,
        }
    }
}

/// Memory form factor
///
/// # References
///
/// - [SMBIOS Type 17 Form Factor](https://www.dmtf.org/standards/smbios)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum MemoryFormFactor {
    /// Standard DIMM
    Dimm,
    /// Small Outline DIMM (laptops)
    SoDimm,
    /// Registered DIMM (servers)
    Rdimm,
    /// Load Reduced DIMM (servers)
    Lrdimm,
    /// Unbuffered DIMM
    Udimm,
    /// Non-volatile DIMM
    Nvdimm,
    /// High Bandwidth Memory
    Hbm,
    /// Unknown form factor
    Unknown,
}

/// Individual memory module (DIMM) information
///
/// Represents a single memory module with comprehensive metadata
/// for CMDB inventory and capacity planning.
///
/// # Detection Methods
///
/// Memory module information is gathered from:
/// 1. **dmidecode -t memory** - SMBIOS Type 17 data (requires privileges)
/// 2. **sysfs /sys/devices/system/memory** - Basic memory info
/// 3. **sysinfo crate** - Total memory fallback
///
/// # Part Number
///
/// The `part_number` field contains the manufacturer's part number,
/// which is essential for:
/// - Procurement and ordering replacements
/// - Warranty tracking
/// - Compatibility verification
///
/// # Example
///
/// ```
/// use hardware_report::MemoryModule;
///
/// // Calculate total memory from modules
/// let total_gb: f64 = modules.iter()
///     .map(|m| m.size_bytes as f64)
///     .sum::<f64>() / (1024.0 * 1024.0 * 1024.0);
/// ```
///
/// # References
///
/// - [JEDEC Memory Standards](https://www.jedec.org/)
/// - [SMBIOS Specification](https://www.dmtf.org/standards/smbios)
/// - [dmidecode](https://www.nongnu.org/dmidecode/)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryModule {
    /// Physical slot/bank locator (e.g., "DIMM_A1", "ChannelA-DIMM0")
    ///
    /// From SMBIOS "Locator" field.
    pub location: String,
    
    /// Bank locator (e.g., "BANK 0", "P0 CHANNEL A")
    ///
    /// From SMBIOS "Bank Locator" field.
    pub bank_locator: Option<String>,
    
    /// Module size in bytes
    ///
    /// Primary size field for calculations.
    pub size_bytes: u64,
    
    /// Module size as human-readable string (e.g., "32 GB")
    ///
    /// Convenience field for display.
    pub size: String,
    
    /// Memory technology type
    pub memory_type: MemoryType,
    
    /// Memory type as string (e.g., "DDR4", "DDR5")
    ///
    /// For backward compatibility.
    pub type_: String,
    
    /// Memory speed in MT/s (megatransfers per second)
    ///
    /// This is the data rate, not the clock frequency.
    /// DDR4-3200 = 3200 MT/s = 1600 MHz clock.
    pub speed_mts: Option<u32>,
    
    /// Configured clock speed in MHz
    pub speed_mhz: Option<u32>,
    
    /// Speed as string (e.g., "3200 MT/s")
    ///
    /// For backward compatibility.
    pub speed: String,
    
    /// Form factor
    pub form_factor: MemoryFormFactor,
    
    /// Manufacturer name (e.g., "Samsung", "Micron", "SK Hynix")
    pub manufacturer: String,
    
    /// Module serial number
    pub serial: String,
    
    /// Manufacturer part number
    ///
    /// Essential for procurement and warranty tracking.
    ///
    /// # Example
    ///
    /// - "M393A4K40EB3-CWE" (Samsung 32GB DDR4-3200)
    /// - "MTA36ASF8G72PZ-3G2E1" (Micron 64GB DDR4-3200)
    pub part_number: Option<String>,
    
    /// Number of memory ranks
    ///
    /// Single rank (1R), Dual rank (2R), Quad rank (4R), Octal rank (8R).
    /// Higher rank counts can affect performance and compatibility.
    pub rank: Option<u32>,
    
    /// Data width in bits (e.g., 64, 72 for ECC)
    pub data_width_bits: Option<u32>,
    
    /// Total width in bits (includes ECC bits if present)
    pub total_width_bits: Option<u32>,
    
    /// Whether ECC (Error Correcting Code) is supported
    pub ecc: Option<bool>,
    
    /// Configured voltage in volts (e.g., 1.2, 1.35)
    pub voltage: Option<f32>,
    
    /// Minimum voltage in volts
    pub voltage_min: Option<f32>,
    
    /// Maximum voltage in volts
    pub voltage_max: Option<f32>,
    
    /// Asset tag (if set by administrator)
    pub asset_tag: Option<String>,
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
            form_factor: MemoryFormFactor::Unknown,
            manufacturer: String::new(),
            serial: String::new(),
            part_number: None,
            rank: None,
            data_width_bits: None,
            total_width_bits: None,
            ecc: None,
            voltage: None,
            voltage_min: None,
            voltage_max: None,
            asset_tag: None,
        }
    }
}

/// System memory information
///
/// Container for overall memory statistics and individual modules.
///
/// # References
///
/// - [/proc/meminfo](https://man7.org/linux/man-pages/man5/proc.5.html)
/// - [SMBIOS Type 16 & 17](https://www.dmtf.org/standards/smbios)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryInfo {
    /// Total system memory in bytes
    pub total_bytes: u64,
    
    /// Total memory as human-readable string (e.g., "256 GB")
    pub total: String,
    
    /// Primary memory type across all modules
    pub type_: String,
    
    /// Primary memory speed
    pub speed: String,
    
    /// Number of populated DIMM slots
    pub populated_slots: u32,
    
    /// Total number of DIMM slots
    pub total_slots: Option<u32>,
    
    /// Maximum supported memory capacity in bytes
    pub max_capacity_bytes: Option<u64>,
    
    /// Whether ECC is enabled system-wide
    pub ecc_enabled: Option<bool>,
    
    /// Individual memory modules
    pub modules: Vec<MemoryModule>,
}
```

---

## Detection Method Details

### Method 1: dmidecode -t memory

**When:** Linux with privileges (primary source)

**Command:**
```bash
dmidecode -t 17  # Memory Device (each DIMM)
dmidecode -t 16  # Physical Memory Array (capacity/slots)
```

**SMBIOS Type 17 Fields:**

| Field | Description | Maps To |
|-------|-------------|---------|
| Size | Module size | `size_bytes`, `size` |
| Locator | Slot name | `location` |
| Bank Locator | Bank name | `bank_locator` |
| Type | DDR4, DDR5, etc. | `type_`, `memory_type` |
| Speed | MT/s rating | `speed_mts`, `speed` |
| Configured Memory Speed | Actual MHz | `speed_mhz` |
| Manufacturer | OEM name | `manufacturer` |
| Serial Number | Serial | `serial` |
| Part Number | OEM part# | `part_number` |
| Rank | 1, 2, 4, 8 | `rank` |
| Configured Voltage | Volts | `voltage` |
| Form Factor | DIMM, SODIMM | `form_factor` |
| Data Width | 64, 72 bits | `data_width_bits` |
| Total Width | 64, 72 bits | `total_width_bits` |

**Example dmidecode output:**
```
Memory Device
	Size: 32 GB
	Locator: DIMM_A1
	Bank Locator: BANK 0
	Type: DDR4
	Speed: 3200 MT/s
	Manufacturer: Samsung
	Serial Number: 12345678
	Part Number: M393A4K40EB3-CWE
	Rank: 2
	Configured Memory Speed: 3200 MT/s
	Configured Voltage: 1.2 V
```

**References:**
- [dmidecode](https://www.nongnu.org/dmidecode/)
- [SMBIOS Specification](https://www.dmtf.org/standards/smbios)

---

### Method 2: sysfs /sys/devices/system/memory

**When:** Basic memory info without privileges

**Paths:**
```
/sys/devices/system/memory/
├── block_size_bytes        # Memory block size
├── memory0/                # First memory block
│   ├── online              # 1 if online
│   ├── state               # online/offline
│   └── phys_index          # Physical address
└── ...
```

**Limitation:** Does not provide DIMM-level details like part number.

---

### Method 3: /proc/meminfo

**When:** Total memory fallback

**Path:** `/proc/meminfo`

**Format:**
```
MemTotal:       263736560 kB
MemFree:         8472348 kB
MemAvailable:  245678912 kB
...
```

---

## Parser Implementation

### File: `src/domain/parsers/memory.rs`

```rust
//! Memory information parsing functions
//!
//! This module provides pure parsing functions for memory information from
//! various sources, primarily dmidecode output.
//!
//! # References
//!
//! - [SMBIOS Specification](https://www.dmtf.org/standards/smbios)
//! - [dmidecode](https://www.nongnu.org/dmidecode/)

use crate::domain::{MemoryFormFactor, MemoryInfo, MemoryModule, MemoryType};

/// Parse dmidecode Type 17 (Memory Device) output
///
/// # Arguments
///
/// * `output` - Output from `dmidecode -t 17`
///
/// # Returns
///
/// Vector of memory modules with all available fields populated.
///
/// # Example
///
/// ```
/// use hardware_report::domain::parsers::memory::parse_dmidecode_memory_device;
///
/// let output = r#"
/// Memory Device
///     Size: 32 GB
///     Locator: DIMM_A1
///     Part Number: M393A4K40EB3-CWE
/// "#;
///
/// let modules = parse_dmidecode_memory_device(output).unwrap();
/// assert_eq!(modules[0].part_number, Some("M393A4K40EB3-CWE".to_string()));
/// ```
///
/// # References
///
/// - [SMBIOS Type 17](https://www.dmtf.org/standards/smbios)
pub fn parse_dmidecode_memory_device(output: &str) -> Result<Vec<MemoryModule>, String> {
    todo!()
}

/// Parse memory size string to bytes
///
/// # Arguments
///
/// * `size_str` - Size string (e.g., "32 GB", "16384 MB", "No Module Installed")
///
/// # Returns
///
/// Size in bytes, or 0 if not installed/unknown.
///
/// # Example
///
/// ```
/// use hardware_report::domain::parsers::memory::parse_memory_size;
///
/// assert_eq!(parse_memory_size("32 GB"), 32 * 1024 * 1024 * 1024);
/// assert_eq!(parse_memory_size("16384 MB"), 16384 * 1024 * 1024);
/// assert_eq!(parse_memory_size("No Module Installed"), 0);
/// ```
pub fn parse_memory_size(size_str: &str) -> u64 {
    let s = size_str.trim();
    
    if s.contains("No Module") || s.contains("Unknown") || s.contains("Not Installed") {
        return 0;
    }
    
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 2 {
        return 0;
    }
    
    let value: u64 = match parts[0].parse() {
        Ok(v) => v,
        Err(_) => return 0,
    };
    
    match parts[1].to_uppercase().as_str() {
        "GB" => value * 1024 * 1024 * 1024,
        "MB" => value * 1024 * 1024,
        "KB" => value * 1024,
        _ => 0,
    }
}

/// Parse memory speed to MT/s
///
/// # Arguments
///
/// * `speed_str` - Speed string (e.g., "3200 MT/s", "2666 MHz")
///
/// # Returns
///
/// Speed in MT/s.
pub fn parse_memory_speed(speed_str: &str) -> Option<u32> {
    let s = speed_str.trim();
    
    if s.contains("Unknown") {
        return None;
    }
    
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    
    parts[0].parse().ok()
}

/// Parse /proc/meminfo for total memory
///
/// # Arguments
///
/// * `content` - Content of /proc/meminfo
///
/// # Returns
///
/// Total memory in bytes.
///
/// # References
///
/// - [/proc/meminfo](https://man7.org/linux/man-pages/man5/proc.5.html)
pub fn parse_proc_meminfo(content: &str) -> Result<u64, String> {
    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(kb) = parts[1].parse::<u64>() {
                    return Ok(kb * 1024); // Convert KB to bytes
                }
            }
        }
    }
    Err("MemTotal not found in /proc/meminfo".to_string())
}

/// Parse form factor string to enum
///
/// # Arguments
///
/// * `ff_str` - Form factor from dmidecode (e.g., "DIMM", "SODIMM")
pub fn parse_form_factor(ff_str: &str) -> MemoryFormFactor {
    match ff_str.trim().to_uppercase().as_str() {
        "DIMM" => MemoryFormFactor::Dimm,
        "SODIMM" | "SO-DIMM" => MemoryFormFactor::SoDimm,
        "RDIMM" => MemoryFormFactor::Rdimm,
        "LRDIMM" => MemoryFormFactor::Lrdimm,
        "UDIMM" => MemoryFormFactor::Udimm,
        "NVDIMM" => MemoryFormFactor::Nvdimm,
        _ => MemoryFormFactor::Unknown,
    }
}
```

---

## Testing Requirements

### Unit Tests

| Test | Description |
|------|-------------|
| `test_parse_dmidecode_memory` | Parse full dmidecode output |
| `test_parse_memory_size` | Size string parsing |
| `test_parse_memory_speed` | Speed string parsing |
| `test_parse_proc_meminfo` | /proc/meminfo parsing |
| `test_memory_type_from_string` | Type enum conversion |
| `test_form_factor_parsing` | Form factor parsing |

### Integration Tests

| Test | Platform | Description |
|------|----------|-------------|
| `test_memory_detection` | Linux | Full memory detection |
| `test_memory_without_sudo` | Linux | Fallback without privileges |

---

## References

### Official Documentation

| Resource | URL |
|----------|-----|
| JEDEC Standards | https://www.jedec.org/ |
| SMBIOS Specification | https://www.dmtf.org/standards/smbios |
| dmidecode | https://www.nongnu.org/dmidecode/ |
| /proc/meminfo | https://man7.org/linux/man-pages/man5/proc.5.html |

---

## Changelog

| Date | Changes |
|------|---------|
| 2024-12-29 | Initial specification |
