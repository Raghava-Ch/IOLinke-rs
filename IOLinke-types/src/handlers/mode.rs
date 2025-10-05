//! # IO-Link Data Link Layer Mode Handler Module
//!
//! This module provides traits and types for handling Data Link Layer (DL) mode indications and
//! read/write operations in IO-Link devices. It defines interfaces for reporting DL operating status
//! and accessing device parameters via the page communication channel.
//!
//! ## Key Traits
//! - [`DlModeInd`]: Handles DL_Mode indication events.
//! - [`DlReadWriteInd`]: Handles DL_Read and DL_Write operations.
//!
//! ## Key Types
//! - [`DlMode`]: Enumerates DL mode states as per IO-Link specification.
//!
//! ## Specification Reference
//! - IO-Link v1.1.4, Section 7.2.1.14 (DL_Mode)
//!
//! This module is intended for use in IO-Link device implementations to monitor and control
//! Data Link Layer operating states and parameter access.

use crate::custom::IoLinkResult;

/// Trait for handling DL_Mode indication events.
///
/// The `DlModeInd` trait defines a callback for reporting when the Data Link Layer (DL)
/// reaches a certain operating status. This is typically used by System Management to
/// monitor the state of the DL. The method corresponds to the DL_Mode service as described
/// in section 7.2.1.14 of the IO-Link specification.
pub trait DlModeInd {
    /// See 7.2.1.14 DL_Mode
    /// The DL uses the DL_Mode service to report to System Management that a certain operating
    /// status has been reached. The parameters of the service primitives are listed in Table 29.
    fn dl_mode_ind(&mut self, mode: DlMode) -> IoLinkResult<()>;
}

/// Trait defining the interface for reading and writing device parameters via the page communication channel.
///
/// This trait provides methods corresponding to the DL_Write and DL_Read services as specified in the IO-Link protocol.
/// Implementors of this trait can handle requests to read from and write to device parameters, typically used by System Management.
///
/// # Methods
/// - [`dl_write_ind`]: Writes a value to a device parameter at the specified address.
/// - [`dl_read_ind`]: Reads a value from a device parameter at the specified address.
///
/// # References
/// - IO-Link Specification 7.2.1.4 DL_Read
/// - IO-Link Specification 7.2.1.5 DL_Write
pub trait DlReadWriteInd {
    /// See 7.2.1.5 DL_Write
    /// The DL_Write service is used by System Management to write a Device parameter value to
    /// the Device via the page communication channel. The parameters of the service primitives are
    /// listed in Table 20.
    fn dl_write_ind(&mut self, address: u8, value: u8) -> IoLinkResult<()>;

    /// 7.2.1.4 DL_Read
    /// The DL_Read service is used by System Management to read a Device parameter value via
    /// the page communication channel. The parameters of the service primitives are listed in Table 19.
    fn dl_read_ind(&mut self, address: u8) -> IoLinkResult<()>;
}

/// Data Link Layer mode as per Section 7.2.1.14.
///
/// The DL mode indicates the current state of the data link
/// layer state machine.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 7.2.1.14: DL_Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DlMode {
    /// Data link layer is inactive
    Inactive,
    /// COM1 mode is established
    Com1,
    /// COM2 mode is established
    Com2,
    /// COM3 mode is established
    Com3,
    /// Communication lost
    Comlost,
    /// Handler changed to the EstablishCom state
    Estabcom,
    /// Handler changed to the STARTUP state
    Startup,
    /// Handler changed to the PREOPERATE state
    PreOperate,
    /// Handler changed to the OPERATE state
    Operate,
}
