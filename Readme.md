# Hardware Report
A Rust utility that automatically collects and reports detailed hardware information from Linux servers, outputting the data in TOML format.

## ⚠️ IMPORTANT BUILD REQUIREMENT ⚠️
**DOCKER MUST BE RUNNING ON YOUR LOCAL MACHINE TO COMPILE FOR LINUX ON NON-LINUX SYSTEMS**
**WITHOUT DOCKER RUNNING, THE BUILD WILL FAIL WHEN EXECUTING `make linux` ON macOS OR WINDOWS**

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
### Required Software
- Rust toolchain (cargo, rustc)
- Make
- **Docker (REQUIRED for cross-compilation on non-Linux systems)**

### Optional System Utilities
- `nvidia-smi` (required for NVIDIA GPU information)
- `ipmitool` (required for BMC information)
- `ethtool` (required for network interface details)
- `numactl` (required for NUMA topology information)
- `lscpu` (required for detailed CPU information)

## Building
The project includes a Makefile that supports building for both Linux and macOS targets.

### Building for Linux
```bash
# Ensure Docker is running first!
docker ps  # Should show Docker is running

# Build Linux binary
make linux

# The binary will be available at:
build/release/hardware_report-linux-x86_64
```

### Building for macOS (if on a Mac)
```bash
# No Docker required for native macOS build
make macos

# The binary will be available at:
build/release/hardware_report-macos-[architecture]
```

### Building for all supported platforms
```bash
# Ensure Docker is running first!
docker ps  # Should show Docker is running

# Build for all supported platforms
make all
```

## Usage
The program requires root privileges to access certain hardware information. Run it using sudo:

```bash
# For Linux binary
sudo ./build/release/hardware_report-linux-x86_64

# For macOS binary
sudo ./build/release/hardware_report-macos-arm64  # For Apple Silicon
# or
sudo ./build/release/hardware_report-macos-x86_64 # For Intel Macs
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
2. Generate a detailed `server_config.toml` file in the current directory

## Output Format
The generated TOML file includes the following main sections:

```toml
[summary]
cpu_summary = "AMD EPYC 7763 (2 Sockets, 64 Cores/Socket, 2 Threads/Core, 8 NUMA Nodes)"
total_memory = "..."
memory_config = "..."
total_storage = "..."
filesystems = [ ... ]
bios = { vendor = "...", version = "...", release_date = "..." }
chassis = { manufacturer = "...", type = "...", serial = "..." }
total_gpus = #
total_nics = #

[summary.cpu_topology]
total_cores = #
total_threads = #
sockets = #
cores_per_socket = #
threads_per_core = #
numa_nodes = #
cpu_model = "..."

[server]
hostname = "..."
os_ip = { ... }  # Network interface IP mapping
bmc_ip = "..."   # Optional
bmc_mac = "..."  # Optional

[hardware.cpu]
model = "..."
cores = #
threads = #
sockets = #
speed = "..."

[hardware.memory]
total = "..."
type = "..."
speed = "..."
modules = [ ... ]

[hardware.storage]
devices = [ ... ]

[hardware.gpus]
devices = [ ... ]

[network]
interfaces = [ ... ]
infiniband = { ... }  # Optional

[summary.numa_topology]
# NUMA node information including CPU and device affinity
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
- SF Compute

## Acknowledgments
- This tool makes use of various Linux system utilities and their output formats
- Inspired by the need for automated hardware inventory in large compute environments
