//! Basic IO-Link Device Example
//!
//! This example demonstrates the basic usage of the IO-Link Device Stack.
//! It creates a device instance and runs a simple polling loop.
//!
//! ## Features Demonstrated
//!
//! - Device initialization and configuration
//! - Basic polling loop operation
//! - Error handling and recovery
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example basic
//! ```
//!
//! ## Expected Behavior
//!
//! The device will start in the Idle state and continuously poll all
//! protocol layers. Since this is a basic example without hardware
//! implementation, most operations will return `NoImplFound` errors,
//! which are handled gracefully by continuing the loop.

use std::{thread::sleep, time::Duration};

use iolinke_device::*;

fn main() {
    // Create a new IO-Link device instance
    let mut io_link_device = IoLinkDevice::new();

    // Set device identification parameters
    // These are required by the IO-Link specification
    io_link_device.set_device_id(0x1234, 0x56789ABC, 0x0001);

    println!("IO-Link Device started. Press Ctrl+C to stop.");

    // Main device loop
    loop {
        // Poll all protocol layers
        match io_link_device.poll() {
            Ok(()) => {
                // Device operating normally
                // In a real implementation, you might add some delay here
                // sleep(Duration::from_millis(10));
            }
            Err(IoLinkError::NoImplFound) => {
                // Feature not implemented yet, continue operation
                // This is expected in the basic example
                continue;
            }
            Err(e) => {
                // Handle other errors
                eprintln!("Device error: {:?}", e);
                break;
            }
        }
    }
}
