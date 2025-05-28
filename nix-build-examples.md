# Nix Build Examples

## Local Development

```bash
# Enter development shell
nix develop

# Build the main package
nix build

# Build Debian package
nix build .#deb

# Build Docker image
nix build .#dockerImage
docker load < result

# Build static binary
nix build .#static

# Build RPM package
nix build .#rpm

# Run all CI checks
nix flake check

# Run specific checks
nix build .#checks.x86_64-linux.cargo-test
nix build .#checks.x86_64-linux.cargo-fmt
nix build .#checks.x86_64-linux.cargo-clippy
```

## CI/CD Benefits

1. **Reproducible Builds**: Every build uses exact same dependencies
2. **Cross-platform**: Build for Linux on macOS and vice versa
3. **Multiple Package Formats**: DEB, RPM, Docker, static binaries
4. **Cached Dependencies**: Use Cachix for faster builds
5. **Integrated Testing**: All tests run in isolated environments

## Package Outputs

After running the GitHub Actions workflows, you'll get:

- `hardware_report-linux-x86_64-{version}.tar.gz` - Linux binary
- `hardware_report-darwin-x86_64-{version}.tar.gz` - macOS binary  
- `hardware_report_{version}_amd64.deb` - Debian package
- `ghcr.io/{owner}/hardware-report:{version}` - Docker image
- SHA256 checksums for all artifacts

## Testing Locally

```bash
# Test the Debian package build
nix build .#deb
ls -la result/

# Test in a Debian container
docker run -it -v $(nix build .#deb --print-out-paths):/tmp/deb debian:latest bash
# Inside container:
apt update && apt install -y /tmp/deb/*.deb
hardware_report --version
```

## Cachix Setup (Optional)

To speed up CI builds:

1. Create account at https://cachix.org
2. Create a cache named `hardware-report`
3. Add secret `CACHIX_AUTH_TOKEN` to GitHub
4. Uncomment the authToken line in workflows

This will cache all Nix builds between CI runs.