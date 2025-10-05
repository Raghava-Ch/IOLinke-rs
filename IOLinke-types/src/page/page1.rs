//! # IO-Link Page 1 Types
//!
//! This module defines types and bitfield structures for IO-Link Direct Parameter Page 1,
//! as specified in IO-Link Specification v1.1.4, Annex B.1. These types are used to encode
//! and decode device parameters exchanged between IO-Link Masters and Devices.
//!
//! ## Contents
//!
//! - `MasterCommand`: Enumeration of all master commands used in IO-Link communication.
//! - `CycleTime`: Bitfield representing the Cycle Time parameter, encoding minimum supported cycle time.
//! - `MsequenceCapability`: Bitfield representing the device's ISDU and M-sequence capabilities.
//! - `RevisionId`: Bitfield for protocol revision identifier (major and minor).
//! - `ProcessDataIn`: Bitfield for Process Data input parameters (Device to Master).
//! - `ProcessDataOut`: Bitfield for Process Data output parameters (Master to Device).
//! - `DeviceIdent`: Structure for device identification (Vendor ID, Device ID, Function ID).
//!
//! ## Specification References
//!
//! - IO-Link Specification v1.1.4, Annex B.1: Direct Parameter Page 1
//! - Section B.1.2: Master Commands
//! - Section B.1.3: Cycle Time
//! - Section B.1.4: M-sequence Capability, RevisionID
//! - Section B.1.6: Process Data In/Out
//! - Section 7.3.4.1: Device Identification
//!
//! ## Usage
//!
//! Use these types to parse, construct, and validate IO-Link Page 1 parameters for device configuration,
//! identification, and communication capability reporting.

use crate::custom::IoLinkError;
use bitfields::bitfield;
use core::convert::From;
use core::result::Result::Ok;

/// All the master commands used in IO-Link
/// See Table B.1.2 – Types of MasterCommands
/// Also see Table 55 – Control codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasterCommand {
    /// MasterCommand {Fallback} = 0x5Au8
    Fallback = 0x5A,
    /// MasterCommand {MasterIdent} = 0x95u8
    MasterIdent = 0x95,
    /// MasterCommand {DeviceIdent} = 0x96u8
    DeviceIdent = 0x96,
    /// MasterCommand {DeviceStartup} = 0x97u8
    DeviceStartup = 0x97,
    /// MasterCommand {ProcessDataOutputOperate} = 0x98u8
    ProcessDataOutputOperate = 0x98,
    /// MasterCommand {DeviceOperate} = 0x99u8
    /// This command is also known as PDOUTINVALID
    DeviceOperate = 0x99,
    /// MasterCommand {DevicePreoperate} = 0x9Au8
    DevicePreOperate = 0x9A,
}

impl Into<u8> for MasterCommand {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for MasterCommand {
    type Error = IoLinkError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x5A => Ok(MasterCommand::Fallback),
            0x95 => Ok(MasterCommand::MasterIdent),
            0x96 => Ok(MasterCommand::DeviceIdent),
            0x97 => Ok(MasterCommand::DeviceStartup),
            0x98 => Ok(MasterCommand::ProcessDataOutputOperate),
            0x99 => Ok(MasterCommand::DeviceOperate),
            0x9A => Ok(MasterCommand::DevicePreOperate),
            _ => Err(IoLinkError::InvalidParameter),
        }
    }
}

/// # CycleTime
///
/// This struct represents the Cycle Time parameter as defined by the IO-Link specification (v1.1.4, Section B.1.3, Table B.3 and Figure B.2).
///
/// The Cycle Time parameter is encoded as a single byte, which informs the IO-Link Master about the minimum supported cycle time of the device.
///
/// ## Bit Layout
/// ```text
/// | Bits 7-6   | Bits 5-0         |
/// |------------|------------------|
/// | Time Base  | Multiplier (M)   |
/// ```
/// - **Bits 7-6 (`time_base`)**: Encodes the time base unit for the cycle time.
///     - `0b00` = 0.1 ms
///     - `0b01` = 0.4 ms
///     - `0b10` = 1.6 ms
///     - `0b11` = Reserved
/// - **Bits 5-0 (`multiplier`)**: Multiplier value (0..=63) to be used with the time base.
///
/// ## Cycle Time Calculation
///
/// The minimum cycle time in milliseconds is calculated as:
///
/// ```text
/// CycleTime = (multiplier * time_base_unit)
/// ```
///
/// Where `time_base_unit` is determined by the value of `time_base`.
///
/// ## Specification Reference
/// - IO-Link Specification v1.1.4, Section B.1.3, Table B.3, Figure B.2
///
/// ## Usage
///
/// Use this struct to encode or decode the Cycle Time parameter for the device's parameter page 1.
/// The fields can be accessed or set using the generated getter/setter methods from the `bitfield` macro.
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct CycleTime {
    /// Bits 0–5: Multiplier (M)
    #[bits(6)]
    pub multiplier: u8,
    /// Bits 6–7: Time Base
    #[bits(2)]
    pub time_base: u8,
}

///
/// Represents the M-sequenceCapability parameter as defined in IO-Link Specification v1.1.4, Section B.1.4 (see Figure B.3).
///
/// This parameter encodes the device's support for ISDU communication and the available M-sequence types
/// during the OPERATE and PREOPERATE states. The structure of the byte is as follows:
///
/// ```text
///  7   6   5   4   3   2   1   0
/// +---+---+---+---+---+---+---+---+
/// | R | R |PRE|PRE| O | O | O | I |
/// +---+---+---+---+---+---+---+---+
///   |   |   |   |   |   |   |   +-- Bit 0: ISDU (0 = not supported, 1 = supported)
///   |   |   |   |   +---+---+------ Bits 1-3: OPERATE M-sequence code
///   |   +---+---+------------------ Bits 4-5: PREOPERATE M-sequence code
///   +---+-------------------------- Bits 6-7: Reserved (must be 0)
/// ```
///
/// - **Bit 0 (ISDU):** Indicates whether the ISDU communication channel is supported.
///   - 0: ISDU not supported
///   - 1: ISDU supported
/// - **Bits 1-3 (OPERATE M-sequence code):** Codes the available M-sequence type during the OPERATE state.
/// - **Bits 4-5 (PREOPERATE M-sequence code):** Codes the available M-sequence type during the PREOPERATE state.
/// - **Bits 6-7 (Reserved):** Reserved, must be set to 0.
///
/// # Reference
/// - IO-Link Specification v1.1.4, Section B.1.4, Figure B.3
/// - Table B.4 – Values of ISDU

#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct MsequenceCapability {
    /// Bit 1: ISDU
    #[bits(1)]
    pub isdu: bool,
    /// Bits 2-4: OPERATE M-sequence code
    #[bits(3)]
    pub operate_m_sequence: u8,
    /// Bits 5-6: PREOPERATE M-sequence code
    #[bits(2)]
    pub preoperate_m_sequence: u8,
    /// Bits 7-8: Reserved
    #[bits(2)]
    __: u8, // Reserved bit, must be 0
}

/// Protocol revision identifier as per Annex B.1.
///
/// This bitfield contains the major and minor revision numbers
/// of the IO-Link protocol version implemented by the device.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex B.1: Direct Parameter Page 1
/// - Section B.1.4: RevisionID parameter
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct RevisionId {
    /// Bits 0 to 3: Minor Revision
    ///
    /// These bits contain the minor digit of the version number,
    /// for example 0 for the protocol version 1.0. Permissible
    /// values for MinorRev are 0x0 to 0xF.
    #[bits(4)]
    pub minor_rev: u8,

    /// Bits 4 to 7: Major Revision
    ///
    /// These bits contain the major digit of the version number,
    /// for example 1 for the protocol version 1.0. Permissible
    /// values for MajorRev are 0x0 to 0xF.
    #[bits(4)]
    pub major_rev: u8,
}

/// Represents the ProcessDataIn parameter as defined in IO-Link Specification v1.1.4 Section B.1.6.
///
/// The ProcessDataIn parameter is a single byte (u8) structured as follows:
///
/// ```text
///  7   6   5   4   3   2   1   0
/// +---+---+---+---+---+---+---+---+
/// | B | S | R |      Length      |
/// +---+---+---+---+---+---+---+---+
///   |   |   |         |
///   |   |   |         +-- Bits 0-4: Length (length of input data)
///   |   |   +------------ Bit 5: Reserved (must be 0)
///   |   +---------------- Bit 6: SIO (0 = SIO mode not supported, 1 = SIO mode supported)
///   +-------------------- Bit 7: BYTE (0 = bit Process Data, 1 = byte Process Data)
/// ```
///
/// - **Bits 0-4 (Length):** Length of the input data (Process Data from Device to Master).
///   - Permissible values depend on the BYTE bit (see Table B.6 below).
/// - **Bit 5 (Reserved):** Reserved, must be set to 0.
/// - **Bit 6 (SIO):** Indicates if SIO (Switching Signal) mode is supported.
///   - 0: SIO mode not supported
///   - 1: SIO mode supported
/// - **Bit 7 (BYTE):** Indicates the unit for Length.
///   - 0: Length is in bits (bit Process Data)
///   - 1: Length is in bytes (octets, byte Process Data)
///
/// # Table B.6 – Permitted combinations of BYTE and Length
///
/// | BYTE | Length      | Definition                                         |
/// |------|-------------|----------------------------------------------------|
/// | 0    | 0           | no Process Data                                    |
/// | 0    | 1           | 1 bit Process Data, structured in bits              |
/// | 0    | n (2–15)    | n bit Process Data, structured in bits              |
/// | 0    | 16          | 16 bit Process Data, structured in bits             |
/// | 0    | 17–31       | Reserved                                           |
/// | 1    | 0, 1        | Reserved                                           |
/// | 1    | 2           | 3 octets Process Data, structured in octets         |
/// | 1    | n (3–30)    | n+1 octets Process Data, structured in octets       |
/// | 1    | 31          | 32 octets Process Data, structured in octets        |
///
/// # Reference
/// - IO-Link Specification v1.1.4, Section B.1.6, Figure B.5
/// - Table B.6 – Permitted combinations of BYTE and Length
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct ProcessDataIn {
    /// Bit 7: BYTE
    #[bits(1)]
    pub byte: bool,
    #[bits(1)]
    __: u8, // Reserved bit, must be 0
    /// Bit 6: SIO
    #[bits(1)]
    pub sio: bool,
    /// Bits 0-4: Length
    #[bits(5)]
    pub length: u8,
}

/// Represents the ProcessDataOut parameter as defined in IO-Link Specification v1.1.4 Section B.1.6.
///
/// The ProcessDataOut parameter is a single byte (u8) structured as follows:
///
/// ```text
///  7   6   5   4   3   2   1   0
/// +---+---+---+---+---+---+---+---+
/// | B | R | R |      Length      |
/// +---+---+---+---+---+---+---+---+
///   |   |   |         |
///   |   |   |         +-- Bits 0-4: Length (length of input data)
///   |   |   +------------ Bit 5: Reserved (must be 0)
///   |   +---------------- Bit 6: Reserved (must be 0)
///   +-------------------- Bit 7: BYTE (0 = bit Process Data, 1 = byte Process Data)
/// ```
///
/// - **Bits 0-4 (Length):** Length of the output data (Process Data from Master to Device).
///   - Permissible values depend on the BYTE bit (see Table B.6 below).
/// - **Bit 5 (Reserved):** Reserved, must be set to 0.
/// - **Bit 6 (Reserved):** Reserved, must be set to 0.
/// - **Bit 7 (BYTE):** Indicates the unit for Length.
///   - 0: Length is in bits (bit Process Data)
///   - 1: Length is in bytes (octets, byte Process Data)
///
/// # Table B.6 – Permitted combinations of BYTE and Length
///
/// | BYTE | Length      | Definition                                         |
/// |------|-------------|----------------------------------------------------|
/// | 0    | 0           | no Process Data                                    |
/// | 0    | 1           | 1 bit Process Data, structured in bits              |
/// | 0    | n (2–15)    | n bit Process Data, structured in bits              |
/// | 0    | 16          | 16 bit Process Data, structured in bits             |
/// | 0    | 17–31       | Reserved                                           |
/// | 1    | 0, 1        | Reserved                                           |
/// | 1    | 2           | 3 octets Process Data, structured in octets         |
/// | 1    | n (3–30)    | n+1 octets Process Data, structured in octets       |
/// | 1    | 31          | 32 octets Process Data, structured in octets        |
///
/// # Reference
/// - IO-Link Specification v1.1.4, Section B.1.6, Figure B.5
/// - Table B.6 – Permitted combinations of BYTE and Length
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct ProcessDataOut {
    /// Bits 0-4: Length
    #[bits(5)]
    pub length: u8,
    #[bits(2)]
    /// Bits 6-7: Reserved
    __: u8, // Reserved bit, must be 0
    /// Bit 7: BYTE
    #[bits(1)]
    pub byte: bool,
}

/// Device identification parameters.
///
/// This struct contains the device identification information
/// including vendor ID, device ID, and function ID.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 7.3.4.1: Device Identification
/// - Annex B.1: Direct Parameter Page 1 (VendorID1, VendorID2, DeviceID1-3)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DeviceIdent {
    /// Vendor ID (VID) - 16-bit vendor identification
    pub vendor_id: [u8; 2],
    /// Device ID (DID) - 24-bit device identification
    pub device_id: [u8; 3],
    /// Function ID (FID) - 16-bit function identification (reserved)
    pub function_id: [u8; 2],
}
