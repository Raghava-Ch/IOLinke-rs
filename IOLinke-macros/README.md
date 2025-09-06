# IOLinke-macros

Procedural macros for the IO-Link Device Stack implementation. This crate provides macros to simplify development of IO-Link devices according to IO-Link Specification v1.1.4.

## Features

### `#[derive(IoLinkDevice)]`
Automatically implements device identification methods:

```rust
use iolinke_macros::IoLinkDevice;

#[derive(IoLinkDevice)]
#[iolink(vendor_id = 0x1234, device_id = 0x56789ABC)]
struct MyDevice {
    // Device-specific fields
}

// Generated methods:
// - MyDevice::vendor_id() -> u16
// - MyDevice::device_id() -> u32
// - MyDevice::function_id() -> u16
// - MyDevice::device_identification() -> DeviceIdentification
```

### `#[iolink_parameter]`
Generates ISDU parameter access methods:

```rust
use iolinke_macros::iolink_parameter;

#[iolink_parameter(index = 0x10, access = "ReadWrite", data_type = "UInt16")]
pub struct VendorName;
```

### `#[iolink_state_machine]`
Creates state machine boilerplate:

```rust
use iolinke_macros::iolink_state_machine;

#[iolink_state_machine]
enum DlModeState {
    Inactive,
    Startup,
    Preoperate,
    Operate,
}
```

### `#[iolink_frame]`
Generates frame validation methods:

```rust
use iolinke_macros::iolink_frame;

#[iolink_frame(frame_type = "MSequence")]
struct MSequenceFrame {
    mc: u8,
    ckt: u8,
    data: [u8; 32],
}
```

## IO-Link Specification Compliance

All macros are designed to generate code that complies with IO-Link Specification v1.1.4:

- Device identification (Section 7.3.4.1)
- ISDU parameters (Section 8.1.3)
- State machines (Section 6.3)
- Frame formats (Section 5.4)
- Event handling (Section 8.3)

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
iolinke-macros = { path = "../IOLinke-macros" }
```

Then import and use the macros in your code:

```rust
use iolinke_macros::{IoLinkDevice, iolink_state_machine, iolink_frame};
```

## License

Licensed under GPL-3 license.
