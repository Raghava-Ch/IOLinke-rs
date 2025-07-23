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
//! ## Usage
//!
//! ```rust,no_run
//! use iolink_device_stack::{ApplicationLayer, PhysicalLayer, DeviceStack};
//!
//! // Initialize the device stack with HAL implementations
//! let mut device = DeviceStack::new(hal_implementation);
//! 
//! // Main application loop
//! loop {
//!     device.poll()?;
//! }
//! ```

pub mod application;
pub mod command;
pub mod dl_mode;
pub mod event;
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

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub use test_utils::MockHal;

// Re-export main traits and types
pub use application::{ApplicationLayer, ApplicationLayerImpl};
pub use hal::{PhysicalLayer};
pub use dl_mode::{DlModeHandler, DlModeState};
pub use message::MessageHandler;
pub use types::*;