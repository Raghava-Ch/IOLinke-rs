# IOLinke-Device Workspace

Complete IO-Link Device Stack implementation in Rust, compliant with IO-Link Specification v1.1.4.

## Workspace Structure

This is a Cargo workspace containing two main crates:

### 📦 [IOLinke-Device](./IOLinke-Device/)
The main IO-Link device stack implementation featuring:
- **12 state machines** as per IO-Link v1.1.4 specification
- **`#![no_std]` compatible** for embedded systems
- **Hardware abstraction layer** for platform portability
- **Complete protocol implementation** including DL-Mode, Message handling, ISDU, etc.
- **Mock HAL** for testing and examples

### 📦 [IOLinke-macros](./IOLinke-macros/)
Procedural macros to simplify IO-Link device development:
- `#[derive(IoLinkDevice)]` - Auto-implement device identification
- `#[iolink_state_machine]` - Generate state machine boilerplate
- `#[iolink_frame]` - Create frame validation methods
- `#[iolink_parameter]` - Generate ISDU parameter access

## Getting Started

### Build the entire workspace:
```bash
cargo build
```

### Run tests:
```bash
cargo test
```

### Run examples:
```bash
# Basic functionality demo
cargo run --example step_by_step

# Macro usage example
cargo run --example macro_example

# Minimal HAL demo
cargo run --example minimal
```

### Run with panic-halt feature (for no_std examples):
```bash
cargo test --example basic_device --features="panic-halt"
```

## Features

✅ **IO-Link v1.1.4 Compliant**
- Complete specification implementation
- All required state machines
- Standard error codes and handling

✅ **Embedded-First Design**
- `#![no_std]` compatible
- Minimal memory footprint
- No heap allocations

✅ **Platform Agnostic**
- Hardware abstraction layer
- Easy integration with real hardware
- Mock implementations for testing

✅ **Developer Friendly**
- Procedural macros for common patterns
- Comprehensive examples
- Well-documented APIs

## Project Status

| Component | Status | Tests |
|-----------|--------|-------|
| Core Library | ✅ Complete | 6/6 passing |
| State Machines | ✅ Implemented | ✅ Working |
| HAL Abstraction | ✅ Complete | ✅ Working |
| Examples | ✅ Working | ✅ Tested |
| Macros | ✅ Implemented | ✅ Ready |

## License

Licensed under either of:
- Apache License, Version 2.0
- MIT license

at your option.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Specification Compliance

This implementation follows IO-Link Specification v1.1.4 (June 2024):
- Section 5.2: Physical Layer
- Section 6.3: State Machines
- Section 7.3.4.1: Device Identification
- Section 8.1.3: ISDU Parameters
- Section 8.3: Event Handling
