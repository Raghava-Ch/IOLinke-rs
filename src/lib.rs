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
mod config;
mod dl;
mod parameter;
mod pl;
mod storage;
mod system_management;
mod utils;
mod data_storage;

#[cfg(feature = "std")]
pub mod ffi;
mod types;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub use test_utils::*;

use crate::system_management::{SmResult, SystemManagementInd};
// Re-export main traits and types
pub use types::*;

/// Simple IO-Link device implementation
pub struct IoLinkDevice<'a> {
    physical_layer: pl::physical_layer::PhysicalLayer,
    dl: dl::DataLinkLayer<'a>,
    system_management: system_management::SystemManagement,
    services: al::services::ApplicationLayerServices,
    application: al::ApplicationLayer<'a>,
}

impl<'a> IoLinkDevice<'a> {
    /// Create a new simple IO-Link device
    pub fn new() -> Self {
        Self {
            system_management: system_management::SystemManagement::default(),
            physical_layer: pl::physical_layer::PhysicalLayer::default(),
            dl: dl::DataLinkLayer::default(),
            services: al::services::ApplicationLayerServices::default(),
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
    pub fn poll(&'a mut self) -> IoLinkResult<()> {
        // Poll all state machines
        self.application.poll(&mut self.dl)?;
        self.dl.poll(
            &mut self.system_management,
            &mut self.physical_layer,
            &mut self.application,
        )?;
        Ok(())
    }
}

impl<'a> system_management::SystemManagementInd for IoLinkDevice<'a> {
    fn sm_device_mode_ind(&mut self, mode: types::DeviceMode) -> SmResult<()> {
        todo!()
    }
}

impl<'a> system_management::SystemManagementCnf for IoLinkDevice<'a> {
    fn sm_set_device_com_cnf(&self, result: SmResult<()>) -> SmResult<()> {
        todo!()
    }
    fn sm_get_device_com_cnf(
        &self,
        result: SmResult<&system_management::DeviceCom>,
    ) -> SmResult<()> {
        todo!()
    }
    fn sm_set_device_ident_cnf(&self, result: SmResult<()>) -> SmResult<()> {
        todo!()
    }
    fn sm_get_device_ident_cnf(
        &self,
        result: SmResult<&system_management::DeviceIdent>,
    ) -> SmResult<()> {
        todo!()
    }
    fn sm_set_device_mode_cnf(&self, result: SmResult<()>) -> SmResult<()> {
        todo!()
    }
}
