# Hardware Report Enhancement Plan

> **Version:** 0.2.0  
> **Target:** Linux (primary), macOS (secondary)  
> **Architecture Focus:** x86_64, aarch64/ARM64

## Table of Contents

1. [Overview](#overview)
2. [Architecture Principles](#architecture-principles)
3. [Enhancement Summary](#enhancement-summary)
4. [Phase 1: Critical Issues](#phase-1-critical-issues)
5. [Phase 2: Data Gaps](#phase-2-data-gaps)
6. [Phase 3: Runtime Metrics](#phase-3-runtime-metrics)
7. [New Dependencies](#new-dependencies)
8. [Implementation Order](#implementation-order)
9. [Related Documents](#related-documents)

---

## Overview

This document outlines the implementation plan for enhancing the `hardware_report` crate to better serve CMDB (Configuration Management Database) inventory use cases, specifically addressing issues encountered in the `metal-agent` project.

### Goals

1. **Eliminate fallback collection methods** - Provide complete, accurate data natively
2. **Multi-architecture support** - Full functionality on x86_64 and aarch64 (ARM64)
3. **Numeric data formats** - Return parseable numeric values, not formatted strings
4. **Multi-method detection** - Use multiple detection strategies with graceful fallbacks
5. **Comprehensive documentation** - Rustdoc for all public APIs with links to official references

### Non-Goals

- Windows support (out of scope for this phase)
- Real-time monitoring (basic runtime metrics only)
- Container/VM detection improvements

---

## Architecture Principles

This implementation strictly follows the **Hexagonal Architecture (Ports and Adapters)** pattern already established in the codebase.

### Layer Responsibilities

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DOMAIN LAYER                                │
│  src/domain/                                                        │
│  ├── entities.rs      # Data structures (platform-agnostic)        │
│  ├── errors.rs        # Domain errors                               │
│  ├── parsers/         # Pure parsing functions (no I/O)            │
│  │   ├── cpu.rs                                                     │
│  │   ├── memory.rs                                                  │
│  │   ├── storage.rs                                                 │
│  │   ├── network.rs                                                 │
│  │   └── gpu.rs       # NEW                                         │
│  └── services/        # Domain services (orchestration)            │
└─────────────────────────────────────────────────────────────────────┘
                                 │
                                 │ implements
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          PORTS LAYER                                │
│  src/ports/                                                         │
│  ├── primary/         # Offered interfaces (what we provide)       │
│  │   └── reporting.rs # HardwareReportingService trait             │
│  └── secondary/       # Required interfaces (what we need)         │
│      ├── system.rs    # SystemInfoProvider trait                   │
│      ├── command.rs   # CommandExecutor trait                      │
│      └── publisher.rs # DataPublisher trait                        │
└─────────────────────────────────────────────────────────────────────┘
                                 │
                                 │ implemented by
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        ADAPTERS LAYER                               │
│  src/adapters/secondary/                                            │
│  ├── system/                                                        │
│  │   ├── linux.rs     # LinuxSystemInfoProvider                    │
│  │   └── macos.rs     # MacOSSystemInfoProvider                    │
│  ├── command/                                                       │
│  │   └── unix.rs      # UnixCommandExecutor                        │
│  └── publisher/                                                     │
│      ├── http.rs      # HttpDataPublisher                          │
│      └── file.rs      # FileDataPublisher                          │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Principles

| Principle | Description |
|-----------|-------------|
| **Domain Independence** | Domain layer has NO knowledge of adapters or I/O |
| **Pure Parsers** | Parsing functions take strings, return Results - no side effects |
| **Trait Abstraction** | All platform-specific code behind trait interfaces |
| **Multi-Method Detection** | Each adapter tries multiple methods, returns best result |
| **Graceful Degradation** | Partial data is better than no data - always return something |

---

## Enhancement Summary

### Critical Issues (Phase 1)

| Issue | Impact | Solution | Doc |
|-------|--------|----------|-----|
| GPU memory returns unparseable string | CMDB shows 0MB VRAM | Numeric fields + multi-method detection | [GPU_DETECTION.md](./GPU_DETECTION.md) |
| Storage empty on ARM/aarch64 | No storage inventory | sysfs + sysinfo fallback chain | [STORAGE_DETECTION.md](./STORAGE_DETECTION.md) |
| CPU frequency not exposed | Hardcoded values | sysfs + raw-cpuid | [CPU_ENHANCEMENTS.md](./CPU_ENHANCEMENTS.md) |

### Data Gaps (Phase 2)

| Missing Field | Category | Priority | Doc |
|---------------|----------|----------|-----|
| CPU cache sizes (L1/L2/L3) | CPU | Medium | [CPU_ENHANCEMENTS.md](./CPU_ENHANCEMENTS.md) |
| DIMM part_number | Memory | Medium | [MEMORY_ENHANCEMENTS.md](./MEMORY_ENHANCEMENTS.md) |
| Storage serial_number | Storage | High | [STORAGE_DETECTION.md](./STORAGE_DETECTION.md) |
| Storage firmware_version | Storage | Medium | [STORAGE_DETECTION.md](./STORAGE_DETECTION.md) |
| GPU driver_version | GPU | High | [GPU_DETECTION.md](./GPU_DETECTION.md) |
| Network driver_version | Network | Low | [NETWORK_ENHANCEMENTS.md](./NETWORK_ENHANCEMENTS.md) |

---

## Phase 1: Critical Issues

### 1.1 GPU Detection Overhaul

**Problem:** GPU memory returned as `"80 GB"` string, consumers can't parse numeric values.

**Solution:** Multi-method detection with numeric output fields.

See: [GPU_DETECTION.md](./GPU_DETECTION.md)

#### Entity Changes

```rust
// src/domain/entities.rs

/// GPU device information
///
/// Represents a discrete or integrated GPU detected in the system.
/// Memory values are provided in megabytes as unsigned integers for
/// reliable parsing by CMDB consumers.
///
/// # Detection Methods
///
/// GPUs are detected using multiple methods in priority order:
/// 1. NVML (NVIDIA Management Library) - most accurate for NVIDIA GPUs
/// 2. nvidia-smi command - fallback for NVIDIA when NVML unavailable
/// 3. ROCm SMI - AMD GPU detection
/// 4. sysfs /sys/class/drm - Linux DRM subsystem
/// 5. lspci - PCI device enumeration
/// 6. sysinfo crate - cross-platform fallback
///
/// # References
///
/// - [NVIDIA NVML Documentation](https://developer.nvidia.com/nvidia-management-library-nvml)
/// - [Linux DRM Subsystem](https://www.kernel.org/doc/html/latest/gpu/drm-uapi.html)
/// - [PCI ID Database](https://pci-ids.ucw.cz/)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuDevice {
    /// GPU index (0-based)
    pub index: u32,
    
    /// GPU product name (e.g., "NVIDIA H100 80GB HBM3")
    pub name: String,
    
    /// GPU UUID (unique identifier)
    pub uuid: String,
    
    /// Total GPU memory in megabytes
    ///
    /// This replaces the previous `memory: String` field which returned
    /// formatted strings like "80 GB" that were difficult to parse.
    pub memory_total_mb: u64,
    
    /// Available GPU memory in megabytes (runtime value, may be None if not queryable)
    pub memory_free_mb: Option<u64>,
    
    /// GPU memory as formatted string (deprecated, for backward compatibility)
    #[deprecated(since = "0.2.0", note = "Use memory_total_mb instead")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    
    /// PCI ID in format "vendor:device" (e.g., "10de:2330")
    pub pci_id: String,
    
    /// PCI bus address (e.g., "0000:01:00.0")
    pub pci_bus_id: Option<String>,
    
    /// Vendor name (e.g., "NVIDIA", "AMD", "Intel")
    pub vendor: String,
    
    /// Driver version (e.g., "535.129.03")
    pub driver_version: Option<String>,
    
    /// CUDA compute capability for NVIDIA GPUs (e.g., "9.0")
    pub compute_capability: Option<String>,
    
    /// GPU architecture (e.g., "Hopper", "Ada Lovelace", "RDNA3")
    pub architecture: Option<String>,
    
    /// NUMA node affinity (-1 if not applicable)
    pub numa_node: Option<i32>,
    
    /// Detection method used to discover this GPU
    pub detection_method: String,
}
```

### 1.2 Storage Detection on ARM

**Problem:** `lsblk` returns empty on some ARM platforms.

**Solution:** Multi-method detection with sysfs as primary on Linux.

See: [STORAGE_DETECTION.md](./STORAGE_DETECTION.md)

#### Entity Changes

```rust
// src/domain/entities.rs

/// Storage device type classification
///
/// # References
///
/// - [Linux Block Device Documentation](https://www.kernel.org/doc/html/latest/block/index.html)
/// - [NVMe Specification](https://nvmexpress.org/specifications/)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum StorageType {
    /// NVMe solid-state drive
    Nvme,
    /// SATA/SAS solid-state drive
    Ssd,
    /// Hard disk drive (rotational)
    Hdd,
    /// Embedded MMC storage
    Emmc,
    /// Unknown or unclassified storage type
    Unknown,
}

/// Storage device information
///
/// # Detection Methods
///
/// Storage devices are detected using multiple methods in priority order:
/// 1. sysfs /sys/block - direct kernel interface (Linux)
/// 2. lsblk command - block device listing
/// 3. sysinfo crate - cross-platform fallback
/// 4. diskutil (macOS)
///
/// # References
///
/// - [Linux sysfs Block Devices](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-block)
/// - [SMART Attributes](https://en.wikipedia.org/wiki/S.M.A.R.T.)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageDevice {
    /// Device name (e.g., "nvme0n1", "sda")
    pub name: String,
    
    /// Device type classification
    pub device_type: StorageType,
    
    /// Legacy type field (deprecated)
    #[deprecated(since = "0.2.0", note = "Use device_type instead")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    
    /// Device size in bytes
    pub size_bytes: u64,
    
    /// Device size in gigabytes (convenience field)
    pub size_gb: f64,
    
    /// Legacy size field as string (deprecated)
    #[deprecated(since = "0.2.0", note = "Use size_bytes or size_gb instead")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    
    /// Device model name
    pub model: String,
    
    /// Device serial number (may require elevated privileges)
    pub serial_number: Option<String>,
    
    /// Device firmware version
    pub firmware_version: Option<String>,
    
    /// Interface type (e.g., "NVMe", "SATA", "SAS", "eMMC")
    pub interface: String,
    
    /// Whether the device is rotational (true = HDD, false = SSD/NVMe)
    pub is_rotational: bool,
    
    /// WWN (World Wide Name) if available
    pub wwn: Option<String>,
    
    /// Detection method used
    pub detection_method: String,
}
```

### 1.3 CPU Frequency and Cache

**Problem:** CPU frequency hardcoded, cache sizes not exposed.

**Solution:** sysfs reads + raw-cpuid for x86.

See: [CPU_ENHANCEMENTS.md](./CPU_ENHANCEMENTS.md)

#### Entity Changes

```rust
// src/domain/entities.rs

/// CPU information with extended details
///
/// # Detection Methods
///
/// CPU information is gathered from multiple sources:
/// 1. sysfs /sys/devices/system/cpu - frequency and cache (Linux)
/// 2. /proc/cpuinfo - model and features (Linux)
/// 3. raw-cpuid crate - x86 CPUID instruction
/// 4. lscpu command - topology information
/// 5. dmidecode - SMBIOS data (requires privileges)
/// 6. sysinfo crate - cross-platform fallback
///
/// # References
///
/// - [Linux CPU sysfs Interface](https://www.kernel.org/doc/Documentation/cpu-freq/user-guide.rst)
/// - [Intel CPUID Reference](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
/// - [ARM CPU Identification](https://developer.arm.com/documentation/ddi0487/latest)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuInfo {
    /// CPU model name (e.g., "AMD EPYC 7763 64-Core Processor")
    pub model: String,
    
    /// CPU vendor (e.g., "GenuineIntel", "AuthenticAMD", "ARM")
    pub vendor: String,
    
    /// Number of physical cores per socket
    pub cores: u32,
    
    /// Number of threads per core (hyperthreading)
    pub threads: u32,
    
    /// Number of CPU sockets
    pub sockets: u32,
    
    /// CPU frequency in MHz (current or max)
    pub frequency_mhz: u32,
    
    /// Legacy speed field as string (deprecated)
    #[deprecated(since = "0.2.0", note = "Use frequency_mhz instead")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<String>,
    
    /// CPU architecture (e.g., "x86_64", "aarch64")
    pub architecture: String,
    
    /// L1 data cache size in kilobytes (per core)
    pub cache_l1d_kb: Option<u32>,
    
    /// L1 instruction cache size in kilobytes (per core)
    pub cache_l1i_kb: Option<u32>,
    
    /// L2 cache size in kilobytes (per core or shared)
    pub cache_l2_kb: Option<u32>,
    
    /// L3 cache size in kilobytes (typically shared)
    pub cache_l3_kb: Option<u32>,
    
    /// CPU flags/features (e.g., "avx2", "sve")
    pub flags: Vec<String>,
    
    /// Microcode version
    pub microcode_version: Option<String>,
}
```

---

## Phase 2: Data Gaps

### 2.1 Memory DIMM Part Number

See: [MEMORY_ENHANCEMENTS.md](./MEMORY_ENHANCEMENTS.md)

```rust
/// Individual memory module (DIMM)
///
/// # References
///
/// - [JEDEC Memory Standards](https://www.jedec.org/)
/// - [SMBIOS Type 17 Memory Device](https://www.dmtf.org/standards/smbios)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryModule {
    pub size: String,
    pub size_bytes: u64,           // NEW
    pub type_: String,
    pub speed: String,
    pub speed_mhz: Option<u32>,    // NEW
    pub location: String,
    pub manufacturer: String,
    pub serial: String,
    pub part_number: Option<String>, // NEW
    pub rank: Option<u32>,           // NEW
    pub configured_voltage: Option<f32>, // NEW (in volts)
}
```

### 2.2 Network Interface Enhancements

See: [NETWORK_ENHANCEMENTS.md](./NETWORK_ENHANCEMENTS.md)

```rust
/// Network interface information
///
/// # References
///
/// - [Linux Netlink Documentation](https://man7.org/linux/man-pages/man7/netlink.7.html)
/// - [ethtool Source](https://mirrors.edge.kernel.org/pub/software/network/ethtool/)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub mac: String,
    pub ip: String,
    pub prefix: String,
    pub speed: Option<String>,
    pub speed_mbps: Option<u32>,      // NEW
    pub type_: String,
    pub vendor: String,
    pub model: String,
    pub pci_id: String,
    pub numa_node: Option<i32>,
    pub driver: Option<String>,        // NEW
    pub driver_version: Option<String>, // NEW
    pub firmware_version: Option<String>, // NEW
    pub mtu: u32,                      // NEW
    pub is_up: bool,                   // NEW
    pub is_virtual: bool,              // NEW
}
```

---

## Phase 3: Runtime Metrics (Optional)

These are lower priority and may be deferred:

| Metric | Category | Notes |
|--------|----------|-------|
| GPU temperature | GPU | Requires NVML or sensors |
| GPU utilization | GPU | Requires NVML |
| GPU power draw | GPU | Requires NVML |
| Storage SMART data | Storage | Requires smartctl or sysfs |
| Network statistics | Network | /sys/class/net/*/statistics |

---

## New Dependencies

### Cargo.toml Changes

```toml
[dependencies]
# Existing dependencies...
sysinfo = "0.32.0"

# NEW: NVIDIA GPU detection via NVML
# Optional - requires NVIDIA driver at runtime
nvml-wrapper = { version = "0.9", optional = true }

# NEW: x86 CPU detection via CPUID
# Only compiled on x86/x86_64 targets
[target.'cfg(any(target_arch = "x86", target_arch = "x86_64"))'.dependencies]
raw-cpuid = { version = "11", optional = true }

[features]
default = []
nvidia = ["nvml-wrapper"]
x86-cpu = ["raw-cpuid"]
full = ["nvidia", "x86-cpu"]
```

### Dependency Rationale

| Crate | Purpose | Why Not Shell Out? |
|-------|---------|-------------------|
| `nvml-wrapper` | NVIDIA GPU detection | Direct API access, no parsing, handles errors properly |
| `raw-cpuid` | x86 CPU cache/features | Direct CPU instruction, no external dependencies |
| `sysinfo` | Cross-platform fallback | Already in use, pure Rust |

### References

- [nvml-wrapper crate](https://crates.io/crates/nvml-wrapper) - [NVML API Docs](https://docs.nvidia.com/deploy/nvml-api/)
- [raw-cpuid crate](https://crates.io/crates/raw-cpuid) - [Intel CPUID](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [sysinfo crate](https://crates.io/crates/sysinfo)

---

## Implementation Order

| Step | Task | Priority | Est. Effort | Files Changed |
|------|------|----------|-------------|---------------|
| 1 | Create `StorageType` enum and update `StorageDevice` | Critical | Low | `entities.rs` |
| 2 | Implement sysfs storage detection for Linux | Critical | Medium | `linux.rs`, `storage.rs` |
| 3 | Update `CpuInfo` with frequency/cache fields | Critical | Low | `entities.rs` |
| 4 | Implement sysfs CPU freq/cache detection | Critical | Medium | `linux.rs`, `cpu.rs` |
| 5 | Update `GpuDevice` with numeric memory fields | Critical | Low | `entities.rs` |
| 6 | Implement multi-method GPU detection | Critical | High | `linux.rs`, `gpu.rs` |
| 7 | Add NVML integration (feature-gated) | Critical | Medium | `linux.rs`, `Cargo.toml` |
| 8 | Update `MemoryModule` with part_number | Medium | Low | `entities.rs`, `memory.rs` |
| 9 | Update `NetworkInterface` with driver info | Medium | Medium | `entities.rs`, `linux.rs`, `network.rs` |
| 10 | Add rustdoc to all public items | High | Medium | All files |
| 11 | Add/update tests for ARM and x86 | High | High | `tests/` |
| 12 | Update examples | Medium | Low | `examples/` |

---

## Related Documents

- [GPU_DETECTION.md](./GPU_DETECTION.md) - Multi-method GPU detection strategy
- [STORAGE_DETECTION.md](./STORAGE_DETECTION.md) - Storage detection with ARM focus
- [CPU_ENHANCEMENTS.md](./CPU_ENHANCEMENTS.md) - CPU frequency and cache detection
- [MEMORY_ENHANCEMENTS.md](./MEMORY_ENHANCEMENTS.md) - Memory module enhancements
- [NETWORK_ENHANCEMENTS.md](./NETWORK_ENHANCEMENTS.md) - Network interface enhancements
- [RUSTDOC_STANDARDS.md](./RUSTDOC_STANDARDS.md) - Documentation standards
- [TESTING_STRATEGY.md](./TESTING_STRATEGY.md) - Testing approach for ARM and x86

---

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2024-12-29 | 0.2.0-plan | Initial enhancement plan |
