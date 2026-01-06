[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org) [![Nix](https://img.shields.io/badge/nix-%23000000.svg?style=for-the-badge&logo=nixos&logoColor=white)](https://nixos.org) [![Linux](https://img.shields.io/badge/linux-%23000000.svg?style=for-the-badge&logo=linux&logoColor=white)](https://kernel.org)

# Hardware Report

**Automated infrastructure discovery tool for Linux servers built with hexagonal architecture.**

Generates structured hardware inventory reports in TOML/JSON format. Designed for CMDB population, infrastructure auditing, and bare-metal server management at scale.

## Table of Contents

- [What Does This Do?](#what-does-this-do)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Architecture Overview](#architecture-overview)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

## What Does This Do?

- **CPU Discovery** - Model, sockets, cores, threads, NUMA topology, cache hierarchy, frequency ranges
- **Memory Detection** - Total capacity, module details, DDR type, speed, slot mapping
- **Storage Enumeration** - NVMe/SSD/HDD detection, capacity, serial numbers, firmware, SMART status
- **GPU Detection** - NVIDIA via nvidia-smi, memory, PCI topology, UUIDs, driver versions
- **Network Interfaces** - MAC/IP, speed (1G-400G+), InfiniBand, driver info, link state
- **System Information** - BIOS, BMC/IPMI, chassis serial, motherboard specs

## Quick Start

Get up and running in minutes:

```bash
# 1. Clone and build
git clone https://github.com/sfcompute/hardware_report.git
cd hardware_report
nix build

# 2. Run hardware discovery
sudo ./result/bin/hardware_report

# Output: <chassis_serial>_hardware_report.toml
```

**Need more details?** See our detailed guides:
- **[Installation](#installation)** - Complete setup instructions for all environments
- **[Architecture Overview](#architecture-overview)** - Hexagonal architecture and design

## Installation

### Option 1: Pre-built Releases (Recommended)

Download from [GitHub Releases](https://github.com/sfcompute/hardware_report/releases):

```bash
# Debian/Ubuntu package
curl -sL https://api.github.com/repos/sfcompute/hardware_report/releases/latest \
  | grep "browser_download_url.*\.deb" | cut -d '"' -f 4 | wget -qi -
sudo apt install -y ./hardware-report_*_amd64.deb
sudo hardware_report
```

### Option 2: Nix Build

```bash
# One-liner: install Nix + build + run
curl -L https://install.determinate.systems/nix | sh -s -- install && \
. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && \
git clone https://github.com/sfcompute/hardware_report.git && cd hardware_report && \
nix build && sudo ./result/bin/hardware_report
```

**Development shell:**
```bash
nix develop
cargo build --release
```

### Option 3: Traditional Cargo Build

**Ubuntu/Debian:**
```bash
sudo apt-get update && sudo apt-get install -y \
  build-essential pkg-config libssl-dev git \
  numactl ipmitool ethtool util-linux pciutils

git clone https://github.com/sfcompute/hardware_report.git && cd hardware_report
cargo build --release
sudo ./target/release/hardware_report
```

**RHEL/Fedora:**
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install pkg-config openssl-devel numactl ipmitool ethtool util-linux pciutils

cargo build --release
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

## Architecture Overview

Built with **hexagonal (ports & adapters) architecture** for clean separation of concerns:

```
                    ┌──────────────────────────────────────┐
                    │         Core Domain (Pure)           │
                    │                                      │
  Primary Ports     │  ┌────────────────────────────┐     │    Secondary Ports
  (Inbound)         │  │    Domain Services         │     │    (Outbound)
                    │  │  • HardwareCollectionSvc   │     │
 ┌─────────────┐    │  │  • ReportGenerationSvc     │     │    ┌──────────────────┐
 │   CLI       │───►│  └────────────────────────────┘     │───►│ System Adapters  │
 │             │    │                                      │    │ • LinuxProvider  │
 │ hardware_   │    │  ┌────────────────────────────┐     │    │ • MacOSProvider  │
 │ report      │    │  │    Domain Entities         │     │    └──────────────────┘
 └─────────────┘    │  │  • CpuInfo, MemoryInfo     │     │
                    │  │  • StorageInfo, GpuInfo    │     │    ┌──────────────────┐
 ┌─────────────┐    │  │  • NetworkInfo, SystemInfo │     │───►│ Command Executor │
 │  Library    │───►│  └────────────────────────────┘     │    │ • UnixExecutor   │
 │             │    │                                      │    └──────────────────┘
 │ create_     │    │  ┌────────────────────────────┐     │
 │ service()   │    │  │    Pure Parsers            │     │    ┌──────────────────┐
 └─────────────┘    │  │  • CPU, Memory, Storage    │     │───►│ Publishers       │
                    │  │  • GPU, Network, System    │     │    │ • FilePublisher  │
                    │  └────────────────────────────┘     │    │ • HttpPublisher  │
                    └──────────────────────────────────────┘    └──────────────────┘
```

**Why Hexagonal Architecture?**
- **Testable** - Mock any external dependency for thorough testing
- **Flexible** - Swap system providers or publishers independently
- **Maintainable** - Clear boundaries between business logic and infrastructure
- **Platform Independent** - Core domain stays pure, adapters handle OS specifics

### Sample Output

```
System Summary:
==============
CPU: AMD EPYC 7763 (2 Sockets, 64 Cores/Socket, 2 Threads/Core)
Memory: 512GB DDR4 @ 3200 MHz
Storage: 15.36 TB (4x 3.84TB NVMe)
GPUs: 8x NVIDIA H100 80GB HBM3
Network: 2x 100GbE, 4x 400Gb InfiniBand
BIOS: AMI 2.4.3 (01/15/2024)
Chassis: SuperMicro (S/N: S454857X9822867)
```

## Documentation

| Document | Description |
|----------|-------------|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Hexagonal architecture deep dive |
| [docs/API.md](docs/API.md) | Library API reference |
| [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) | Production deployment guide |

## Contributing

Found a bug or want to add something? We welcome contributions!

**Quick Development Workflow:**
```bash
# 1. Fork and clone
git clone https://github.com/your-username/hardware_report.git

# 2. Set up development environment
nix develop  # or follow traditional Rust setup

# 3. Make changes and test
cargo test && cargo clippy && cargo fmt

# 4. Submit PR with descriptive commit messages
```

## License

[MIT](LICENSE)

---

**Built for infrastructure management at scale** | [Issues](https://github.com/sfcompute/hardware_report/issues) | [Releases](https://github.com/sfcompute/hardware_report/releases)
