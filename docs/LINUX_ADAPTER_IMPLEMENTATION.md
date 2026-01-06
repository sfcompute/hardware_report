# Linux Adapter Implementation Guide

> **File:** `src/adapters/secondary/system/linux.rs`  
> **Purpose:** Platform-specific hardware detection for Linux (x86_64 and aarch64)  
> **Architecture:** Adapter layer in Hexagonal/Ports-and-Adapters pattern

## Table of Contents

1. [Overview](#overview)
2. [Architecture Context](#architecture-context)
3. [Prerequisites](#prerequisites)
4. [Implementation Steps](#implementation-steps)
   - [Step 1: Update Imports](#step-1-update-imports)
   - [Step 2: Storage Detection](#step-2-storage-detection)
   - [Step 3: CPU Detection](#step-3-cpu-detection)
   - [Step 4: GPU Detection](#step-4-gpu-detection)
   - [Step 5: Network Detection](#step-5-network-detection)
5. [Helper Functions](#helper-functions)
6. [Testing](#testing)
7. [LeetCode Pattern Summary](#leetcode-pattern-summary)

---

## Overview

The `LinuxSystemInfoProvider` is an **adapter** that implements the `SystemInfoProvider` **port** (trait). It translates abstract hardware queries into Linux-specific operations (sysfs reads, command execution).

### What Changes?

| Method | Current | New |
|--------|---------|-----|
| `get_storage_info` | lsblk only | sysfs primary + lsblk enrichment + sysinfo fallback |
| `get_cpu_info` | lscpu + dmidecode | + sysfs for frequency/cache |
| `get_gpu_info` | nvidia-smi + lspci | + multi-method chain with numeric memory |
| `get_network_info` | ip command | + sysfs for driver/MTU/state |

---

## Architecture Context

```
┌─────────────────────────────────────────────────────────────────────┐
│                         YOUR CODE CHANGES                           │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│  ADAPTER LAYER: src/adapters/secondary/system/linux.rs             │
│                                                                     │
│  LinuxSystemInfoProvider                                            │
│  ├── get_storage_info()  ← MODIFY: Add sysfs detection             │
│  ├── get_cpu_info()      ← MODIFY: Add frequency/cache             │
│  ├── get_gpu_info()      ← MODIFY: Multi-method + numeric memory   │
│  └── get_network_info()  ← MODIFY: Add driver/MTU                  │
│                                                                     │
│  Helper methods (NEW):                                              │
│  ├── detect_storage_sysfs()                                        │
│  ├── detect_storage_lsblk()                                        │
│  ├── detect_cpu_sysfs_frequency()                                  │
│  ├── detect_cpu_sysfs_cache()                                      │
│  ├── detect_gpus_nvidia_smi()                                      │
│  ├── detect_gpus_lspci()                                           │
│  ├── detect_gpus_sysfs_drm()                                       │
│  ├── detect_network_sysfs()                                        │
│  ├── read_sysfs_file()                                             │
│  └── merge_*_info()                                                │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  │ implements
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│  PORT LAYER: src/ports/secondary/system.rs                         │
│                                                                     │
│  trait SystemInfoProvider {                                         │
│      fn get_storage_info() -> Result<StorageInfo, SystemError>     │
│      fn get_cpu_info() -> Result<CpuInfo, SystemError>             │
│      fn get_gpu_info() -> Result<GpuInfo, SystemError>             │
│      fn get_network_info() -> Result<NetworkInfo, SystemError>     │
│  }                                                                  │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  │ uses
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│  DOMAIN LAYER: src/domain/                                          │
│                                                                     │
│  entities.rs - StorageDevice, CpuInfo, GpuDevice, etc.             │
│  parsers/    - Pure parsing functions (no I/O)                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Prerequisites

Before modifying `linux.rs`, ensure you have:

1. **Updated `entities.rs`** with:
   - `StorageType` enum
   - Updated `StorageDevice` struct
   - Updated `CpuInfo` struct with cache/frequency
   - `GpuVendor` enum
   - Updated `GpuDevice` struct with numeric memory
   - Updated `NetworkInterface` struct

2. **Updated `parsers/storage.rs`** with:
   - `parse_sysfs_size()`
   - `parse_sysfs_rotational()`
   - `parse_lsblk_json()`
   - `is_virtual_device()`

3. **Created `parsers/gpu.rs`** with:
   - `parse_nvidia_smi_output()`
   - `parse_lspci_gpu_output()`

4. **Updated `parsers/cpu.rs`** with:
   - `parse_sysfs_freq_khz()`
   - `parse_sysfs_cache_size()`
   - `parse_proc_cpuinfo()`

---

## Implementation Steps

### Step 1: Update Imports

**Location:** Top of `linux.rs` (lines 17-30)

**Replace the existing imports with:**

```rust
// =============================================================================
// IMPORTS
// =============================================================================
//
// ARCHITECTURE NOTE:
// - We import from `domain` (entities and parsers)
// - We import from `ports` (the trait we implement)
// - We DO NOT import from other adapters (adapters are independent)
//
// LEETCODE CONNECTION: Dependency management is like LC #210 Course Schedule II
// - There's an ordering: domain → ports → adapters
// - Circular dependencies would break the build
// =============================================================================

//! Linux system information provider
//!
//! Implements `SystemInfoProvider` for Linux systems using:
//! - sysfs (`/sys`) for direct kernel data
//! - procfs (`/proc`) for process/system info  
//! - Command execution for tools like lsblk, nvidia-smi
//!
//! # Platform Support
//!
//! - x86_64: Full support including raw-cpuid
//! - aarch64: Full support via sysfs (ARM servers, DGX Spark)
//!
//! # Detection Strategy
//!
//! Each hardware type uses multiple detection methods:
//! 1. Primary: sysfs (most reliable, always available)
//! 2. Secondary: Command output (lsblk, nvidia-smi, etc.)
//! 3. Fallback: sysinfo crate (cross-platform)

use crate::domain::{
    // Existing imports - keep these
    combine_cpu_info, determine_memory_speed, determine_memory_type, 
    parse_dmidecode_bios_info, parse_dmidecode_chassis_info, parse_dmidecode_cpu, 
    parse_dmidecode_memory, parse_dmidecode_system_info, parse_free_output, 
    parse_hostname_output, parse_ip_output, parse_lscpu_output,
    BiosInfo, ChassisInfo, MemoryInfo, MotherboardInfo, NumaNode, SystemInfo,
    
    // NEW imports for enhanced entities
    CpuInfo, CpuCacheInfo, GpuInfo, GpuDevice, GpuVendor, 
    StorageInfo, StorageDevice, StorageType,
    NetworkInfo, NetworkInterface, NetworkInterfaceType,
    
    // NEW imports for parsers
    SystemError,
};

// NEW: Import parser functions
use crate::domain::parsers::storage::{
    parse_sysfs_size, parse_sysfs_rotational, parse_lsblk_json, is_virtual_device,
};
use crate::domain::parsers::cpu::{
    parse_sysfs_freq_khz, parse_sysfs_cache_size, parse_proc_cpuinfo,
};
use crate::domain::parsers::gpu::{
    parse_nvidia_smi_output, parse_lspci_gpu_output,
};

use crate::ports::{CommandExecutor, SystemCommand, SystemInfoProvider};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// NEW: Standard library imports for sysfs reading
use std::fs;
use std::path::{Path, PathBuf};
```

---

### Step 2: Storage Detection

**Location:** Replace `get_storage_info` method (around line 153-170)

**LeetCode Patterns:**
- **Chain of Responsibility**: Try sysfs → lsblk → sysinfo
- **Merge/Combine** (LC #88): Combine results from multiple sources
- **Tree Traversal** (LC #102): Walk /sys/block directory

```rust
// =============================================================================
// STORAGE DETECTION
// =============================================================================
//
// PROBLEM SOLVED:
// - Old code used only lsblk, which fails on some ARM platforms
// - New code uses sysfs as primary (works everywhere on Linux)
//
// DETECTION CHAIN (Chain of Responsibility pattern):
// 1. sysfs /sys/block - Primary, most reliable
// 2. lsblk -J - Enrichment (WWN, transport type)
// 3. sysinfo crate - Fallback if above fail
//
// LEETCODE CONNECTION:
// - LC #88 Merge Sorted Array: we merge info from multiple sources
// - LC #200 Number of Islands: walking the sysfs "grid"
// =============================================================================

async fn get_storage_info(&self) -> Result<StorageInfo, SystemError> {
    // =========================================================================
    // STEP 1: Primary detection via sysfs
    // =========================================================================
    //
    // WHY SYSFS FIRST?
    // - Direct kernel interface - always available on Linux
    // - No external tools required (lsblk might not be installed)
    // - Works identically on x86_64 and aarch64
    // - Doesn't require parsing command output (more reliable)
    //
    // sysfs structure:
    // /sys/block/
    // ├── sda/
    // │   ├── size           # Size in 512-byte sectors
    // │   ├── queue/
    // │   │   └── rotational # 0=SSD, 1=HDD
    // │   └── device/
    // │       ├── model      # Device model
    // │       └── serial     # Serial number (may need root)
    // ├── nvme0n1/
    // └── mmcblk0/           # eMMC on ARM
    // =========================================================================
    
    let mut devices = Vec::new();
    
    // if let chaining - cleaner than match for "try or log warning"
    if let Ok(sysfs_devices) = self.detect_storage_sysfs().await {
        log::debug!("sysfs detected {} storage devices", sysfs_devices.len());
        devices = sysfs_devices;
    } else {
        log::warn!("sysfs storage detection failed, trying next method");
    }
    
    // =========================================================================
    // STEP 2: Enrichment via lsblk
    // =========================================================================
    //
    // Even if sysfs worked, lsblk may have additional data:
    // - WWN (World Wide Name)
    // - Transport type (nvme, sata, usb)
    // - Serial (sometimes easier to get via lsblk)
    //
    // MERGE STRATEGY:
    // - Match by device name
    // - Fill in missing fields from lsblk
    // - Don't overwrite existing data (sysfs is more reliable)
    //
    // LEETCODE CONNECTION: This is the merge pattern
    // Similar to LC #88 Merge Sorted Array, but merging by key (device name)
    // =========================================================================
    
    if let Ok(lsblk_devices) = self.detect_storage_lsblk().await {
        log::debug!(
            "lsblk found {} devices for enrichment", 
            lsblk_devices.len()
        );
        self.merge_storage_info(&mut devices, lsblk_devices);
    }
    
    // =========================================================================
    // STEP 3: Fallback via sysinfo crate
    // =========================================================================
    //
    // If we still have no devices, something unusual is happening.
    // Try sysinfo as a cross-platform fallback.
    //
    // This can happen in:
    // - Containers with limited /sys access
    // - Unusual system configurations
    // =========================================================================
    
    if devices.is_empty() {
        log::warn!("No devices from sysfs/lsblk, trying sysinfo fallback");
        if let Ok(sysinfo_devices) = self.detect_storage_sysinfo() {
            devices = sysinfo_devices;
        }
    }
    
    // =========================================================================
    // POST-PROCESSING
    // =========================================================================
    //
    // 1. Filter virtual devices (loop, ram, dm-*)
    // 2. Ensure size fields are calculated
    // 3. Sort for consistent output
    //
    // LEETCODE CONNECTION:
    // - Filtering is like LC #283 Move Zeroes (filter in-place)
    // - Sorting is standard LC pattern
    // =========================================================================
    
    // Filter out virtual devices - they're not physical hardware
    // PATTERN: retain() is more efficient than filter() + collect()
    devices.retain(|d| d.device_type != StorageType::Virtual);
    
    // Ensure all calculated fields are populated
    for device in &mut devices {
        if device.size_gb == 0.0 && device.size_bytes > 0 {
            device.calculate_size_fields();
        }
        device.set_device_path();
    }
    
    // Sort by name for consistent, predictable output
    devices.sort_by(|a, b| a.name.cmp(&b.name));
    
    log::info!("Detected {} storage devices", devices.len());
    Ok(StorageInfo { devices })
}
```

**Add these helper methods to `impl LinuxSystemInfoProvider`:**

```rust
// =============================================================================
// STORAGE HELPER METHODS
// =============================================================================

impl LinuxSystemInfoProvider {
    /// Detect storage devices via sysfs /sys/block.
    ///
    /// # How It Works
    ///
    /// 1. Read directory listing of /sys/block
    /// 2. For each device, read attributes from sysfs files
    /// 3. Build StorageDevice struct
    ///
    /// # sysfs Paths Used
    ///
    /// | Path | Content | Example |
    /// |------|---------|---------|
    /// | `/sys/block/{dev}/size` | Sectors (×512=bytes) | "3907029168" |
    /// | `/sys/block/{dev}/queue/rotational` | 0=SSD, 1=HDD | "0" |
    /// | `/sys/block/{dev}/device/model` | Model name | "Samsung SSD 980" |
    /// | `/sys/block/{dev}/device/serial` | Serial (may need root) | "S5GXNF0N1234" |
    ///
    /// # LeetCode Connection
    ///
    /// This is **directory traversal** similar to:
    /// - LC #200 Number of Islands (grid traversal)
    /// - LC #130 Surrounded Regions
    /// - LC #417 Pacific Atlantic Water Flow
    ///
    /// We're walking a tree structure (filesystem) and extracting data.
    async fn detect_storage_sysfs(&self) -> Result<Vec<StorageDevice>, SystemError> {
        let mut devices = Vec::new();
        
        // Path to block devices in sysfs
        let sys_block = Path::new("/sys/block");
        
        // Check if sysfs is mounted/accessible
        if !sys_block.exists() {
            return Err(SystemError::NotAvailable {
                resource: "/sys/block".to_string(),
            });
        }
        
        // Read directory entries
        // PATTERN: This is the "traversal" part - we visit each node (device)
        let entries = fs::read_dir(sys_block).map_err(|e| {
            SystemError::IoError {
                path: "/sys/block".to_string(),
                message: e.to_string(),
            }
        })?;
        
        // Process each block device
        for entry in entries.flatten() {
            let device_name = entry.file_name().to_string_lossy().to_string();
            
            // ─────────────────────────────────────────────────────────────
            // EARLY FILTERING: Skip virtual devices
            // ─────────────────────────────────────────────────────────────
            // WHY EARLY? Saves I/O - don't read attributes for devices we'll skip
            // PATTERN: This is like LC #283 Move Zeroes - filter early
            if is_virtual_device(&device_name) {
                log::trace!("Skipping virtual device: {}", device_name);
                continue;
            }
            
            let device_path = entry.path();
            
            // ─────────────────────────────────────────────────────────────
            // READ SIZE (required field)
            // ─────────────────────────────────────────────────────────────
            // If we can't get size, skip this device (probably not real storage)
            // 
            // PATTERN: let-else for early return/continue on failure
            // This is cleaner than nested match statements
            let size_path = device_path.join("size");
            let Ok(content) = self.read_sysfs_file(&size_path) else {
                continue;
            };
            let Ok(size_bytes) = parse_sysfs_size(&content) else {
                log::trace!("Cannot parse size for {}: invalid format", device_name);
                continue;
            };
            
            // Skip tiny devices (< 1GB) - probably not real storage
            // USB sticks, boot partitions, etc.
            const MIN_SIZE: u64 = 1_000_000_000; // 1 GB
            if size_bytes < MIN_SIZE {
                log::trace!("Skipping small device {}: {} bytes", device_name, size_bytes);
                continue;
            }
            
            // ─────────────────────────────────────────────────────────────
            // READ ROTATIONAL FLAG
            // ─────────────────────────────────────────────────────────────
            // 0 = SSD/NVMe (no spinning platters)
            // 1 = HDD (spinning platters)
            let rotational_path = device_path.join("queue/rotational");
            let is_rotational = self.read_sysfs_file(&rotational_path)
                .map(|content| parse_sysfs_rotational(&content))
                .unwrap_or(false); // Default to SSD if unknown
            
            // ─────────────────────────────────────────────────────────────
            // DETERMINE DEVICE TYPE
            // ─────────────────────────────────────────────────────────────
            // Combines name pattern + rotational flag
            let device_type = StorageType::from_device(&device_name, is_rotational);
            
            // ─────────────────────────────────────────────────────────────
            // READ OPTIONAL FIELDS
            // ─────────────────────────────────────────────────────────────
            // These may fail (especially serial without root) - that's OK
            
            // Model name
            let model = self.read_sysfs_file(&device_path.join("device/model"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            
            // Serial number (may require root)
            let serial_number = self.read_sysfs_file(&device_path.join("device/serial"))
                .map(|s| s.trim().to_string())
                .ok()
                .filter(|s| !s.is_empty());
            
            // Firmware version
            let firmware_version = self.read_sysfs_file(&device_path.join("device/firmware_rev"))
                .map(|s| s.trim().to_string())
                .ok()
                .filter(|s| !s.is_empty());
            
            // For NVMe, try alternate paths
            let (serial_number, firmware_version) = if device_type == StorageType::Nvme {
                self.read_nvme_sysfs_attrs(&device_name, serial_number, firmware_version)
            } else {
                (serial_number, firmware_version)
            };
            
            // ─────────────────────────────────────────────────────────────
            // DETERMINE INTERFACE TYPE
            // ─────────────────────────────────────────────────────────────
            let interface = match &device_type {
                StorageType::Nvme => "NVMe".to_string(),
                StorageType::Emmc => "eMMC".to_string(),
                StorageType::Hdd | StorageType::Ssd => "SATA".to_string(),
                _ => "Unknown".to_string(),
            };
            
            // ─────────────────────────────────────────────────────────────
            // BUILD THE DEVICE STRUCT
            // ─────────────────────────────────────────────────────────────
            // PATTERN: Builder pattern - set required fields, then optional
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
            
            // Calculate derived fields
            device.calculate_size_fields();
            
            devices.push(device);
        }
        
        Ok(devices)
    }
    
    /// Read NVMe-specific sysfs attributes.
    ///
    /// NVMe devices have attributes in a different location:
    /// `/sys/class/nvme/nvme0/serial` instead of `/sys/block/nvme0n1/device/serial`
    ///
    /// # Arguments
    ///
    /// * `device_name` - Block device name (e.g., "nvme0n1")
    /// * `existing_serial` - Serial from block device path (may be None)
    /// * `existing_firmware` - Firmware from block device path (may be None)
    fn read_nvme_sysfs_attrs(
        &self,
        device_name: &str,
        existing_serial: Option<String>,
        existing_firmware: Option<String>,
    ) -> (Option<String>, Option<String>) {
        // Extract controller name: "nvme0n1" -> "nvme0"
        // PATTERN: String manipulation - find pattern and extract
        let controller = device_name
            .chars()
            .take_while(|c| !c.is_ascii_digit() || device_name.starts_with("nvme"))
            .take_while(|&c| c != 'n' || device_name.find("nvme").is_some())
            .collect::<String>();
        
        // Try to extract just "nvme0" from "nvme0n1"
        let controller = if device_name.starts_with("nvme") {
            // Find position of 'n' that's followed by a digit (the namespace)
            if let Some(pos) = device_name[4..].find('n') {
                &device_name[..4 + pos]
            } else {
                &device_name
            }
        } else {
            &device_name
        };
        
        let nvme_path = PathBuf::from("/sys/class/nvme").join(controller);
        
        // Try to get serial from NVMe class path
        let serial = existing_serial.or_else(|| {
            self.read_sysfs_file(&nvme_path.join("serial"))
                .map(|s| s.trim().to_string())
                .ok()
                .filter(|s| !s.is_empty())
        });
        
        // Try to get firmware from NVMe class path
        let firmware = existing_firmware.or_else(|| {
            self.read_sysfs_file(&nvme_path.join("firmware_rev"))
                .map(|s| s.trim().to_string())
                .ok()
                .filter(|s| !s.is_empty())
        });
        
        (serial, firmware)
    }
    
    /// Detect storage via lsblk command (JSON output).
    ///
    /// # Command
    ///
    /// ```bash
    /// lsblk -J -b -o NAME,SIZE,TYPE,MODEL,SERIAL,ROTA,TRAN,WWN
    /// ```
    ///
    /// # Flags
    ///
    /// - `-J` = JSON output (easier to parse than text)
    /// - `-b` = Size in bytes (not human-readable)
    /// - `-o` = Specify columns
    ///
    /// # When to Use
    ///
    /// - Enrichment after sysfs (WWN, transport)
    /// - Fallback if sysfs fails
    async fn detect_storage_lsblk(&self) -> Result<Vec<StorageDevice>, SystemError> {
        let cmd = SystemCommand::new("lsblk")
            .args(&[
                "-J",           // JSON output
                "-b",           // Bytes (not human readable)
                "-d",           // No partitions
                "-o", "NAME,SIZE,TYPE,MODEL,SERIAL,ROTA,TRAN,WWN",
            ])
            .timeout(Duration::from_secs(10));
        
        // PATTERN: Combined error mapping with and_then
        // Execute command and check success in one chain
        let output = self.command_executor.execute(&cmd).await.map_err(|e| {
            SystemError::CommandFailed {
                command: "lsblk".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            }
        })?;
        
        // PATTERN: Guard clause with let-else for cleaner flow
        let output = if output.success { output } else {
            return Err(SystemError::CommandFailed {
                command: "lsblk".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr,
            });
        };
        
        parse_lsblk_json(&output.stdout).map_err(SystemError::ParseError)
    }
    
    /// Detect storage via sysinfo crate (cross-platform fallback).
    ///
    /// # Limitations
    ///
    /// sysinfo provides:
    /// - Mounted filesystems (not raw block devices)
    /// - Limited metadata (no serial, model, etc.)
    ///
    /// Use only as last resort.
    fn detect_storage_sysinfo(&self) -> Result<Vec<StorageDevice>, SystemError> {
        use sysinfo::Disks;
        
        let disks = Disks::new_with_refreshed_list();
        let mut devices = Vec::new();
        
        for disk in disks.iter() {
            let size_bytes = disk.total_space();
            
            // Skip small devices
            if size_bytes < 1_000_000_000 {
                continue;
            }
            
            let name = disk.name().to_string_lossy().to_string();
            let name = if name.is_empty() {
                disk.mount_point().to_string_lossy().to_string()
            } else {
                name
            };
            
            let mut device = StorageDevice {
                name,
                size_bytes,
                detection_method: "sysinfo".to_string(),
                ..Default::default()
            };
            
            device.calculate_size_fields();
            devices.push(device);
        }
        
        Ok(devices)
    }
    
    /// Merge storage info from secondary source into primary.
    ///
    /// # Strategy
    ///
    /// 1. Match devices by name
    /// 2. Fill in missing fields from secondary
    /// 3. Don't overwrite existing data (primary is authoritative)
    ///
    /// # LeetCode Connection
    ///
    /// This is the **merge** pattern:
    /// - LC #88 Merge Sorted Array
    /// - LC #21 Merge Two Sorted Lists
    /// - LC #56 Merge Intervals
    ///
    /// Key insight: we're merging by KEY (device name), not by position.
    ///
    /// # Complexity
    ///
    /// Current: O(n × m) where n = primary.len(), m = secondary.len()
    ///
    /// Could optimize with HashMap for O(n + m), but device lists are small
    /// (typically < 20), so linear search is fine and simpler.
    fn merge_storage_info(
        &self,
        primary: &mut Vec<StorageDevice>,
        secondary: Vec<StorageDevice>,
    ) {
        for sec_device in secondary {
            // PATTERN: if-let-else for merge-or-insert
            if let Some(pri_device) = primary.iter_mut().find(|d| d.name == sec_device.name) {
                // PATTERN: Option::or() for null coalescing - much cleaner!
                // Before: if pri.field.is_none() { pri.field = sec.field; }
                // After:  pri.field = pri.field.take().or(sec.field);
                pri_device.serial_number = pri_device.serial_number.take().or(sec_device.serial_number);
                pri_device.firmware_version = pri_device.firmware_version.take().or(sec_device.firmware_version);
                pri_device.wwn = pri_device.wwn.take().or(sec_device.wwn);
                
                // PATTERN: Conditional assignment with && guard
                if pri_device.model.is_empty() && !sec_device.model.is_empty() {
                    pri_device.model = sec_device.model;
                }
            } else {
                // Device not in primary - add it
                primary.push(sec_device);
            }
        }
    }
}
```

---

### Step 3: CPU Detection

**Location:** Update `get_cpu_info` method (around line 67-102)

```rust
// =============================================================================
// CPU DETECTION
// =============================================================================
//
// ENHANCEMENTS:
// - Add frequency_mhz (numeric, not string)
// - Add cache sizes (L1d, L1i, L2, L3)
// - Add CPU flags/features
// - Better ARM support via /proc/cpuinfo
//
// DETECTION CHAIN:
// 1. sysfs for frequency and cache
// 2. /proc/cpuinfo for model, vendor, flags
// 3. lscpu for topology
// 4. dmidecode for additional data (with privileges)
// =============================================================================

async fn get_cpu_info(&self) -> Result<CpuInfo, SystemError> {
    // Start with basic info from lscpu (existing code)
    let lscpu_cmd = SystemCommand::new("lscpu").timeout(Duration::from_secs(10));
    let lscpu_output = self
        .command_executor
        .execute(&lscpu_cmd)
        .await
        .map_err(|e| SystemError::CommandFailed {
            command: "lscpu".to_string(),
            exit_code: None,
            stderr: e.to_string(),
        })?;

    let mut cpu_info = parse_lscpu_output(&lscpu_output.stdout)
        .map_err(SystemError::ParseError)?;
    
    // =========================================================================
    // ENHANCEMENT 1: Frequency from sysfs
    // =========================================================================
    //
    // sysfs provides exact frequency in kHz
    // More reliable than parsing lscpu string output
    //
    // Paths:
    // - /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq (max frequency)
    // - /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq (current)
    // =========================================================================
    
    // PATTERN: if-let with destructuring for tuple results
    // Clean way to handle optional enhancement without nested blocks
    if let Ok((freq_mhz, freq_min, freq_max)) = self.detect_cpu_sysfs_frequency().await {
        cpu_info.frequency_mhz = freq_mhz;
        cpu_info.frequency_min_mhz = freq_min;
        cpu_info.frequency_max_mhz = freq_max;
        cpu_info.set_speed_string();
        cpu_info.detection_methods.push("sysfs_freq".to_string());
    }
    
    // =========================================================================
    // ENHANCEMENT 2: Cache from sysfs
    // =========================================================================
    //
    // sysfs provides detailed cache hierarchy:
    // /sys/devices/system/cpu/cpu0/cache/index0/  (L1d typically)
    // /sys/devices/system/cpu/cpu0/cache/index1/  (L1i typically)
    // /sys/devices/system/cpu/cpu0/cache/index2/  (L2)
    // /sys/devices/system/cpu/cpu0/cache/index3/  (L3)
    //
    // Each has: level, type, size, ways_of_associativity, etc.
    // =========================================================================
    
    // PATTERN: if-let + for loop with tuple matching
    // Avoids deep nesting by using tuple pattern matching
    if let Ok(caches) = self.detect_cpu_sysfs_cache().await {
        for cache in &caches {
            // Tuple matching is cleaner than nested if-else
            match (cache.level, cache.cache_type.as_str()) {
                (1, "Data") => cpu_info.cache_l1d_kb = Some(cache.size_kb),
                (1, "Instruction") => cpu_info.cache_l1i_kb = Some(cache.size_kb),
                (2, _) => cpu_info.cache_l2_kb = Some(cache.size_kb),
                (3, _) => cpu_info.cache_l3_kb = Some(cache.size_kb),
                _ => {} // L4 or unified caches - ignored for now
            }
        }
        cpu_info.caches = caches;
        cpu_info.detection_methods.push("sysfs_cache".to_string());
    }
    
    // =========================================================================
    // ENHANCEMENT 3: Flags and vendor from /proc/cpuinfo
    // =========================================================================
    //
    // /proc/cpuinfo format differs by architecture:
    //
    // x86_64:
    //   vendor_id : GenuineIntel
    //   flags     : fpu vme de pse avx avx2 avx512f ...
    //
    // aarch64:
    //   CPU implementer : 0x41
    //   CPU part        : 0xd0c
    //   Features        : fp asimd evtstrm aes ...
    // =========================================================================
    
    // PATTERN: if-let with && chaining for conditional field updates
    // Each field update only happens if condition is met
    if let Ok(proc_info) = self.read_proc_cpuinfo().await {
        // PATTERN: Short-circuit with && for conditional assignment
        if !proc_info.flags.is_empty() { cpu_info.flags = proc_info.flags; }
        
        // PATTERN: && chaining avoids nested if blocks
        if cpu_info.vendor.is_empty() && !proc_info.vendor.is_empty() {
            cpu_info.vendor = proc_info.vendor;
        }
        
        // PATTERN: Option::is_some() then take - or use or_else
        cpu_info.microarchitecture = cpu_info.microarchitecture.or(proc_info.microarchitecture);
        
        cpu_info.detection_methods.push("proc_cpuinfo".to_string());
    }
    
    // =========================================================================
    // EXISTING: dmidecode enrichment (with privileges)
    // =========================================================================
    
    let dmidecode_cmd = SystemCommand::new("dmidecode")
        .args(&["-t", "processor"])
        .timeout(Duration::from_secs(10));

    // PATTERN: Nested if-let flattened with && conditions
    // Original: if let Ok { if success { if let Ok { ... } } }
    // Refactored: Single if-let chain with && guard
    if let Ok(output) = self.command_executor.execute_with_privileges(&dmidecode_cmd).await {
        if output.success && let Ok(dmidecode_info) = parse_dmidecode_cpu(&output.stdout) {
            cpu_info = combine_cpu_info(cpu_info, dmidecode_info);
            cpu_info.detection_methods.push("dmidecode".to_string());
        }
    }
    
    // Calculate totals
    cpu_info.calculate_totals();
    
    // Set architecture
    cpu_info.architecture = std::env::consts::ARCH.to_string();
    
    Ok(cpu_info)
}
```

**Add CPU helper methods:**

```rust
// =============================================================================
// CPU HELPER METHODS
// =============================================================================

impl LinuxSystemInfoProvider {
    /// Detect CPU frequency from sysfs.
    ///
    /// # sysfs Paths
    ///
    /// - `/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq` - Max frequency (kHz)
    /// - `/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_min_freq` - Min frequency (kHz)
    /// - `/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq` - Current (kHz)
    ///
    /// # Returns
    ///
    /// Tuple of (primary_mhz, min_mhz, max_mhz)
    ///
    /// # LeetCode Connection
    ///
    /// File I/O with error handling is like parsing problems:
    /// - Handle missing files gracefully
    /// - Convert units (kHz → MHz)
    async fn detect_cpu_sysfs_frequency(&self) 
        -> Result<(u32, Option<u32>, Option<u32>), SystemError> 
    {
        let cpu_path = Path::new("/sys/devices/system/cpu/cpu0/cpufreq");
        
        if !cpu_path.exists() {
            return Err(SystemError::NotAvailable {
                resource: "/sys/devices/system/cpu/cpu0/cpufreq".to_string(),
            });
        }
        
        // Read max frequency (primary)
        let max_freq = self.read_sysfs_file(&cpu_path.join("cpuinfo_max_freq"))
            .ok()
            .and_then(|s| parse_sysfs_freq_khz(&s).ok());
        
        // Read min frequency
        let min_freq = self.read_sysfs_file(&cpu_path.join("cpuinfo_min_freq"))
            .ok()
            .and_then(|s| parse_sysfs_freq_khz(&s).ok());
        
        // Read current frequency (fallback for primary)
        let cur_freq = self.read_sysfs_file(&cpu_path.join("scaling_cur_freq"))
            .ok()
            .and_then(|s| parse_sysfs_freq_khz(&s).ok());
        
        // Use max as primary, fall back to current
        let primary = max_freq.or(cur_freq).unwrap_or(0);
        
        Ok((primary, min_freq, max_freq))
    }
    
    /// Detect CPU cache hierarchy from sysfs.
    ///
    /// # sysfs Structure
    ///
    /// ```text
    /// /sys/devices/system/cpu/cpu0/cache/
    /// ├── index0/    # Usually L1 Data
    /// │   ├── level  # "1"
    /// │   ├── type   # "Data"
    /// │   └── size   # "32K"
    /// ├── index1/    # Usually L1 Instruction
    /// ├── index2/    # Usually L2
    /// └── index3/    # Usually L3 (shared)
    /// ```
    ///
    /// # LeetCode Connection
    ///
    /// Directory traversal with structured data extraction:
    /// - Similar to LC #102 Level Order Traversal (visiting nodes at each level)
    /// - Cache hierarchy IS a tree structure!
    async fn detect_cpu_sysfs_cache(&self) -> Result<Vec<CpuCacheInfo>, SystemError> {
        let cache_path = Path::new("/sys/devices/system/cpu/cpu0/cache");
        
        if !cache_path.exists() {
            return Err(SystemError::NotAvailable {
                resource: cache_path.to_string_lossy().to_string(),
            });
        }
        
        let mut caches = Vec::new();
        
        // Iterate through index0, index1, index2, index3
        for i in 0..10 {
            let index_path = cache_path.join(format!("index{}", i));
            
            if !index_path.exists() {
                break;
            }
            
            // Read cache attributes
            let level: u8 = self.read_sysfs_file(&index_path.join("level"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
            
            let cache_type = self.read_sysfs_file(&index_path.join("type"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "Unknown".to_string());
            
            let size_kb = self.read_sysfs_file(&index_path.join("size"))
                .ok()
                .and_then(|s| parse_sysfs_cache_size(&s).ok())
                .unwrap_or(0);
            
            let ways = self.read_sysfs_file(&index_path.join("ways_of_associativity"))
                .ok()
                .and_then(|s| s.trim().parse().ok());
            
            let line_size = self.read_sysfs_file(&index_path.join("coherency_line_size"))
                .ok()
                .and_then(|s| s.trim().parse().ok());
            
            caches.push(CpuCacheInfo {
                level,
                cache_type,
                size_kb,
                ways_of_associativity: ways,
                line_size_bytes: line_size,
                sets: None,
                shared: Some(level >= 3), // L3 is typically shared
            });
        }
        
        Ok(caches)
    }
    
    /// Read and parse /proc/cpuinfo.
    ///
    /// # Why?
    ///
    /// /proc/cpuinfo contains:
    /// - CPU flags/features (important for workload compatibility)
    /// - Vendor identification
    /// - ARM CPU part numbers (for microarchitecture detection)
    async fn read_proc_cpuinfo(&self) -> Result<CpuInfo, SystemError> {
        let content = fs::read_to_string("/proc/cpuinfo").map_err(|e| {
            SystemError::IoError {
                path: "/proc/cpuinfo".to_string(),
                message: e.to_string(),
            }
        })?;
        
        parse_proc_cpuinfo(&content).map_err(SystemError::ParseError)
    }
}
```

---

### Step 4: GPU Detection

**Location:** Replace `get_gpu_info` method (around line 172-232)

```rust
// =============================================================================
// GPU DETECTION
// =============================================================================
//
// MULTI-METHOD CHAIN (Chain of Responsibility pattern):
//
// Priority 1: NVML (if feature enabled)
//   - Most accurate for NVIDIA
//   - Direct library, no parsing
//
// Priority 2: nvidia-smi
//   - Fallback for NVIDIA
//   - Parse CSV output
//
// Priority 3: rocm-smi
//   - AMD GPU detection
//   - Parse JSON output
//
// Priority 4: sysfs /sys/class/drm
//   - Universal Linux
//   - Works for all vendors
//   - Limited memory info
//
// Priority 5: lspci
//   - Basic enumeration
//   - No memory/driver info
//   - Last resort
//
// LEETCODE CONNECTION:
// - Chain of Responsibility is like trying multiple approaches
// - LC #322 Coin Change: try different options
// - LC #70 Climbing Stairs: multiple ways to reach goal
// =============================================================================

async fn get_gpu_info(&self) -> Result<GpuInfo, SystemError> {
    let mut devices = Vec::new();
    
    // =========================================================================
    // METHOD 1: nvidia-smi (NVIDIA GPUs)
    // =========================================================================
    //
    // Command: nvidia-smi --query-gpu=... --format=csv,noheader,nounits
    //
    // Key flags:
    // - nounits: Returns "81920" instead of "81920 MiB"
    // - noheader: Skip column headers
    // - csv: Comma-separated for easy parsing
    //
    // Fields we query:
    // - index, name, uuid, memory.total, memory.free
    // - pci.bus_id, driver_version, compute_cap
    // =========================================================================
    
    if let Ok(nvidia_devices) = self.detect_gpus_nvidia_smi().await {
        log::debug!("nvidia-smi detected {} GPUs", nvidia_devices.len());
        devices.extend(nvidia_devices);
    }
    
    // =========================================================================
    // METHOD 2: rocm-smi (AMD GPUs)
    // =========================================================================
    //
    // Only try if we don't have NVIDIA GPUs (or want both)
    // AMD GPUs won't show up via nvidia-smi
    // =========================================================================
    
    if let Ok(amd_devices) = self.detect_gpus_rocm_smi().await {
        log::debug!("rocm-smi detected {} GPUs", amd_devices.len());
        // Merge AMD GPUs (they won't conflict with NVIDIA by name)
        devices.extend(amd_devices);
    }
    
    // =========================================================================
    // METHOD 3: sysfs /sys/class/drm (enrichment or fallback)
    // =========================================================================
    //
    // sysfs provides:
    // - PCI vendor/device IDs
    // - NUMA node
    // - AMD: Memory info via mem_info_vram_total
    //
    // Use to:
    // - Enrich existing devices with NUMA info
    // - Fallback detection if commands failed
    // =========================================================================
    
    if let Ok(drm_devices) = self.detect_gpus_sysfs_drm().await {
        log::debug!("sysfs DRM detected {} GPUs", drm_devices.len());
        self.merge_gpu_info(&mut devices, drm_devices);
    }
    
    // =========================================================================
    // METHOD 4: lspci (last resort)
    // =========================================================================
    //
    // If we still have no GPUs, try lspci
    // This only gives us basic enumeration (no memory, driver)
    // =========================================================================
    
    if devices.is_empty() {
        if let Ok(lspci_devices) = self.detect_gpus_lspci().await {
            log::debug!("lspci detected {} GPUs", lspci_devices.len());
            devices = lspci_devices;
        }
    }
    
    // =========================================================================
    // POST-PROCESSING
    // =========================================================================
    
    // Re-index devices
    for (i, device) in devices.iter_mut().enumerate() {
        device.index = i as u32;
        
        // Ensure legacy memory field is set
        #[allow(deprecated)]
        if device.memory.is_empty() {
            device.set_memory_string();
        }
    }
    
    // Sort by index for consistent output
    devices.sort_by_key(|d| d.index);
    
    log::info!("Detected {} GPUs", devices.len());
    Ok(GpuInfo { devices })
}
```

**Add GPU helper methods:**

```rust
// =============================================================================
// GPU HELPER METHODS
// =============================================================================

impl LinuxSystemInfoProvider {
    /// Detect NVIDIA GPUs via nvidia-smi command.
    ///
    /// # Command
    ///
    /// ```bash
    /// nvidia-smi --query-gpu=index,name,uuid,memory.total,memory.free,pci.bus_id,driver_version,compute_cap \
    ///   --format=csv,noheader,nounits
    /// ```
    ///
    /// # Output Format
    ///
    /// ```text
    /// 0, NVIDIA H100 80GB HBM3, GPU-xxxx, 81920, 81000, 00000000:01:00.0, 535.129.03, 9.0
    /// ```
    ///
    /// # LeetCode Connection
    ///
    /// CSV parsing is like string manipulation problems:
    /// - LC #68 Text Justification
    /// - LC #722 Remove Comments
    async fn detect_gpus_nvidia_smi(&self) -> Result<Vec<GpuDevice>, SystemError> {
        let cmd = SystemCommand::new("nvidia-smi")
            .args(&[
                "--query-gpu=index,name,uuid,memory.total,memory.free,pci.bus_id,driver_version,compute_cap",
                "--format=csv,noheader,nounits",
            ])
            .timeout(Duration::from_secs(10));
        
        let output = self.command_executor.execute(&cmd).await.map_err(|e| {
            SystemError::CommandFailed {
                command: "nvidia-smi".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            }
        })?;
        
        // PATTERN: let-else for guard clause
        // Cleaner than if !success { return Err } 
        let output = if output.success { output } else {
            return Err(SystemError::CommandFailed {
                command: "nvidia-smi".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr,
            });
        };
        
        parse_nvidia_smi_output(&output.stdout).map_err(SystemError::ParseError)
    }
    
    /// Detect AMD GPUs via rocm-smi command.
    ///
    /// # Command
    ///
    /// ```bash
    /// rocm-smi --showproductname --showmeminfo vram --showdriver --json
    /// ```
    async fn detect_gpus_rocm_smi(&self) -> Result<Vec<GpuDevice>, SystemError> {
        let cmd = SystemCommand::new("rocm-smi")
            .args(&["--showproductname", "--showmeminfo", "vram", "--showdriver", "--json"])
            .timeout(Duration::from_secs(10));
        
        // PATTERN: Match with guard clause for conditional success
        // This is idiomatic when you need both Ok AND a condition
        let Ok(output) = self.command_executor.execute(&cmd).await else {
            return Ok(Vec::new()); // rocm-smi not available - not an AMD system
        };
        if !output.success {
            return Ok(Vec::new()); // Command failed - not an AMD system
        }
        
        // Parse rocm-smi JSON output
        // TODO: Implement parse_rocm_smi_output in parsers/gpu.rs
        self.parse_rocm_smi_json(&output.stdout)
    }
    
    /// Parse rocm-smi JSON output (inline for now).
    fn parse_rocm_smi_json(&self, output: &str) -> Result<Vec<GpuDevice>, SystemError> {
        let json: serde_json::Value = serde_json::from_str(output)
            .map_err(|e| SystemError::ParseError(format!("rocm-smi JSON parse error: {}", e)))?;
        
        // PATTERN: let-else for early return when required data missing
        let Some(obj) = json.as_object() else {
            return Ok(Vec::new()); // Not a JSON object - no GPUs
        };
        
        // PATTERN: filter_map + enumerate for index tracking
        // Cleaner than manual index increment
        let devices: Vec<GpuDevice> = obj.iter()
            .filter(|(key, _)| key.starts_with("card"))
            .enumerate()
            .map(|(index, (_, value))| {
                // PATTERN: and_then chains for nested Option extraction
                let name = value.get("Card series")
                    .and_then(|v| v.as_str())
                    .unwrap_or("AMD GPU")
                    .to_string();
                
                let memory_bytes = value.get("VRAM Total Memory (B)")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                
                let driver_version = value.get("Driver version")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                GpuDevice {
                    index: index as u32,
                    name,
                    uuid: format!("amd-gpu-{}", index),
                    memory_total_mb: memory_bytes / (1024 * 1024),
                    driver_version,
                    vendor: GpuVendor::Amd,
                    vendor_name: "AMD".to_string(),
                    detection_method: "rocm-smi".to_string(),
                    ..Default::default()
                }
            })
            .collect();
        
        Ok(devices)
    }
    
    /// Detect GPUs via sysfs DRM interface.
    ///
    /// # sysfs Paths
    ///
    /// ```text
    /// /sys/class/drm/card0/device/
    /// ├── vendor            # PCI vendor ID ("0x10de" = NVIDIA)
    /// ├── device            # PCI device ID
    /// ├── numa_node         # NUMA affinity
    /// └── mem_info_vram_total  # AMD: VRAM size in bytes
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Get NUMA node info for all GPUs
    /// - Get memory info for AMD GPUs
    /// - Fallback enumeration
    async fn detect_gpus_sysfs_drm(&self) -> Result<Vec<GpuDevice>, SystemError> {
        let drm_path = Path::new("/sys/class/drm");
        
        if !drm_path.exists() {
            return Err(SystemError::NotAvailable {
                resource: "/sys/class/drm".to_string(),
            });
        }
        
        let mut devices = Vec::new();
        let mut index = 0;
        
        let entries = fs::read_dir(drm_path).map_err(|e| SystemError::IoError {
            path: "/sys/class/drm".to_string(),
            message: e.to_string(),
        })?;
        
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Only process card* entries (not renderD*)
            if !name.starts_with("card") || name.contains("-") {
                continue;
            }
            
            let card_path = entry.path();
            let device_path = card_path.join("device");
            
            if !device_path.exists() {
                continue;
            }
            
            // Read PCI vendor ID
            let vendor_id = self.read_sysfs_file(&device_path.join("vendor"))
                .map(|s| s.trim().trim_start_matches("0x").to_string())
                .unwrap_or_default();
            
            let vendor = GpuVendor::from_pci_vendor(&vendor_id);
            
            // Skip if not a GPU vendor we recognize
            if vendor == GpuVendor::Unknown {
                continue;
            }
            
            // Read PCI device ID
            let device_id = self.read_sysfs_file(&device_path.join("device"))
                .map(|s| s.trim().trim_start_matches("0x").to_string())
                .unwrap_or_default();
            
            // Read NUMA node
            let numa_node = self.read_sysfs_file(&device_path.join("numa_node"))
                .ok()
                .and_then(|s| s.trim().parse::<i32>().ok());
            
            // AMD-specific: Read VRAM size
            let memory_total_mb = if vendor == GpuVendor::Amd {
                self.read_sysfs_file(&device_path.join("mem_info_vram_total"))
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok())
                    .map(|bytes| bytes / (1024 * 1024))
                    .unwrap_or(0)
            } else {
                0
            };
            
            let device = GpuDevice {
                index,
                name: format!("{} GPU ({})", vendor.name(), name),
                uuid: format!("drm-{}", name),
                memory_total_mb,
                pci_id: format!("{}:{}", vendor_id, device_id),
                vendor: vendor.clone(),
                vendor_name: vendor.name().to_string(),
                numa_node,
                detection_method: "sysfs".to_string(),
                ..Default::default()
            };
            
            devices.push(device);
            index += 1;
        }
        
        Ok(devices)
    }
    
    /// Detect GPUs via lspci command (last resort).
    async fn detect_gpus_lspci(&self) -> Result<Vec<GpuDevice>, SystemError> {
        let cmd = SystemCommand::new("lspci")
            .args(&["-nn"])
            .timeout(Duration::from_secs(5));
        
        let output = self.command_executor.execute(&cmd).await.map_err(|e| {
            SystemError::CommandFailed {
                command: "lspci".to_string(),
                exit_code: None,
                stderr: e.to_string(),
            }
        })?;
        
        if !output.success {
            return Err(SystemError::CommandFailed {
                command: "lspci".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr.clone(),
            });
        }
        
        parse_lspci_gpu_output(&output.stdout).map_err(SystemError::ParseError)
    }
    
    /// Merge GPU info from secondary source into primary.
    ///
    /// Match by PCI bus ID or name, fill in missing fields.
    fn merge_gpu_info(&self, primary: &mut Vec<GpuDevice>, secondary: Vec<GpuDevice>) {
        for sec_gpu in secondary {
            // PATTERN: and_then + find with predicate chaining
            // Cleaner than nested if-let
            let matched = sec_gpu.pci_bus_id.as_ref().and_then(|sec_bus_id| {
                primary.iter_mut().find(|g| {
                    g.pci_bus_id.as_ref().is_some_and(|id| id == sec_bus_id)
                })
            });
            
            // PATTERN: if-let-else for merge-or-insert logic
            if let Some(pri_gpu) = matched {
                // PATTERN: or/or_else for null coalescing
                pri_gpu.numa_node = pri_gpu.numa_node.or(sec_gpu.numa_node);
                if pri_gpu.pci_id.is_empty() { pri_gpu.pci_id = sec_gpu.pci_id; }
                if pri_gpu.memory_total_mb == 0 { pri_gpu.memory_total_mb = sec_gpu.memory_total_mb; }
            } else if !primary.iter().any(|g| g.name == sec_gpu.name) {
                primary.push(sec_gpu);
            }
        }
    }
}
```

---

### Step 5: Network Detection

**Location:** Update `get_network_info` method (around line 234-252)

```rust
// =============================================================================
// NETWORK DETECTION
// =============================================================================
//
// ENHANCEMENTS:
// - Add driver and driver_version
// - Add MTU
// - Add is_up, is_virtual
// - Add speed_mbps (numeric)
// =============================================================================

async fn get_network_info(&self) -> Result<NetworkInfo, SystemError> {
    // Get basic interface info from ip command (existing code)
    let ip_cmd = SystemCommand::new("ip")
        .args(&["addr", "show"])
        .timeout(Duration::from_secs(5));
    
    let ip_output = self.command_executor.execute(&ip_cmd).await.map_err(|e| {
        SystemError::CommandFailed {
            command: "ip".to_string(),
            exit_code: None,
            stderr: e.to_string(),
        }
    })?;

    let mut interfaces = parse_ip_output(&ip_output.stdout)
        .map_err(SystemError::ParseError)?;
    
    // =========================================================================
    // ENHANCEMENT: Enrich with sysfs data
    // =========================================================================
    //
    // sysfs provides:
    // - /sys/class/net/{iface}/operstate (up/down)
    // - /sys/class/net/{iface}/speed (Mbps, may be -1)
    // - /sys/class/net/{iface}/mtu
    // - /sys/class/net/{iface}/device/driver -> symlink to driver
    // =========================================================================
    
    for iface in &mut interfaces {
        self.enrich_network_interface_sysfs(iface).await;
    }
    
    Ok(NetworkInfo {
        interfaces,
        infiniband: None, // TODO: Add infiniband detection
    })
}
```

**Add network helper methods:**

```rust
// =============================================================================
// NETWORK HELPER METHODS
// =============================================================================

impl LinuxSystemInfoProvider {
    /// Enrich network interface with sysfs data.
    ///
    /// # sysfs Paths
    ///
    /// ```text
    /// /sys/class/net/{iface}/
    /// ├── operstate         # "up", "down", "unknown"
    /// ├── speed             # Mbps (may be -1 if unknown)
    /// ├── mtu               # MTU in bytes
    /// ├── carrier           # 1 = link detected
    /// └── device/
    ///     └── driver/       # Symlink to driver module
    ///         └── module/
    ///             └── version  # Driver version
    /// ```
    async fn enrich_network_interface_sysfs(&self, iface: &mut NetworkInterface) {
        let iface_path = PathBuf::from("/sys/class/net").join(&iface.name);
        
        if !iface_path.exists() {
            return;
        }
        
        // ─────────────────────────────────────────────────────────────
        // OPERATIONAL STATE
        // ─────────────────────────────────────────────────────────────
        iface.is_up = self.read_sysfs_file(&iface_path.join("operstate"))
            .map(|s| s.trim().to_lowercase() == "up")
            .unwrap_or(false);
        
        // ─────────────────────────────────────────────────────────────
        // SPEED
        // ─────────────────────────────────────────────────────────────
        // May be -1 if link is down or speed is unknown
        // PATTERN: if-let chain with && for multiple conditions
        if let Ok(speed_str) = self.read_sysfs_file(&iface_path.join("speed"))
            && let Ok(speed) = speed_str.trim().parse::<i32>()
            && speed > 0
        {
            iface.speed_mbps = Some(speed as u32);
            iface.speed = Some(format!("{} Mbps", speed));
        }
        
        // ─────────────────────────────────────────────────────────────
        // MTU
        // ─────────────────────────────────────────────────────────────
        // PATTERN: if-let chain - parse only if read succeeds
        if let Ok(mtu_str) = self.read_sysfs_file(&iface_path.join("mtu"))
            && let Ok(mtu) = mtu_str.trim().parse::<u32>()
        {
            iface.mtu = mtu;
        }
        
        // ─────────────────────────────────────────────────────────────
        // CARRIER (link detected)
        // ─────────────────────────────────────────────────────────────
        iface.carrier = self.read_sysfs_file(&iface_path.join("carrier"))
            .map(|s| s.trim() == "1")
            .ok();
        
        // ─────────────────────────────────────────────────────────────
        // VIRTUAL INTERFACE DETECTION
        // ─────────────────────────────────────────────────────────────
        // Virtual interfaces don't have a physical device
        let device_path = iface_path.join("device");
        iface.is_virtual = !device_path.exists()
            || iface.name.starts_with("lo")
            || iface.name.starts_with("veth")
            || iface.name.starts_with("br")
            || iface.name.starts_with("docker")
            || iface.name.starts_with("virbr");
        
        // ─────────────────────────────────────────────────────────────
        // DRIVER INFORMATION (only for physical interfaces)
        // ─────────────────────────────────────────────────────────────
        // PATTERN: Negated guard with if-let chain
        // Only process driver info for non-virtual interfaces
        if !iface.is_virtual {
            let driver_link = device_path.join("driver");
            // PATTERN: if-let chain with && for nested conditionals
            if let Ok(driver_path) = fs::read_link(&driver_link)
                && let Some(driver_name) = driver_path.file_name()
            {
                let driver_str = driver_name.to_string_lossy().to_string();
                iface.driver = Some(driver_str.clone());
                
                // Chain: driver version lookup
                let version_path = PathBuf::from("/sys/module").join(&driver_str).join("version");
                if let Ok(version) = self.read_sysfs_file(&version_path) {
                    iface.driver_version = Some(version.trim().to_string());
                }
            }
        }
        
        // ─────────────────────────────────────────────────────────────
        // INTERFACE TYPE
        // ─────────────────────────────────────────────────────────────
        // PATTERN: Match-like if-else chain for classification
        iface.interface_type = if iface.name == "lo" {
            NetworkInterfaceType::Loopback
        } else if iface.name.starts_with("br") || iface.name.starts_with("virbr") {
            NetworkInterfaceType::Bridge
        } else if iface.name.starts_with("veth") {
            NetworkInterfaceType::Veth
        } else if iface.name.starts_with("bond") {
            NetworkInterfaceType::Bond
        } else if iface.name.contains(".") {
            NetworkInterfaceType::Vlan
        } else if iface.name.starts_with("wl") {
            NetworkInterfaceType::Wireless
        } else if iface.name.starts_with("ib") {
            NetworkInterfaceType::Infiniband
        } else {
            NetworkInterfaceType::Ethernet
        };
    }
}
```

---

## Helper Functions

**Add this general-purpose helper at the end of the impl block:**

```rust
// =============================================================================
// GENERAL HELPER METHODS
// =============================================================================

impl LinuxSystemInfoProvider {
    /// Read a sysfs file and return contents as String.
    ///
    /// # Error Handling
    ///
    /// Returns Err if:
    /// - File doesn't exist
    /// - Permission denied
    /// - Any I/O error
    ///
    /// # Why This Helper?
    ///
    /// - Centralizes error handling
    /// - Consistent logging
    /// - Can add caching later if needed
    fn read_sysfs_file(&self, path: &Path) -> Result<String, std::io::Error> {
        fs::read_to_string(path)
    }
}
```

---

## Testing

After implementing, verify with:

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Test on real hardware (run binary)
cargo run --bin hardware_report

# Check specific detection
cargo run --bin hardware_report 2>&1 | grep -A 5 "storage"
cargo run --bin hardware_report 2>&1 | grep -A 5 "gpus"
```

---

## Rust Idioms: if-let Chaining & let-else

This implementation uses modern Rust patterns to reduce nesting:

### let-else (Early Exit Pattern)

```rust
// BEFORE: Nested match/if for required values
let size_bytes = match self.read_sysfs_file(&path) {
    Ok(content) => match parse_sysfs_size(&content) {
        Ok(size) => size,
        Err(_) => continue,
    },
    Err(_) => continue,
};

// AFTER: let-else for early exit
let Ok(content) = self.read_sysfs_file(&path) else { continue; };
let Ok(size_bytes) = parse_sysfs_size(&content) else { continue; };
```

### if-let Chaining with &&

```rust
// BEFORE: Nested if-let
if let Ok(speed_str) = read_file(&path) {
    if let Ok(speed) = speed_str.parse::<i32>() {
        if speed > 0 {
            iface.speed = Some(speed);
        }
    }
}

// AFTER: Chained if-let with &&
if let Ok(speed_str) = read_file(&path)
    && let Ok(speed) = speed_str.parse::<i32>()
    && speed > 0
{
    iface.speed = Some(speed);
}
```

### Option::or() for Null Coalescing

```rust
// BEFORE: Verbose conditional assignment
if pri.serial.is_none() {
    pri.serial = sec.serial;
}

// AFTER: Functional style
pri.serial = pri.serial.take().or(sec.serial);
```

### and_then Chains

```rust
// BEFORE: Nested if-let for Option extraction
if let Some(bus_id) = &sec_gpu.pci_bus_id {
    if let Some(pri) = primary.iter_mut().find(|g| ...) {
        // merge
    }
}

// AFTER: and_then chain
let matched = sec_gpu.pci_bus_id.as_ref().and_then(|bus_id| {
    primary.iter_mut().find(|g| g.pci_bus_id.as_ref().is_some_and(|id| id == bus_id))
});
```

---

## LeetCode Pattern Summary

| Pattern | Problems | Where Used |
|---------|----------|------------|
| **Chain of Responsibility** | - | All detection methods (sysfs → command → fallback) |
| **Merge/Combine** | LC #88, #21, #56 | `merge_storage_info`, `merge_gpu_info` |
| **Tree Traversal** | LC #102, #200 | sysfs directory walking, cache hierarchy |
| **Filtering** | LC #283, #27 | `devices.retain()` for virtual devices |
| **Hash Map Lookup** | LC #1, #49 | Vendor ID → vendor name |
| **String Parsing** | LC #8, #65 | sysfs file parsing |
| **Pattern Matching** | LC #28, #10 | lspci PCI ID extraction |
| **Two Pointers** | LC #88 | Merge operations |

---

## Implementation Checklist

Use this to track your progress:

```markdown
## Storage Detection
- [ ] Update imports
- [ ] Replace get_storage_info method
- [ ] Add detect_storage_sysfs helper
- [ ] Add detect_storage_lsblk helper  
- [ ] Add detect_storage_sysinfo helper
- [ ] Add read_nvme_sysfs_attrs helper
- [ ] Add merge_storage_info helper
- [ ] Test: cargo check

## CPU Detection
- [ ] Update get_cpu_info method
- [ ] Add detect_cpu_sysfs_frequency helper
- [ ] Add detect_cpu_sysfs_cache helper
- [ ] Add read_proc_cpuinfo helper
- [ ] Test: cargo check

## GPU Detection
- [ ] Replace get_gpu_info method
- [ ] Add detect_gpus_nvidia_smi helper
- [ ] Add detect_gpus_rocm_smi helper
- [ ] Add parse_rocm_smi_json helper
- [ ] Add detect_gpus_sysfs_drm helper
- [ ] Add detect_gpus_lspci helper
- [ ] Add merge_gpu_info helper
- [ ] Test: cargo check

## Network Detection
- [ ] Update get_network_info method
- [ ] Add enrich_network_interface_sysfs helper
- [ ] Test: cargo check

## Final
- [ ] Add read_sysfs_file general helper
- [ ] Run full test suite: cargo test
- [ ] Test on real hardware
```

Good luck with your implementation!
