//! # IO-Link Device Handler Modules
//!
//! This directory contains submodules for handling various aspects of IO-Link device communication,
//! including command processing, data storage, event signaling, ISDU transport, message handling,
//! mode management, on-request data, process data, protocol timing, parameter memory, and system management.
//!
//! Each submodule provides traits, types, and logic for implementing protocol-compliant device services
//! and state machines as specified in the IO-Link standard.
//!
//! ## Submodules
//! - [`command`]: Master command and control code handling.
//! - [`ds`]: Data storage command handling.
//! - [`event`]: Event signaling and memory management.
//! - [`isdu`]: Indexed Service Data Unit transport.
//! - [`message`]: Message handler operations.
//! - [`mode`]: Data Link Layer mode management.
//! - [`od`]: On-request data services.
//! - [`pd`]: Process data input/output handling.
//! - [`pl`]: Protocol timer management.
//! - [`pm`]: Parameter memory and index assignment.
//! - [`sm`]: System management operations.
//!
//! These modules are intended for use in IO-Link device implementations to provide protocol-compliant
//! service interfaces and state management for all major device functions.

pub mod command;
pub mod ds;
pub mod event;
pub mod isdu;
pub mod message;
pub mod mode;
pub mod od;
pub mod pd;
pub mod pl;
pub mod pm;
pub mod sm;
