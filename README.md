# Hardware Report

A system information collection tool that generates comprehensive hardware inventory reports in TOML format for Linux servers.

## Overview

Hardware Report collects detailed system configuration data and outputs it as `<chassis_serial>_hardware_report.toml`, enabling automated infrastructure management and standardization across heterogeneous hardware deployments.

## Installation

### Using Nix (Recommended)

```bash
# Build and run directly
git clone https://github.com/sfcompute/hardware_report.git
cd hardware_report
nix build
sudo ./result/bin/hardware_report
```

### As Debian Package

```bash
# Build .deb package
nix build .#deb
sudo apt install -y ./result/hardware-report_0.1.7_amd64.deb
```

### Traditional Build

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt install -y pkg-config libssl-dev

# Build with cargo
cargo build --release
sudo ./target/release/hardware_report
```

## Collected Information

- **CPU**: Model, topology, NUMA configuration, socket/core/thread counts
- **Memory**: Total capacity, type, speed, module details
- **Storage**: Device inventory, capacity, models
- **Network**: Interface details, speeds, Infiniband configuration
- **GPU**: NVIDIA GPU detection and specifications
- **System**: BIOS, chassis, motherboard, BMC information
- **Filesystems**: Mount points and utilization

## Output Format

The tool generates a structured TOML file containing all collected hardware information. See the [sample output](#sample-output) section for a complete example.

## Requirements

### Build Dependencies
- Rust toolchain (1.70+)
- pkg-config
- OpenSSL development libraries

### Runtime Dependencies
When built with Nix, all dependencies are included. For other build methods, ensure these tools are available:
- `numactl` - NUMA topology
- `ipmitool` - BMC information
- `ethtool` - Network interface details
- `lscpu` - CPU information
- `nvidia-smi` - GPU information (if applicable)

## Development

```bash
# Enter development environment with all dependencies
nix develop

# Run tests
cargo test

# Build
cargo build --release
```

## CI/CD

The project uses GitHub Actions for automated testing and releases. Tagged commits automatically generate Linux binaries and tarballs.

## Sample Output

<details>
<summary>Click to expand sample TOML output</summary>

```toml
hostname = "gpu-node-01"
bmc_ip = "10.0.0.100"
bmc_mac = "ac:1f:6b:00:00:00"

[summary]
total_memory = "512GB"
memory_config = "DDR4 @ 3200 MT/s"
total_storage = "15.36 TB"
total_gpus = 8
total_nics = 4
cpu_summary = "AMD EPYC 7763 (2 Sockets, 64 Cores/Socket, 2 Threads/Core)"

[summary.bios]
vendor = "American Megatrends Inc."
version = "2.4.3"
release_date = "01/15/2024"

[summary.chassis]
manufacturer = "Supermicro"
type_ = "Other"
serial = "S123456789"

[hardware.cpu]
model = "AMD EPYC 7763"
cores = 64
threads = 2
sockets = 2

[[hardware.gpus.devices]]
index = 0
name = "NVIDIA H100 80GB HBM3"
uuid = "GPU-12345678-1234-1234-1234-123456789012"
memory = "81559 MiB"
pci_id = "10de:2330"

# ... additional hardware details ...
```

</details>

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome. Please open an issue to discuss significant changes before submitting a PR.

## Author

Kenny Sheridan, Supercomputing Engineer