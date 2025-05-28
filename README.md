# Hardware Report
A Rust utility that automatically collects and reports detailed hardware information from Linux servers, outputting the data in TOML format.

The collected data is saved as `<chassis_serialnumber>_hardware_report.toml`, which ensures scalability by creating distinct reports for each server. These reports are useful for infrastructure standardization across heterogeneous bare-metal hardware, allowing operators to automate and manage configurations consistently.

This tool is designed to help the open-source GPU infrastructure community by providing a uniform method for gathering and serializing system data, which can be particularly beneficial when managing diverse clusters of GPUs and servers with varying configurations.

## Quick Start

### Preferred Method: Build with Nix
The easiest and most reproducible way to build `hardware_report` is using Nix, which automatically handles all dependencies:

```bash
# Install Nix and build in one go:
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install && \
. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && \
git clone -b add-nix-build-support https://github.com/sfcompute/hardware_report.git && \
cd hardware_report && \
nix build && \
echo "Build complete! Run with: sudo ./result/bin/hardware_report"
```

**Note**: The Nix build support is currently in the `add-nix-build-support` branch. Once merged to main, remove the `-b add-nix-build-support` flag from the clone command.

### Alternative: Use Development Shell
For development work, enter a shell with all dependencies:

```bash
# Enter development environment
nix develop

# Build with cargo
cargo build --release

# Run the binary
sudo ./target/release/hardware_report
```

### With direnv (Recommended for Development)
If you have direnv installed:

```bash
# Allow direnv to automatically load the environment
direnv allow

# Dependencies are now available in your shell
cargo build --release
```

## Features
- Comprehensive system information collection including:
    - Detailed CPU topology and configuration:
        - Model information
        - Socket, core, and thread counts
        - NUMA node configuration
        - Per-socket core counts
        - Threading details
    - System summary with BIOS and chassis information
    - Basic system details (hostname, IP addresses)
    - BMC (Baseboard Management Controller) information
    - Memory details (size, type, speed, individual modules)
    - Storage information (devices, size, model)
    - GPU details (when NVIDIA GPUs are present)
    - Network interface information
    - Infiniband configuration (if available)
    - Filesystem information and mount points
    - NUMA topology with device affinity

## Prerequisites

### For Nix Users (Recommended)
- Nix package manager - see installation instructions below
- That's it! Nix handles ALL dependencies including runtime tools

#### Installing Nix (if not already installed)
```bash
# On Linux or macOS - Install Nix with the official installer
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install

# After installation, restart your terminal or run:
. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh

# Verify installation
nix --version
```

### For Traditional Build Methods
- Rust toolchain (cargo, rustc)
- pkg-config
- OpenSSL development libraries
- Make (for Makefile builds)
- Docker (for cross-compilation to Linux on non-Linux systems)

### Required System Utilities (Runtime)

**When built with Nix**: All runtime dependencies are automatically included! The Nix-built binary comes with a wrapper that provides:
- `numactl` - for NUMA topology information
- `ipmitool` - for BMC information  
- `ethtool` - for network interface details
- `lscpu` - for detailed CPU information
- `lspci` - for PCI device information

**Note**: `nvidia-smi` must still be provided by your system's NVIDIA driver installation.

**When built without Nix** (cargo, make, or pre-built binaries):
```bash
# Install required runtime dependencies on Ubuntu/Debian:
sudo apt-get update && sudo apt-get install -y \
  numactl \
  ipmitool \
  ethtool \
  util-linux  # for lscpu
  pciutils    # for lspci
```

## Building

### Building with Nix (Recommended)
```bash
# One-liner for fresh systems (installs Nix, clones, and builds):
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install && \
. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && \
git clone -b add-nix-build-support https://github.com/sfcompute/hardware_report.git && \
cd hardware_report && \
nix build

# The binary will be available at:
./result/bin/hardware_report

# Optional: Install system-wide
sudo cp ./result/bin/hardware_report /usr/local/bin/
```

For step-by-step instructions:
```bash
# 1. Install Nix (skip if already installed)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install

# 2. Source Nix in current shell (no need to restart terminal)
. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh

# 3. Clone the repository with Nix support branch
git clone -b add-nix-build-support https://github.com/sfcompute/hardware_report.git
cd hardware_report

# 4. Build the project
nix build

# 5. Run the binary
sudo ./result/bin/hardware_report
```

### Building with Cargo (Alternative)
If you prefer to use cargo directly:

```bash
# Install dependencies first (example for Ubuntu/Debian)
sudo apt-get install pkg-config libssl-dev

# Build for your platform
cargo build --release

# The binary will be available at:
target/release/hardware_report
```

### Cross-Platform Builds with Make (Alternative)
The project includes a Makefile that supports building for both Linux and macOS targets.

#### ⚠️ IMPORTANT BUILD REQUIREMENT ⚠️
**DOCKER MUST BE RUNNING ON YOUR LOCAL MACHINE TO COMPILE FOR LINUX ON NON-LINUX SYSTEMS**

#### Building for Linux
```bash
# Ensure Docker is running first!
docker ps  # Should show Docker is running

# Build Linux binary
make linux

# The binary will be available at:
build/release/hardware_report-linux-x86_64
```

#### Building for macOS
```bash
# No Docker required for native macOS build
make macos

# The binary will be available at:
build/release/hardware_report-macos-[architecture]
```

#### Building for all supported platforms
```bash
# Ensure Docker is running first!
make all
```

## Continuous Integration and Releases

### Automated Builds
The project uses GitHub Actions for continuous integration and release management:
- All pull requests are automatically tested
- Code formatting and linting are checked
- Releases are automatically built and published when version tags are pushed

### Creating a Release
1. Tag the commit you want to release:
```bash
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

2. The GitHub Actions workflow will automatically:
   - Create a new GitHub Release
   - Build the Linux binary
   - Create a tarball with the binary
   - Generate SHA256 checksums
   - Upload the artifacts to the release

### Downloading Releases
You can download the latest release from the GitHub Releases page or use wget:
```bash
# Get the latest release URL
RELEASE_URL=$(curl -s https://api.github.com/repos/sfcompute/hardware_report/releases/latest | grep "browser_download_url.*tar.gz\"" | cut -d '"' -f 4)

# Download and verify the tarball
wget $RELEASE_URL
wget $RELEASE_URL.sha256
sha256sum -c hardware_report-linux-x86_64-*.tar.gz.sha256

# Extract and make executable
tar xzf hardware_report-linux-x86_64-*.tar.gz
chmod +x hardware_report-linux-x86_64
```

## Usage
The program requires root privileges to access certain hardware information. Run it using sudo:

```bash
# If built with Nix (includes all dependencies!)
sudo ./result/bin/hardware_report

# If built with cargo (install dependencies first)
sudo apt-get update && sudo apt-get install -y numactl ipmitool ethtool util-linux pciutils
sudo ./target/release/hardware_report

# If using pre-built binaries (install dependencies first)
sudo apt-get update && sudo apt-get install -y numactl ipmitool ethtool util-linux pciutils
sudo ./hardware_report-linux-x86_64
```

The program will:
1. Display a summary of system information including:
    - CPU configuration and topology
    - Memory configuration
    - Storage capacity
    - BIOS information
    - Chassis details
    - GPU count
    - Network interface count
    - Filesystem information
2. Generate a detailed `<chassis_serial>_hardware_report.toml` file in the current directory

## Summarized Node Hardware Output
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

[Additional NUMA nodes omitted for brevity...]

Filesystems:
  /dev/nvme0n1p2 (xfs) - 3.8T total, 2.1T used, 1.7T available, mounted on /
  /dev/nvme1n1 (xfs) - 3.8T total, 1.9T used, 1.9T available, mounted on /data
```

## (Sample) Serialized Output
The generated TOML file includes the following main sections:

```toml
hostname = "gpu120B"
bmc_ip = "10.49.136.120"
bmc_mac = "7c:c2:55:50:ca:55"

[summary]
total_memory = "1.0Ti"
memory_config = "DDR5 @ 4800 MT/s"
total_storage = "2.6 TB"
total_storage_tb = 2.6200195312494543
filesystems = [
    "tmpfs (tmpfs) - 101G total, 3.9M used, 101G available, mounted on /run",
    "/dev/mapper/vgroot-lvroot (ext4) - 879G total, 30G used, 805G available, mounted on /",
    "tmpfs (tmpfs) - 504G total, 0 used, 504G available, mounted on /dev/shm",
    "tmpfs (tmpfs) - 5.0M total, 0 used, 5.0M available, mounted on /run/lock",
    "/dev/nvme0n1p1 (vfat) - 511M total, 6.1M used, 505M available, mounted on /boot/efi",
    "tmpfs (tmpfs) - 101G total, 4.0K used, 101G available, mounted on /run/user/1001",
    "tmpfs (tmpfs) - 101G total, 4.0K used, 101G available, mounted on /run/user/0",
]
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
features = "Unknown"
location = "Part Component"
type_ = "Motherboard"

[summary.numa_topology.0]
id = 0
cpus = [
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
    13,
    14,
    15,
    16,
    17,
    18,
    19,
    20,
    21,
    22,
    23,
    24,
    25,
    26,
    27,
    28,
    29,
    30,
    31,
    64,
    65,
    66,
    67,
    68,
    69,
    70,
    71,
    72,
    73,
    74,
    75,
    76,
    77,
    78,
    79,
    80,
    81,
    82,
    83,
    84,
    85,
    86,
    87,
    88,
    89,
    90,
    91,
    92,
    93,
    94,
    95,
]
memory = "515656 MB"
devices = []

[summary.numa_topology.0.distances]
0 = 1

[summary.numa_topology.1]
id = 1
cpus = [
    32,
    33,
    34,
    35,
    36,
    37,
    38,
    39,
    40,
    41,
    42,
    43,
    44,
    45,
    46,
    47,
    48,
    49,
    50,
    51,
    52,
    53,
    54,
    55,
    56,
    57,
    58,
    59,
    60,
    61,
    62,
    63,
    96,
    97,
    98,
    99,
    100,
    101,
    102,
    103,
    104,
    105,
    106,
    107,
    108,
    109,
    110,
    111,
    112,
    113,
    114,
    115,
    116,
    117,
    118,
    119,
    120,
    121,
    122,
    123,
    124,
    125,
    126,
    127,
]
memory = "516072 MB"
devices = []

[summary.numa_topology.1.distances]

[summary.cpu_topology]
total_cores = 64
total_threads = 128
sockets = 2
cores_per_socket = 32
threads_per_core = 2
numa_nodes = 2
cpu_model = "Intel(R) Xeon(R) Platinum 8462Y+"

[os_ip]
enp217s0f0np0 = "10.49.11.120"
tailscale0 = "100.85.182.53"

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

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMB1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMB2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMC1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMC2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMD1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMD2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMME1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMME2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMF1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMF2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMG1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMG2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMH1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P1-DIMMH2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMA1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMA2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMB1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMB2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMC1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMC2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMD1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMD2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMME1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMME2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMF1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMF2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMG1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMG2"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMH1"

[[hardware.memory.modules]]
size = "32 GB"
type_ = "DDR5"
speed = "4800 MT/s"
location = "P2-DIMMH2"

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

[[hardware.storage.devices]]
name = "nvme2n1"
type_ = "disk"
size = "894.3G"
model = "SAMSUNG MZQL2960HCJR-00A07"

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

[[hardware.gpus.devices]]
index = 2
name = "NVIDIA H100 80GB HBM3"
uuid = "GPU-09f2d8e0-6794-c36d-72c1-6188c9fd3ca8"
memory = "81559 MiB"
pci_id = "10de:2330"
vendor = "NVIDIA Corporation"

[[hardware.gpus.devices]]
index = 3
name = "NVIDIA H100 80GB HBM3"
uuid = "GPU-d8d290ff-f504-df9a-542f-e36fcd79c5fd"
memory = "81559 MiB"
pci_id = "10de:2330"
vendor = "NVIDIA Corporation"

[[hardware.gpus.devices]]
index = 4
name = "NVIDIA H100 80GB HBM3"
uuid = "GPU-fc4b72ea-18d3-170e-3cac-9892a9f88605"
memory = "81559 MiB"
pci_id = "10de:2330"
vendor = "NVIDIA Corporation"

[[hardware.gpus.devices]]
index = 5
name = "NVIDIA H100 80GB HBM3"
uuid = "GPU-a937677e-4f9d-4774-3a2f-1f56d83559b7"
memory = "81559 MiB"
pci_id = "10de:2330"
vendor = "NVIDIA Corporation"

[[hardware.gpus.devices]]
index = 6
name = "NVIDIA H100 80GB HBM3"
uuid = "GPU-17a894de-70b6-7a64-e321-b2f5eb3f7ea2"
memory = "81559 MiB"
pci_id = "10de:2330"
vendor = "NVIDIA Corporation"

[[hardware.gpus.devices]]
index = 7
name = "NVIDIA H100 80GB HBM3"
uuid = "GPU-699b6e6a-8b22-e653-3c37-a744bd686ba1"
memory = "81559 MiB"
pci_id = "10de:2330"
vendor = "NVIDIA Corporation"

[[network.interfaces]]
name = "enp220s0f0"
mac = "7c:c2:55:79:6e:a2"
ip = ""
speed = "Unknown!"
type_ = "ether"
vendor = "Intel Corporation"
model = "Ethernet Controller 10G X550T"
pci_id = "8086:1563"

[[network.interfaces]]
name = "enp220s0f1"
mac = "7c:c2:55:79:6e:a3"
ip = ""
speed = "Unknown!"
type_ = "ether"
vendor = "Intel Corporation"
model = "Ethernet Controller 10G X550T"
pci_id = "8086:1563"

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
name = "enp217s0f1np1"
mac = "a0:88:c2:09:7c:c9"
ip = ""
speed = "Unknown!"
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

[[network.interfaces]]
name = "ibp44s0"
mac = "00:00:10:49:fe:80:00:00:00:00:00:00:a0:88:c2:03:00:49:ad:8a"
ip = ""
speed = "400000Mb/s"
type_ = "infiniband"
vendor = "Mellanox Technologies"
model = "MT2910 Family [ConnectX-7]"
pci_id = "15b3:1021"

[[network.interfaces]]
name = "ibp64s0"
mac = "00:00:10:49:fe:80:00:00:00:00:00:00:a0:88:c2:03:00:49:af:1a"
ip = ""
speed = "400000Mb/s"
type_ = "infiniband"
vendor = "Mellanox Technologies"
model = "MT2910 Family [ConnectX-7]"
pci_id = "15b3:1021"

[[network.interfaces]]
name = "ibp93s0f0"
mac = "00:00:0d:31:fe:80:00:00:00:00:00:00:94:6d:ae:03:00:6f:02:30"
ip = ""
speed = "200000Mb/s"
type_ = "infiniband"
vendor = "Mellanox Technologies"
model = "MT28908 Family [ConnectX-6]"
pci_id = "15b3:101b"

[[network.interfaces]]
name = "ibp93s0f1"
mac = "00:00:0d:01:fe:80:00:00:00:00:00:00:94:6d:ae:03:00:6f:02:31"
ip = ""
speed = "Unknown!"
type_ = "infiniband"
vendor = "Mellanox Technologies"
model = "MT28908 Family [ConnectX-6]"
pci_id = "15b3:101b"

[[network.interfaces]]
name = "ibp101s0"
mac = "00:00:10:49:fe:80:00:00:00:00:00:00:a0:88:c2:03:00:49:ae:1a"
ip = ""
speed = "400000Mb/s"
type_ = "infiniband"
vendor = "Mellanox Technologies"
model = "MT2910 Family [ConnectX-7]"
pci_id = "15b3:1021"

[[network.interfaces]]
name = "ibp156s0"
mac = "00:00:10:49:fe:80:00:00:00:00:00:00:a0:88:c2:03:00:49:ad:aa"
ip = ""
speed = "400000Mb/s"
type_ = "infiniband"
vendor = "Mellanox Technologies"
model = "MT2910 Family [ConnectX-7]"
pci_id = "15b3:1021"

[[network.interfaces]]
name = "ibp173s0"
mac = "00:00:10:49:fe:80:00:00:00:00:00:00:a0:88:c2:03:00:4b:63:f0"
ip = ""
speed = "400000Mb/s"
type_ = "infiniband"
vendor = "Mellanox Technologies"
model = "MT2910 Family [ConnectX-7]"
pci_id = "15b3:1021"

[[network.interfaces]]
name = "ibp192s0"
mac = "00:00:10:49:fe:80:00:00:00:00:00:00:a0:88:c2:03:00:49:a1:fa"
ip = ""
speed = "400000Mb/s"
type_ = "infiniband"
vendor = "Mellanox Technologies"
model = "MT2910 Family [ConnectX-7]"
pci_id = "15b3:1021"

[[network.interfaces]]
name = "ibp227s0"
mac = "00:00:0a:04:fe:80:00:00:00:00:00:00:a0:88:c2:03:00:49:a2:92"
ip = ""
speed = "400000Mb/s"
type_ = "infiniband"
vendor = "Mellanox Technologies"
model = "MT2910 Family [ConnectX-7]"
pci_id = "15b3:1021"

[[network.interfaces]]
name = "tailscale0"
mac = ""
ip = "100.85.182.53"
speed = "Unknown!"
type_ = "none"
vendor = ""
model = ""
pci_id = ""
```

## Error Handling
The program handles various error cases gracefully:
- Missing system utilities
- Insufficient permissions
- Unavailable hardware components
- Parse errors in system command output
- Network interface collection failures

## Dependencies
- `regex` - For parsing command output
- `serde` - For serialization
- `toml` - For TOML format handling

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## Authors
- Kenny Sheridan, Supercomputing Engineer

## Acknowledgments
- This tool makes use of various Linux system utilities and their output formats
- Inspired by the need for automated hardware inventory in large compute environments