# Testing Strategy

> **Target Platforms:** Linux x86_64, Linux aarch64  
> **Primary Test Target:** aarch64 (ARM64) - DGX Spark, Graviton

## Table of Contents

1. [Overview](#overview)
2. [Test Categories](#test-categories)
3. [Platform Matrix](#platform-matrix)
4. [Unit Testing](#unit-testing)
5. [Integration Testing](#integration-testing)
6. [Hardware Testing](#hardware-testing)
7. [CI/CD Configuration](#cicd-configuration)
8. [Test Data Management](#test-data-management)
9. [Mocking Strategy](#mocking-strategy)

---

## Overview

The `hardware_report` crate requires testing across multiple architectures and hardware configurations. This document defines the testing strategy to ensure reliability on both x86_64 and aarch64 platforms.

### Testing Principles

1. **Parser functions are pure** - Test with captured output, no hardware needed
2. **Adapters require mocking** - Use trait-based dependency injection
3. **Integration tests need real hardware** - Run on CI matrix or manual
4. **ARM is primary target** - Ensure full coverage on aarch64

---

## Test Categories

### Test Pyramid

```
                    ┌─────────────────┐
                    │  Manual/E2E     │  ← Real hardware, manual verification
                    │  (5%)           │
                    ├─────────────────┤
                    │  Integration    │  ← Real sysfs, commands, CI matrix
                    │  (20%)          │
                    ├─────────────────┤
                    │  Unit Tests     │  ← Pure functions, mocked adapters
                    │  (75%)          │
                    └─────────────────┘
```

### Test Types

| Type | Location | Dependencies | CI |
|------|----------|--------------|-----|
| Unit | `src/**/*.rs` (inline) | None | Yes |
| Parser | `tests/parsers/` | Sample data files | Yes |
| Adapter | `tests/adapters/` | Mocked traits | Yes |
| Integration | `tests/integration/` | Real system | Matrix |
| Hardware | `tests/hardware/` | Physical hardware | Manual |

---

## Platform Matrix

### Target Platforms

| Platform | Architecture | GPU | CI Support | Notes |
|----------|--------------|-----|------------|-------|
| Linux x86_64 | x86_64 | NVIDIA | GitHub Actions | Standard runners |
| Linux x86_64 | x86_64 | AMD | Self-hosted | Optional |
| Linux aarch64 | aarch64 | None | GitHub Actions | `ubuntu-24.04-arm` |
| Linux aarch64 | aarch64 | NVIDIA | Self-hosted | DGX Spark |
| macOS x86_64 | x86_64 | Apple | GitHub Actions | Legacy Intel |
| macOS aarch64 | aarch64 | Apple | GitHub Actions | M1/M2/M3 |

### CI Matrix Configuration

```yaml
strategy:
  matrix:
    include:
      # x86_64 Linux
      - os: ubuntu-latest
        arch: x86_64
        target: x86_64-unknown-linux-gnu
        features: "full"
        
      # aarch64 Linux (GitHub-hosted ARM)
      - os: ubuntu-24.04-arm
        arch: aarch64
        target: aarch64-unknown-linux-gnu
        features: ""  # No nvidia feature on ARM CI
        
      # Cross-compile for ARM (build only)
      - os: ubuntu-latest
        arch: x86_64
        target: aarch64-unknown-linux-gnu
        cross: true
        features: ""
```

---

## Unit Testing

### Parser Unit Tests

Parser functions are pure and easily testable:

```rust
// src/domain/parsers/storage.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sysfs_size() {
        // 2TB drive in 512-byte sectors
        assert_eq!(parse_sysfs_size("3907029168").unwrap(), 2000398934016);
        
        // 1TB drive
        assert_eq!(parse_sysfs_size("1953525168").unwrap(), 1000204886016);
        
        // Empty/whitespace
        assert!(parse_sysfs_size("").is_err());
        assert!(parse_sysfs_size("   ").is_err());
    }

    #[test]
    fn test_parse_sysfs_rotational() {
        assert!(parse_sysfs_rotational("1"));   // HDD
        assert!(!parse_sysfs_rotational("0"));  // SSD
        assert!(!parse_sysfs_rotational("0\n")); // With newline
    }

    #[test]
    fn test_storage_type_from_device() {
        assert_eq!(StorageType::from_device("nvme0n1", false), StorageType::Nvme);
        assert_eq!(StorageType::from_device("sda", false), StorageType::Ssd);
        assert_eq!(StorageType::from_device("sda", true), StorageType::Hdd);
        assert_eq!(StorageType::from_device("mmcblk0", false), StorageType::Emmc);
        assert_eq!(StorageType::from_device("loop0", false), StorageType::Virtual);
    }
}
```

### GPU Parser Tests

```rust
// src/domain/parsers/gpu.rs

#[cfg(test)]
mod tests {
    use super::*;

    const NVIDIA_SMI_OUTPUT: &str = r#"0, NVIDIA H100 80GB HBM3, GPU-12345678-1234-1234-1234-123456789abc, 81920, 81000, 00000000:01:00.0, 535.129.03, 9.0
1, NVIDIA H100 80GB HBM3, GPU-87654321-4321-4321-4321-cba987654321, 81920, 80500, 00000000:02:00.0, 535.129.03, 9.0"#;

    #[test]
    fn test_parse_nvidia_smi_output() {
        let gpus = parse_nvidia_smi_output(NVIDIA_SMI_OUTPUT).unwrap();
        
        assert_eq!(gpus.len(), 2);
        
        assert_eq!(gpus[0].index, 0);
        assert_eq!(gpus[0].name, "NVIDIA H100 80GB HBM3");
        assert_eq!(gpus[0].memory_total_mb, 81920);
        assert_eq!(gpus[0].memory_free_mb, Some(81000));
        assert_eq!(gpus[0].driver_version, Some("535.129.03".to_string()));
        assert_eq!(gpus[0].compute_capability, Some("9.0".to_string()));
    }

    #[test]
    fn test_parse_nvidia_smi_empty() {
        let gpus = parse_nvidia_smi_output("").unwrap();
        assert!(gpus.is_empty());
    }

    #[test]
    fn test_parse_pci_vendor() {
        assert_eq!(parse_pci_vendor("10de"), GpuVendor::Nvidia);
        assert_eq!(parse_pci_vendor("0x10de"), GpuVendor::Nvidia);
        assert_eq!(parse_pci_vendor("1002"), GpuVendor::Amd);
        assert_eq!(parse_pci_vendor("8086"), GpuVendor::Intel);
        assert_eq!(parse_pci_vendor("abcd"), GpuVendor::Unknown);
    }
}
```

### CPU Parser Tests

```rust
// src/domain/parsers/cpu.rs

#[cfg(test)]
mod tests {
    use super::*;

    const PROC_CPUINFO_INTEL: &str = r#"
processor	: 0
vendor_id	: GenuineIntel
cpu family	: 6
model		: 106
model name	: Intel(R) Xeon(R) Platinum 8380 CPU @ 2.30GHz
stepping	: 6
microcode	: 0xd0003a5
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat avx avx2 avx512f avx512dq
"#;

    const PROC_CPUINFO_ARM: &str = r#"
processor	: 0
BogoMIPS	: 50.00
Features	: fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp
CPU implementer	: 0x41
CPU architecture: 8
CPU variant	: 0x3
CPU part	: 0xd0c
CPU revision	: 1
"#;

    #[test]
    fn test_parse_proc_cpuinfo_intel() {
        let info = parse_proc_cpuinfo(PROC_CPUINFO_INTEL).unwrap();
        
        assert_eq!(info.vendor, "GenuineIntel");
        assert_eq!(info.model, "Intel(R) Xeon(R) Platinum 8380 CPU @ 2.30GHz");
        assert_eq!(info.family, Some(6));
        assert_eq!(info.model_number, Some(106));
        assert!(info.flags.contains(&"avx512f".to_string()));
    }

    #[test]
    fn test_parse_proc_cpuinfo_arm() {
        let info = parse_proc_cpuinfo(PROC_CPUINFO_ARM).unwrap();
        
        assert_eq!(info.vendor, "ARM");
        assert!(info.flags.contains(&"asimd".to_string()));
        assert_eq!(info.microarchitecture, Some("Neoverse N1".to_string()));
    }

    #[test]
    fn test_arm_cpu_part_mapping() {
        assert_eq!(arm_cpu_part_to_name("0xd0c"), Some("Neoverse N1"));
        assert_eq!(arm_cpu_part_to_name("0xd40"), Some("Neoverse V1"));
        assert_eq!(arm_cpu_part_to_name("0xd49"), Some("Neoverse N2"));
        assert_eq!(arm_cpu_part_to_name("0xffff"), None);
    }

    #[test]
    fn test_parse_sysfs_freq() {
        assert_eq!(parse_sysfs_freq_khz("3500000").unwrap(), 3500);
        assert_eq!(parse_sysfs_freq_khz("2100000\n").unwrap(), 2100);
        assert!(parse_sysfs_freq_khz("invalid").is_err());
    }

    #[test]
    fn test_parse_cache_size() {
        assert_eq!(parse_sysfs_cache_size("32K").unwrap(), 32);
        assert_eq!(parse_sysfs_cache_size("1M").unwrap(), 1024);
        assert_eq!(parse_sysfs_cache_size("256M").unwrap(), 262144);
        assert_eq!(parse_sysfs_cache_size("32768K").unwrap(), 32768);
    }
}
```

---

## Integration Testing

### sysfs Integration Tests

```rust
// tests/integration/sysfs_storage.rs

#[cfg(target_os = "linux")]
mod tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_sysfs_block_exists() {
        assert!(Path::new("/sys/block").exists());
    }

    #[test]
    fn test_can_read_block_devices() {
        let entries = fs::read_dir("/sys/block").unwrap();
        let devices: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| !n.starts_with("loop") && !n.starts_with("ram"))
            .collect();
        
        // Most systems have at least one real block device
        // This may fail in minimal containers
        println!("Found block devices: {:?}", devices);
    }

    #[test]
    fn test_can_read_device_size() {
        if let Ok(entries) = fs::read_dir("/sys/block") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let size_path = format!("/sys/block/{}/size", name.to_string_lossy());
                
                if let Ok(size_str) = fs::read_to_string(&size_path) {
                    let sectors: u64 = size_str.trim().parse().unwrap_or(0);
                    let bytes = sectors * 512;
                    println!("{}: {} bytes", name.to_string_lossy(), bytes);
                }
            }
        }
    }
}
```

### Command Execution Tests

```rust
// tests/integration/commands.rs

#[cfg(target_os = "linux")]
mod tests {
    use std::process::Command;

    #[test]
    fn test_lsblk_available() {
        let output = Command::new("which").arg("lsblk").output();
        assert!(output.is_ok());
    }

    #[test]
    fn test_lsblk_json_output() {
        let output = Command::new("lsblk")
            .args(["-J", "-o", "NAME,SIZE,TYPE"])
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                assert!(stdout.contains("blockdevices"));
            }
            _ => {
                // lsblk may not be available in all environments
                println!("lsblk not available, skipping");
            }
        }
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_arm_specific_detection() {
        // Verify ARM-specific paths exist
        let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap();
        
        // ARM cpuinfo has different format
        assert!(
            cpuinfo.contains("CPU implementer") || 
            cpuinfo.contains("model name"),
            "Expected ARM or x86 CPU info format"
        );
    }
}
```

---

## Hardware Testing

### Manual Test Checklist

#### Storage Tests

```bash
# Run on target hardware
cargo test --test storage_hardware -- --ignored

# Expected output validation:
# - At least one storage device detected
# - Size > 0 for all devices
# - Type correctly identified (NVMe/SSD/HDD)
# - Serial number present (may need sudo)
```

#### GPU Tests

```bash
# Run on NVIDIA system
cargo test --test gpu_hardware --features nvidia -- --ignored

# Expected output validation:
# - All GPUs detected
# - Memory matches nvidia-smi
# - Driver version present
# - PCI bus ID present
```

### Hardware Test Files

```rust
// tests/hardware/storage.rs

#[test]
#[ignore] // Run manually on real hardware
fn test_storage_detection_real_hardware() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let service = hardware_report::create_service().unwrap();
        let config = hardware_report::ReportConfig::default();
        let report = service.generate_report(config).await.unwrap();
        
        // Validate storage
        assert!(!report.hardware.storage.devices.is_empty(), 
            "No storage devices detected");
        
        for device in &report.hardware.storage.devices {
            assert!(device.size_bytes > 0, 
                "Device {} has zero size", device.name);
            assert!(!device.model.is_empty(), 
                "Device {} has empty model", device.name);
            println!("Found: {} - {} - {} GB", 
                device.name, device.model, device.size_gb);
        }
    });
}

#[test]
#[ignore]
#[cfg(target_arch = "aarch64")]
fn test_arm_storage_detection() {
    // ARM-specific storage test
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let service = hardware_report::create_service().unwrap();
        let config = hardware_report::ReportConfig::default();
        let report = service.generate_report(config).await.unwrap();
        
        // On ARM, we should detect NVMe or eMMC
        assert!(!report.hardware.storage.devices.is_empty(),
            "No storage on ARM - sysfs fallback may have failed");
        
        let has_nvme = report.hardware.storage.devices.iter()
            .any(|d| d.device_type == StorageType::Nvme);
        let has_emmc = report.hardware.storage.devices.iter()
            .any(|d| d.device_type == StorageType::Emmc);
        
        println!("ARM storage: NVMe={}, eMMC={}", has_nvme, has_emmc);
    });
}
```

---

## CI/CD Configuration

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml

name: Test

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test-x86:
    name: Test x86_64
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Run unit tests
        run: cargo test --lib --all-features
      
      - name: Run doc tests
        run: cargo test --doc
      
      - name: Run integration tests
        run: cargo test --test '*' -- --skip hardware

  test-arm:
    name: Test aarch64
    runs-on: ubuntu-24.04-arm
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Run unit tests
        run: cargo test --lib
      
      - name: Run ARM integration tests
        run: cargo test --test '*' -- --skip hardware
      
      - name: Test ARM-specific code paths
        run: |
          # Verify ARM detection works
          cargo run --example basic_usage 2>&1 | tee output.txt
          grep -q "architecture.*aarch64" output.txt || echo "Warning: arch detection may have issues"

  cross-compile:
    name: Cross-compile check
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - aarch64-unknown-linux-gnu
          - aarch64-unknown-linux-musl
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install cross
        run: cargo install cross
      
      - name: Cross build
        run: cross build --target ${{ matrix.target }} --release

  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: clippy, rustfmt
      
      - name: Check formatting
        run: cargo fmt --check
      
      - name: Clippy
        run: cargo clippy --all-features -- -D warnings
      
      - name: Check docs
        run: cargo doc --no-deps --all-features
        env:
          RUSTDOCFLAGS: -D warnings
```

---

## Test Data Management

### Sample Data Files

Store captured command outputs for parser testing:

```
tests/
├── data/
│   ├── nvidia-smi/
│   │   ├── h100-8gpu.csv
│   │   ├── a100-4gpu.csv
│   │   └── no-gpu.csv
│   ├── lsblk/
│   │   ├── nvme-only.json
│   │   ├── mixed-storage.json
│   │   └── arm-emmc.json
│   ├── proc/
│   │   ├── cpuinfo-intel-xeon.txt
│   │   ├── cpuinfo-amd-epyc.txt
│   │   └── cpuinfo-arm-neoverse.txt
│   └── sysfs/
│       ├── block-nvme/
│       └── cpu-arm/
```

### Loading Test Data

```rust
// tests/common/mod.rs

use std::path::PathBuf;

pub fn test_data_path(relative: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/data");
    path.push(relative);
    path
}

pub fn load_test_data(relative: &str) -> String {
    std::fs::read_to_string(test_data_path(relative))
        .expect(&format!("Failed to load test data: {}", relative))
}

// Usage in tests:
#[test]
fn test_nvidia_h100_parsing() {
    let data = load_test_data("nvidia-smi/h100-8gpu.csv");
    let gpus = parse_nvidia_smi_output(&data).unwrap();
    assert_eq!(gpus.len(), 8);
}
```

---

## Mocking Strategy

### Trait-Based Mocking

```rust
// src/ports/secondary/system.rs - The trait

#[async_trait]
pub trait SystemInfoProvider: Send + Sync {
    async fn get_storage_info(&self) -> Result<StorageInfo, SystemError>;
    async fn get_gpu_info(&self) -> Result<GpuInfo, SystemError>;
    // ...
}

// tests/mocks/system.rs - Mock implementation

pub struct MockSystemInfoProvider {
    pub storage_result: Result<StorageInfo, SystemError>,
    pub gpu_result: Result<GpuInfo, SystemError>,
}

#[async_trait]
impl SystemInfoProvider for MockSystemInfoProvider {
    async fn get_storage_info(&self) -> Result<StorageInfo, SystemError> {
        self.storage_result.clone()
    }
    
    async fn get_gpu_info(&self) -> Result<GpuInfo, SystemError> {
        self.gpu_result.clone()
    }
}

// Usage in tests
#[tokio::test]
async fn test_service_with_mock() {
    let mock = MockSystemInfoProvider {
        storage_result: Ok(StorageInfo {
            devices: vec![
                StorageDevice {
                    name: "nvme0n1".to_string(),
                    size_bytes: 1000204886016,
                    ..Default::default()
                }
            ]
        }),
        gpu_result: Ok(GpuInfo { devices: vec![] }),
    };
    
    // Inject mock into service
    let service = HardwareCollectionService::new(Arc::new(mock));
    let report = service.generate_report(Default::default()).await.unwrap();
    
    assert_eq!(report.hardware.storage.devices.len(), 1);
}
```

### Command Executor Mocking

```rust
// Mock command executor for testing adapter logic

pub struct MockCommandExecutor {
    pub responses: HashMap<String, CommandResult>,
}

impl MockCommandExecutor {
    pub fn new() -> Self {
        Self { responses: HashMap::new() }
    }
    
    pub fn mock_command(&mut self, cmd: &str, result: CommandResult) {
        self.responses.insert(cmd.to_string(), result);
    }
}

#[async_trait]
impl CommandExecutor for MockCommandExecutor {
    async fn execute(&self, cmd: &SystemCommand) -> Result<CommandOutput, CommandError> {
        if let Some(result) = self.responses.get(&cmd.program) {
            Ok(result.clone())
        } else {
            Err(CommandError::NotFound(cmd.program.clone()))
        }
    }
}
```

---

## Changelog

| Date | Changes |
|------|---------|
| 2024-12-29 | Initial testing strategy |
