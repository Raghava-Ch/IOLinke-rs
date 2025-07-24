# IO-Link Device Stack

A modular, maintainable, and portable IO-Link Device/Slave stack implementation compliant with IO-Link Specification Version 1.1.4 (June 2024).

## Features

- **Embedded-first**: `#![no_std]` compatible for bare-metal microcontrollers
- **State machine driven**: 12 protocol state machines using polling architecture
- **Platform portable**: Hardware abstraction layer (HAL) traits
- **C interoperability**: Complete C FFI bindings with generated headers
- **Protocol compliant**: Implements IO-Link Specification v1.1.4
- **Frame parsing**: Built-in nom-based frame parsing

## Architecture

The stack consists of 12 state machines handling different protocol aspects:

### Data Link Layer
- **DL-Mode Handler**: Communication mode management (SIO/COM1/COM2/COM3)
- **Message Handler**: Frame parsing and message routing

### Application Layer  
- **Process Data Handler**: Cyclic data exchange
- **On-request Data Handler**: Acyclic data exchange
- **ISDU Handler**: Parameter access via Index Service Data Units
- **Command Handler**: Device command processing
- **Event Handler**: Event management and queuing

### System Layer
- **Application Layer**: Main API interface
- **Event State Machine**: Event state management
- **System Management**: Overall system control
- **Parameter Manager**: Parameter storage and access
- **Data Storage**: Non-volatile data management

## Quick Start

### Rust Usage

```rust
use iolink_device_stack::{ApplicationLayer, DeviceStack};

// Initialize with your HAL implementation
let mut device = DeviceStack::new(your_hal_impl);

// Main application loop
loop {
    device.poll()?;
}
```

### C Usage

```c
#include "iolink_device_stack.h"

// Create device instance
iolink_device_handle_t* device = iolink_device_create();

// Main loop
while (1) {
    iolink_device_poll(device);
    
    // Get process data
    iolink_process_data_t data;
    iolink_get_input_data(device, &data);
    
    // Set output data
    iolink_set_output_data(device, &data);
}

// Cleanup
iolink_device_destroy(device);
```

## Building

### For embedded targets

```bash
# For ARM Cortex-M
cargo build --target thumbv7em-none-eabihf --release

# For other no_std targets
cargo build --target your-target --release
```

### Generate C headers

```bash
cargo build
# Headers generated in target/include/iolink_device_stack.h
```

## Hardware Abstraction

Implement the required HAL traits for your platform:

```rust
use iolink_device_stack::hal::*;

struct MyHal {
    // Your hardware-specific fields
}

impl PhysicalLayer for MyHal {
    fn pl_set_mode(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
        // Configure UART for the specified mode
    }
    
    fn pl_transfer(&mut self, tx_data: &[u8], rx_buffer: &mut [u8]) -> IoLinkResult<usize> {
        // Transfer data over UART
    }
    
    // ... implement other required methods
}
```

## Dependencies

- `smlang`: State machine framework
- `heapless`: No-std collections
- `embedded-hal`: Hardware abstraction
- `nom`: Protocol parsing
- `cstr_core`: C string compatibility
- `log`: Logging framework

## Documentation

All APIs include references to specific sections of the IO-Link Specification v1.1.4:

```rust
/// See IO-Link v1.1.4 Section 8.4.2.1 for AL_GetInput_req behavior.
pub fn al_get_input_req(&mut self) -> IoLinkResult<ProcessData>
```

## Testing

```bash
# Run all tests
cargo test

# Run with mock HAL
cargo test --features mock-hal
```

## License

Licensed under either of

- GPL-3.0

"IO‑Link® is a registered trademark of the IO‑Link Community. This project is not affiliated with or endorsed by IO‑Link Community."

## Compliance

This implementation follows IO-Link Interface Specification Order No: 10.002, Version 1.1.4, June 2024.

## Contributing

Contributions are welcome! Please ensure:

1. All code references relevant specification sections
2. Tests are included for new functionality  
3. Documentation follows the established patterns
4. Code compiles for `no_std` targets
