# IO-Link Device Stack - AI Assistant Instructions

## Project Overview
This is a Rust implementation of an IO-Link Device/Slave stack compliant with IO-Link Specification v1.1.4 (June 2024). The project targets embedded microcontrollers using `#![no_std]` and provides C-compatible FFI bindings.

## Architecture Principles

### Core Constraints
- **No-std embedded Rust**: All code must be `#![no_std]` compatible for bare-metal targets (Cortex-M)
- **State machine driven**: Use `smlang` crate for all 12 protocol state machines with polling-based execution
- **HAL abstraction**: Platform portability through trait-based hardware abstraction
- **C interoperability**: Expose C-compatible API via `#[no_mangle] extern "C"` functions

### Key Components (12 State Machines)
Each component uses `smlang` with polling methods, never interrupts or async:
- DL-Mode Handler, Message Handler, Process Data Handler
- On-request Data Handler, ISDU Handler, Command Handler
- Event Handler, Application Layer, Event State Machine
- System Management, Parameter Manager, Data Storage

## Development Patterns

### State Machine Implementation
```rust
// Always use smlang with polling pattern
use smlang::statemachine;

statemachine! {
    transitions: {
        // State transitions based on IO-Link spec
    }
}

impl MyStateMachine {
    pub fn poll(&mut self) -> Result<(), Error> {
        // Polling-based state progression
    }
}
```

### Public API Structure
- **ApplicationLayer trait**: Implements Section 8.4 methods (AL_GetInput_req, AL_SetOutput_req, ISDU access, events)
- **PhysicalLayer trait**: Implements Section 5.2/Figure 13 (PL_SetMode, PL_Transfer, PL_WakeUp, PL_Status)

### Documentation Requirements
All public APIs must reference specific IO-Link specification sections:
```rust
/// See IO-Link v1.1.4 Section 8.4.2.1 for AL_GetInput_req behavior.
pub fn al_get_input_req(&mut self) -> Result<InputData, Error> { }
```

## Key Dependencies
- `smlang`: State machine framework (use latest version)
- `heapless`: No-std collections
- `embedded-hal`: Hardware abstraction
- `cstr_core`: C string compatibility
- `log`: Logging framework

## File Organization
```
src/
├── lib.rs          # Main library entry point
├── hal.rs          # Hardware abstraction traits
├── types.rs        # IO-Link protocol types and enums
├── ffi.rs          # C FFI bindings
├── application.rs  # Application Layer API
├── dl_mode.rs      # Data Link Mode Handler
├── message.rs      # Message Handler
├── process_data.rs # Process Data Handler
├── on_request.rs   # On-request Data Handler
├── isdu.rs         # ISDU Handler
├── command.rs      # Command Handler
├── event.rs        # Event Handler
├── event_sm.rs     # Event State Machine
├── sm.rs           # System Management
├── parameter.rs    # Parameter Manager
├── storage.rs      # Data Storage
└── test_utils.rs   # Testing utilities
```

## Code Generation
- Use `cbindgen` to generate C headers from Rust FFI functions
- Ensure all C-exported functions use `#[no_mangle] extern "C"`
- Maintain consistent error handling across FFI boundary

## Testing Strategy
- Unit tests for each state machine component
- Integration tests for protocol compliance
- Mock HAL implementations for testing without hardware
- Reference IO-Link specification sections in test documentation

## Specification Reference
All implementation decisions should reference the IO-Link Interface Specification v1.1.4 (IOL-Interface-Spec_10002_V114_Jun24.pdf) in the project root.
