# Makefile for hardware_report - System Hardware Information Tool

# Define the name of the binary (must match your [[bin]] name in Cargo.toml)
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

# Check if Docker is installed (non-empty if found)
DOCKER_CMD := $(shell command -v docker 2> /dev/null)

# Check if Docker is running (if installed). We'll store '1' if running, '0' otherwise
DOCKER_RUNNING := $(shell if [ -n "$(DOCKER_CMD)" ]; then docker info >/dev/null 2>&1 && echo 1 || echo 0; else echo 0; fi)

# Default target: build for both Linux and macOS on whichever platform we are on
.PHONY: all
all:
ifeq ($(UNAME_S),Darwin)
	@$(MAKE) linux macos
else ifeq ($(UNAME_S),Linux)
	@$(MAKE) linux macos
else
	@echo "Unsupported operating system: $(UNAME_S)"
	@exit 1
endif

# Target for building Linux x86_64 binary
.PHONY: linux
linux: install-tools
	@echo "Building for Linux (x86_64)..."
	@mkdir -p $(RELEASE_DIR)

ifeq ($(UNAME_S),Darwin)
	# On macOS → only cross-compile with Docker if Docker is running
ifeq ($(DOCKER_RUNNING),1)
	@echo "Cross-compiling for Linux using Docker..."
	docker run --rm \
		-v "$(PWD)":/usr/src/myapp \
		-w /usr/src/myapp \
		--platform linux/amd64 \
		$(DOCKER_IMAGE) \
		bash -c "rustup target add $(LINUX_TARGET) && \
		         cargo build --release --target=$(LINUX_TARGET)"
	@cp target/$(LINUX_TARGET)/release/$(BINARY_NAME) \
		$(RELEASE_DIR)/$(BINARY_NAME)-linux-x86_64
	@chmod +x $(RELEASE_DIR)/$(BINARY_NAME)-linux-x86_64
	@echo "Linux binary built (docker cross-compile): $(RELEASE_DIR)/$(BINARY_NAME)-linux-x86_64"
else
	@echo "WARNING: Docker is not running or not available on macOS. Skipping Linux build."
endif

else ifeq ($(UNAME_S),Linux)
	# On Linux → build natively, no Docker
	$(CARGO_BUILD) --target=$(LINUX_TARGET)
	@cp target/$(LINUX_TARGET)/release/$(BINARY_NAME) \
		$(RELEASE_DIR)/$(BINARY_NAME)-linux-x86_64
	@echo "Linux binary built (native): $(RELEASE_DIR)/$(BINARY_NAME)-linux-x86_64"
else
	@echo "Unsupported operating system for Linux build: $(UNAME_S)"
	@exit 1
endif


# Target for building macOS binary
.PHONY: macos
macos: install-tools
ifeq ($(UNAME_S),Darwin)
	@echo "Building for macOS ($(UNAME_M))..."
	@mkdir -p $(RELEASE_DIR)
ifeq ($(UNAME_M),arm64)
	# Apple Silicon
	$(CARGO_BUILD) --target=$(MACOS_TARGET)
	@cp target/$(MACOS_TARGET)/release/$(BINARY_NAME) \
		$(RELEASE_DIR)/$(BINARY_NAME)-macos-arm64
	@echo "macOS binary built: $(RELEASE_DIR)/$(BINARY_NAME)-macos-arm64"
else ifeq ($(UNAME_M),x86_64)
	# Intel Mac
	$(CARGO_BUILD) --target=x86_64-apple-darwin
	@cp target/x86_64-apple-darwin/release/$(BINARY_NAME) \
		$(RELEASE_DIR)/$(BINARY_NAME)-macos-x86_64
	@echo "macOS binary built: $(RELEASE_DIR)/$(BINARY_NAME)-macos-x86_64"
else
	@echo "Unknown Mac architecture: $(UNAME_M)."
endif
else
	@echo "Skipping macOS build because we're not on macOS."
endif

# Target for installing required tools
.PHONY: install-tools
install-tools:
	@echo "Installing required tools..."
	rustup target add $(LINUX_TARGET)

ifeq ($(UNAME_S),Darwin)
	rustup target add $(MACOS_TARGET)

	# Only pull Docker image if Docker is installed & running
ifeq ($(DOCKER_RUNNING),1)
	@echo "Docker is running on macOS...pulling image for cross-builds."
	docker pull --platform linux/amd64 $(DOCKER_IMAGE) || true
else
	@echo "Docker not running (or not installed). Will skip Docker-based Linux builds."
endif

else ifeq ($(UNAME_S),Linux)
	@echo "Building on Linux natively. No Docker usage needed."
else
	@echo "Unsupported operating system: $(UNAME_S)"
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
	@echo "  all          - Build for both Linux and macOS on your current platform"
	@echo "  linux        - Build for Linux x86_64 (Docker on macOS if running, native on Linux)"
	@echo "  macos        - Build for macOS (only works on an actual Mac)"
	@echo "  test         - Run tests"
	@echo "  fmt          - Check code format"
	@echo "  lint         - Run clippy linter"
	@echo "  doc          - Generate documentation"
	@echo "  clean        - Clean build artifacts"
	@echo "  install-tools- Install required build tools"
	@echo "  help         - Show this help message"

