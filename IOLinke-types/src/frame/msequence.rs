use crate::custom::IoLinkError;
use bitfields::bitfield;
use iolinke_macros::bitfield_support;

/// # M-sequence control (MC)
///
/// The M-sequence control octet (MC) defines how user data is transmitted in an IO-Link frame,
/// as specified in IO-Link v1.1.4, Section A.1.2 and Figure A.1. This octet encodes:
/// - The transmission direction (read or write)
/// - The communication channel
/// - The address (offset) of the data or the communication channel
///
/// ## Bit Layout (see Figure A.1)
/// ```text
///  7   6   5   4   3   2   1   0
/// +---+---+---+---+---+---+---+---+
/// | R |   Communication   | Address |
/// |/W |    Channel (2)    | (5)     |
/// +---+---+---+---+---+---+---+---+
///  |   |<------2------>|<---5----->|
///  |
///  +-- Bit 7: R/W (Read/Write)
///      - 0: Read access (transmission of user data from Device to Master)
///      - 1: Write access (transmission of user data from Master to Device)
///
///  Bits 5-6: Communication Channel
///      - 0: Process
///      - 1: Page
///      - 2: Diagnosis
///      - 3: ISDU
///
///  Bits 0-4: Address/Flow Control
///      - Offset of the user data on the specified communication channel,
///        or flow control for ISDU channel.
/// ```
///
/// ## Specification Reference
/// - IO-Link v1.1.4 Section A.1.2, Figure A.1, Table A.1, Table A.2
///
/// ## Example
/// ```rust
/// let mut mc = MsequenceControl::new();
/// mc.set_read_write(types::RwDirection::Read);
/// mc.set_comm_channel(types::ComChannel::Process);
/// mc.set_address_fctrl(0x03);
/// ```
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct MsequenceControl {
    /// Bit 7: R/W (Read/Write)
    /// - 0: Read access (Device to Master)
    /// - 1: Write access (Master to Device)
    #[bits(1)]
    pub read_write: RwDirection,
    /// Bits 5-6: Communication Channel
    /// - 0: Process
    /// - 1: Page
    /// - 2: Diagnosis
    /// - 3: ISDU
    #[bits(2)]
    pub comm_channel: ComChannel,
    #[bits(5)]
    pub address_fctrl: u8,
}

/// # Checksum / M-sequence type (CKT) Octet
///
/// This struct represents the Checksum/M-sequence type octet as defined in IO-Link Specification v1.1.4,
/// Section A.1.3 (see Figure A.2 and Table A.3 in the spec and the provided image).
///
/// The octet is structured as follows:
///
/// ```text
///  Bit 7   | Bit 6   | Bits 5-0
/// +--------+---------+----------------------+
/// | M-seq  | M-seq   |     Checksum         |
/// | type   | type    |   (6 bits)           |
/// +--------+---------+----------------------+
/// |<--2--->|<--------6--------------------->|
/// ```
///
/// - **Bits 0 to 5: Checksum**
///   - 6-bit message checksum to ensure data integrity (see A.1.6 and Clause I.1).
///
/// - **Bits 6 to 7: M-sequence type**
///   - Indicates the M-sequence type. The Master uses this to specify how messages within the M-sequence are structured.
///   - Defined values (see Table A.3):
///     - `0`: Type 0
///     - `1`: Type 1
///     - `2`: Type 2 (see NOTE: subtypes depend on PD configuration and PD direction)
///     - `3`: Reserved
///
/// # Specification Reference
/// - IO-Link v1.1.4 Section A.1.3, Figure A.2, Table A.3
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct ChecksumMsequenceType {
    /// Bits 0-5: 6-bit checksum for data integrity
    #[bits(6)]
    pub checksum: u8,
    /// Bits 6-7: M-sequence type (see Table A.3)
    #[bits(2)]
    pub m_seq_type: MsequenceBaseType,
}

/// Checksum / status (CKS) octet structure for IO-Link device reply messages.
///
/// This structure represents the reply message's checksum/status octet sent from the Device to the Master,
/// as described in IO-Link specification section A.1.5 (see Figure A.3 in the spec).
///
/// The octet is composed as follows:
/// - **Bits 0 to 5: Checksum**
///   - These 6 bits contain a 6-bit checksum to ensure data integrity of the reply message.
///   - The checksum is calculated as specified in section A.1.6 of the IO-Link specification.
///
/// - **Bit 6: PD status**
///   - This bit indicates whether the Device can provide valid Process Data (PD) or not.
///   - The flag should be used for Devices with input Process Data. Devices with only output Process Data
///     always indicate "Process Data valid".
///   - If the PD status flag is set to "Process Data invalid" within a message, all the input Process Data
///     of the complete Process Data cycle are invalid.
///   - See Table A.5 for values:
///     - 0: Process Data valid
///     - 1: Process Data invalid
///
/// - **Bit 7: Event flag**
///   - This bit indicates a Device-initiated event for the data category "Event" to be retrieved by the Master
///     via the diagnostic communication channel.
///   - The Device can report additional information such as errors, warnings, or events via Event response messages.
///   - See Table A.6 for values:
///     - 0: No Event
///     - 1: Event
///
/// # Layout (bit order)
/// ```text
///  7      6         5 4 3 2 1 0
/// +------+------+----------------+
/// |Event | PD   |   Checksum     |
/// |flag  |status|   (6 bits)     |
/// +------+------+----------------+
/// ```
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct ChecksumStatus {
    /// Bit 7: Event flag
    /// - 0: No Event
    /// - 1: Event
    #[bits(1)]
    pub event_flag: bool,
    /// Bit 6: PD status
    /// - 0: Process Data valid
    /// - 1: Process Data invalid
    #[bits(1)]
    pub pd_status: PdStatus,
    /// Bits 0-5: Checksum
    /// - 6-bit message checksum to ensure data integrity (see A.1.6 and Clause I.1).
    #[bits(6)]
    pub checksum: u8,
}

/// Read/Write direction for parameter operations as per Section 7.2.2.2.
///
/// This enum specifies whether a parameter operation is a read or write
/// request from the master to the device.
#[bitfield_support]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RwDirection {
    /// Write operation - Master sends parameter value to device
    Write,
    /// Read operation - Master requests parameter value from device
    Read,
}

impl RwDirection {
    /// Create a new RwDirection enum variant
    ///
    /// # Examples
    ///
    /// ```rust
    /// let direction = RwDirection::new();
    /// assert_eq!(direction, RwDirection::Write);
    /// ```
    pub const fn new() -> Self {
        Self::Write
    }
}

/// Implements conversion from `u8` to [`RwDirection`] with error handling.
///
/// This implementation allows infallible conversion from a `u8` value to the corresponding
/// [`RwDirection`] enum variant, as specified by the IO-Link protocol (Section 7.2.2.2).
///
/// # Mapping
/// - `0` → [`RwDirection::Write`]
/// - `1` → [`RwDirection::Read`]
///
/// Any other value will result in an [`IoLinkError::InvalidParameter`] error.
///
/// # Examples
/// ```rust
/// use crate::frame::msequence::{RwDirection, IoLinkError};
/// use core::convert::TryFrom;
///
/// assert_eq!(RwDirection::try_from(0), Ok(RwDirection::Write));
/// assert_eq!(RwDirection::try_from(1), Ok(RwDirection::Read));
/// assert_eq!(RwDirection::try_from(2), Err(IoLinkError::InvalidParameter));
/// ```
impl TryFrom<u8> for RwDirection {
    type Error = IoLinkError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RwDirection::Write),
            1 => Ok(RwDirection::Read),
            _ => Err(IoLinkError::InvalidParameter),
        }
    }
}

/// Communication channel identifier as per Annex A.1.2.
///
/// The communication channel determines the type of data being
/// transmitted in an M-sequence frame.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex A.1.2: M-sequence control (MC)
/// - Table A.1: Values of communication channel
#[bitfield_support]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ComChannel {
    /// Process data channel - Real-time process data exchange
    Process = 0,
    /// Page channel - Parameter read/write operations
    Page = 1,
    /// Diagnosis channel - Event and status information
    Diagnosis = 2,
    /// ISDU channel - Index-based service data unit communication
    Isdu = 3,
}

impl ComChannel {
    pub const fn new() -> Self {
        Self::Process
    }
}

impl TryFrom<u8> for ComChannel {
    type Error = IoLinkError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ComChannel::Process),
            1 => Ok(ComChannel::Page),
            2 => Ok(ComChannel::Diagnosis),
            3 => Ok(ComChannel::Isdu),
            _ => Err(IoLinkError::InvalidParameter),
        }
    }
}

/// M-sequence base type as per Annex A.1.3.
///
/// The M-sequence type determines the timing and structure of
/// communication sequences between master and device.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex A.1.3: Checksum / M-sequence type (CKT)
/// - Table A.2: M-sequence type definitions
#[bitfield_support]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsequenceBaseType {
    /// Type 0 - Standard M-sequence with basic timing
    Type0 = 0,
    /// Type 1 - Extended M-sequence with additional timing
    Type1 = 1,
    /// Type 2 - Advanced M-sequence with optimized timing
    Type2 = 2,
    /// Reserved for future use
    Reserved = 3,
}

impl MsequenceBaseType {
    /// Create a new MsequenceBaseType enum variant
    ///
    /// # Examples
    ///
    /// ```rust
    /// let base_type = MsequenceBaseType::new();
    /// assert_eq!(base_type, MsequenceBaseType::Type0);
    /// ```
    pub const fn new() -> Self {
        Self::Type0
    }
}

/// Process Data (PD) status flag as per IO-Link v1.1.4 Annex A.1.5, Table A.5 (bit 6 of the checksum/status octet).
///
/// This bit indicates whether the Device can provide valid Process Data or not.
/// - Value 0: Process Data valid (the Device provides valid input Process Data)
/// - Value 1: Process Data invalid (all input Process Data of the complete Process Data cycle are invalid)
///
/// Devices with only output Process Data shall always indicate "Process Data valid".
///
/// # Specification Reference
/// - IO-Link v1.1.4 Annex A.1.5: Checksum/status (CKS), Table A.5
#[bitfield_support]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdStatus {
    /// Process Data valid (bit 6 = 0)
    VALID = 0,
    /// Process Data invalid (bit 6 = 1)
    INVALID = 1,
}

impl PdStatus {
    /// Create a new PdStatus enum variant
    ///
    /// # Examples
    ///
    /// ```rust
    /// let status = PdStatus::new();
    /// assert_eq!(status, PdStatus::INVALID);
    /// ```
    pub const fn new() -> Self {
        Self::INVALID
    }
}

/// See 7.3.3.2 M-sequences
/// Also see A.2.6 M-sequence type usage for STARTUP, PREOPERATE and OPERATE modes
///     Table A.7 – M-sequence types for the STARTUP mode
///     Table A.8 – M-sequence types for the PREOPERATE mode
///     Table A.9 – M-sequence types for the OPERATE mode (legacy protocol)
///     Table A.10 – M-sequence types for the OPERATE mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MsequenceType {
    /// TYPE_0:
    /// ```ignore
    /// MC | CKT | --OD-- | CKS
    ///          | RD WR  |    
    /// ```
    Type0 = 0,

    /// TYPE_1_1:
    /// ```ignore
    /// MC | CKT | --PD₀ PD₁-- | CKS
    ///          |    RD WR    |    
    /// ```
    Type11,

    /// TYPE_1_2:
    /// ```ignore
    /// MC | CKT | --OD₀ OD₁-- | CKS
    ///          |    RD WR    |    
    /// ```
    Type12,

    /// TYPE_1_V:
    /// ```ignore
    /// MC | CKT | --OD₀ ... ODₙ-- | CKS
    ///          |      RD WR      |    
    /// ```
    Type1V,

    /// TYPE_2_1:
    /// ```ignore
    /// MC | CKT |  --OD-- | PD | CKS
    ///          |  RD WR  |    
    /// ```
    Type21,

    /// TYPE_2_2:
    /// ```ignore
    /// MC | CKT |  --OD-- | PD₀ PD₁ | CKS
    ///          |  RD WR  |    
    /// ```
    Type22,

    /// TYPE_2_3:
    /// ```ignore
    /// MC | CKT | PD |  --OD-- | CKS
    ///               |  RD WR  |    
    /// ```
    Type23,

    /// TYPE_2_4:
    /// ```ignore
    /// MC | CKT | PD₀ PD₁ |  --OD-- | CKS
    ///                    |  RD WR  |    
    /// ```
    Type24,

    /// TYPE_2_5:
    /// ```ignore
    /// MC | CKT | PD |  --OD-- | PD | CKS
    ///               |  RD WR  |    
    /// ```
    Type25,

    /// TYPE_2_V:
    /// ```ignore
    /// MC | CKT | PD₀ ... PDₙ₋₁ | --OD₀ ... ODₘ₋₁-- | PD₀ ... PDₖ₋₁ | CKS
    ///                          |       RD WR       |
    /// ```
    Type2V = 9,
}

impl Into<u8> for MsequenceType {
    fn into(self) -> u8 {
        self as u8
    }
}

/// Transmission rate configuration for IO-Link communication.
///
/// The transmission rate determines the baud rate used for
/// master-device communication.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 5.2.2: Communication Modes
/// - Table 5.1: Communication mode characteristics
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransmissionRate {
    /// COM1 mode - 1200 baud effective rate
    Com1,
    /// COM2 mode - 2400 baud effective rate
    Com2,
    /// COM3 mode - 4800 baud effective rate
    Com3,
}

impl Default for TransmissionRate {
    /// Default transmission rate is COM1 (1200 baud).
    fn default() -> Self {
        Self::Com1
    }
}

impl TransmissionRate {
    /// Returns the baud rate for the given UART mode.
    ///
    /// # IO-Link Specification Reference
    ///
    /// - Section A.3.2: Bit time
    /// - Equation (A.2): TBIT = 1 / (transmission rate)
    /// - Table 9: Values for TBIT
    ///
    /// The baud rate is the number of bits transmitted per second. This value is used for timing calculations in the IO-Link protocol.
    ///
    /// # Returns
    ///
    /// Baud rate in bits per second (bps) for the selected UART mode.
    ///
    /// # Example
    ///
    /// ```
    /// let baud_rate = IoLinkUartMode::Com1.get_baud_rate();
    /// assert_eq!(baud_rate, 4800); // 4.8 kbaud
    /// ```
    pub const fn get_baud_rate(self) -> u32 {
        match self {
            Self::Com1 => 4800,
            Self::Com2 => 38400,
            Self::Com3 => 230400,
        }
    }

    /// Returns the bit time (T_BIT) in microseconds for the given UART mode.
    ///
    /// # IO-Link Specification Reference
    ///
    /// - Section A.3.2: Bit time
    /// - Equation (A.2): TBIT = 1 / (transmission rate)
    /// - Table 9: Values for TBIT
    ///
    /// The bit time (T_BIT) is the time required to transmit a single bit and is the inverse of the
    /// transmission rate. This value is used for timing calculations in the IO-Link protocol.
    ///
    /// # Returns
    ///
    /// Bit time in microseconds (µs) for the selected UART mode.
    ///
    /// # Example
    ///
    /// ```
    /// let tbit = IoLinkUartMode::Com1.get_t_bit_in_us();
    /// assert_eq!(tbit, 20833); // 1/4800 s = 20833 µs
    /// ```
    pub const fn get_t_bit_in_us(self) -> u32 {
        match self {
            Self::Com1 => 20833,
            Self::Com2 => 2604,
            Self::Com3 => 434,
        }
    }
}
