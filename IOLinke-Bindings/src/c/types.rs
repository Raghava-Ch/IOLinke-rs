//! # IO-Link C Types Bindings
//!
//! This module provides C-compatible types and structures for representing IO-Link device parameters and communication settings,
//! as defined in the IO-Link Specification v1.1.4 (Annex B.1). These types are designed for interoperability with C code and
//! facilitate safe and efficient FFI between Rust and C environments.
//!
//! ## Overview
//!
//! The types in this module cover:
//! - Device action and operation states
//! - Communication result types and result unions
//! - Device handle representation
//! - Cycle time encoding and calculation
//! - M-sequence capability and protocol revision identification
//! - Process data input/output configuration
//! - Device communication parameter grouping
//!
//! Each struct and enum is annotated with `#[repr(C)]` to ensure layout compatibility with C, and detailed documentation is provided
//! for each type, referencing the relevant sections of the IO-Link specification.
//!
//! ## Specification Reference
//!
//! - IO-Link Specification v1.1.4, Annex B.1: Direct Parameter Page 1
//! - Section B.1.1: Communication Parameters
//! - Section B.1.3: Cycle Time
//! - Section B.1.4: M-sequenceCapability, RevisionID
//! - Section B.1.6: ProcessDataIn, ProcessDataOut
//!
//! ## Usage
//!
//! Use these types to encode, decode, and transfer IO-Link device parameters and communication settings between Rust and C code.
//! The documentation for each type provides guidance on field usage and specification compliance.
use iolinke_device::{SioMode, TransmissionRate};

pub use core::result::{
    Result,
    Result::{Err, Ok},
};

/// Device action states.
#[repr(C)]
pub enum DeviceActionState {
    /// No action is currently being performed.
    Busy,
    /// An action is currently being performed.
    Done,
    /// No device is present.
    NoDevice,
}

/// Result of an operation.
#[repr(C)]
pub enum OperationResult {
    /// Operation completed successfully.
    Ok,
    /// An error occurred during the operation.
    Error,
    /// A parameter conflict was detected.
    ParameterConflict,
}

/// Type alias for IO-Link device handle.
pub type IOLinkeDeviceHandle = i16; // Using i16 to match C's int16_t

/// Result types for system management operations.
#[repr(C)]
pub enum SmResultType {
    /// Operation completed successfully.
    Ok,
    /// An error occurred during the operation.
    Err,
    /// Communication data for the device.
    DeviceCom,
    /// Identification information for the device.
    DeviceIdent,
}

#[repr(C)]
/// A union representing the result of a state machine operation.
///
/// This union can hold one of the following:
/// - `device_com`: Communication data for the device.
/// - `device_ident`: Identification information for the device.
/// - `err_code`: Error code indicating the result of the operation.
pub union SmResult {
    /// Communication parameters for the device.
    pub device_com: DeviceCom,
    /// Identification information for the device.
    pub device_ident: iolinke_device::DeviceIdent,
    /// Error code for the operation.
    pub err_code: i8,
}

/// Wrapper struct combining the result type and the result union.
#[repr(C)]
pub struct SmResultWrapper {
    /// The type of result.
    pub result_type: SmResultType,
    /// The result data, which can be one of several types.
    pub result: SmResult,
}

/// # CycleTime
///
/// This struct represents the Cycle Time parameter as defined by the IO-Link specification (v1.1.4, Section B.1.3, Table B.3 and Figure B.2).
///
/// The Cycle Time parameter is encoded as a single byte, which informs the IO-Link Master about the minimum supported cycle time of the device.
///
/// ## Bit Layout
///
/// | Bits 7-6   | Bits 5-0         |
/// |------------|------------------|
/// | Time Base  | Multiplier (M)   |
///
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CycleTime {
    /// Bits 0–5: Multiplier (M)
    pub multiplier: u8,
    /// Bits 6–7: Time Base
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MsequenceCapability {
    /// Bit 1: ISDU
    pub isdu: bool,
    /// Bits 2-4: OPERATE M-sequence code
    pub operate_m_sequence: u8,
    /// Bits 5-6: PREOPERATE M-sequence code
    pub preoperate_m_sequence: u8,
    // Bits 7-8: Reserved
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RevisionId {
    /// Bits 0 to 3: Minor Revision
    ///
    /// These bits contain the minor digit of the version number,
    /// for example 0 for the protocol version 1.0. Permissible
    /// values for MinorRev are 0x0 to 0xF.
    pub minor_rev: u8,

    /// Bits 4 to 7: Major Revision
    ///
    /// These bits contain the major digit of the version number,
    /// for example 1 for the protocol version 1.0. Permissible
    /// values for MajorRev are 0x0 to 0xF.
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessDataIn {
    /// Bit 7: BYTE
    pub byte: bool,
    // Reserved bit 5
    /// Bit 6: SIO
    pub sio: bool,
    /// Bits 0-4: Length
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessDataOut {
    /// Bits 0-4: Length
    pub length: u8,
    /// Bits 6-7: Reserved
    // Reserved bit, must be 0
    /// Bit 7: BYTE
    pub byte: bool,
}

/// Device communication configuration parameters.
///
/// This struct contains all the communication-related parameters
/// that define the device's communication capabilities and settings.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex B.1: Direct Parameter Page 1
/// - Section B.1.1: Communication Parameters
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DeviceCom {
    /// Supported SIO mode configuration
    pub suppported_sio_mode: SioMode,
    /// Transmission rate for IO-Link communication
    pub transmission_rate: TransmissionRate,
    /// Minimum cycle time configuration
    pub min_cycle_time: CycleTime,
    /// M-sequence capability configuration
    pub msequence_capability: MsequenceCapability,
    /// Protocol revision identifier
    pub revision_id: RevisionId,
    /// Process data input configuration
    pub process_data_in: ProcessDataIn,
    /// Process data output configuration
    pub process_data_out: ProcessDataOut,
}
