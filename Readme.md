# Hardware Report
A Rust utility that automatically collects and reports detailed hardware information from Linux servers, outputting the data in TOML format.

## Features
- Comprehensive system information collection including:
  - System summary with BIOS and chassis information
  - Basic system details (hostname, IP addresses)
  - BMC (Baseboard Management Controller) information
  - CPU specifications (model, cores, threads, speed)
  - Memory details (size, type, speed, individual modules)
  - Storage information (devices, size, model)
  - GPU details (when NVIDIA GPUs are present)
  - Network interface information
  - Infiniband configuration (if available)
  - Filesystem information and mount points

## Prerequisites
- Linux operating system
- Rust toolchain (cargo, rustc)
- Make (for using the Makefile)
- Docker (optional, for cross-compilation)
- The following system utilities must be installed:
  - `hostname`
  - `ip`
  - `lscpu`
  - `dmidecode` (requires root privileges)
  - `lsblk`
  - `df`
  - `ethtool`
  - `ipmitool` (for BMC information)
  - `nvidia-smi` (optional, for GPU information)
  - `ibstat` (optional, for Infiniband information)

## Building
The project includes a Makefile that supports building for both Linux and macOS targets.

### Building for Linux
```bash
# Build Linux binary
make linux

# The binary will be available at:
build/release/hardware_report-linux-x86_64
```

### Building for macOS (if on a Mac)
```bash
# Build macOS binary
make macos

# The binary will be available at:
build/release/hardware_report-macos-[architecture]
```

### Building for all supported platforms
```bash
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
total_memory = "..."
memory_config = "..."
total_storage = "..."
filesystems = [ ... ]
bios = { vendor = "...", version = "...", release_date = "..." }
chassis = { manufacturer = "...", type = "...", serial = "..." }
total_gpus = #
total_nics = #

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
```

## Error Handling
The program handles various error cases gracefully:
- Missing system utilities
- Insufficient permissions
- Unavailable hardware components
- Parse errors in system command output

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
