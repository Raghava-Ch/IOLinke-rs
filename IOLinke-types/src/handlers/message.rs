//! # IO-Link Message Handler Module
//!
//! This module defines traits and types for handling message-related operations in IO-Link devices.
//! It provides interfaces for signaling exceptional operations and managing message handler states,
//! including communication mode and reset/inactive commands.
//!
//! ## Key Traits
//! - [`MsgHandlerInfo`]: Handles message handler information signaling.
//!
//! ## Key Types
//! - [`MHInfo`]: Enumerates message handler information types (e.g., communication lost, illegal message type).
//! - [`MhConfState`]: Represents the configuration state of the message handler, including IO-Link mode.
//!
//! ## Specification Reference
//! - IO-Link v1.1.4, Section 7.2.2.6 (MHInfo)
//!
//! This module is intended for use in IO-Link device implementations to process message handler events
//! and manage communication state transitions.

use crate::handlers::sm;

/// Trait for message handler operations in bw modules
pub trait MsgHandlerInfo {
    /// See 7.2.2.6 MHInfo
    /// The service MHInfo signals an exceptional operation within the message handler. The
    /// parameters of the service are listed in Table 39.
    fn mh_info(&mut self, mh_info: MHInfo);
}

/// All the message handler information type
/// See 7.2.2.6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MHInfo {
    /// lost communication
    COMlost,
    /// unexpected M-sequence type detected
    IllegalMessagetype,
    /// Checksum error detected
    ChecksumMismatch,
}

/// The `IoLinkMode` specifies the operational mode for the IO-Link communication.
/// This variant is used to indicate or change the mode of the IO-Link device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MhConfState {
    /// Represents the IO-Link communication mode.
    /// This variant holds an `IoLinkMode` value from the `sm` module,
    /// which specifies the current mode of IO-Link communication.
    /// Represents a command message containing the current IO-Link mode.
    Com(sm::IoLinkMode),
    /// Represents a command message to reset the device.
    /// This variant indicates that the device should perform a reset operation.
    Active,
    /// Represents a command message to set the device to an inactive state.
    /// This variant indicates that the device should transition to an inactive state.
    Inactive,
}
