# IOLinke-Device Workspace

A complete IO-Link Device Stack implementation in Rust, compliant with IO-Link Specification v1.1.4.

## Overview

IOLinke-Device provides a production-ready (Work in progress), embedded-first IO-Link device stack designed for modularity, compliance, and ease of integration. The workspace consists of multiple Rust crates, each implementing a specific layer or utility for building IO-Link-compliant devices. also check out project status [PROJECT_STATUS.adoc](./PROJECT_STATUS.adoc).

## Features

✅ **IO-Link v1.1.4 Compliant**
- Complete specification implementation
- All required state machines and protocol handlers
- Standard error codes and comprehensive error handling

✅ **Embedded-First Design**
- `#![no_std]` compatible across core crates
- Minimal memory footprint optimized for embedded systems
- Zero heap allocations
- C language bindings for integration with microcontroller firmware with auto generation of C headers.


✅ **Platform Agnostic**
- Hardware abstraction layer for easy porting
- Mock implementations for development and testing
- Support for both embedded and host environments

✅ **Developer Friendly**
- Procedural macros to reduce boilerplate
- Comprehensive examples and documentation
- Well-documented APIs with usage guides
- Configuaration-driven design for easy adaptation to different hardware and application requirements
- Configure and validate device parameters and memory layout using a toon file

## Getting Started

### Prerequisites

- Rust toolchain (1.70.0 or later recommended)
- For embedded targets: appropriate target toolchain installed

### Configuation
Edit the [IOLinke-Dev-config/device_config.toon](IOLinke-Dev-config/device_config.toon) file to configure your device parameters, memory layout, and hardware-specific settings. This configuration will be validated and used to generate optimized stack code.

To validate it against the IO-Link specification, And update the stack source code run:
```bash
# Navigate to the root of the workspace
cargo configuration
```

### C Language Integration
```bash
# Navigate to bindings crate
cd IOLinke-Bindings/

# Build for specific target (example: ARM Cortex-M4)
cargo build --release --target thumbv7em-none-eabihf

# Other common targets:
# thumbv6m-none-eabi        (ARM Cortex-M0/M0+)
# thumbv7m-none-eabi         (ARM Cortex-M3)
# thumbv7em-none-eabihf      (ARM Cortex-M4/M7 with FPU)
# riscv32imac-unknown-none-elf (RISC-V)
```
Get the generated C header file from the target/include folder and include it in your C project to access the IO-Link device stack APIs.

Get the lib file from the `target/release/libiolinke_bindings.a` or `target/debug/libiolinke_bindings.a` folder and link it with your C project to use the IO-Link device stack functionalities.
```C
#include "iolinke_device.h"

/// Detailed API documentation and examples can be found in the IOLinke-Examples crate and the generated C header file.

// Use the provided APIs to initialize and operate the IO-Link device stack in your C application.
```
### Rust Integration
```bash
    cd IOLinke-DEVICE
    cargo build --release
```

Include the `IOLinke-DEVICE` crate from the workspace in your Rust project and use the provided APIs to initialize and operate the IO-Link device stack.

```rust
use IOLinke_DEVICE::*;

// Implement hardware abstraction layer
// Initialize the IO-Link stack
// Handle communication events
```

### Testing in Host Environment

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p IOLinke-DEVICE

# Run with verbose output
cargo test -- --nocapture
```

Refer to the `IOLinke-Examples` crate for complete integration examples.

### Testing in IO-Link Conformance Test Tool
Formal conformance testing has not yet been completed. Users and organizations are encouraged to validate the stack using the IO-Link Conformance Test Tool provided by the IO-Link Consortium, which verifies compliance with the IO-Link specification through a comprehensive suite of tests.
If any issues are discovered, please report them in the repository’s issue tracker with detailed information about the test case and observed behavior. This feedback is valuable for improving reliability and ensuring compliance.

Organizations interested in sponsoring formal conformance testing are welcome to contact [ch.raghava44@gmail.com](mailto:ch.raghava44@gmail.com)

## Architecture Overview

The workspace follows a layered architecture designed for compliance with IO-Link Specification v1.1.4.

- **Layered Protocol Stack**: Each crate corresponds to a specific layer or aspect of the IO-Link protocol, from physical layer abstraction to application layer interfaces.
- **Modular Design**: Crates are loosely coupled and can be developed, tested, and updated independently.
- **Configuration Driven**: Device behavior and capabilities are configured via the [IOLinke-Dev-config/device_config.toon](IOLinke-Dev-config/device_config.toon) file, enabling easy adaptation to different hardware and application requirements.
- **Testing and Validation**: Extensive testing utilities and example implementations facilitate validation of protocol compliance and interoperability.

Refer to individual crate documentation for detailed architecture and API information.

## Workspace Structure

### Core Crates

#### **IOLinke-types**
Protocol types, traits, and state machines for IO-Link device communication. This crate defines the foundational protocol logic and includes:

- Command handlers and message processing
- Data storage abstractions
- Event management system
- ISDU (Index Service Data Unit) operations
- Mode management and state machines
- Process data and on-request data handlers
- Protocol timing and system management

#### **IOLinke-DEVICE**
The main device implementation integrating all protocol layers. Provides:

- Complete IO-Link device state machine
- System management and configuration
- Process data exchange mechanisms
- ISDU service implementation
- Event and diagnostic handling
- And all core functionalities

#### **IOLinke-Bindings**
C FFI bindings for integration with C-based firmware and toolchains. Features:

- Foreign Function Interface (FFI) for all core APIs
- C header generation
- Example C applications demonstrating integration
- Cross-language compatibility layer

#### **IOLinke-Dev-config**
Vendor-specific configuration management. Integrators configure:

- Device parameters and capabilities
- Memory layouts and storage maps
- Hardware-specific settings
- Application-specific data structures
- Toon file to configure the device

#### **IOLinke-Derived-config**
Configuration derivation and validation layer that:

- Validates device configuration against IO-Link specifications
- Generates optimized stack code based on configuration
- Declares parameter memory map
- Provides compile-time configuration checks

### Development & Testing Crates

#### **IOLinke-Test-utils**
Comprehensive testing utilities including:

- Mock hardware implementations
- Protocol test sequences
- Validation helpers
- Test environment setup utilities

#### **IOLinke-Examples**
Example applications and demos:

- Rust usage examples
- C integration examples
- Common device implementations
- Quick-start templates

### Utility Crates

#### **IOLinke-macros**
Procedural macros simplifying development:

- Bitfield definition macros
- Protocol trait derivation
- Configuration helpers

#### **IOLinke-util**
Cross-platform utilities for:

- Logging and diagnostics
- Event formatting
- Protocol debugging helpers
- Common type conversions

## Licensing Model

Copyright (C) 2025 Raghava Challari

IOLinke-Device is currently licensed under the GNU General Public License v3.0 (GPLv3) for evaluation purposes. This allows you to use, modify, and distribute the software under the terms of GPLv3 during the evaluation phase.

For more details on GPLv3, refer to the [LICENSE](./LICENSE) file in the repository or visit the [GPLv3 official site](https://www.gnu.org/licenses/gpl-3.0.en.html).

### Disclaimer
This is an independent Rust implementation of the IO-Link specification and is not affiliated with the IO-Link Consortium.


## Contact

For any inquiries or support, please reach out to:

**Name:** Raghava Ch  
**Email:** [ch.raghava44@gmail.com](mailto:ch.raghava44@gmail.com)