//! # ISDU Frame Types for IO-Link Communication
//!
//! This module provides types and utilities for working with ISDU (Index-based Service Data Unit)
//! frames as defined in the IO-Link Specification v1.1.4. It includes representations for the
//! I-Service octet, I-Service codes, ISDU length codes, and flow control values used in ISDU
//! communication between IO-Link Master and Device.
//!
//! ## Contents
//! - [`IsduService`]: Bitfield struct representing the I-Service octet, which encodes both the
//!   I-Service type and the ISDU length.
//! - [`IsduIServiceCode`]: Enum of all valid I-Service codes (nibble values) as per Table A.16 and
//!   Table A.18 of the specification.
//! - [`IsduLengthCode`]: Enum for ISDU length codes as defined in Table A.17.
//! - [`IsduFlowCtrl`]: Enum for ISDU flow control values, including sequence counters, start, idle,
//!   reserved, and abort states.
//!
//! ## Specification References
//! - IO-Link Specification v1.1.4, Section A.5.2
//!   - Figure A.16: I-Service Octet Structure
//!   - Table A.16: Definitions of the nibble "I-Service"
//!   - Table A.17: ISDU length codes
//!   - Table A.18: ISDU syntax
//!
//! ## Usage
//! Use these types to encode, decode, and interpret ISDU frames for IO-Link protocol
//! implementations. The bitfield and enum representations ensure type safety and adherence to
//! protocol requirements.
//!
//! ## Example
//! ```rust
//! use iolinke_types::frame::isdu::{IsduService, IsduIServiceCode, IsduLengthCode, IsduFlowCtrl};
//!
//! // Create an ISDU Service octet for a Read Request with extended length
//! let mut service = IsduService::new();
//! service.set_i_service(IsduIServiceCode::ReadRequestIndex);
//! service.set_length(IsduLengthCode::Extended.into());
//!
//! // Interpret flow control value
//! let flow = IsduFlowCtrl::from_u8(0x10).unwrap();
//! assert_eq!(flow, IsduFlowCtrl::Start);
//! ```

use bitfields::bitfield;
use iolinke_macros::bitfield_support;

use core::option::{
    Option,
    Option::{None, Some},
};
use core::result::{
    Result,
    Result::{Err, Ok},
};

/// # ISDU I-Service Octet Structure
///
/// This struct represents the I-Service octet as defined in IO-Link Specification v1.1.4,
/// Section A.5.2 (see Figure A.16 and Tables A.16–A.18 in the spec).
///
/// The I-Service octet is structured as follows:
///
/// ```text
///  7       6      5     4     3    2     1     0  
/// +-----+-----+-----+-----+-----+-----+-----+-----+
/// |  I-Service (bits 7-4) | Length (bits 3-0) |
/// +-----------------------+---------------------+
/// |<------4 bits--------->|<----4 bits------->|
/// ```
///
/// - **Bits 0 to 3: Length**
///   - Encodes the length of the ISDU, as specified in Table A.17.
///   - This value determines the number of bytes in the ISDU transfer (see Table A.17 for valid codes).
///
/// - **Bits 4 to 7: I-Service**
///   - Encodes the I-Service type, as specified in Table A.16.
///   - This value determines the ISDU service (e.g., Read Request, Write Request, Success, Failure, etc.).
///   - See Table A.16 for the mapping of codes to service types.
///
/// ## Specification Reference
/// - IO-Link Specification v1.1.4, Section A.5.2, Figure A.16
/// - Table A.16: Definitions of the nibble "I-Service"
/// - Table A.17: ISDU length codes
/// - Table A.18: ISDU syntax
///
/// ## Usage
/// Use this struct to encode or decode the I-Service octet for ISDU communication.
/// The fields can be accessed or set using the generated getter/setter methods from the `bitfield` macro.
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct IsduService {
    /// Bits 0–3: Length (see Table A.17)
    #[bits(4)]
    pub length: u8,
    /// Bits 4–7: I-Service (see Table A.16)
    #[bits(4)]
    pub i_service: IsduIServiceCode,
}

/// ISDU I-Service codes as defined in IO-Link Specification v1.1.4, Table A.16 and Table A.18.
///
/// This enum represents the possible I-Service types for ISDU communication.
/// The discriminant values match the 4-bit codes used in the protocol.
///
/// # Specification Reference
/// - Table A.16: Definitions of the nibble "I-Service"
/// - Table A.18: ISDU syntax
#[bitfield_support]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsduIServiceCode {
    /// 0x0: No Service
    ///
    /// See Table A.16 – I-Service: No Service
    NoService = 0x0,

    /// 0x1: Write Request (Index)
    ///
    /// See Table A.16 – I-Service: Write Request (Index)
    WriteRequestIndex = 0x1,

    /// 0x2: Write Request (Index, Subindex)
    ///
    /// See Table A.16 – I-Service: Write Request (Index, Subindex)
    WriteRequestIndexSubindex = 0x2,

    /// 0x3: Write Request (Index, Index, Subindex)
    ///
    /// See Table A.16 – I-Service: Write Request (Index, Index, Subindex)
    WriteRequestIndexIndexSubindex = 0x3,

    /// 0x4: Write Failure
    ///
    /// See Table A.16 – I-Service: Write Failure
    WriteFailure = 0x4,

    /// 0x5: Write Success
    ///
    /// See Table A.16 – I-Service: Write Success
    WriteSuccess = 0x5,

    /// 0x9: Read Request (Index)
    ///
    /// See Table A.16 – I-Service: Read Request (Index)
    ReadRequestIndex = 0x9,

    /// 0xA: Read Request (Index, Subindex)
    ///
    /// See Table A.16 – I-Service: Read Request (Index, Subindex)
    ReadRequestIndexSubindex = 0xA,

    /// 0xB: Read Request (Index, Index, Subindex)
    ///
    /// See Table A.16 – I-Service: Read Request (Index, Index, Subindex)
    ReadRequestIndexIndexSubindex = 0xB,

    /// 0xC: Read Failure
    ///
    /// See Table A.16 – I-Service: Read Failure
    ReadFailure = 0xC,

    /// 0xD: Read Success
    ///
    /// See Table A.16 – I-Service: Read Success
    ReadSuccess = 0xD,
}

impl IsduIServiceCode {
    /// Create a new `IsduIServiceCode` enum variant
    pub const fn new() -> Self {
        Self::NoService
    }
    /// Try to convert a u8 value (lower 4 bits) to an `IsduIService` variant.
    ///
    /// Returns `None` if the value does not correspond to a defined I-Service code.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value & 0xF {
            0x0 => Some(Self::NoService),
            0x1 => Some(Self::WriteRequestIndex),
            0x2 => Some(Self::WriteRequestIndexSubindex),
            0x3 => Some(Self::WriteRequestIndexIndexSubindex),
            0x4 => Some(Self::WriteFailure),
            0x5 => Some(Self::WriteSuccess),
            0x9 => Some(Self::ReadRequestIndex),
            0xA => Some(Self::ReadRequestIndexSubindex),
            0xB => Some(Self::ReadRequestIndexIndexSubindex),
            0xC => Some(Self::ReadFailure),
            0xD => Some(Self::ReadSuccess),
            _ => None,
        }
    }

    /// Get the u8 for this I-Service.
    pub const fn into(self) -> u8 {
        self as u8
    }
}

/// ISDU Length codes as defined in IO-Link Specification v1.1.4, Table A.17.
///
/// This enum represents the possible length codes for ISDU communication.
/// The discriminant values match the 4-bit codes used in the protocol.
///
/// # Specification Reference
/// - Table A.17: ISDU length codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsduLengthCode {
    /// 0x1: Extended length
    ///
    /// See Table A.17 – ISDU length code: Extended length
    Extended = 0x1,
    // Other length codes (0x2..=0xF) can be added as needed.
}

impl IsduLengthCode {
    /// Try to convert a u8 value (lower 4 bits) to an `IsduLengthCode` variant.
    ///
    /// Returns `None` if the value does not correspond to a defined length code.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value & 0xF {
            0x1 => Some(Self::Extended),
            _ => None,
        }
    }

    /// Get the 4-bit code for this length.
    pub fn into(self) -> u8 {
        self as u8
    }
}

/// ISDU Flow Control values according to IO-Link Specification v1.1.4
///
/// Flow control values are used in ISDU (Index-based Service Data Unit) communication
/// to manage the transmission of service data between Master and Device.
///
/// ## Flow Control Definitions
///
/// | Value | Name | Description |
/// |-------|------|-------------|
/// | 0x00u8-0x0Fu8 | COUNT | M-sequence counter within an ISDU. Increments beginning with 1 after an ISDU START. Jumps back from 15 to 0 in the event of an overflow. |
/// | 0x10u8 | START | Start of an ISDU I-Service, i.e., start of a request or a response. For the start of a request, any previously incomplete services may be rejected. For a start request associated with a response, a Device shall send "No Service" until its application returns response data. |
/// | 0x11u8 | IDLE_1 | No request for ISDU transmission. |
/// | 0x12u8 | IDLE_2 | Reserved for future use. No request for ISDU transmission. |
/// | 0x13u8-0x1Eu8 | Reserved | Reserved for future use. |
/// | 0x1Fu8 | ABORT | Abort entire service. The Master responds by rejecting received response data. The Device responds by rejecting received request data and may generate an abort. |
///
/// # Examples
///
/// ```rust
/// use iolinke_types::frame::isdu::IsduFlowCtrl;
///
/// let start_value = IsduFlowCtrl::Start.as_u8();
/// assert_eq!(start_value, 0x10u8);
///
/// let abort_value = IsduFlowCtrl::Abort.as_u8();
/// assert_eq!(abort_value, 0x1Fu8);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsduFlowCtrl {
    /// M-sequence counter within an ISDU (0x00..=0x0F)
    Count(u8),
    /// Start of an ISDU I-Service (0x10)
    Start = 0x10,
    /// No request for ISDU transmission (0x11)
    Idle1 = 0x11,
    /// Reserved for future use. No request for ISDU transmission (0x12)
    Idle2 = 0x12,
    /// Reserved (0x13..=0x1E)
    Reserved(u8),
    /// Abort entire service (0x1F)
    Abort = 0x1F,
}

impl IsduFlowCtrl {
    /// Try to convert a u8 value to an `IsduFlowCtrl` variant.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00..=0x0F => Some(IsduFlowCtrl::Count(value)),
            0x10 => Some(IsduFlowCtrl::Start),
            0x11 => Some(IsduFlowCtrl::Idle1),
            0x12 => Some(IsduFlowCtrl::Idle2),
            0x13..=0x1E => Some(IsduFlowCtrl::Reserved(value)),
            0x1F => Some(IsduFlowCtrl::Abort),
            _ => None,
        }
    }

    /// Get the u8 value for this flow control.
    pub fn as_u8(&self) -> u8 {
        match *self {
            IsduFlowCtrl::Count(v) => v,
            IsduFlowCtrl::Start => 0x10,
            IsduFlowCtrl::Idle1 => 0x11,
            IsduFlowCtrl::Idle2 => 0x12,
            IsduFlowCtrl::Reserved(v) => v,
            IsduFlowCtrl::Abort => 0x1F,
        }
    }

    /// Get the u8 value for this flow control.
    pub fn into_bits(self) -> u8 {
        match self {
            IsduFlowCtrl::Count(v) => v,
            IsduFlowCtrl::Start => 0x10,
            IsduFlowCtrl::Idle1 => 0x11,
            IsduFlowCtrl::Idle2 => 0x12,
            IsduFlowCtrl::Reserved(v) => v,
            IsduFlowCtrl::Abort => 0x1F,
        }
    }
}
