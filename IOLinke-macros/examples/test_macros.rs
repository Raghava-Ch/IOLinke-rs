//! Test Macros Example
//!
//! This example demonstrates the usage of the basic IO-Link macros
//! for parameter addressing and master commands.
//!
//! ## Features Demonstrated
//!
//! - `direct_parameter_address!` macro for parameter addressing
//! - `master_command!` macro for master command values
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example test_macros
//! ```
//!
//! ## Expected Output
//!
//! The example will print the hexadecimal addresses and command values
//! for various IO-Link parameters and commands as defined in the
//! IO-Link specification v1.1.4.
//!
//! ## Specification Reference
//!
//! - Annex B.1: Direct Parameter Page 1 addresses
//! - Section 7.2: Master Commands and values

use iolinke_macros::{direct_parameter_address, master_command};

fn main() {
    // Test direct_parameter_address macro
    // This macro resolves parameter names to their standardized addresses
    let vendor_id1_addr = direct_parameter_address!(VendorID1);
    println!("VendorID1 address: 0x{:02X}", vendor_id1_addr);
    
    let device_id1_addr = direct_parameter_address!(DeviceID1);
    println!("DeviceID1 address: 0x{:02X}", device_id1_addr);
    
    // Test master_command macro
    // This macro resolves command names to their standardized values
    let fallback_cmd = master_command!(Fallback);
    println!("Fallback command: 0x{:02X}", fallback_cmd);
    
    let device_ident_cmd = master_command!(DeviceIdent);
    println!("DeviceIdent command: 0x{:02X}", device_ident_cmd);
}
