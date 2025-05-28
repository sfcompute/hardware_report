# Hardware Report: Automated Infrastructure Discovery Tool

## Executive Summary

**Hardware Report** is a Rust-based utility that automatically discovers, catalogs, and reports detailed hardware information from Linux servers. The tool generates standardized TOML reports that enable consistent infrastructure management across heterogeneous bare-metal hardware environments.

### Business Value
- **Infrastructure Standardization**: Uniform hardware inventory across diverse server configurations
- **Operational Efficiency**: Automated discovery eliminates manual hardware auditing
- **Scalability**: Generates unique reports per server using chassis serial numbers
- **Cost Management**: Enables better capacity planning and resource allocation
- **Compliance**: Provides detailed audit trails for hardware configurations

### Target Use Cases
- GPU compute clusters and AI/ML infrastructure
- Heterogeneous bare-metal server environments
- Data center inventory management
- Infrastructure compliance and auditing
- Capacity planning and resource optimization

---

## Technical Overview

### Core Capabilities
The tool provides comprehensive system discovery including:

**CPU Architecture**
- Model information and socket configuration
- Core, thread, and NUMA topology mapping
- Per-socket core counts and threading details

**Memory Subsystem**
- Total capacity and module-level details
- Memory type, speed, and slot mapping
- Individual DIMM information

**Storage Infrastructure**
- Device enumeration with capacity and models
- Filesystem information and mount points
- Storage type identification (NVMe, SSD, HDD)

**GPU Resources**
- NVIDIA GPU detection and configuration
- Memory capacity and PCI topology
- Device UUID tracking

**Network Infrastructure**
- Interface discovery with speed capabilities
- MAC address and IP configuration
- InfiniBand and high-speed networking support

**System Information**
- BIOS and firmware details
- BMC (Baseboard Management Controller) data
- Chassis and motherboard specifications
- NUMA topology with device affinity mapping

### Output Format
Reports are generated as structured TOML files named `<chassis_serial>_hardware_report.toml`, ensuring unique identification and preventing data collisions in large deployments.

---

## Quick Start Guide

### Option 1: Nix Build (Recommended)

**One-Line Installation with Nix**
```bash
# Install Nix, build, and run in one command:
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install && \
. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && \
git clone https://github.com/sfcompute/hardware_report.git && \
cd hardware_report && \
nix build && \
echo "Build complete! Run with: sudo ./result/bin/hardware_report" && \
sudo ./result/bin/hardware_report
```

**Production Deployment (Debian Package with Nix)**
```bash
# Build and install system-wide package:
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install && \
. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && \
git clone https://github.com/sfcompute/hardware_report.git && \
cd hardware_report && \
nix build .#deb && \
sudo apt --fix-broken install -y && \
sudo apt remove -y hardware-report 2>/dev/null || true && \
sudo apt install -y ./result/hardware-report_0.1.7_amd64.deb && \
sudo hardware_report
```

### Option 2: Traditional Build (No Nix Required)

**Ubuntu/Debian Systems**
```bash
# 1. Install Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Install build dependencies
sudo apt-get update && sudo apt-get install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  git

# 3. Install runtime dependencies
sudo apt-get install -y \
  numactl \
  ipmitool \
  ethtool \
  util-linux \
  pciutils

# 4. Clone and build
git clone https://github.com/sfcompute/hardware_report.git
cd hardware_report
cargo build --release

# 5. Run the tool
sudo ./target/release/hardware_report
```

**RHEL/CentOS/Fedora Systems**
```bash
# 1. Install Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Install build dependencies
# For RHEL/CentOS 8+
sudo dnf groupinstall "Development Tools"
sudo dnf install pkg-config openssl-devel git

# For older RHEL/CentOS 7
# sudo yum groupinstall "Development Tools"
# sudo yum install pkg-config openssl-devel git

# 3. Install runtime dependencies
sudo dnf install numactl ipmitool ethtool util-linux pciutils
# For CentOS 7: sudo yum install numactl ipmitool ethtool util-linux pciutils

# 4. Clone and build
git clone https://github.com/sfcompute/hardware_report.git
cd hardware_report
cargo build --release

# 5. Run the tool
sudo ./target/release/hardware_report
```

**One-Liner for Ubuntu/Debian (Traditional Build)**
```bash
# Complete setup and build without Nix:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && \
source ~/.cargo/env && \
sudo apt-get update && \
sudo apt-get install -y build-essential pkg-config libssl-dev git numactl ipmitool ethtool util-linux pciutils && \
git clone https://github.com/sfcompute/hardware_report.git && \
cd hardware_report && \
cargo build --release && \
echo "Build complete! Run with: sudo ./target/release/hardware_report" && \
sudo ./target/release/hardware_report
```

### Option 3: Pre-built Releases (Recommended for Quick Setup)

Instead of building from source, you can download pre-built binaries and Debian packages from our [GitHub Releases](https://github.com/sfcompute/hardware_report/releases) page.

**Available Release Artifacts:**
- `hardware_report-linux-x86_64-{version}.tar.gz` - Pre-compiled Linux binary
- `hardware-report_{version}_amd64.deb` - Debian package with automatic dependency management
- SHA256 checksums for all artifacts

**Option 3a: Pre-built Binary**
```bash
# Download the latest release
curl -s https://api.github.com/repos/sfcompute/hardware_report/releases/latest \
  | grep "browser_download_url.*tar.gz" \
  | cut -d '"' -f 4 \
  | wget -i -

# Download and verify checksum
curl -s https://api.github.com/repos/sfcompute/hardware_report/releases/latest \
  | grep "browser_download_url.*tar.gz.sha256" \
  | cut -d '"' -f 4 \
  | wget -i -

sha256sum -c hardware_report-linux-x86_64-*.tar.gz.sha256

# Install runtime dependencies
sudo apt-get update && sudo apt-get install -y numactl ipmitool ethtool util-linux pciutils

# Extract and run
tar xzf hardware_report-linux-x86_64-*.tar.gz
chmod +x hardware_report-linux-x86_64
sudo ./hardware_report-linux-x86_64
```

**Option 3b: Debian Package (Ubuntu/Debian)**
```bash
# Download the latest Debian package
curl -s https://api.github.com/repos/sfcompute/hardware_report/releases/latest \
  | grep "browser_download_url.*\.deb" \
  | cut -d '"' -f 4 \
  | wget -i -

# Install with automatic dependency resolution
sudo apt update
sudo apt install -y ./hardware-report_*_amd64.deb

# Run the tool
sudo hardware_report
```

### Advanced Build Methods

**For Teams with Existing Nix Infrastructure**
```bash
git clone https://github.com/sfcompute/hardware_report.git && \
cd hardware_report && \
nix build && \
sudo ./result/bin/hardware_report
```

**Development Environment Setup (Nix)**
```bash
# Enter development shell with all dependencies
nix develop

# Build with cargo
cargo build --release

# Execute binary
sudo ./target/release/hardware_report
```

**Development with direnv Integration**
```bash
# Automatic environment loading
direnv allow
cargo build --release
```

---

## Build Systems & CI/CD

### Build System Comparison

| Method | Pros | Cons | Best For |
|--------|------|------|----------|
| **Nix** | Self-contained, reproducible, automatic dependencies | Learning curve, additional tool | Production, reproducible builds |
| **Cargo** | Native Rust, familiar to developers, fast | Manual dependency management | Development, existing Rust workflows |
| **Docker** | Cross-platform, isolated builds | Requires Docker setup | CI/CD, cross-compilation |
| **Pre-built** | No build required, instant setup | Less flexibility, trust requirements | Quick evaluation, production deployment |

### Nix Build System (Recommended for Production)
The project leverages Nix for reproducible builds that automatically handle all dependencies:

**Benefits of Nix Build:**
- **Dependency Management**: Automatically includes `numactl`, `ipmitool`, `ethtool`, `lscpu`, `lspci`
- **Reproducibility**: Consistent builds across different environments
- **Self-Contained**: No system dependency installation required
- **Cross-Platform**: Supports Linux and macOS targets

### Traditional Cargo Build System
For teams preferring standard Rust tooling:

**System Prerequisites by Distribution:**

**Ubuntu/Debian:**
```bash
sudo apt-get update && sudo apt-get install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  numactl \
  ipmitool \
  ethtool \
  util-linux \
  pciutils
```

**RHEL/CentOS/Fedora:**
```bash
# Build dependencies
sudo dnf groupinstall "Development Tools"
sudo dnf install pkg-config openssl-devel

# Runtime dependencies
sudo dnf install numactl ipmitool ethtool util-linux pciutils
```

**Alpine Linux:**
```bash
# Build dependencies
apk add --no-cache \
  build-base \
  pkgconfig \
  openssl-dev \
  git

# Runtime dependencies
apk add --no-cache \
  numactl \
  ipmitool \
  ethtool \
  util-linux \
  pciutils
```

### Cross-Platform Builds with Docker
```bash
# Ensure Docker is running
docker ps

# Build Linux binary
make linux

# Build for all supported platforms
make all
```

### Static Binary Builds
For maximum portability, create static binaries:

```bash
# Install musl target for static linking
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --target x86_64-unknown-linux-musl --release

# The static binary requires no runtime dependencies except system tools
```

### Automated Release Pipeline
The project includes GitHub Actions workflows for:
- Automated testing on pull requests
- Code formatting and linting validation
- Release builds triggered by version tags
- Binary artifact generation with SHA256 checksums

---

## Sample Output

### Console Summary
```bash
System Summary:
==============
CPU: AMD EPYC 7763 (2 Sockets, 64 Cores/Socket, 2 Threads/Core, 8 NUMA Nodes)
Total: 128 Cores, 256 Threads
Memory: 512GB DDR4 @ 3200 MHz
Storage: 15.36 TB (Total: 15.37 TB)
Storage Devices: 3.84TB + 3.84TB + 3.84TB + 3.84TB
BIOS: AMI 2.4.3 (01/15/2024)
Chassis: SuperMicro SC847BE2C-R1K28LPB (S/N: S454857X9822867)
Motherboard: Supermicro X13DEG-OAD v1.01 (S/N: OM237S046931)

Network Interfaces:
  enp1s0 - Intel Corporation E810-XXVDA4 (E4:43:4B:43:07:24) [Speed: 100000baseT/Full] [NUMA: 0]
  enp2s0 - Intel Corporation E810-XXVDA4 (E4:43:4B:43:07:25) [Speed: 100000baseT/Full] [NUMA: 0]

GPUs:
  NVIDIA H100 PCIe - NVIDIA (86:00.0) [NUMA: 0]
  NVIDIA H100 PCIe - NVIDIA (87:00.0) [NUMA: 1]

NUMA Topology:
  Node 0:
    Memory: 128G
    CPUs: 0-15,128-143
    Devices:
      GPU - NVIDIA H100 PCIe (PCI ID: 86:00.0)
      NIC - Intel E810-XXVDA4 (PCI ID: A1:00.0)

  Node 1:
    Memory: 128G
    CPUs: 16-31,144-159
    Devices:
      GPU - NVIDIA H100 PCIe (PCI ID: 87:00.0)

Filesystems:
  /dev/nvme0n1p2 (xfs) - 3.8T total, 2.1T used, 1.7T available, mounted on /
  /dev/nvme1n1 (xfs) - 3.8T total, 1.9T used, 1.9T available, mounted on /data
```

### Structured TOML Report (Sample Sections)

```toml
hostname = "gpu120B"
bmc_ip = "10.49.136.120"
bmc_mac = "7c:c2:55:50:ca:55"

[summary]
total_memory = "1.0Ti"
memory_config = "DDR5 @ 4800 MT/s"
total_storage = "2.6 TB"
total_storage_tb = 2.6200195312494543
total_gpus = 8
total_nics = 15
cpu_summary = "Intel(R) Xeon(R) Platinum 8462Y+ (2 Sockets, 32 Cores/Socket, 2 Threads/Core, 2 NUMA Nodes)"

[summary.bios]
vendor = "American Megatrends International, LLC."
version = "2.1.V1"
release_date = "03/20/2024"
firmware_version = "N/A"

[summary.chassis]
manufacturer = "Supermicro"
type_ = "Other"
serial = "C8010MM29A40285"

[summary.motherboard]
manufacturer = "Supermicro"
product_name = "X13DEG-OAD"
version = "1.01"
serial = "OM234S043296"

[summary.cpu_topology]
total_cores = 64
total_threads = 128
sockets = 2
cores_per_socket = 32
threads_per_core = 2
numa_nodes = 2
cpu_model = "Intel(R) Xeon(R) Platinum 8462Y+"

[hardware.cpu]
model = "Intel(R) Xeon(R) Platinum 8462Y+"
cores = 32
threads = 2
sockets = 2
speed = " MHz"

[hardware.memory]
total = "1.0Ti"
type_ = "DDR5"
speed = "4800 MT/s"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMA1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMA2"

[[hardware.storage.devices]]
name = "nvme0n1"
type_ = "disk"
size = "894.3G"
model = "Micron_7450_MTFDKBA960TFR"

[[hardware.storage.devices]]
name = "nvme1n1"
type_ = "disk"
size = "894.3G"
model = "Micron_7450_MTFDKBA960TFR"

[[hardware.gpus.devices]]
index = 0
name = "NVIDIA H100 80GB HBM3"
uuid = "GPU-9856b125-7617-39e4-f265-a21213c1d988"
memory = "81559 MiB"
pci_id = "10de:2330"
vendor = "NVIDIA Corporation"

[[hardware.gpus.devices]]
index = 1
name = "NVIDIA H100 80GB HBM3"
uuid = "GPU-84a1aceb-6bec-9067-95a8-32bf31bb9b2c"
memory = "81559 MiB"
pci_id = "10de:2330"
vendor = "NVIDIA Corporation"

[[network.interfaces]]
name = "enp217s0f0np0"
mac = "a0:88:c2:09:7c:c8"
ip = "10.49.11.120"
speed = "100000Mb/s"
type_ = "ether"
vendor = "Mellanox Technologies"
model = "MT2892 Family [ConnectX-6 Dx]"
pci_id = "15b3:101d"

[[network.interfaces]]
name = "ibp26s0"
mac = "00:00:10:49:fe:80:00:00:00:00:00:00:a0:88:c2:03:00:4b:64:18"
ip = ""
speed = "400000Mb/s"
type_ = "infiniband"
vendor = "Mellanox Technologies"
model = "MT2910 Family [ConnectX-7]"
pci_id = "15b3:1021"
```

---

## Architecture & Dependencies

### System Requirements
- **Operating System**: Linux (primary target)
- **Privileges**: Root access required for hardware introspection
- **Architecture**: x86_64 (primary), with cross-compilation support

### Runtime Dependencies
**Nix-built binaries include all dependencies automatically:**
- `numactl` - NUMA topology information
- `ipmitool` - BMC information extraction
- `ethtool` - Network interface details
- `lscpu` - CPU configuration data
- `lspci` - PCI device enumeration

**Note**: NVIDIA drivers must be installed separately for GPU detection.

### Core Libraries
- **regex** - System command output parsing
- **serde** - Data serialization framework
- **toml** - TOML format generation

---

## Deployment Considerations

### Production Deployment Options

**Option 1: Pre-built Debian Package (Recommended for Production)**
```bash
# Download latest Debian package from GitHub Releases
curl -s https://api.github.com/repos/sfcompute/hardware_report/releases/latest \
  | grep "browser_download_url.*\.deb" \
  | cut -d '"' -f 4 \
  | wget -i -

# Install with automatic dependency resolution
sudo apt update && sudo apt install -y ./hardware-report_*_amd64.deb

# Deploy across multiple servers
ansible servers -m copy -a "src=hardware-report_*_amd64.deb dest=/tmp/"
ansible servers -m apt -a "deb=/tmp/hardware-report_*_amd64.deb state=present"
```

**Option 2: Pre-built Binary Deployment**
```bash
# Download and verify pre-built binary
curl -s https://api.github.com/repos/sfcompute/hardware_report/releases/latest \
  | grep "browser_download_url.*tar.gz" \
  | cut -d '"' -f 4 \
  | wget -i -

# Extract and deploy to target systems
tar xzf hardware_report-linux-x86_64-*.tar.gz
scp hardware_report-linux-x86_64 user@target:/usr/local/bin/hardware_report

# Ensure runtime dependencies are installed on all target systems
ansible all -m package -a "name=numactl,ipmitool,ethtool,util-linux,pciutils state=present"
```

**Option 3: Build-from-Source Deployment**
```bash
# For environments that require building from source
# Using Nix-built package (includes all dependencies)
nix build .#deb
sudo apt install -y ./result/hardware-report_*_amd64.deb

# Or using traditional cargo build
cargo build --release
sudo cp target/release/hardware_report /usr/local/bin/
```

### Automation Integration
1. **Scheduled Execution**: Use cron or systemd timers for regular inventory updates
2. **Configuration Management**: Deploy via Ansible, Puppet, or Chef
3. **Report Collection**: Centralize TOML files using rsync, SFTP, or object storage
4. **Version Control**: Track infrastructure changes through Git

### Dependency Management Strategies

**Nix Approach** (Zero manual dependency management):
- All dependencies bundled automatically
- Consistent across all environments
- No version conflicts or missing tools

**Traditional Approach** (Manual dependency management):
- Explicit control over system packages
- Integration with existing package management
- Requires dependency installation on each target system

**Container Approach** (Isolated execution):
```dockerfile
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y \
    numactl ipmitool ethtool util-linux pciutils
COPY hardware_report /usr/local/bin/
CMD ["hardware_report"]
```

### Error Handling
The tool gracefully handles common failure scenarios:
- Missing system utilities
- Insufficient privileges
- Unavailable hardware components
- Command output parsing errors
- Network interface collection failures

### Troubleshooting
**Debian Package Issues**
```bash
# Remove broken installation
sudo apt remove -y hardware-report

# Rebuild and reinstall
git pull
rm -rf result
nix build .#deb --rebuild
sudo apt install -y ./result/hardware-report_0.1.7_amd64.deb
```

---

## Project Information

**Repository**: https://github.com/sfcompute/hardware_report  
**Author**: Kenny Sheridan, Supercomputing Engineer  
**License**: Open Source  
**Target Community**: Open-source GPU infrastructure community

### Contributing
Pull requests welcome. For major changes, please open an issue first to discuss proposed modifications.

### Support
For issues and feature requests, please use the GitHub issue tracker.
