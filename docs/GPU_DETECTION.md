# GPU Detection Enhancement Plan

> **Category:** Critical Issue  
> **Target Platforms:** Linux (x86_64, aarch64)  
> **Related Files:** `src/domain/entities.rs`, `src/adapters/secondary/system/linux.rs`, `src/domain/parsers/gpu.rs` (new)

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Current Implementation](#current-implementation)
3. [Multi-Method Detection Strategy](#multi-method-detection-strategy)
4. [Entity Changes](#entity-changes)
5. [Detection Method Details](#detection-method-details)
6. [Adapter Implementation](#adapter-implementation)
7. [Parser Implementation](#parser-implementation)
8. [Error Handling](#error-handling)
9. [Testing Requirements](#testing-requirements)
10. [References](#references)

---

## Problem Statement

### Current Issue

The current GPU memory field returns a formatted string that consumers cannot reliably parse:

```rust
// Current output
GpuDevice {
    memory: "80 GB",  // String - cannot parse reliably
    // ...
}

// Consumer code that fails:
let memory_mb = gpu.memory.parse::<f64>().unwrap_or(0.0) as u32 * 1024;
// Result: memory_mb = 0 (parse fails on "80 GB")
```

### Impact

- CMDB inventory shows 0MB VRAM for all GPUs
- `metal-agent` must fall back to shelling out to `nvidia-smi`
- No support for AMD or Intel GPUs
- Detection fails silently

### Requirements

1. Return numeric memory values in MB (u64)
2. Detect GPUs using multiple methods with fallback chain
3. Support NVIDIA, AMD, and Intel GPUs
4. Work on both x86_64 and aarch64 architectures
5. Provide driver version information
6. Include detection method in output for debugging

---

## Current Implementation

### Location

- **Entity:** `src/domain/entities.rs:235-251`
- **Adapter:** `src/adapters/secondary/system/linux.rs:172-231`

### Current Detection Flow

```
┌─────────────────────────────────────────┐
│ LinuxSystemInfoProvider::get_gpu_info() │
└─────────────────────────────────────────┘
                    │
                    ▼
         ┌──────────────────┐
         │ Try nvidia-smi   │
         │ --query-gpu=...  │
         └──────────────────┘
                    │
          success?  │
         ┌──────────┴──────────┐
         │ YES                 │ NO
         ▼                     ▼
    Parse CSV output     ┌──────────────────┐
    Return devices       │ Try lspci -nn    │
                        └──────────────────┘
                                   │
                         Parse VGA/3D lines
                         Return basic info
```

### Current Limitations

| Limitation | Impact |
|------------|--------|
| Only two detection methods | Misses AMD ROCm, Intel GPUs |
| String memory format | Breaks consumer parsing |
| No driver version | Missing CMDB field |
| No PCI bus ID | Can't correlate with NUMA |
| nvidia-smi parsing fragile | Format changes break detection |

---

## Multi-Method Detection Strategy

### Detection Priority Chain

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        GPU DETECTION CHAIN                                   │
│                                                                              │
│  Priority 1: NVML (nvml-wrapper crate)                                      │
│  ├── Most accurate for NVIDIA GPUs                                          │
│  ├── Direct API access, no parsing                                          │
│  ├── Memory in bytes, convert to MB                                         │
│  └── Feature-gated: #[cfg(feature = "nvidia")]                             │
│                          │                                                   │
│                          ▼ (if unavailable or no NVIDIA GPUs)               │
│  Priority 2: nvidia-smi command                                             │
│  ├── Fallback for NVIDIA when NVML unavailable                             │
│  ├── Common on systems without development headers                          │
│  └── Parse --query-gpu output with nounits flag                            │
│                          │                                                   │
│                          ▼ (if unavailable or no NVIDIA GPUs)               │
│  Priority 3: ROCm SMI (rocm-smi)                                            │
│  ├── AMD GPU detection                                                       │
│  ├── Parse JSON output when available                                       │
│  └── Common on AMD GPU systems                                              │
│                          │                                                   │
│                          ▼ (if unavailable or no AMD GPUs)                  │
│  Priority 4: sysfs /sys/class/drm                                           │
│  ├── Linux DRM subsystem                                                     │
│  ├── Works for all GPU vendors                                              │
│  ├── Memory info from /sys/class/drm/card*/device/mem_info_*               │
│  └── Vendor from /sys/class/drm/card*/device/vendor                        │
│                          │                                                   │
│                          ▼ (if no GPUs found)                               │
│  Priority 5: lspci with PCI ID database                                     │
│  ├── Enumerate all VGA/3D controllers                                       │
│  ├── Look up vendor:device in PCI ID database                              │
│  └── No memory info available                                               │
│                          │                                                   │
│                          ▼ (if lspci unavailable)                           │
│  Priority 6: sysinfo crate                                                  │
│  ├── Cross-platform fallback                                                │
│  └── Limited GPU information                                                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Method Capabilities Matrix

| Method | NVIDIA | AMD | Intel | Memory | Driver | PCI Bus | NUMA |
|--------|--------|-----|-------|--------|--------|---------|------|
| NVML | Yes | No | No | Exact | Yes | Yes | Yes |
| nvidia-smi | Yes | No | No | Exact | Yes | Yes | No |
| rocm-smi | No | Yes | No | Exact | Yes | Yes | No |
| sysfs DRM | Yes | Yes | Yes | Varies | No | Yes | Yes |
| lspci | Yes | Yes | Yes | No | No | Yes | No |
| sysinfo | Limited | Limited | Limited | No | No | No | No |

---

## Entity Changes

### New GpuDevice Structure

```rust
// src/domain/entities.rs

/// GPU vendor classification
///
/// # References
///
/// - [PCI Vendor IDs](https://pci-ids.ucw.cz/)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GpuVendor {
    /// NVIDIA Corporation (PCI vendor 0x10de)
    Nvidia,
    /// Advanced Micro Devices (PCI vendor 0x1002)
    Amd,
    /// Intel Corporation (PCI vendor 0x8086)
    Intel,
    /// Apple Inc. (integrated GPUs)
    Apple,
    /// Unknown or unrecognized vendor
    Unknown,
}

impl GpuVendor {
    /// Convert PCI vendor ID to GpuVendor
    ///
    /// # Arguments
    ///
    /// * `vendor_id` - PCI vendor ID as hexadecimal string (e.g., "10de")
    ///
    /// # Example
    ///
    /// ```
    /// use hardware_report::GpuVendor;
    /// 
    /// assert_eq!(GpuVendor::from_pci_vendor("10de"), GpuVendor::Nvidia);
    /// assert_eq!(GpuVendor::from_pci_vendor("1002"), GpuVendor::Amd);
    /// ```
    pub fn from_pci_vendor(vendor_id: &str) -> Self {
        match vendor_id.to_lowercase().as_str() {
            "10de" => GpuVendor::Nvidia,
            "1002" => GpuVendor::Amd,
            "8086" => GpuVendor::Intel,
            _ => GpuVendor::Unknown,
        }
    }
}

/// GPU device information
///
/// Represents a discrete or integrated GPU detected in the system.
/// Memory values are provided in megabytes as unsigned integers for
/// reliable parsing by CMDB consumers.
///
/// # Detection Methods
///
/// GPUs are detected using multiple methods in priority order:
/// 1. **NVML** - NVIDIA Management Library (most accurate for NVIDIA)
/// 2. **nvidia-smi** - NVIDIA command-line tool (fallback)
/// 3. **rocm-smi** - AMD ROCm System Management Interface
/// 4. **sysfs** - Linux `/sys/class/drm` interface
/// 5. **lspci** - PCI device enumeration
/// 6. **sysinfo** - Cross-platform fallback
///
/// # Memory Format
///
/// Memory is always reported in **megabytes** as a `u64`. The previous
/// `memory: String` field (e.g., "80 GB") is deprecated.
///
/// # Example
///
/// ```
/// use hardware_report::GpuDevice;
/// 
/// // Calculate memory in GB from the numeric field
/// let memory_gb = gpu.memory_total_mb as f64 / 1024.0;
/// println!("GPU has {} GB memory", memory_gb);
/// ```
///
/// # References
///
/// - [NVIDIA NVML API](https://docs.nvidia.com/deploy/nvml-api/)
/// - [AMD ROCm SMI](https://rocm.docs.amd.com/projects/rocm_smi_lib/en/latest/)
/// - [Linux DRM Subsystem](https://www.kernel.org/doc/html/latest/gpu/drm-uapi.html)
/// - [PCI ID Database](https://pci-ids.ucw.cz/)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuDevice {
    /// GPU index (0-based, unique per system)
    pub index: u32,
    
    /// GPU product name
    ///
    /// Examples:
    /// - "NVIDIA H100 80GB HBM3"
    /// - "AMD Instinct MI250X"
    /// - "Intel Arc A770"
    pub name: String,
    
    /// GPU UUID (globally unique identifier)
    ///
    /// Format varies by vendor:
    /// - NVIDIA: "GPU-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
    /// - AMD: May be empty or use different format
    pub uuid: String,
    
    /// Total GPU memory in megabytes
    ///
    /// This is the primary memory field and should be used for all
    /// programmatic access. Multiply by 1024 for KB, divide by 1024
    /// for GB.
    ///
    /// # Note
    ///
    /// Returns 0 if memory could not be determined (e.g., lspci-only detection).
    pub memory_total_mb: u64,
    
    /// Available (free) GPU memory in megabytes
    ///
    /// This is a runtime value that reflects current memory usage.
    /// Returns `None` if not queryable (requires NVML or ROCm).
    pub memory_free_mb: Option<u64>,
    
    /// Used GPU memory in megabytes
    ///
    /// Calculated as `memory_total_mb - memory_free_mb` when available.
    pub memory_used_mb: Option<u64>,
    
    /// PCI vendor:device ID (e.g., "10de:2330")
    ///
    /// Format: `{vendor_id}:{device_id}` in lowercase hexadecimal.
    ///
    /// # References
    ///
    /// - [PCI ID Database](https://pci-ids.ucw.cz/)
    pub pci_id: String,
    
    /// PCI bus address (e.g., "0000:01:00.0")
    ///
    /// Format: `{domain}:{bus}:{device}.{function}`
    ///
    /// Useful for correlating with NUMA topology and other PCI devices.
    pub pci_bus_id: Option<String>,
    
    /// GPU vendor classification
    pub vendor: GpuVendor,
    
    /// Vendor name as string (e.g., "NVIDIA", "AMD", "Intel")
    ///
    /// Provided for serialization compatibility. Use `vendor` field
    /// for programmatic comparisons.
    pub vendor_name: String,
    
    /// GPU driver version
    ///
    /// Examples:
    /// - NVIDIA: "535.129.03"
    /// - AMD: "6.3.6"
    pub driver_version: Option<String>,
    
    /// CUDA compute capability (NVIDIA only)
    ///
    /// Format: "major.minor" (e.g., "9.0" for Hopper, "8.9" for Ada)
    ///
    /// # References
    ///
    /// - [CUDA Compute Capability](https://developer.nvidia.com/cuda-gpus)
    pub compute_capability: Option<String>,
    
    /// GPU architecture name
    ///
    /// Examples:
    /// - NVIDIA: "Hopper", "Ada Lovelace", "Ampere"
    /// - AMD: "CDNA2", "RDNA3"
    /// - Intel: "Xe-HPG"
    pub architecture: Option<String>,
    
    /// NUMA node affinity
    ///
    /// The NUMA node this GPU is attached to. Important for optimal
    /// CPU-GPU memory transfers.
    ///
    /// Returns `None` on non-NUMA systems or if not determinable.
    pub numa_node: Option<i32>,
    
    /// Power limit in watts (if available)
    pub power_limit_watts: Option<u32>,
    
    /// Current temperature in Celsius (if available)
    pub temperature_celsius: Option<u32>,
    
    /// Detection method that discovered this GPU
    ///
    /// One of: "nvml", "nvidia-smi", "rocm-smi", "sysfs", "lspci", "sysinfo"
    ///
    /// Useful for debugging and understanding data accuracy.
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
            pci_id: String::new(),
            pci_bus_id: None,
            vendor: GpuVendor::Unknown,
            vendor_name: "Unknown".to_string(),
            driver_version: None,
            compute_capability: None,
            architecture: None,
            numa_node: None,
            power_limit_watts: None,
            temperature_celsius: None,
            detection_method: String::new(),
        }
    }
}
```

---

## Detection Method Details

### Method 1: NVML (nvml-wrapper)

**When:** Feature `nvidia` enabled, NVIDIA driver installed

**Pros:**
- Most accurate data
- Direct API, no parsing
- Memory in bytes (exact)
- Full metadata

**Cons:**
- Requires NVIDIA driver
- NVML library must be present
- NVIDIA GPUs only

**sysfs paths used:**
- None (direct library calls)

**References:**
- [nvml-wrapper crate](https://crates.io/crates/nvml-wrapper)
- [NVML API Reference](https://docs.nvidia.com/deploy/nvml-api/)
- [NVML Header](https://github.com/NVIDIA/nvidia-settings/blob/main/src/nvml.h)

---

### Method 2: nvidia-smi

**When:** NVML unavailable, `nvidia-smi` command available

**Command:**
```bash
nvidia-smi --query-gpu=index,name,uuid,memory.total,memory.free,pci.bus_id,driver_version,compute_cap --format=csv,noheader,nounits
```

**Output format:**
```
0, NVIDIA H100 80GB HBM3, GPU-xxxx, 81920, 81000, 00000000:01:00.0, 535.129.03, 9.0
```

**Parsing notes:**
- Use `nounits` flag to get numeric values
- Memory is in MiB (mebibytes)
- Fields are comma-separated

**References:**
- [nvidia-smi Documentation](https://developer.nvidia.com/nvidia-system-management-interface)

---

### Method 3: ROCm SMI

**When:** AMD GPU detected, `rocm-smi` command available

**Command:**
```bash
rocm-smi --showproductname --showmeminfo vram --showdriver --json
```

**Output format (JSON):**
```json
{
  "card0": {
    "Card series": "AMD Instinct MI250X",
    "VRAM Total Memory (B)": "137438953472",
    "Driver version": "6.3.6"
  }
}
```

**References:**
- [ROCm SMI Documentation](https://rocm.docs.amd.com/projects/rocm_smi_lib/en/latest/)
- [ROCm GitHub](https://github.com/RadeonOpenCompute/rocm_smi_lib)

---

### Method 4: sysfs DRM

**When:** Linux, GPUs present in `/sys/class/drm`

**sysfs paths:**
```
/sys/class/drm/card{N}/device/
├── vendor              # PCI vendor ID (e.g., "0x10de")
├── device              # PCI device ID (e.g., "0x2330")
├── subsystem_vendor    # Subsystem vendor ID
├── subsystem_device    # Subsystem device ID
├── numa_node           # NUMA node affinity
├── mem_info_vram_total # AMD: VRAM total in bytes
├── mem_info_vram_used  # AMD: VRAM used in bytes
└── driver/             # Symlink to driver
    └── module/
        └── version     # Driver version (some drivers)
```

**Vendor detection:**
- NVIDIA: vendor = 0x10de
- AMD: vendor = 0x1002
- Intel: vendor = 0x8086

**Memory detection:**
- AMD: `/sys/class/drm/card*/device/mem_info_vram_total`
- Intel: `/sys/class/drm/card*/gt/addr_range` (varies)
- NVIDIA: Not available via sysfs (use NVML)

**References:**
- [Linux DRM sysfs](https://www.kernel.org/doc/html/latest/gpu/drm-uapi.html)
- [sysfs ABI Documentation](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-drm)

---

### Method 5: lspci

**When:** Other methods unavailable or for initial enumeration

**Command:**
```bash
lspci -nn -d ::0300  # VGA compatible controller
lspci -nn -d ::0302  # 3D controller (NVIDIA Tesla/compute)
```

**Output format:**
```
01:00.0 3D controller [0302]: NVIDIA Corporation GH100 [H100 SXM5 80GB] [10de:2330] (rev a1)
```

**Parsing:**
- PCI bus ID: `01:00.0`
- Class: `3D controller [0302]`
- Vendor:Device: `[10de:2330]`
- Name: Everything between `:` and `[vendor:device]`

**References:**
- [lspci man page](https://man7.org/linux/man-pages/man8/lspci.8.html)
- [PCI Class Codes](https://pci-ids.ucw.cz/read/PD/)

---

### Method 6: sysinfo crate

**When:** Last resort fallback

**Usage:**
```rust
use sysinfo::System;

let sys = System::new_all();
// sysinfo doesn't currently expose GPU info
// but may in future versions
```

**Current limitation:** sysinfo does not expose GPU information as of v0.32.

**References:**
- [sysinfo crate](https://crates.io/crates/sysinfo)

---

## Adapter Implementation

### File: `src/adapters/secondary/system/linux.rs`

```rust
// Pseudocode for new implementation

impl SystemInfoProvider for LinuxSystemInfoProvider {
    async fn get_gpu_info(&self) -> Result<GpuInfo, SystemError> {
        let mut devices = Vec::new();
        
        // Method 1: Try NVML (feature-gated)
        #[cfg(feature = "nvidia")]
        {
            if let Ok(nvml_gpus) = self.detect_gpus_nvml().await {
                devices.extend(nvml_gpus);
            }
        }
        
        // Method 2: Try nvidia-smi (if no NVML results)
        if devices.is_empty() {
            if let Ok(smi_gpus) = self.detect_gpus_nvidia_smi().await {
                devices.extend(smi_gpus);
            }
        }
        
        // Method 3: Try rocm-smi for AMD
        if let Ok(rocm_gpus) = self.detect_gpus_rocm_smi().await {
            devices.extend(rocm_gpus);
        }
        
        // Method 4: Try sysfs DRM
        if let Ok(drm_gpus) = self.detect_gpus_sysfs_drm().await {
            // Merge with existing or add new
            self.merge_gpu_info(&mut devices, drm_gpus);
        }
        
        // Method 5: Try lspci (for devices not yet found)
        if let Ok(pci_gpus) = self.detect_gpus_lspci().await {
            // Merge with existing or add new
            self.merge_gpu_info(&mut devices, pci_gpus);
        }
        
        // Enrich with NUMA info
        self.enrich_gpu_numa_info(&mut devices).await;
        
        // Re-index
        for (i, gpu) in devices.iter_mut().enumerate() {
            gpu.index = i as u32;
        }
        
        Ok(GpuInfo { devices })
    }
}
```

### Helper Methods

```rust
impl LinuxSystemInfoProvider {
    /// Detect GPUs using NVML library
    ///
    /// # Requirements
    ///
    /// - Feature `nvidia` must be enabled
    /// - NVIDIA driver must be installed
    /// - NVML library must be loadable
    #[cfg(feature = "nvidia")]
    async fn detect_gpus_nvml(&self) -> Result<Vec<GpuDevice>, SystemError> {
        // Implementation using nvml-wrapper
        todo!()
    }
    
    /// Detect GPUs using nvidia-smi command
    ///
    /// # Requirements
    ///
    /// - `nvidia-smi` must be in PATH
    /// - NVIDIA driver must be installed
    async fn detect_gpus_nvidia_smi(&self) -> Result<Vec<GpuDevice>, SystemError> {
        // Implementation using command execution
        todo!()
    }
    
    /// Detect AMD GPUs using rocm-smi command
    ///
    /// # Requirements
    ///
    /// - `rocm-smi` must be in PATH
    /// - ROCm must be installed
    async fn detect_gpus_rocm_smi(&self) -> Result<Vec<GpuDevice>, SystemError> {
        // Implementation using command execution
        todo!()
    }
    
    /// Detect GPUs using sysfs DRM interface
    ///
    /// # References
    ///
    /// - [sysfs DRM ABI](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-drm)
    async fn detect_gpus_sysfs_drm(&self) -> Result<Vec<GpuDevice>, SystemError> {
        // Implementation reading /sys/class/drm
        todo!()
    }
    
    /// Detect GPUs using lspci command
    ///
    /// # Requirements
    ///
    /// - `lspci` must be in PATH (pciutils package)
    async fn detect_gpus_lspci(&self) -> Result<Vec<GpuDevice>, SystemError> {
        // Implementation using command execution
        todo!()
    }
    
    /// Merge GPU info from multiple sources
    ///
    /// GPUs are matched by PCI bus ID. Information from higher-priority
    /// sources takes precedence, but missing fields are filled in from
    /// lower-priority sources.
    fn merge_gpu_info(&self, primary: &mut Vec<GpuDevice>, secondary: Vec<GpuDevice>) {
        // Implementation
        todo!()
    }
    
    /// Enrich GPU devices with NUMA node information
    ///
    /// Reads NUMA affinity from `/sys/class/drm/card{N}/device/numa_node`
    async fn enrich_gpu_numa_info(&self, devices: &mut [GpuDevice]) {
        // Implementation
        todo!()
    }
}
```

---

## Parser Implementation

### New File: `src/domain/parsers/gpu.rs`

```rust
//! GPU information parsing functions
//!
//! This module provides pure parsing functions for GPU information from
//! various sources. All functions take string input and return parsed
//! results without performing I/O.
//!
//! # Supported Formats
//!
//! - nvidia-smi CSV output
//! - rocm-smi JSON output
//! - lspci text output
//! - sysfs file contents
//!
//! # Example
//!
//! ```
//! use hardware_report::domain::parsers::gpu::parse_nvidia_smi_output;
//!
//! let output = "0, NVIDIA H100, GPU-xxx, 81920, 81000, 00:01:00.0, 535.129.03, 9.0";
//! let gpus = parse_nvidia_smi_output(output).unwrap();
//! assert_eq!(gpus[0].memory_total_mb, 81920);
//! ```

use crate::domain::{GpuDevice, GpuVendor};

/// Parse nvidia-smi CSV output into GPU devices
///
/// # Arguments
///
/// * `output` - Output from `nvidia-smi --query-gpu=... --format=csv,noheader,nounits`
///
/// # Expected Format
///
/// ```text
/// index, name, uuid, memory.total, memory.free, pci.bus_id, driver_version, compute_cap
/// 0, NVIDIA H100 80GB HBM3, GPU-xxxx, 81920, 81000, 00000000:01:00.0, 535.129.03, 9.0
/// ```
///
/// # Errors
///
/// Returns an error if the output format is invalid or cannot be parsed.
///
/// # References
///
/// - [nvidia-smi Query Options](https://developer.nvidia.com/nvidia-system-management-interface)
pub fn parse_nvidia_smi_output(output: &str) -> Result<Vec<GpuDevice>, String> {
    todo!()
}

/// Parse rocm-smi JSON output into GPU devices
///
/// # Arguments
///
/// * `output` - JSON output from `rocm-smi --json`
///
/// # Expected Format
///
/// ```json
/// {
///   "card0": {
///     "Card series": "AMD Instinct MI250X",
///     "VRAM Total Memory (B)": "137438953472",
///     "Driver version": "6.3.6"
///   }
/// }
/// ```
///
/// # Errors
///
/// Returns an error if the JSON is invalid or required fields are missing.
///
/// # References
///
/// - [ROCm SMI Documentation](https://rocm.docs.amd.com/projects/rocm_smi_lib/en/latest/)
pub fn parse_rocm_smi_output(output: &str) -> Result<Vec<GpuDevice>, String> {
    todo!()
}

/// Parse lspci output for GPU devices
///
/// # Arguments
///
/// * `output` - Output from `lspci -nn`
///
/// # Expected Format
///
/// ```text
/// 01:00.0 3D controller [0302]: NVIDIA Corporation GH100 [H100 SXM5 80GB] [10de:2330] (rev a1)
/// ```
///
/// # Note
///
/// This method cannot determine GPU memory. The `memory_total_mb` field
/// will be set to 0 for GPUs detected via lspci only.
///
/// # References
///
/// - [lspci man page](https://man7.org/linux/man-pages/man8/lspci.8.html)
pub fn parse_lspci_gpu_output(output: &str) -> Result<Vec<GpuDevice>, String> {
    todo!()
}

/// Parse PCI vendor ID to determine GPU vendor
///
/// # Arguments
///
/// * `vendor_id` - PCI vendor ID in hexadecimal (e.g., "10de", "0x10de")
///
/// # Returns
///
/// The corresponding `GpuVendor` enum value.
///
/// # Example
///
/// ```
/// use hardware_report::domain::parsers::gpu::parse_pci_vendor;
/// use hardware_report::GpuVendor;
///
/// assert_eq!(parse_pci_vendor("10de"), GpuVendor::Nvidia);
/// assert_eq!(parse_pci_vendor("0x1002"), GpuVendor::Amd);
/// ```
///
/// # References
///
/// - [PCI Vendor IDs](https://pci-ids.ucw.cz/)
pub fn parse_pci_vendor(vendor_id: &str) -> GpuVendor {
    todo!()
}

/// Parse sysfs DRM memory info for AMD GPUs
///
/// # Arguments
///
/// * `content` - Content of `/sys/class/drm/card*/device/mem_info_vram_total`
///
/// # Returns
///
/// Memory size in megabytes.
///
/// # References
///
/// - [AMDGPU sysfs](https://www.kernel.org/doc/html/latest/gpu/amdgpu/driver-misc.html)
pub fn parse_sysfs_vram_total(content: &str) -> Result<u64, String> {
    todo!()
}
```

---

## Error Handling

### Error Types

```rust
/// GPU detection-specific errors
#[derive(Debug, thiserror::Error)]
pub enum GpuDetectionError {
    /// NVML library initialization failed
    #[error("NVML initialization failed: {0}")]
    NvmlInitFailed(String),
    
    /// No GPUs found by any method
    #[error("No GPUs detected")]
    NoGpusFound,
    
    /// Command execution failed
    #[error("GPU detection command failed: {command}: {reason}")]
    CommandFailed {
        command: String,
        reason: String,
    },
    
    /// Output parsing failed
    #[error("Failed to parse GPU info from {source}: {reason}")]
    ParseFailed {
        source: String,
        reason: String,
    },
    
    /// sysfs read failed
    #[error("Failed to read sysfs path {path}: {reason}")]
    SysfsFailed {
        path: String,
        reason: String,
    },
}
```

### Error Handling Strategy

1. **Never fail completely** - Return partial results if some methods work
2. **Log warnings** - Log failures at detection methods for debugging
3. **Include detection_method** - So consumers know data accuracy
4. **Return empty GpuInfo** - If no GPUs found (not an error condition)

---

## Testing Requirements

### Unit Tests

| Test | Description |
|------|-------------|
| `test_parse_nvidia_smi_output` | Parse valid nvidia-smi CSV |
| `test_parse_nvidia_smi_empty` | Handle empty nvidia-smi output |
| `test_parse_rocm_smi_output` | Parse valid rocm-smi JSON |
| `test_parse_lspci_output` | Parse lspci with multiple GPUs |
| `test_parse_pci_vendor` | Vendor ID to enum conversion |
| `test_gpu_merge` | Merging info from multiple sources |

### Integration Tests

| Test | Platform | Description |
|------|----------|-------------|
| `test_gpu_detection_nvidia` | x86_64 + NVIDIA | Full detection with real GPU |
| `test_gpu_detection_amd` | x86_64 + AMD | Full detection with AMD GPU |
| `test_gpu_detection_arm` | aarch64 | Detection on ARM (DGX Spark) |
| `test_gpu_detection_no_gpu` | Any | Graceful handling of no GPU |

### Test Hardware Matrix

| Platform | GPU | Test Target |
|----------|-----|-------------|
| x86_64 Linux | NVIDIA H100 | CI + Manual |
| x86_64 Linux | AMD MI250X | Manual |
| aarch64 Linux | NVIDIA (DGX Spark) | Manual |
| aarch64 Linux | No GPU | CI |

---

## References

### Official Documentation

| Resource | URL |
|----------|-----|
| NVIDIA NVML API | https://docs.nvidia.com/deploy/nvml-api/ |
| NVIDIA SMI | https://developer.nvidia.com/nvidia-system-management-interface |
| AMD ROCm SMI | https://rocm.docs.amd.com/projects/rocm_smi_lib/en/latest/ |
| Linux DRM | https://www.kernel.org/doc/html/latest/gpu/drm-uapi.html |
| PCI ID Database | https://pci-ids.ucw.cz/ |
| CUDA Compute Capability | https://developer.nvidia.com/cuda-gpus |

### Crate Documentation

| Crate | URL |
|-------|-----|
| nvml-wrapper | https://docs.rs/nvml-wrapper |
| sysinfo | https://docs.rs/sysinfo |

### Kernel Documentation

| Path | Description |
|------|-------------|
| `/sys/class/drm/` | DRM subsystem sysfs |
| `/sys/class/drm/card*/device/vendor` | PCI vendor ID |
| `/sys/class/drm/card*/device/numa_node` | NUMA affinity |

---

## Changelog

| Date | Changes |
|------|---------|
| 2024-12-29 | Initial specification |
