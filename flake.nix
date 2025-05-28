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
          version = "0.1.1";
          
          src = ./.;
          
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
      });
}