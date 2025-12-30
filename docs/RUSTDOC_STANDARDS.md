# Rustdoc Standards and Guidelines

> **Purpose:** Define documentation standards for the `hardware_report` crate  
> **Audience:** Contributors and maintainers

## Table of Contents

1. [Overview](#overview)
2. [Documentation Requirements](#documentation-requirements)
3. [Rustdoc Format](#rustdoc-format)
4. [External References](#external-references)
5. [Examples](#examples)
6. [Module Documentation](#module-documentation)
7. [Linting and CI](#linting-and-ci)

---

## Overview

All public APIs in `hardware_report` must be documented with rustdoc comments. This ensures:

1. **Discoverability** - Engineers can find what they need
2. **Correctness** - Examples are tested via `cargo test --doc`
3. **Traceability** - Links to official specifications and kernel docs
4. **Maintainability** - Clear contracts for each component

### Guiding Principles

- **Every public item gets a doc comment** - structs, enums, functions, traits, modules
- **Link to official references** - kernel docs, hardware specs, crate docs
- **Include examples** - runnable code in doc comments
- **Explain "why" not just "what"** - context for design decisions

---

## Documentation Requirements

### Required for ALL Public Items

| Item Type | Required Sections |
|-----------|-------------------|
| Module | Purpose, contents overview |
| Struct | Description, fields, example usage |
| Enum | Description, variants, when to use each |
| Function | Purpose, arguments, returns, errors, example |
| Trait | Purpose, implementors, example |
| Constant | Purpose, value explanation |

### Required External Links

When documenting hardware-related items, include links to:

| Topic | Link To |
|-------|---------|
| sysfs paths | Kernel documentation |
| PCI IDs | pci-ids.ucw.cz |
| SMBIOS fields | DMTF SMBIOS spec |
| NVMe | nvmexpress.org |
| GPU (NVIDIA) | NVIDIA developer docs |
| GPU (AMD) | ROCm documentation |
| Memory specs | JEDEC |
| CPU (x86) | Intel/AMD SDM |
| CPU (ARM) | ARM developer documentation |

---

## Rustdoc Format

### Basic Structure

```rust
/// Short one-line description.
///
/// Longer description that explains the purpose, context, and usage
/// of this item. Can span multiple paragraphs.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// Description of return value.
///
/// # Errors
///
/// Description of error conditions.
///
/// # Panics
///
/// Conditions under which this function panics (if any).
///
/// # Safety
///
/// For unsafe functions, explain the invariants.
///
/// # Example
///
/// ```rust
/// use hardware_report::SomeItem;
///
/// let result = some_function(arg1, arg2);
/// assert!(result.is_ok());
/// ```
///
/// # References
///
/// - [Link Text](https://url)
/// - [Another Reference](https://url)
pub fn some_function(param1: Type1, param2: Type2) -> Result<Output, Error> {
    // ...
}
```

### Struct Documentation

```rust
/// GPU device information.
///
/// Represents a discrete or integrated GPU detected in the system.
/// Memory values are provided in megabytes as unsigned integers for
/// reliable parsing by CMDB consumers.
///
/// # Detection Methods
///
/// GPUs are detected using multiple methods in priority order:
/// 1. **NVML** - NVIDIA Management Library (most accurate)
/// 2. **nvidia-smi** - NVIDIA CLI tool
/// 3. **rocm-smi** - AMD GPU tool
/// 4. **sysfs** - Linux `/sys/class/drm`
/// 5. **lspci** - PCI enumeration
///
/// # Memory Format
///
/// Memory is reported in **megabytes** as `u64`. The previous string
/// format (e.g., "80 GB") is deprecated.
///
/// # Example
///
/// ```rust
/// use hardware_report::GpuDevice;
///
/// fn process_gpu(gpu: &GpuDevice) {
///     // Calculate memory in GB
///     let memory_gb = gpu.memory_total_mb as f64 / 1024.0;
///     println!("{}: {} GB", gpu.name, memory_gb);
///     
///     // Check vendor
///     if gpu.vendor == GpuVendor::Nvidia {
///         println!("CUDA Compute: {:?}", gpu.compute_capability);
///     }
/// }
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
    /// GPU index (0-based, unique per system).
    pub index: u32,
    
    /// GPU product name.
    ///
    /// Examples:
    /// - "NVIDIA H100 80GB HBM3"
    /// - "AMD Instinct MI250X"
    pub name: String,
    
    // ... more fields with individual documentation
}
```

### Enum Documentation

```rust
/// Storage device type classification.
///
/// Classifies storage devices by their underlying technology.
/// Used for inventory categorization and performance expectations.
///
/// # Detection
///
/// Type is determined by:
/// 1. Device name prefix (`nvme*`, `sd*`, `mmcblk*`)
/// 2. sysfs rotational flag
/// 3. Interface type
///
/// # Example
///
/// ```rust
/// use hardware_report::StorageType;
///
/// let device_type = StorageType::from_device("nvme0n1", false);
/// assert_eq!(device_type, StorageType::Nvme);
///
/// match device_type {
///     StorageType::Nvme | StorageType::Ssd => println!("Fast storage"),
///     StorageType::Hdd => println!("Rotational storage"),
///     _ => println!("Other"),
/// }
/// ```
///
/// # References
///
/// - [Linux Block Devices](https://www.kernel.org/doc/html/latest/block/index.html)
/// - [NVMe Specification](https://nvmexpress.org/specifications/)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StorageType {
    /// NVMe solid-state drive.
    ///
    /// Detected by `nvme*` device name prefix.
    /// Typically provides highest performance (PCIe interface).
    Nvme,
    
    /// SATA/SAS solid-state drive.
    ///
    /// Detected by `rotational=0` on `sd*` devices.
    Ssd,
    
    /// Hard disk drive (rotational media).
    ///
    /// Detected by `rotational=1` on `sd*` devices.
    Hdd,
    
    /// Embedded MMC storage.
    ///
    /// Common on ARM platforms. Detected by `mmcblk*` prefix.
    Emmc,
    
    /// Unknown or unclassified storage type.
    Unknown,
}
```

### Function Documentation

```rust
/// Parse sysfs frequency file to MHz.
///
/// Converts kernel cpufreq values (in kHz) to MHz for consistent
/// representation across the crate.
///
/// # Arguments
///
/// * `content` - Content of a cpufreq file (e.g., `scaling_max_freq`)
///
/// # Returns
///
/// Frequency in MHz as `u32`.
///
/// # Errors
///
/// Returns an error if the content cannot be parsed as an integer.
///
/// # Example
///
/// ```rust
/// use hardware_report::domain::parsers::cpu::parse_sysfs_freq_khz;
///
/// // 3.5 GHz in kHz
/// let freq_mhz = parse_sysfs_freq_khz("3500000").unwrap();
/// assert_eq!(freq_mhz, 3500);
///
/// // Invalid input
/// assert!(parse_sysfs_freq_khz("invalid").is_err());
/// ```
///
/// # References
///
/// - [cpufreq sysfs](https://www.kernel.org/doc/Documentation/cpu-freq/user-guide.rst)
pub fn parse_sysfs_freq_khz(content: &str) -> Result<u32, String> {
    let khz: u32 = content
        .trim()
        .parse()
        .map_err(|e| format!("Invalid frequency: {}", e))?;
    Ok(khz / 1000)
}
```

---

## External References

### Reference Link Format

Use markdown links in the `# References` section:

```rust
/// # References
///
/// - [Link Text](https://full.url.here)
```

### Standard Reference URLs

#### Linux Kernel

| Topic | URL Pattern |
|-------|-------------|
| sysfs ABI | `https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-*` |
| Block devices | `https://www.kernel.org/doc/html/latest/block/index.html` |
| Networking | `https://www.kernel.org/doc/html/latest/networking/index.html` |
| DRM/GPU | `https://www.kernel.org/doc/html/latest/gpu/drm-uapi.html` |
| CPU | `https://www.kernel.org/doc/Documentation/cpu-freq/user-guide.rst` |

#### Hardware Specifications

| Topic | URL |
|-------|-----|
| NVMe | `https://nvmexpress.org/specifications/` |
| SMBIOS | `https://www.dmtf.org/standards/smbios` |
| PCI IDs | `https://pci-ids.ucw.cz/` |
| JEDEC (Memory) | `https://www.jedec.org/` |

#### Vendor Documentation

| Vendor | Topic | URL |
|--------|-------|-----|
| NVIDIA | NVML | `https://docs.nvidia.com/deploy/nvml-api/` |
| NVIDIA | CUDA CC | `https://developer.nvidia.com/cuda-gpus` |
| AMD | ROCm SMI | `https://rocm.docs.amd.com/projects/rocm_smi_lib/en/latest/` |
| Intel | CPUID | `https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html` |
| ARM | CPU ID | `https://developer.arm.com/documentation/ddi0487/latest` |

#### Crate Documentation

| Crate | URL |
|-------|-----|
| nvml-wrapper | `https://docs.rs/nvml-wrapper` |
| raw-cpuid | `https://docs.rs/raw-cpuid` |
| sysinfo | `https://docs.rs/sysinfo` |
| serde | `https://docs.rs/serde` |

### Intra-doc Links

Use Rust's intra-doc links to reference other items in the crate:

```rust
/// See [`GpuDevice`] for GPU information.
/// See [`StorageType::Nvme`] for NVMe detection.
/// See [`parse_sysfs_size`](crate::domain::parsers::storage::parse_sysfs_size).
```

---

## Examples

### Testable Examples

All examples in doc comments should be testable:

```rust
/// # Example
///
/// ```rust
/// use hardware_report::StorageType;
///
/// let st = StorageType::from_device("nvme0n1", false);
/// assert_eq!(st, StorageType::Nvme);
/// ```
```

Run with:
```bash
cargo test --doc
```

### Non-runnable Examples

For examples that can't be run (require hardware, external commands):

```rust
/// # Example
///
/// ```rust,no_run
/// use hardware_report::create_service;
///
/// #[tokio::main]
/// async fn main() {
///     let service = create_service().unwrap();
///     let report = service.generate_report(Default::default()).await.unwrap();
///     println!("{:?}", report);
/// }
/// ```
```

### Examples That Should Not Compile

For showing incorrect usage:

```rust
/// # Example of what NOT to do
///
/// ```rust,compile_fail
/// // This won't compile because memory is u64, not String
/// let memory: String = gpu.memory_total_mb;
/// ```
```

---

## Module Documentation

### Module-Level Documentation

Every module should have a `//!` comment at the top:

```rust
//! GPU information parsing functions.
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
//! # Architecture
//!
//! These functions are part of the **domain layer** in the hexagonal
//! architecture. They have no dependencies on adapters or I/O.
//!
//! # Example
//!
//! ```rust
//! use hardware_report::domain::parsers::gpu::parse_nvidia_smi_output;
//!
//! let output = "0, NVIDIA H100, GPU-xxx, 81920, 81000";
//! let gpus = parse_nvidia_smi_output(output).unwrap();
//! assert_eq!(gpus[0].memory_total_mb, 81920);
//! ```
//!
//! # References
//!
//! - [nvidia-smi](https://developer.nvidia.com/nvidia-system-management-interface)
//! - [rocm-smi](https://rocm.docs.amd.com/projects/rocm_smi_lib/en/latest/)

use crate::domain::{GpuDevice, GpuVendor};

// ... module contents
```

### Re-exports Documentation

Document re-exports in `lib.rs`:

```rust
//! # Hardware Report
//!
//! A library for collecting hardware information on Linux systems.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use hardware_report::{create_service, ReportConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let service = create_service()?;
//!     let report = service.generate_report(ReportConfig::default()).await?;
//!     println!("Hostname: {}", report.hostname);
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! This crate follows the Hexagonal Architecture (Ports and Adapters):
//!
//! - **Domain**: Core entities and pure parsing functions
//! - **Ports**: Trait definitions for required/provided interfaces
//! - **Adapters**: Platform-specific implementations
//!
//! ## Feature Flags
//!
//! - `nvidia` - Enable NVML support for NVIDIA GPUs
//! - `x86-cpu` - Enable raw-cpuid for x86 CPU detection

pub use domain::entities::*;
pub use domain::errors::*;
```

---

## Linting and CI

### Rustdoc Lints

Enable documentation lints in `lib.rs`:

```rust
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::private_intra_doc_links)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::invalid_html_tags)]
```

### CI Checks

Add to CI workflow:

```yaml
- name: Check documentation
  run: |
    cargo doc --no-deps --document-private-items
    cargo test --doc

- name: Check for broken links
  run: |
    cargo rustdoc -- -D warnings
```

### Local Documentation

Generate and view docs locally:

```bash
# Generate docs
cargo doc --no-deps --open

# Generate with private items
cargo doc --no-deps --document-private-items --open

# Test doc examples
cargo test --doc
```

---

## Checklist

Before submitting code, verify:

- [ ] All public items have `///` doc comments
- [ ] Modules have `//!` documentation
- [ ] Examples compile and pass (`cargo test --doc`)
- [ ] External references are included where relevant
- [ ] Intra-doc links work (`cargo doc` succeeds)
- [ ] `#[deprecated]` items explain migration path
- [ ] Complex types have usage examples
- [ ] Error conditions are documented

---

## Changelog

| Date | Changes |
|------|---------|
| 2024-12-29 | Initial standards document |
