# Architecture Overview

Hardware Report is built with **hexagonal (ports & adapters) architecture** for clean separation of concerns.

## Table of Contents

- [Design Principles](#design-principles)
- [Layer Structure](#layer-structure)
- [Component Diagram](#component-diagram)
- [Key Benefits](#key-benefits)

## Design Principles

| Principle | Description |
|-----------|-------------|
| **Domain Independence** | Domain layer has no knowledge of adapters or I/O |
| **Pure Parsers** | Parsing functions take strings, return Results - no side effects |
| **Trait Abstraction** | All platform-specific code behind trait interfaces |
| **Multi-Method Detection** | Each adapter tries multiple methods, returns best result |
| **Graceful Degradation** | Partial data is better than no data - always return something |

## Layer Structure

```
src/
├── domain/           # Core business logic (platform-agnostic)
│   ├── entities.rs   # Data structures (CpuInfo, MemoryInfo, etc.)
│   ├── errors.rs     # Domain errors
│   ├── parsers/      # Pure parsing functions (no I/O)
│   │   ├── cpu.rs
│   │   ├── memory.rs
│   │   ├── storage.rs
│   │   ├── network.rs
│   │   └── gpu.rs
│   └── services/     # Domain services (orchestration)
│
├── ports/            # Interface definitions
│   ├── primary/      # Offered interfaces (what we provide)
│   │   └── reporting.rs
│   └── secondary/    # Required interfaces (what we need)
│       ├── system.rs
│       ├── command.rs
│       └── publisher.rs
│
└── adapters/         # Implementations
    ├── primary/      # CLI, library entry points
    └── secondary/    # Platform-specific implementations
        ├── system/
        │   ├── linux.rs
        │   └── macos.rs
        ├── command/
        │   └── unix.rs
        └── publisher/
            ├── file.rs
            └── http.rs
```

## Component Diagram

```
                    ┌──────────────────────────────────────┐
                    │         Core Domain (Pure)           │
                    │                                      │
  Primary Ports     │  ┌────────────────────────────┐     │    Secondary Ports
  (Inbound)         │  │    Domain Services         │     │    (Outbound)
                    │  │  • HardwareCollectionSvc   │     │
 ┌─────────────┐    │  │  • ReportGenerationSvc     │     │    ┌──────────────────┐
 │   CLI       │───►│  └────────────────────────────┘     │───►│ System Adapters  │
 │             │    │                                      │    │ • LinuxProvider  │
 │ hardware_   │    │  ┌────────────────────────────┐     │    │ • MacOSProvider  │
 │ report      │    │  │    Domain Entities         │     │    └──────────────────┘
 └─────────────┘    │  │  • CpuInfo, MemoryInfo     │     │
                    │  │  • StorageInfo, GpuInfo    │     │    ┌──────────────────┐
 ┌─────────────┐    │  │  • NetworkInfo, SystemInfo │     │───►│ Command Executor │
 │  Library    │───►│  └────────────────────────────┘     │    │ • UnixExecutor   │
 │             │    │                                      │    └──────────────────┘
 │ create_     │    │  ┌────────────────────────────┐     │
 │ service()   │    │  │    Pure Parsers            │     │    ┌──────────────────┐
 └─────────────┘    │  │  • CPU, Memory, Storage    │     │───►│ Publishers       │
                    │  │  • GPU, Network, System    │     │    │ • FilePublisher  │
                    │  └────────────────────────────┘     │    │ • HttpPublisher  │
                    └──────────────────────────────────────┘    └──────────────────┘
```

## Key Benefits

- **Testable** - Mock any external dependency for thorough testing
- **Flexible** - Swap system providers or publishers independently
- **Maintainable** - Clear boundaries between business logic and infrastructure
- **Platform Independent** - Core domain stays pure, adapters handle OS specifics
