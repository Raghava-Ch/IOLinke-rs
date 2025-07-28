#![no_std]
#![warn(missing_docs)]

//! # IO-Link Device Stack
//!
//! A modular, maintainable, and portable IO-Link Device/Slave stack implementation
//! compliant with IO-Link Specification Version 1.1.4 (June 2024).
//!
//! This library provides a complete IO-Link device implementation targeting
//! embedded microcontrollers with `#![no_std]` compatibility.
//!
//! ## Architecture
//!
//! The stack is built around 12 state machines that handle different aspects
//! of the IO-Link protocol:
//!
//! - Data Link Layer: DL-Mode Handler, Message Handler
//! - Application Layer: Process Data, On-request Data, ISDU, Command, Event handlers
//! - System: Application Layer, Event State Machine, System Management
//! - Storage: Parameter Manager, Data Storage
//!
//! ## Macros
//!
//! This crate integrates with `iolinke-macros` to provide convenient procedural
//! macros for common IO-Link patterns.

// Re-export macros for convenience
pub use iolinke_macros::*;

mod al;
mod dl;
mod pl;
mod parameter;
mod sm;
mod storage;

#[cfg(feature = "std")]
pub mod ffi;
mod types;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub use test_utils::MockHal;

// Re-export main traits and types
pub use al::{ApplicationLayer,};
pub use types::*;

/// Simple IO-Link device implementation
pub struct IoLinkDevice {
    system_management: sm::SystemManagement,
    physical_layer: pl::physical_layer::PhysicalLayer,
    dl: dl::DataLinkLayer,
    application: al::ApplicationLayer,
}

impl IoLinkDevice {
    /// Create a new simple IO-Link device
    pub fn new() -> Self {
        Self {
            system_management: sm::SystemManagement::default(),
            physical_layer: pl::physical_layer::PhysicalLayer::default(),
            dl: dl::DataLinkLayer::default(),
            application: al::ApplicationLayer::default(),
        }
    }

    /// Step 1: Set device identification
    pub fn set_device_id(&mut self, vendor_id: u16, device_id: u32, function_id: u16) {
        let _device_identification = DeviceIdentification {
            vendor_id,
            device_id,
            function_id,
            reserved: 0,
        };
    }

    /// Step 3: Basic polling function
    pub fn poll(&mut self) -> IoLinkResult<()> {
        // Poll all state machines
        self.dl.poll(&mut self.system_management, &mut self.physical_layer)?;
        self.application.poll()?;
        Ok(())
    }
}