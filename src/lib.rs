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

pub mod application;
pub mod command;
pub mod dl_mode;
pub mod event_handler;
pub mod event_sm;
#[cfg(feature = "std")]
pub mod ffi;
pub mod hal;
pub mod isdu;
pub mod message;
pub mod on_request;
pub mod parameter;
pub mod process_data;
pub mod sm;
pub mod storage;
pub mod types;

mod pl;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub use test_utils::MockHal;

// Re-export main traits and types
pub use application::{ApplicationLayer, ApplicationLayerImpl};
pub use hal::{IoLinkHal};
pub use dl_mode::{DlModeHandler, DlModeState};
pub use message::MessageHandler;
pub use types::*;

/// Simple IO-Link device implementation
pub struct IoLinkDevice {
    dl_mode: DlModeHandler,
    message_handler: MessageHandler,
    application: ApplicationLayerImpl,
}

impl IoLinkDevice {
    /// Create a new simple IO-Link device
    pub fn new() -> Self {
        Self {
            dl_mode: DlModeHandler::new(),
            message_handler: MessageHandler::new(),
            application: ApplicationLayerImpl::new(),
        }
    }

    /// Step 1: Set device identification
    pub fn set_device_id(&mut self, vendor_id: u16, device_id: u32, function_id: u16) {
        let device_identification = DeviceIdentification {
            vendor_id,
            device_id,
            function_id,
            reserved: 0,
        };
        self.application.set_device_id(device_identification);
    }

    /// Step 3: Basic polling function
    pub fn poll(&mut self) -> IoLinkResult<()> {
        // Poll all state machines
        self.dl_mode.poll()?;
        self.message_handler.poll(&mut self.dl_mode)?;
        self.application.poll()?;
        Ok(())
    }
}