{
  description = "Hardware Report - A tool for generating hardware information reports";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            # Increase download buffer size to prevent warnings
            download-buffer-size = 256 * 1024 * 1024; # 256 MB
          };
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };
        
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];
        
        buildInputs = with pkgs; [
          openssl
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];
        
        # Runtime dependencies that the binary needs
        runtimeDeps = with pkgs; [
          numactl
          ipmitool
          ethtool
          util-linux  # for lscpu
          pciutils    # for lspci
        ];
        
        hardware_report_unwrapped = pkgs.rustPlatform.buildRustPackage {
          pname = "hardware_report_unwrapped";
          version = "0.1.7";
          
          src = builtins.path { 
            path = ./.; 
            name = "hardware-report-source"; 
          };
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          inherit nativeBuildInputs buildInputs;
          
          meta = with pkgs.lib; {
            description = "A tool for generating hardware information reports";
            homepage = "https://github.com/yourusername/hardware_report";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      in
      {
        packages.default = pkgs.writeShellScriptBin "hardware_report" ''
          export PATH="${pkgs.lib.makeBinPath runtimeDeps}:$PATH"
          exec ${hardware_report_unwrapped}/bin/hardware_report "$@"
        '';

        packages.hardware_report = self.packages.${system}.default;
        
        packages.deb = pkgs.stdenv.mkDerivation {
          pname = "hardware-report";
          version = "0.1.7";
          
          src = self.packages.${system}.default;
          
          nativeBuildInputs = with pkgs; [ dpkg ];
          
          unpackPhase = "true";
          
          buildPhase = ''
            # Create debian package structure
            mkdir -p hardware-report_0.1.7_amd64/{DEBIAN,usr/bin,usr/share/doc/hardware-report}
            
            # Copy the actual binary (not the wrapper)
            cp ${hardware_report_unwrapped}/bin/hardware_report hardware-report_0.1.7_amd64/usr/bin/
            
            # Create control file
            cat > hardware-report_0.1.7_amd64/DEBIAN/control << EOF
Package: hardware-report
Version: 0.1.7
Architecture: amd64
Maintainer: Kenny Sheridan <kenny@sfcompute.com>
Description: Hardware information collection tool
 A tool for generating detailed hardware information reports from Linux servers,
 outputting the data in TOML format for infrastructure standardization.
Depends: numactl, ipmitool, ethtool, util-linux, pciutils
Priority: optional
Section: utils
EOF
            
            # Create copyright file
            cat > hardware-report_0.1.7_amd64/usr/share/doc/hardware-report/copyright << EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: hardware_report
Source: https://github.com/sfcompute/hardware_report

Files: *
Copyright: 2024 Kenny Sheridan
License: MIT
EOF
            
            # Build the deb package
            dpkg-deb --build hardware-report_0.1.7_amd64
          '';
          
          installPhase = ''
            mkdir -p $out
            cp hardware-report_0.1.7_amd64.deb $out/
          '';
        };
        
        # Docker image
        packages.dockerImage = pkgs.dockerTools.buildImage {
          name = "hardware-report";
          tag = "latest";
          
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ self.packages.${system}.default ] ++ runtimeDeps;
            pathsToLink = [ "/bin" ];
          };
          
          config = {
            Cmd = [ "/bin/hardware_report" ];
          };
        };
        
        # Static binary for Alpine/musl
        packages.static = let
          muslPkgs = import nixpkgs {
            inherit system;
            crossSystem = {
              config = "x86_64-unknown-linux-musl";
            };
          };
        in muslPkgs.rustPlatform.buildRustPackage {
          pname = "hardware_report-static";
          version = "0.1.7";
          
          src = builtins.path { 
            path = ./.; 
            name = "hardware-report-source"; 
          };
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = with muslPkgs; [
            pkg-config
          ];
          
          buildInputs = with muslPkgs; [
            openssl.dev
          ];
          
          OPENSSL_STATIC = "1";
          OPENSSL_LIB_DIR = "${muslPkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${muslPkgs.openssl.dev}/include";
          
          meta = with pkgs.lib; {
            description = "Static build of hardware_report";
            homepage = "https://github.com/sfcompute/hardware_report";
            license = licenses.mit;
          };
        };
        
        # RPM package
        packages.rpm = pkgs.stdenv.mkDerivation {
          pname = "hardware-report";
          version = "0.1.7";
          
          src = self.packages.${system}.default;
          
          nativeBuildInputs = with pkgs; [ rpm ];
          
          unpackPhase = "true";
          
          buildPhase = ''
            mkdir -p rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
            
            cat > rpmbuild/SPECS/hardware-report.spec << EOF
Name:           hardware-report
Version:        0.1.7
Release:        1%{?dist}
Summary:        Hardware information collection tool
License:        MIT
URL:            https://github.com/sfcompute/hardware_report

%description
A tool for generating detailed hardware information reports from Linux servers,
outputting the data in TOML format for infrastructure standardization.

%install
mkdir -p %{buildroot}%{_bindir}
cp ${hardware_report_unwrapped}/bin/hardware_report %{buildroot}%{_bindir}/

%files
%{_bindir}/hardware_report

%changelog
* $(date +"%a %b %d %Y") Kenny Sheridan <kenny@sfcompute.com> - 0.1.7-1
- Initial RPM release
EOF
            
            rpmbuild --define "_topdir $(pwd)/rpmbuild" \
                     --define "_bindir /usr/bin" \
                     --define "buildroot $(pwd)/buildroot" \
                     -bb rpmbuild/SPECS/hardware-report.spec
          '';
          
          installPhase = ''
            mkdir -p $out
            cp rpmbuild/RPMS/*/*.rpm $out/
          '';
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ runtimeDeps ++ (with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-edit
          ]);
          
          shellHook = ''
            echo "Hardware Report development environment"
            echo "Run 'cargo build' to build the project"
            echo "Run 'cargo run' to run the project"
            echo ""
            echo "Runtime dependencies are available in PATH:"
            echo "- numactl, ipmitool, ethtool, lscpu, lspci"
          '';
        };
        
        # CI checks
        checks = {
          # Build the package
          package = self.packages.${system}.default;
          
          # Run cargo tests
          cargo-test = pkgs.runCommand "cargo-test" {
            inherit nativeBuildInputs buildInputs;
            src = builtins.path { 
              path = ./.; 
              name = "hardware-report-source"; 
            };
          } ''
            cd $src
            export HOME=$TMPDIR
            ${rustToolchain}/bin/cargo test --release
            touch $out
          '';
          
          # Check formatting
          cargo-fmt = pkgs.runCommand "cargo-fmt-check" {
            inherit nativeBuildInputs;
            src = builtins.path { 
              path = ./.; 
              name = "hardware-report-source"; 
            };
          } ''
            cd $src
            ${rustToolchain}/bin/cargo fmt -- --check
            touch $out
          '';
          
          # Run clippy
          cargo-clippy = pkgs.runCommand "cargo-clippy" {
            inherit nativeBuildInputs buildInputs;
            src = builtins.path { 
              path = ./.; 
              name = "hardware-report-source"; 
            };
          } ''
            cd $src
            export HOME=$TMPDIR
            ${rustToolchain}/bin/cargo clippy --all-targets -- -D warnings
            touch $out
          '';
        };
      });
}