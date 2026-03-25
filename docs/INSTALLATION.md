# Installation Guide

## Table of Contents

- [Nix Build (Recommended)](#nix-build-recommended)
- [Pre-built Releases](#pre-built-releases)
- [Cargo Install](#cargo-install)
- [Traditional Cargo Build](#traditional-cargo-build)
- [Runtime Dependencies](#runtime-dependencies)

## Nix Build (Recommended)

Nix automatically handles all build and runtime dependencies.

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

**Development shell:**
```bash
nix develop
cargo build --release
```

## Pre-built Releases

Download from [GitHub Releases](https://github.com/sfcompute/hardware_report/releases):

**Linux tarball** (pick the archive for your architecture — `x86_64` or `aarch64`):
```bash
curl -sL https://api.github.com/repos/sfcompute/hardware_report/releases/latest \
  | grep "browser_download_url.*hardware_report-linux-x86_64.*\.tar\.gz" | cut -d '"' -f 4 | wget -qi -
tar xzf hardware_report-linux-x86_64-*.tar.gz
sudo ./hardware_report-linux-x86_64
```

On ARM64, use `hardware_report-linux-aarch64` in the `grep` pattern and tarball / binary names instead.

## Cargo Install

```bash
git clone https://github.com/sfcompute/hardware_report.git
cd hardware_report
cargo install --path .
sudo hardware_report
```

## Traditional Cargo Build

**Ubuntu/Debian:**
```bash
sudo apt-get update && sudo apt-get install -y \
  build-essential pkg-config libssl-dev git \
  numactl ipmitool ethtool util-linux pciutils

git clone https://github.com/sfcompute/hardware_report.git
cd hardware_report
cargo build --release
sudo ./target/release/hardware_report
```

**RHEL/Fedora:**
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install pkg-config openssl-devel numactl ipmitool ethtool util-linux pciutils

git clone https://github.com/sfcompute/hardware_report.git
cd hardware_report
cargo build --release
sudo ./target/release/hardware_report
```

## Runtime Dependencies

| Tool | Purpose |
|------|---------|
| `numactl` | NUMA topology |
| `ipmitool` | BMC/IPMI data |
| `ethtool` | Network interface details |
| `lspci` | PCI device enumeration |
| `dmidecode` | System/BIOS/memory info |
| `nvidia-smi` | GPU detection (optional) |

> **Note:** Nix builds bundle all dependencies automatically.
