# API Reference

Hardware Report can be used as a library in your Rust projects.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core Types](#core-types)
- [Service Creation](#service-creation)
- [Examples](#examples)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hardware_report = { git = "https://github.com/sfcompute/hardware_report.git" }
```

## Quick Start

```rust
use hardware_report::create_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = create_service()?;
    let report = service.collect_hardware_info().await?;
    
    println!("CPU: {}", report.hardware.cpu.model);
    println!("Memory: {}", report.hardware.memory.total);
    println!("GPUs: {}", report.hardware.gpus.devices.len());
    
    Ok(())
}
```

## Core Types

### HardwareReport

The main report structure containing all collected hardware information.

```rust
pub struct HardwareReport {
    pub hostname: String,
    pub hardware: HardwareInfo,
    pub network: NetworkInfo,
    pub summary: Summary,
}
```

### CpuInfo

```rust
pub struct CpuInfo {
    pub model: String,
    pub cores: u32,
    pub threads: u32,
    pub sockets: u32,
    pub speed: String,
    pub vendor: String,
    pub architecture: String,
    pub frequency_mhz: u32,
    pub cache_l1d_kb: Option<u32>,
    pub cache_l2_kb: Option<u32>,
    pub cache_l3_kb: Option<u32>,
}
```

### MemoryInfo

```rust
pub struct MemoryInfo {
    pub total: String,
    pub type_: String,
    pub speed: String,
    pub modules: Vec<MemoryModule>,
}
```

### StorageDevice

```rust
pub struct StorageDevice {
    pub name: String,
    pub device_type: StorageType,  // Nvme, Ssd, Hdd
    pub size: String,
    pub size_bytes: u64,
    pub model: String,
    pub serial_number: Option<String>,
}
```

### GpuDevice

```rust
pub struct GpuDevice {
    pub index: u32,
    pub name: String,
    pub uuid: String,
    pub memory: String,
    pub memory_total_mb: u32,
    pub vendor: String,
    pub pci_id: String,
}
```

### NetworkInterface

```rust
pub struct NetworkInterface {
    pub name: String,
    pub mac: String,
    pub ip: String,
    pub speed: String,
    pub mtu: Option<u32>,
    pub link_state: Option<String>,
}
```

## Service Creation

```rust
use hardware_report::{create_service, create_service_with_config};

// Default configuration
let service = create_service()?;

// Custom configuration
let config = ServiceConfig {
    timeout_seconds: 30,
    collect_gpu: true,
    collect_network: true,
};
let service = create_service_with_config(config)?;
```

## Examples

See the `examples/` directory for complete usage examples:

- `examples/basic_usage.rs` - Simple hardware collection
- `examples/library_usage.rs` - Library integration patterns
- `examples/custom_analysis.rs` - Custom data processing
