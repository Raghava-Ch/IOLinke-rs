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
mod utils;
mod config;
mod storage;
mod system_management;

#[cfg(feature = "std")]
pub mod ffi;
mod types;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub use test_utils::*;

// Re-export main traits and types
pub use types::*;

/// Simple IO-Link device implementation
pub struct IoLinkDevice {
    physical_layer: pl::physical_layer::PhysicalLayer,
    dl: dl::DataLinkLayer,
    system_management: system_management::SystemManagement,
    application_layer: al::ApplicationLayer,
}

impl IoLinkDevice {
    /// Create a new simple IO-Link device
    pub fn new() -> Self {
        Self {
            system_management: system_management::SystemManagement::default(),
            physical_layer: pl::physical_layer::PhysicalLayer::default(),
            dl: dl::DataLinkLayer::default(),
            application_layer: al::ApplicationLayer::default(),
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
        self.application_layer.poll(&mut self.dl)?;
        self.dl.poll(
            &mut self.system_management,
            &mut self.physical_layer,
            &mut self.application_layer,
        )?;
        self.system_management.poll(
            &mut self.application_layer,
            &mut self.physical_layer,
        )?;
        Ok(())
    }
}

impl al::ApplicationLayerReadWriteInd for IoLinkDevice {
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()> {
        self.application_layer.al_read_ind(index, sub_index)
    }

    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()> {
        self.application_layer.al_write_ind(index, sub_index, data)
    }

    fn al_abort_ind(&mut self) -> IoLinkResult<()> {
        Err(IoLinkError::FuncNotAvailable)
    }
}

impl al::ApplicationLayerProcessDataInd for IoLinkDevice {
    fn al_set_input_ind(&mut self) -> IoLinkResult<()> {
        todo!();
    }

    fn al_pd_cycle_ind(&mut self) {
        todo!();
    }

    fn al_get_output_ind(&mut self) -> IoLinkResult<()> {
        todo!();
    }

    fn al_new_output_ind(&mut self) -> IoLinkResult<()> {
        todo!();
    }

    fn al_control(&mut self, control_code: u8) -> IoLinkResult<()> {
        todo!();
    }
    
}
