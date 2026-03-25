# Deployment Guide

## Table of Contents

- [Production Deployment](#production-deployment)
- [Automation Integration](#automation-integration)
- [Troubleshooting](#troubleshooting)

## Production Deployment

### Pre-built binary (recommended)

```bash
# Download tarball (x86_64 example; use aarch64 asset names on ARM)
curl -sL https://api.github.com/repos/sfcompute/hardware_report/releases/latest \
  | grep "browser_download_url.*hardware_report-linux-x86_64.*\.tar\.gz" | cut -d '"' -f 4 | wget -qi -
tar xzf hardware_report-linux-x86_64-*.tar.gz

# Install to PATH
sudo install -m 755 hardware_report-linux-x86_64 /usr/local/bin/hardware_report

# Runtime tools (Debian/Ubuntu example)
sudo apt-get install -y numactl ipmitool ethtool pciutils

sudo hardware_report
```

### Multi-server deployment with Ansible

```bash
# After extracting the release tarball locally:
ansible servers -m copy -a "src=hardware_report-linux-x86_64 dest=/usr/local/bin/hardware_report mode=0755"
ansible servers -m apt -a "name=numactl,ipmitool,ethtool,pciutils state=present" --become
ansible servers -m shell -a "sudo /usr/local/bin/hardware_report" --become
```

### Binary deployment (scp)

```bash
curl -sL https://api.github.com/repos/sfcompute/hardware_report/releases/latest \
  | grep "browser_download_url.*hardware_report-linux-x86_64.*\.tar\.gz" | cut -d '"' -f 4 | wget -qi -
tar xzf hardware_report-linux-x86_64-*.tar.gz

scp hardware_report-linux-x86_64 user@target:/tmp/
ssh user@target "sudo install -m 755 /tmp/hardware_report-linux-x86_64 /usr/local/bin/hardware_report"
ssh user@target "sudo apt-get install -y numactl ipmitool ethtool pciutils"
```

## Automation Integration

### Scheduled Execution (systemd timer)

```ini
# /etc/systemd/system/hardware-report.service
[Unit]
Description=Hardware Report Collection

[Service]
Type=oneshot
ExecStart=/usr/bin/hardware_report
WorkingDirectory=/var/lib/hardware-report

# /etc/systemd/system/hardware-report.timer
[Unit]
Description=Run Hardware Report daily

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

```bash
sudo systemctl enable --now hardware-report.timer
```

### Cron

```bash
# Daily at 2 AM
echo "0 2 * * * root /usr/bin/hardware_report" | sudo tee /etc/cron.d/hardware-report
```

## Troubleshooting

### Permission Issues

```bash
# Most hardware detection requires root
sudo hardware_report

# Or add user to required groups
sudo usermod -aG disk,video $USER
```

### Missing Dependencies

```bash
# Check which tools are available
which numactl ipmitool ethtool lspci dmidecode nvidia-smi

# Install missing (Ubuntu/Debian)
sudo apt-get install -y numactl ipmitool ethtool pciutils dmidecode
```

### Incomplete GPU Detection

```bash
# Verify NVIDIA driver is loaded
nvidia-smi

# If not working, install drivers
sudo apt-get install -y nvidia-driver-535  # or appropriate version
```

### Debug Output

```bash
# Enable verbose logging
RUST_LOG=debug sudo hardware_report
```
