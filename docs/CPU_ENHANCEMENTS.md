# CPU Enhancement Plan

> **Category:** Critical Issue  
> **Target Platforms:** Linux (x86_64, aarch64)  
> **Priority:** Critical - CPU frequency not exposed, cache sizes missing

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Current Implementation](#current-implementation)
3. [Multi-Method Detection Strategy](#multi-method-detection-strategy)
4. [Entity Changes](#entity-changes)
5. [Detection Method Details](#detection-method-details)
6. [Adapter Implementation](#adapter-implementation)
7. [Parser Implementation](#parser-implementation)
8. [Architecture-Specific Handling](#architecture-specific-handling)
9. [Testing Requirements](#testing-requirements)
10. [References](#references)

---

## Problem Statement

### Current Issue

The `CpuInfo` structure lacks critical fields for CMDB inventory:

```rust
// Current struct - limited fields
pub struct CpuInfo {
    pub model: String,
    pub cores: u32,
    pub threads: u32,
    pub sockets: u32,
    pub speed: String,  // String format, unreliable
}
```

Issues:
1. **Frequency as String** - `speed: "2300.000 MHz"` cannot be parsed reliably
2. **No cache information** - L1/L2/L3 cache sizes missing
3. **No architecture field** - Cannot distinguish x86_64 vs aarch64
4. **No CPU flags** - Missing feature detection (AVX, SVE, etc.)

### Impact

- CMDB uses hardcoded 2100 MHz for CPU frequency
- Cannot assess cache hierarchy for performance analysis
- Cannot verify CPU features for workload compatibility

### Requirements

1. **Numeric frequency field** - `frequency_mhz: u32`
2. **Cache size fields** - L1d, L1i, L2, L3 in kilobytes
3. **Architecture detection** - x86_64, aarch64, etc.
4. **CPU flags/features** - Vector extensions, virtualization, etc.
5. **Multi-method detection** - sysfs, CPUID, lscpu, sysinfo

---

## Current Implementation

### Location

- **Entity:** `src/domain/entities.rs:163-175`
- **Adapter:** `src/adapters/secondary/system/linux.rs:67-102`
- **Parser:** `src/domain/parsers/cpu.rs`

### Current Detection Flow

```
┌─────────────────────────────────────────┐
│ LinuxSystemInfoProvider::get_cpu_info() │
└─────────────────────────────────────────┘
                    │
                    ▼
         ┌──────────────────┐
         │ lscpu            │
         └──────────────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │ dmidecode -t processor│
         │ (requires privileges) │
         └──────────────────────┘
                    │
                    ▼
         Combine and return CpuInfo
```

### Current Limitations

| Limitation | Impact |
|------------|--------|
| No sysfs reads | Misses cpufreq data |
| No CPUID access | Misses cache details on x86 |
| Speed as string | Consumer parsing issues |
| No cache info | Missing CMDB fields |
| No flags/features | Cannot verify capabilities |

---

## Multi-Method Detection Strategy

### Detection Priority Chain

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CPU DETECTION CHAIN                                   │
│                                                                              │
│  Priority 1: sysfs /sys/devices/system/cpu                                  │
│  ├── Most reliable for frequency and cache                                  │
│  ├── Works on all Linux architectures                                       │
│  ├── cpufreq for frequency                                                  │
│  └── cache/index* for cache sizes                                          │
│                          │                                                   │
│                          ▼ (for x86 detailed cache info)                    │
│  Priority 2: raw-cpuid crate (x86/x86_64 only)                             │
│  ├── Direct CPUID instruction access                                        │
│  ├── Accurate cache line/associativity info                                │
│  ├── CPU features and flags                                                 │
│  └── Feature-gated: #[cfg(feature = "x86-cpu")]                            │
│                          │                                                   │
│                          ▼ (for model and topology)                         │
│  Priority 3: /proc/cpuinfo                                                  │
│  ├── Model name                                                             │
│  ├── Vendor                                                                 │
│  ├── Flags (x86)                                                           │
│  └── Features (ARM)                                                         │
│                          │                                                   │
│                          ▼ (for topology)                                   │
│  Priority 4: lscpu command                                                  │
│  ├── Socket/core/thread topology                                           │
│  ├── NUMA information                                                       │
│  └── Architecture detection                                                 │
│                          │                                                   │
│                          ▼ (for SMBIOS data)                                │
│  Priority 5: dmidecode -t processor                                        │
│  ├── Serial number (on some systems)                                       │
│  ├── Max frequency                                                          │
│  └── Requires privileges                                                    │
│                          │                                                   │
│                          ▼ (cross-platform fallback)                        │
│  Priority 6: sysinfo crate                                                  │
│  ├── Basic CPU info                                                         │
│  ├── Cross-platform                                                         │
│  └── Limited detail                                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Method Capabilities Matrix

| Method | Frequency | Cache | Model | Vendor | Flags | Topology | Arch |
|--------|-----------|-------|-------|--------|-------|----------|------|
| sysfs | Yes | Yes | No | No | No | Partial | Yes |
| raw-cpuid | Yes | Yes | Yes | Yes | Yes | No | x86 only |
| /proc/cpuinfo | No | No | Yes | Yes | Yes | Partial | Yes |
| lscpu | Partial | Partial | Yes | Yes | Partial | Yes | Yes |
| dmidecode | Yes | No | Yes | Yes | No | Partial | Yes |
| sysinfo | Yes | No | Partial | No | No | Yes | Yes |

---

## Entity Changes

### New CpuInfo Structure

```rust
// src/domain/entities.rs

/// CPU cache level information
///
/// Represents a single cache level (L1d, L1i, L2, L3).
///
/// # References
///
/// - [Intel CPUID](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
/// - [Linux cache sysfs](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-system-cpu)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuCacheInfo {
    /// Cache level (1, 2, or 3)
    pub level: u8,
    
    /// Cache type: "Data", "Instruction", or "Unified"
    pub cache_type: String,
    
    /// Cache size in kilobytes
    pub size_kb: u32,
    
    /// Number of ways of associativity
    pub ways_of_associativity: Option<u32>,
    
    /// Cache line size in bytes
    pub line_size_bytes: Option<u32>,
    
    /// Number of sets
    pub sets: Option<u32>,
    
    /// Whether this cache is shared across cores
    pub shared_cpu_map: Option<String>,
}

/// CPU information with extended details
///
/// Provides comprehensive CPU information for CMDB inventory,
/// including frequency, cache hierarchy, and feature flags.
///
/// # Detection Methods
///
/// CPU information is gathered from multiple sources in priority order:
/// 1. **sysfs** - `/sys/devices/system/cpu` (frequency, cache)
/// 2. **raw-cpuid** - CPUID instruction (x86 only, cache details)
/// 3. **/proc/cpuinfo** - Model, vendor, flags
/// 4. **lscpu** - Topology information
/// 5. **dmidecode** - SMBIOS data (requires privileges)
/// 6. **sysinfo** - Cross-platform fallback
///
/// # Frequency Values
///
/// Multiple frequency values are provided:
/// - `frequency_mhz` - Current or maximum frequency (primary field)
/// - `frequency_min_mhz` - Minimum scaling frequency
/// - `frequency_max_mhz` - Maximum scaling frequency
/// - `frequency_base_mhz` - Base (non-turbo) frequency
///
/// # Cache Hierarchy
///
/// Cache sizes are provided per-core in kilobytes:
/// - `cache_l1d_kb` - L1 data cache
/// - `cache_l1i_kb` - L1 instruction cache  
/// - `cache_l2_kb` - L2 cache (may be per-core or shared)
/// - `cache_l3_kb` - L3 cache (typically shared)
///
/// # Example
///
/// ```
/// use hardware_report::CpuInfo;
///
/// // Calculate total L3 cache
/// let total_l3_mb = cpu.cache_l3_kb.unwrap_or(0) as f64 / 1024.0;
///
/// // Check for AVX-512 support
/// let has_avx512 = cpu.flags.iter().any(|f| f.starts_with("avx512"));
/// ```
///
/// # References
///
/// - [Linux CPU sysfs](https://www.kernel.org/doc/Documentation/cpu-freq/user-guide.rst)
/// - [Intel CPUID](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
/// - [ARM CPU ID registers](https://developer.arm.com/documentation/ddi0487/latest)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuInfo {
    /// CPU model name
    ///
    /// Examples:
    /// - "AMD EPYC 7763 64-Core Processor"
    /// - "Intel(R) Xeon(R) Platinum 8380 CPU @ 2.30GHz"
    /// - "Neoverse-N1"
    pub model: String,
    
    /// CPU vendor identifier
    ///
    /// Values:
    /// - "GenuineIntel" (Intel)
    /// - "AuthenticAMD" (AMD)
    /// - "ARM" (ARM/Ampere/etc)
    pub vendor: String,
    
    /// Number of physical cores per socket
    pub cores: u32,
    
    /// Number of threads per core (hyperthreading/SMT)
    pub threads: u32,
    
    /// Number of CPU sockets
    pub sockets: u32,
    
    /// Total physical cores (cores * sockets)
    pub total_cores: u32,
    
    /// Total logical CPUs (cores * threads * sockets)
    pub total_threads: u32,
    
    /// CPU frequency in MHz (current or max)
    ///
    /// Primary frequency field. This is the most useful value for
    /// general CMDB purposes.
    pub frequency_mhz: u32,
    
    /// Minimum scaling frequency in MHz
    ///
    /// From cpufreq scaling_min_freq.
    pub frequency_min_mhz: Option<u32>,
    
    /// Maximum scaling frequency in MHz
    ///
    /// From cpufreq scaling_max_freq. This is the turbo/boost frequency.
    pub frequency_max_mhz: Option<u32>,
    
    /// Base (non-turbo) frequency in MHz
    ///
    /// From cpufreq base_frequency or CPUID.
    pub frequency_base_mhz: Option<u32>,
    
    /// CPU architecture
    ///
    /// Values: "x86_64", "aarch64", "armv7l", etc.
    pub architecture: String,
    
    /// CPU microarchitecture name
    ///
    /// Examples:
    /// - "Zen3" (AMD)
    /// - "Ice Lake" (Intel)
    /// - "Neoverse N1" (ARM)
    pub microarchitecture: Option<String>,
    
    /// L1 data cache size in kilobytes (per core)
    pub cache_l1d_kb: Option<u32>,
    
    /// L1 instruction cache size in kilobytes (per core)
    pub cache_l1i_kb: Option<u32>,
    
    /// L2 cache size in kilobytes (per core typically)
    pub cache_l2_kb: Option<u32>,
    
    /// L3 cache size in kilobytes (typically shared)
    ///
    /// Note: This is often the total L3 across all cores in a socket.
    pub cache_l3_kb: Option<u32>,
    
    /// Detailed cache information for each level
    pub caches: Vec<CpuCacheInfo>,
    
    /// CPU flags/features
    ///
    /// Examples (x86): "avx", "avx2", "avx512f", "aes", "sse4_2"
    /// Examples (ARM): "fp", "asimd", "sve", "sve2"
    ///
    /// # References
    ///
    /// - [x86 CPUID flags](https://en.wikipedia.org/wiki/CPUID)
    /// - [ARM HWCAP](https://www.kernel.org/doc/html/latest/arm64/elf_hwcaps.html)
    pub flags: Vec<String>,
    
    /// Microcode/firmware version
    pub microcode_version: Option<String>,
    
    /// CPU stepping (revision level)
    pub stepping: Option<u32>,
    
    /// CPU family number
    pub family: Option<u32>,
    
    /// CPU model number (not the name)
    pub model_number: Option<u32>,
    
    /// Virtualization support
    ///
    /// Values: "VT-x", "AMD-V", "none", etc.
    pub virtualization: Option<String>,
    
    /// NUMA nodes count
    pub numa_nodes: u32,
    
    /// Detection methods used
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
```

---

## Detection Method Details

### Method 1: sysfs /sys/devices/system/cpu

**When:** Linux systems (always primary for freq/cache)

**sysfs paths:**

```
/sys/devices/system/cpu/
├── cpu0/
│   ├── cpufreq/
│   │   ├── cpuinfo_max_freq      # Max frequency in kHz
│   │   ├── cpuinfo_min_freq      # Min frequency in kHz
│   │   ├── scaling_cur_freq      # Current frequency in kHz
│   │   ├── scaling_max_freq      # Scaling max in kHz
│   │   ├── scaling_min_freq      # Scaling min in kHz
│   │   └── base_frequency        # Base (non-turbo) freq
│   ├── cache/
│   │   ├── index0/               # L1d typically
│   │   │   ├── level             # Cache level (1, 2, 3)
│   │   │   ├── type              # Data, Instruction, Unified
│   │   │   ├── size              # Size with unit (e.g., "32K")
│   │   │   ├── ways_of_associativity
│   │   │   ├── coherency_line_size
│   │   │   └── number_of_sets
│   │   ├── index1/               # L1i typically
│   │   ├── index2/               # L2 typically
│   │   └── index3/               # L3 typically
│   └── topology/
│       ├── physical_package_id   # Socket ID
│       ├── core_id               # Core ID within socket
│       └── thread_siblings_list  # SMT siblings
├── possible                      # Possible CPU range
├── present                       # Present CPU range
└── online                        # Online CPU range
```

**Frequency parsing:**

```rust
// sysfs reports frequency in kHz, convert to MHz
let freq_khz: u32 = read_sysfs("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")?
    .trim()
    .parse()?;
let freq_mhz = freq_khz / 1000;
```

**Cache size parsing:**

```rust
// sysfs reports size as "32K", "512K", "32768K", "16M", etc.
fn parse_cache_size(size_str: &str) -> Option<u32> {
    let s = size_str.trim();
    if s.ends_with('K') {
        s[..s.len()-1].parse().ok()
    } else if s.ends_with('M') {
        s[..s.len()-1].parse::<u32>().ok().map(|v| v * 1024)
    } else {
        s.parse().ok()
    }
}
```

**References:**
- [CPU sysfs Documentation](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-system-cpu)
- [cpufreq User Guide](https://www.kernel.org/doc/Documentation/cpu-freq/user-guide.rst)

---

### Method 2: raw-cpuid (x86/x86_64 only)

**When:** x86/x86_64 architecture, feature enabled

**Cargo.toml:**
```toml
[target.'cfg(any(target_arch = "x86", target_arch = "x86_64"))'.dependencies]
raw-cpuid = { version = "11", optional = true }

[features]
x86-cpu = ["raw-cpuid"]
```

**Usage:**
```rust
#[cfg(all(feature = "x86-cpu", any(target_arch = "x86", target_arch = "x86_64")))]
fn get_cpu_info_cpuid() -> CpuInfo {
    use raw_cpuid::CpuId;
    
    let cpuid = CpuId::new();
    
    let model = cpuid.get_processor_brand_string()
        .map(|b| b.as_str().trim().to_string())
        .unwrap_or_default();
    
    let vendor = cpuid.get_vendor_info()
        .map(|v| v.as_str().to_string())
        .unwrap_or_default();
    
    // Cache info
    if let Some(cache_params) = cpuid.get_cache_parameters() {
        for cache in cache_params {
            let size_kb = (cache.associativity()
                * cache.physical_line_partitions()
                * cache.coherency_line_size()
                * cache.sets()) as u32 / 1024;
            // ...
        }
    }
    
    // Feature flags
    if let Some(features) = cpuid.get_feature_info() {
        // Check SSE, AVX, etc.
    }
    
    // ...
}
```

**References:**
- [raw-cpuid crate](https://docs.rs/raw-cpuid)
- [Intel CPUID Reference](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)

---

### Method 3: /proc/cpuinfo

**When:** Linux, for model name and flags

**Path:** `/proc/cpuinfo`

**Format (x86):**
```
processor	: 0
vendor_id	: GenuineIntel
cpu family	: 6
model		: 106
model name	: Intel(R) Xeon(R) Platinum 8380 CPU @ 2.30GHz
stepping	: 6
microcode	: 0xd0003a5
cpu MHz		: 2300.000
cache size	: 61440 KB
flags		: fpu vme de pse ... avx avx2 avx512f avx512dq ...
```

**Format (ARM):**
```
processor	: 0
BogoMIPS	: 50.00
Features	: fp asimd evtstrm aes pmull sha1 sha2 crc32 ...
CPU implementer	: 0x41
CPU architecture: 8
CPU variant	: 0x3
CPU part	: 0xd0c
CPU revision	: 1
```

**References:**
- [/proc/cpuinfo](https://man7.org/linux/man-pages/man5/proc.5.html)

---

### Method 4: lscpu

**When:** For topology information

**Command:**
```bash
lscpu -J  # JSON output (preferred)
lscpu     # Text output (fallback)
```

**JSON output:**
```json
{
   "lscpu": [
      {"field": "Architecture:", "data": "x86_64"},
      {"field": "CPU(s):", "data": "128"},
      {"field": "Thread(s) per core:", "data": "2"},
      {"field": "Core(s) per socket:", "data": "32"},
      {"field": "Socket(s):", "data": "2"},
      {"field": "NUMA node(s):", "data": "2"},
      {"field": "Model name:", "data": "AMD EPYC 7763 64-Core Processor"},
      {"field": "CPU max MHz:", "data": "3500.0000"},
      {"field": "L1d cache:", "data": "2 MiB"},
      {"field": "L1i cache:", "data": "2 MiB"},
      {"field": "L2 cache:", "data": "32 MiB"},
      {"field": "L3 cache:", "data": "256 MiB"}
   ]
}
```

**References:**
- [lscpu man page](https://man7.org/linux/man-pages/man1/lscpu.1.html)

---

### Method 5: sysinfo crate

**When:** Cross-platform fallback

**Usage:**
```rust
use sysinfo::System;

let sys = System::new_all();

let frequency = sys.cpus().first()
    .map(|cpu| cpu.frequency() as u32)
    .unwrap_or(0);

let physical_cores = sys.physical_core_count().unwrap_or(0);
let logical_cpus = sys.cpus().len();
```

**References:**
- [sysinfo crate](https://docs.rs/sysinfo)

---

## Architecture-Specific Handling

### x86_64

```rust
#[cfg(target_arch = "x86_64")]
fn detect_cpu_x86(info: &mut CpuInfo) {
    // Use raw-cpuid if available
    #[cfg(feature = "x86-cpu")]
    {
        use raw_cpuid::CpuId;
        let cpuid = CpuId::new();
        // Extract cache, features, etc.
    }
    
    // Read flags from /proc/cpuinfo
    // Parse "flags" line
}
```

### aarch64

```rust
#[cfg(target_arch = "aarch64")]
fn detect_cpu_arm(info: &mut CpuInfo) {
    // Read from /proc/cpuinfo
    // "Features" line instead of "flags"
    // "CPU part" for microarchitecture detection
    
    // Map CPU part to microarchitecture
    // 0xd0c -> "Neoverse N1"
    // 0xd40 -> "Neoverse V1"
    // etc.
}

/// Map ARM CPU part ID to microarchitecture name
///
/// # References
///
/// - [ARM CPU Part Numbers](https://developer.arm.com/documentation/ddi0487/latest)
fn arm_cpu_part_to_name(part: &str) -> Option<&'static str> {
    match part.to_lowercase().as_str() {
        "0xd03" => Some("Cortex-A53"),
        "0xd07" => Some("Cortex-A57"),
        "0xd08" => Some("Cortex-A72"),
        "0xd09" => Some("Cortex-A73"),
        "0xd0a" => Some("Cortex-A75"),
        "0xd0b" => Some("Cortex-A76"),
        "0xd0c" => Some("Neoverse N1"),
        "0xd0d" => Some("Cortex-A77"),
        "0xd40" => Some("Neoverse V1"),
        "0xd41" => Some("Cortex-A78"),
        "0xd44" => Some("Cortex-X1"),
        "0xd49" => Some("Neoverse N2"),
        "0xd4f" => Some("Neoverse V2"),
        _ => None,
    }
}
```

---

## Parser Implementation

### File: `src/domain/parsers/cpu.rs`

```rust
//! CPU information parsing functions
//!
//! This module provides pure parsing functions for CPU information from
//! various sources. All functions take string input and return parsed
//! results without performing I/O.
//!
//! # Supported Formats
//!
//! - sysfs frequency/cache files
//! - /proc/cpuinfo
//! - lscpu text and JSON output
//! - dmidecode processor output
//!
//! # References
//!
//! - [Linux CPU sysfs](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-system-cpu)
//! - [/proc/cpuinfo](https://man7.org/linux/man-pages/man5/proc.5.html)

use crate::domain::{CpuCacheInfo, CpuInfo};

/// Parse sysfs frequency file (in kHz) to MHz
///
/// # Arguments
///
/// * `content` - Content of cpufreq file (e.g., scaling_max_freq)
///
/// # Returns
///
/// Frequency in MHz.
///
/// # Example
///
/// ```
/// use hardware_report::domain::parsers::cpu::parse_sysfs_freq_khz;
///
/// assert_eq!(parse_sysfs_freq_khz("3500000").unwrap(), 3500);
/// ```
pub fn parse_sysfs_freq_khz(content: &str) -> Result<u32, String> {
    let khz: u32 = content.trim().parse()
        .map_err(|e| format!("Invalid frequency: {}", e))?;
    Ok(khz / 1000)
}

/// Parse sysfs cache size (e.g., "32K", "1M")
///
/// # Arguments
///
/// * `content` - Content of cache size file
///
/// # Returns
///
/// Size in kilobytes.
///
/// # Example
///
/// ```
/// use hardware_report::domain::parsers::cpu::parse_sysfs_cache_size;
///
/// assert_eq!(parse_sysfs_cache_size("32K").unwrap(), 32);
/// assert_eq!(parse_sysfs_cache_size("1M").unwrap(), 1024);
/// assert_eq!(parse_sysfs_cache_size("256M").unwrap(), 262144);
/// ```
pub fn parse_sysfs_cache_size(content: &str) -> Result<u32, String> {
    let s = content.trim();
    if s.ends_with('K') {
        s[..s.len()-1].parse()
            .map_err(|e| format!("Invalid cache size: {}", e))
    } else if s.ends_with('M') {
        s[..s.len()-1].parse::<u32>()
            .map(|v| v * 1024)
            .map_err(|e| format!("Invalid cache size: {}", e))
    } else if s.ends_with('G') {
        s[..s.len()-1].parse::<u32>()
            .map(|v| v * 1024 * 1024)
            .map_err(|e| format!("Invalid cache size: {}", e))
    } else {
        s.parse()
            .map_err(|e| format!("Invalid cache size: {}", e))
    }
}

/// Parse /proc/cpuinfo content
///
/// # Arguments
///
/// * `content` - Full content of /proc/cpuinfo
///
/// # Returns
///
/// Partial CpuInfo with fields from cpuinfo.
///
/// # References
///
/// - [/proc filesystem](https://man7.org/linux/man-pages/man5/proc.5.html)
pub fn parse_proc_cpuinfo(content: &str) -> Result<CpuInfo, String> {
    let mut info = CpuInfo::default();
    
    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        
        let key = parts[0].trim();
        let value = parts[1].trim();
        
        match key {
            "model name" => info.model = value.to_string(),
            "vendor_id" => info.vendor = value.to_string(),
            "cpu family" => info.family = value.parse().ok(),
            "model" => info.model_number = value.parse().ok(),
            "stepping" => info.stepping = value.parse().ok(),
            "microcode" => info.microcode_version = Some(value.to_string()),
            "flags" | "Features" => {
                info.flags = value.split_whitespace()
                    .map(String::from)
                    .collect();
            }
            "CPU implementer" => {
                if info.vendor.is_empty() {
                    info.vendor = "ARM".to_string();
                }
            }
            "CPU part" => {
                // ARM CPU part number
                if let Some(arch) = arm_cpu_part_to_name(value) {
                    info.microarchitecture = Some(arch.to_string());
                }
            }
            _ => {}
        }
    }
    
    Ok(info)
}

/// Parse lscpu JSON output
///
/// # Arguments
///
/// * `output` - JSON output from `lscpu -J`
///
/// # References
///
/// - [lscpu](https://man7.org/linux/man-pages/man1/lscpu.1.html)
pub fn parse_lscpu_json(output: &str) -> Result<CpuInfo, String> {
    todo!()
}

/// Parse lscpu text output
///
/// # Arguments
///
/// * `output` - Text output from `lscpu`
pub fn parse_lscpu_text(output: &str) -> Result<CpuInfo, String> {
    todo!()
}

/// Map ARM CPU part ID to microarchitecture name
///
/// # Arguments
///
/// * `part` - CPU part from /proc/cpuinfo (e.g., "0xd0c")
///
/// # References
///
/// - [ARM CPU Part Numbers](https://developer.arm.com/documentation/ddi0487/latest)
pub fn arm_cpu_part_to_name(part: &str) -> Option<&'static str> {
    match part.to_lowercase().as_str() {
        "0xd03" => Some("Cortex-A53"),
        "0xd07" => Some("Cortex-A57"),
        "0xd08" => Some("Cortex-A72"),
        "0xd09" => Some("Cortex-A73"),
        "0xd0a" => Some("Cortex-A75"),
        "0xd0b" => Some("Cortex-A76"),
        "0xd0c" => Some("Neoverse N1"),
        "0xd0d" => Some("Cortex-A77"),
        "0xd40" => Some("Neoverse V1"),
        "0xd41" => Some("Cortex-A78"),
        "0xd44" => Some("Cortex-X1"),
        "0xd49" => Some("Neoverse N2"),
        "0xd4f" => Some("Neoverse V2"),
        "0xd80" => Some("Cortex-A520"),
        "0xd81" => Some("Cortex-A720"),
        _ => None,
    }
}
```

---

## Testing Requirements

### Unit Tests

| Test | Description |
|------|-------------|
| `test_parse_sysfs_freq` | Parse frequency in kHz to MHz |
| `test_parse_cache_size` | Parse K/M/G suffixes |
| `test_parse_proc_cpuinfo_intel` | Parse Intel cpuinfo |
| `test_parse_proc_cpuinfo_amd` | Parse AMD cpuinfo |
| `test_parse_proc_cpuinfo_arm` | Parse ARM cpuinfo |
| `test_parse_lscpu_json` | Parse lscpu JSON |
| `test_arm_cpu_part_mapping` | ARM part to name |

### Integration Tests

| Test | Platform | Description |
|------|----------|-------------|
| `test_cpu_detection_x86` | x86_64 | Full detection on x86 |
| `test_cpu_detection_arm` | aarch64 | Full detection on ARM |
| `test_cpuid_features` | x86_64 | raw-cpuid feature detection |
| `test_sysfs_cache` | Linux | sysfs cache reading |

---

## References

### Official Documentation

| Resource | URL |
|----------|-----|
| Linux CPU sysfs | https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-system-cpu |
| Linux cpufreq | https://www.kernel.org/doc/Documentation/cpu-freq/user-guide.rst |
| Intel CPUID | https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html |
| AMD CPUID | https://www.amd.com/en/support/tech-docs |
| ARM CPU ID | https://developer.arm.com/documentation/ddi0487/latest |
| ARM HWCAP | https://www.kernel.org/doc/html/latest/arm64/elf_hwcaps.html |

### Crate Documentation

| Crate | URL |
|-------|-----|
| raw-cpuid | https://docs.rs/raw-cpuid |
| sysinfo | https://docs.rs/sysinfo |

---

## Changelog

| Date | Changes |
|------|---------|
| 2024-12-29 | Initial specification |
