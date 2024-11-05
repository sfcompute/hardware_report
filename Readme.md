# Hardware Report

A Rust utility that automatically collects and reports detailed hardware information from Linux servers, outputting the data in TOML format.

## Features

- Comprehensive system information collection:
  - Basic system details (hostname, IP addresses)
  - BMC (Baseboard Management Controller) information
  - CPU specifications (model, cores, threads, speed)
  - Memory details (size, type, speed, individual modules)
  - Storage information (devices, size, model)
  - GPU details (when NVIDIA GPUs are present)
  - Network interface information
  - Infiniband configuration (if available)

## Prerequisites

- Linux operating system
- Rust toolchain (cargo, rustc)
- The following system utilities must be installed:
  - `hostname`
  - `ip`
  - `lscpu`
  - `dmidecode` (requires root privileges)
  - `lsblk`
  - `ethtool`
  - `ipmitool` (for BMC information)
  - `nvidia-smi` (optional, for GPU information)
  - `ibstat` (optional, for Infiniband information)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/sfcompute/hardware_report.git
cd hardware_report
```

2. Build the project:
```bash
cargo build --release
```

The compiled binary will be available at `target/release/hardware_report`

## Usage

Run the program with appropriate permissions (some hardware information requires root access):

```bash
sudo ./target/release/hardware_report
```

The program will generate a `server_config.toml` file in the current directory containing all collected system information.

## Output Format

The generated TOML file includes the following main sections:

```toml
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
- `serde_json` - For parsing JSON output from system commands

## License

[Insert your chosen license here]

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## Authors

- SF Compute

## Acknowledgments

- This tool makes use of various Linux system utilities and their output formats
- Inspired by the need for automated hardware inventory in large compute environments
