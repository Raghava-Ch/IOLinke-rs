//! Parameter Storage Example
//!
//! This example demonstrates the usage of the `declare_parameter_storage!` macro
//! to create a complete parameter storage system for IO-Link devices.
//!
//! ## Features Demonstrated
//!
//! - `declare_parameter_storage!` macro for creating parameter storage
//! - Custom data type support
//! - Parameter access control (ReadOnly, WriteOnly, ReadWrite)
//! - Index and subindex memory operations
//! - Parameter validation and constraints
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example test_parameter_storage
//! ```
//!
//! ## Expected Output
//!
//! The example will demonstrate parameter storage operations including:
//! - Setting and getting parameters
//! - Access control validation
//! - Custom data type handling
//! - Index memory operations
//! - Parameter information retrieval
//!
//! ## Specification Reference
//!
//! - Annex B.1: Direct Parameter Page 1 and 2
//! - Section 8.1.3: Parameter Access and Validation
//! - Annex B.8: Index assignment of data objects

use iolinke_macros::declare_parameter_storage;

use core::option::{
    Option,
    Option::{None, Some},
};
use core::result::{
    Result,
    Result::{Err, Ok},
};

/// Custom data type example for demonstration.
///
/// This struct shows how custom types can be used with the parameter
/// storage system. It implements conversion to/from bytes for storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SadReal {
    value: u8,
    value2: u8,
}

impl SadReal {
    /// Creates a new SadReal instance.
    fn new(value: u8) -> Self {
        Self { value, value2: 0 }
    }

    /// Converts the struct to a byte array for storage.
    fn to_bytes(&self) -> [u8; 2] {
        [self.value, self.value2]
    }
}

// Declare parameter storage using the macro
// This creates a complete parameter storage system with validation
declare_parameter_storage! {
    // Note: Index 0x0000 and 0x0001 can only have subindexes 0x00-0x0F
    // Index, Subindex, Length, Range, Access, Type, DefaultValue
    (0x0000, 0x00, 2, 0..1, ReadOnly, SadReal, &[100, 11]),  // Direct Parameter Page 1
    (0x0000, 0x0C, 1, 0..0, WriteOnly, u8, &[0]),      // Split subindex space
    (0x0001, 0x09, 2, 0..1, ReadWrite, u16, &[42, 1]),   // Fixed: length 2 for range 0..=1
    (0x0002, 0x00, 1, 0..0, WriteOnly, u8, &[0]),      // System-Command, 0..=0 is 1 value
}

fn main() {
    println!("Macro expanded successfully");
    let mut storage = ParameterStorage::new();
    println!("Storage created successfully");

    // Test setting a parameter
    match storage.set_parameter(0x0000, 0x0C, &[42]) {
        Ok(()) => println!("Successfully set parameter 0x0000:0x0C to 42"),
        Err(e) => println!("Failed to set parameter: {:?}", e),
    }

    // Test generic get/set functions
    match storage.get_parameter(0x0000, 0x00) {
        Ok(data) => println!("Parameter 0x0000:00 value: {:?}", data),
        Err(e) => println!("Failed to get parameter: {:?}", e),
    }

    // Test setting a parameter (length 2 for u16)
    match storage.set_parameter(0x0001, 0x09, &[42, 0]) {
        Ok(()) => println!("Successfully set parameter 0x0001:0x09 to [42, 0]"),
        Err(e) => println!("Failed to set parameter: {:?}", e),
    }

    // Test getting the value we just set
    match storage.get_parameter(0x0001, 0x09) {
        Ok(data) => println!("Parameter 0x0001:0x09 value: {:?}", data),
        Err(e) => println!("Failed to get parameter: {:?}", e),
    }

    // Test parameter info lookup
    match storage.get_parameter_info(0x0000, 0x00) {
        Ok(info) => println!(
            "Parameter 0x0000:00 info: length={}, range={:?}",
            info.length, info.range
        ),
        Err(e) => println!("Failed to get parameter info: {:?}", e),
    }

    // Test setting a read-only parameter (should fail)
    match storage.set_parameter(0x0000, 0x00, &[50]) {
        Ok(()) => println!("Unexpectedly set read-only parameter"),
        Err(e) => println!("Correctly denied access to read-only parameter: {:?}", e),
    }

    // Test setting a write-only parameter
    match storage.set_parameter(0x0002, 0x00, &[123]) {
        Ok(()) => println!("Successfully set write-only parameter 0x0002:00 to 123"),
        Err(e) => println!("Failed to set write-only parameter: {:?}", e),
    }

    // Test setting a custom type (this should work for write-only)
    let sad_real = SadReal::new(100);
    match storage.set_parameter(0x0000, 0x0C, &sad_real.to_bytes()) {
        Ok(()) => println!("Successfully set custom type parameter"),
        Err(e) => println!("Failed to set custom type parameter: {:?}", e),
    }

    // Test reading entire index memory
    match storage.read_index_memory(0x0000) {
        Ok(data) => println!("Index 0x0000 memory: {:?}", data),
        Err(e) => println!("Failed to read index memory: {:?}", e),
    }

    // Test writing entire index memory
    let index_data = [100, 150]; // Two bytes for the two subindexes in index 0x0000
    match storage.write_index_memory(0x0000, &index_data) {
        Ok(()) => println!("Successfully wrote index 0x0000 memory"),
        Err(e) => println!("Failed to write index memory: {:?}", e),
    }

    // Test getting all parameter information
    let all_params = storage.get_all_parameters();
    println!("Total parameters defined: {}", all_params.len());
    for param in all_params {
        println!(
            "  Index 0x{:04X}:0x{:02X} - {} ({} bytes, {:?})",
            param.index, param.subindex, param.data_type, param.length, param.access
        );
    }

    // Test constraint validation
    match storage.validate_constraints() {
        Ok(()) => println!("All parameter constraints are valid"),
        Err(e) => println!("Parameter constraint validation failed: {:?}", e),
    }

    // Test getting parameters by index
    // let index_0x0000_params = storage.get_parameters_by_index(0x0000);
    // println!("Parameters for index 0x0000: {}", index_0x0000_params.len());
    // for param in index_0x0000_params {
    //     println!("  Subindex 0x{:02X}: {} ({} bytes, {:?})",
    //             param.subindex, param.data_type, param.length, param.access);
    // }

    println!("All tests completed successfully!");
}
