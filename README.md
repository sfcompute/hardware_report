[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org) [![Nix](https://img.shields.io/badge/nix-%23000000.svg?style=for-the-badge&logo=nixos&logoColor=white)](https://nixos.org) [![Linux](https://img.shields.io/badge/linux-%23000000.svg?style=for-the-badge&logo=linux&logoColor=white)](https://kernel.org)

# Hardware Report

**Automated infrastructure discovery tool for Linux servers built with hexagonal architecture.**

Generates structured hardware inventory reports in TOML/JSON format. Designed for CMDB population, infrastructure auditing, and bare-metal server management at scale.

## Table of Contents

- [What Does This Do?](#what-does-this-do)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

## What Does This Do?

- **CPU Discovery** - Model, sockets, cores, threads, NUMA topology, cache hierarchy
- **Memory Detection** - Total capacity, module details, DDR type, speed, slot mapping
- **Storage Enumeration** - NVMe/SSD/HDD detection, capacity, serial numbers, SMART status
- **GPU Detection** - NVIDIA via nvidia-smi, memory, PCI topology, UUIDs
- **Network Interfaces** - MAC/IP, speed (1G-400G+), InfiniBand, driver info
- **System Information** - BIOS, BMC/IPMI, chassis serial, motherboard specs

## Quick Start

**Recommended: Nix build** (automatically handles all dependencies)

```bash
# Install Nix (if not already installed)
curl -L https://install.determinate.systems/nix | sh -s -- install
. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh

# Clone, build, and run
git clone https://github.com/sfcompute/hardware_report.git
cd hardware_report
nix build
sudo ./result/bin/hardware_report
```

Output: `<chassis_serial>_hardware_report.toml`

**Other installation methods:** See [docs/INSTALLATION.md](docs/INSTALLATION.md)

## Documentation

| Document | Description |
|----------|-------------|
| [docs/INSTALLATION.md](docs/INSTALLATION.md) | All installation methods (Nix, Cargo, pre-built releases) |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Hexagonal architecture overview |
| [docs/API.md](docs/API.md) | Library API reference |

## Contributing

```bash
nix develop
cargo test && cargo clippy && cargo fmt
```

## License

[MIT](LICENSE)

---

**Built for infrastructure management at scale** | [Issues](https://github.com/sfcompute/hardware_report/issues) | [Releases](https://github.com/sfcompute/hardware_report/releases)
