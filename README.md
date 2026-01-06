# Hardware Report

Automated infrastructure discovery tool for Linux servers. Generates structured hardware inventory reports in TOML/JSON format.

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

## Features

- **CPU**: Model, sockets, cores, threads, NUMA topology, cache hierarchy
- **Memory**: Total capacity, module details, type, speed, slot mapping
- **Storage**: NVMe/SSD/HDD detection, capacity, serial numbers, SMART status
- **GPU**: NVIDIA detection via nvidia-smi, memory, PCI topology, UUIDs
- **Network**: Interface discovery, MAC/IP, speed, InfiniBand support
- **System**: BIOS, BMC/IPMI, chassis, motherboard specifications

## Quick Start

### Pre-built Release (Recommended)

```bash
# Download latest release
curl -sL https://api.github.com/repos/sfcompute/hardware_report/releases/latest \
  | grep "browser_download_url.*\.deb" | cut -d '"' -f 4 | wget -qi -

# Install and run
sudo apt install -y ./hardware-report_*_amd64.deb
sudo hardware_report
```

### Nix Build

```bash
git clone https://github.com/sfcompute/hardware_report.git && cd hardware_report
nix build && sudo ./result/bin/hardware_report
```

### Cargo Build

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get install -y build-essential pkg-config libssl-dev numactl ipmitool ethtool pciutils

# Build and run
git clone https://github.com/sfcompute/hardware_report.git && cd hardware_report
cargo build --release
sudo ./target/release/hardware_report
```

## Installation

### Option 1: Pre-built Releases

Download from [GitHub Releases](https://github.com/sfcompute/hardware_report/releases):
- `hardware-report_*_amd64.deb` - Debian/Ubuntu package
- `hardware_report-linux-x86_64-*.tar.gz` - Standalone binary

### Option 2: Nix

```bash
# One-liner: install Nix + build + run
curl -L https://install.determinate.systems/nix | sh -s -- install && \
. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && \
git clone https://github.com/sfcompute/hardware_report.git && cd hardware_report && \
nix build && sudo ./result/bin/hardware_report
```

Development shell:
```bash
nix develop
cargo build --release
```

### Option 3: Traditional Build

**Ubuntu/Debian:**
```bash
sudo apt-get update && sudo apt-get install -y \
  build-essential pkg-config libssl-dev git \
  numactl ipmitool ethtool util-linux pciutils

cargo build --release
```

**RHEL/Fedora:**
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install pkg-config openssl-devel numactl ipmitool ethtool util-linux pciutils

cargo build --release
```

## Usage

```bash
# Run with sudo for full hardware access
sudo ./target/release/hardware_report

# Output: <chassis_serial>_hardware_report.toml
```

### Sample Output

```
System Summary:
==============
CPU: AMD EPYC 7763 (2 Sockets, 64 Cores/Socket, 2 Threads/Core)
Memory: 512GB DDR4 @ 3200 MHz
Storage: 15.36 TB (4x 3.84TB NVMe)
GPUs: 8x NVIDIA H100 80GB HBM3
Network: 2x 100GbE, 4x 400Gb InfiniBand
```

### Runtime Dependencies

| Tool | Purpose |
|------|---------|
| `numactl` | NUMA topology |
| `ipmitool` | BMC/IPMI data |
| `ethtool` | Network interface details |
| `lspci` | PCI device enumeration |
| `dmidecode` | System/BIOS/memory info |
| `nvidia-smi` | GPU detection (optional) |

> **Note:** Nix builds bundle all dependencies automatically.

## Documentation

| Document | Description |
|----------|-------------|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Hexagonal architecture overview |
| [docs/API.md](docs/API.md) | Library API reference |
| [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) | Production deployment guide |

## Contributing

Pull requests welcome. For major changes, please open an issue first.

```bash
# Development
nix develop
cargo test
cargo clippy
cargo fmt
```

## License

[MIT](LICENSE)

---

**Repository:** https://github.com/sfcompute/hardware_report  
**Author:** SF Compute
