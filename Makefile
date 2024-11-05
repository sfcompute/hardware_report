# Makefile for hardware_report - System Hardware Information Tool

# Define the name of the binary
BINARY_NAME := hardware_report

# Define the Rust compiler and Cargo
RUSTC := rustc
CARGO := cargo

# Define target architectures
LINUX_TARGET := x86_64-unknown-linux-gnu
MACOS_TARGET := aarch64-apple-darwin

# Define output directories
BUILD_DIR := build
RELEASE_DIR := $(BUILD_DIR)/release

# Define version
VERSION := 0.1.0

# Define flags for release builds
RELEASE_FLAGS := --release

# Define common cargo build command
CARGO_BUILD := $(CARGO) build $(RELEASE_FLAGS)

# Define docker image for Linux builds
DOCKER_IMAGE := rust:latest

# Detect the host system
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Check if Docker is available
DOCKER_AVAILABLE := $(shell command -v docker 2> /dev/null)

# Default target
.PHONY: all
all: 
ifeq ($(UNAME_S),Darwin)
ifeq ($(DOCKER_AVAILABLE),)
	@echo "Docker is not available. Skipping Linux build and proceeding with macOS build only."
	@$(MAKE) macos
else
	@$(MAKE) linux macos
endif
else
	@$(MAKE) linux macos
endif

# Target for building Linux x86_64 binary
.PHONY: linux
linux:
	@echo "Building for Linux (x86_64)..."
	@mkdir -p $(RELEASE_DIR)
ifeq ($(UNAME_S),Linux)
	@echo "Building natively on Linux..."
	$(CARGO_BUILD) --target=$(LINUX_TARGET)
	@cp target/$(LINUX_TARGET)/release/$(BINARY_NAME) $(RELEASE_DIR)/$(BINARY_NAME)-linux-x86_64
else
ifeq ($(DOCKER_AVAILABLE),)
	@echo "Error: Docker is not available. Cannot build for Linux on non-Linux systems without Docker."
	@exit 1
else
	@echo "Cross-compiling for Linux using Docker..."
	@docker run --rm -v $(PWD):/usr/src/myapp -w /usr/src/myapp --platform linux/amd64 $(DOCKER_IMAGE) \
		bash -c "rustup target add $(LINUX_TARGET) && \
		cargo build --release --target=$(LINUX_TARGET)"
	@cp target/$(LINUX_TARGET)/release/$(BINARY_NAME) $(RELEASE_DIR)/$(BINARY_NAME)-linux-x86_64
	@chmod +x $(RELEASE_DIR)/$(BINARY_NAME)-linux-x86_64
endif
endif
	@echo "Linux binary built: $(RELEASE_DIR)/$(BINARY_NAME)-linux-x86_64"

# Target for building macOS binary
.PHONY: macos
macos:
ifeq ($(UNAME_S),Darwin)
	@echo "Building for macOS ($(UNAME_M))..."
	@mkdir -p $(RELEASE_DIR)
ifeq ($(UNAME_M),arm64)  # If on Apple Silicon
	$(CARGO_BUILD) --target=aarch64-apple-darwin
	@cp target/$(MACOS_TARGET)/release/$(BINARY_NAME) $(RELEASE_DIR)/$(BINARY_NAME)-macos-arm64
else ifeq ($(UNAME_M),x86_64)  # If on Intel Mac
	$(CARGO_BUILD) --target=x86_64-apple-darwin
	@cp target/x86_64-apple-darwin/release/$(BINARY_NAME) $(RELEASE_DIR)/$(BINARY_NAME)-macos-x86_64
endif
	@echo "macOS binary built: $(RELEASE_DIR)/$(BINARY_NAME)-macos-$(UNAME_M)"
else
	@echo "Error: macOS build can only be performed on a Mac."
	@exit 1
endif

# Target for installing required tools
.PHONY: install-tools
install-tools:
	@echo "Installing required tools..."
	rustup target add $(LINUX_TARGET)
ifeq ($(UNAME_S),Darwin)
	rustup target add $(MACOS_TARGET)
ifeq ($(UNAME_M),x86_64)
	@echo "Note: For cross-compilation to ARM64, additional setup may be required."
endif
ifeq ($(DOCKER_AVAILABLE),)
	@echo "Warning: Docker is not installed. It is required for building Linux binaries on macOS."
else
	@echo "Docker is available for Linux builds."
	docker pull --platform linux/amd64 $(DOCKER_IMAGE)
endif
else ifeq ($(UNAME_S),Linux)
	@echo "Building on Linux, no additional tools needed."
else
	@echo "Unsupported operating system"
	@exit 1
endif

# Test target
.PHONY: test
test:
	$(CARGO) test

# Format check target
.PHONY: fmt
fmt:
	$(CARGO) fmt --all -- --check

# Lint target
.PHONY: lint
lint:
	$(CARGO) clippy -- -D warnings

# Documentation target
.PHONY: doc
doc:
	$(CARGO) doc --no-deps

# Clean target
.PHONY: clean
clean:
	$(CARGO) clean
	rm -rf $(BUILD_DIR)

# Help target
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  all          - Build for all platforms (default)"
	@echo "  linux        - Build for Linux x86_64"
	@echo "  macos        - Build for macOS (native architecture)"
	@echo "  test         - Run tests"
	@echo "  fmt          - Check code format"
	@echo "  lint         - Run clippy linter"
	@echo "  doc          - Generate documentation"
	@echo "  clean        - Clean build artifacts"
	@echo "  install-tools- Install required build tools"
	@echo "  help         - Show this help message"

# Ensure all required tools are installed before building
linux macos: install-tools
