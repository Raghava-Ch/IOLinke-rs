//! IO-Link protocol types and enums
//!
//! This module defines all the data types, enums, and structures used throughout
//! the IO-Link protocol implementation, based on IO-Link Specification v1.1.4.
//!
//! ## Key Types
//!
//! - **Communication Modes**: SIO, COM1-3, DI, DO modes as per Section 5.2.2
//! - **Device States**: Idle, Startup, Preoperate, Operate states from Section 6.3
//! - **Error Handling**: Standardized error codes and result types
//! - **Timing**: Protocol timing constants and timer abstractions
//! - **Data Structures**: Process data, parameters, and event structures
//!
//! ## Specification Compliance
//!
//! All types follow the IO-Link v1.1.4 specification:
//! - Section 5.2: Physical Layer and Communication Modes
//! - Section 6.3: State Machines and Device Modes
//! - Section 7.3: Device Identification and Parameters
//! - Section 8.1: ISDU Communication and Parameters
//! - Annex A: Protocol Details and Timing
//! - Annex B: Parameter Definitions

use heapless::Vec;
use bitfields::bitfield;
use iolinke_macros::bitfield_support;

/// Maximum length of IO-Link message data in bytes.
///
/// This constant defines the maximum size of any IO-Link message
/// including headers, data, and checksums as per Section 5.4.
pub const MAX_MESSAGE_LENGTH: usize = 32;

/// Maximum number of process data bytes supported by the device.
///
/// Process data is limited to 32 bytes per cycle as defined in
/// Section 8.2 and Annex B.6 of the IO-Link specification.
pub const MAX_PROCESS_DATA_LENGTH: usize = 32;

/// Maximum number of events that can be queued in the event memory.
///
/// Events are stored in a circular buffer with this maximum capacity
/// to prevent memory overflow during high event rates.
pub const MAX_EVENT_QUEUE_SIZE: usize = 16;

/// IO-Link communication mode as defined in Section 5.2.2.
///
/// The communication mode determines the baud rate and protocol
/// characteristics used for master-device communication.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 5.2.2: Communication Modes
/// - Table 5.1: Communication mode characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IoLinkMode {
    /// SIO mode (Standard I/O) - Digital input/output without communication
    Sio = 0,
    /// COM1 mode - 4.8 kbaud communication (1200 baud effective)
    Com1 = 1,
    /// COM2 mode - 38.4 kbaud communication (2400 baud effective)
    Com2 = 2,
    /// COM3 mode - 230.4 kbaud communication (4800 baud effective)
    Com3 = 3,
    /// DI mode - Digital input only
    Di = 4,
    /// DO mode - Digital output only
    Do = 5,
    /// INACTIVE mode - No communication, high impedance
    Inactive = 0xFF
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

/// Event instance identifier as per Annex A.6.4.
///
/// The event instance specifies the source of an event within
/// the device system.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex A.6.4: EventQualifier
/// - Table A.17: Event instance values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventInstance {
    /// Unknown event source
    Unknown = 0,
    // 1 to 3 Reserved
    /// Application layer event
    Application = 4,
    /// System layer event
    System = 5,
    // 6 to 7 Reserved
}

impl Into<u8> for EventInstance {
    fn into(self) -> u8 {
        self as u8
    }
}

/// Event type classification as per Annex A.6.4.
///
/// The event type indicates the severity and nature of an event
/// for proper handling by the master.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex A.6.4: EventQualifier
/// - Table A.20: Event type values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Reserved value
    /// Notification event - Informational message
    Notification = 1,
    /// Warning event - Requires attention but not critical
    Warning = 2,
    /// Error event - Critical issue requiring immediate action
    Error = 3,
}

impl Into<u8> for EventType {
    fn into(self) -> u8 {
        self as u8
    }
}

/// Event mode specification as per Annex A.6.4.
///
/// The event mode determines how the event is reported and
/// whether it persists or is transient.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex A.6.4: EventQualifier
/// - Table A.20: Event mode values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventMode {
    /// Reserved value
    /// Single shot event - Reported once and cleared
    SingleShot = 1,
    /// Event disappears - Event condition no longer active
    Disappears = 2,
    /// Event appears - New event condition detected
    Appears = 3,
}

impl Into<u8> for EventMode {
    fn into(self) -> u8 {
        self as u8
    }
}

/// Device operating mode as per Section 6.3.
///
/// The device mode represents the current state of the IO-Link
/// device state machine.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 6.3: State Machines
/// - Table 94: SM_DeviceMode state definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceMode {
    /// Device is idle and waiting for configuration
    Idle,
    /// Device is in SIO mode (Standard I/O)
    Sio,
    /// Device is establishing communication with master
    Estabcom,
    /// Device is in COM1 communication mode
    Com1,
    /// Device is in COM2 communication mode
    Com2,
    /// Device is in COM3 communication mode
    Com3,
    /// Device is starting up and initializing
    Startup,
    /// Device is in identification startup phase
    Identstartup,
    /// Device is checking identification parameters
    Identchange,
    /// Device is in preoperate mode (parameterized but not operational)
    Preoperate,
    /// Device is in full operate mode (fully operational)
    Operate,
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

/// All the message handler confirmation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MhConfState {
    Com(IoLinkMode),
    Active,
    Inactive,
}

/// All the Command handler configuration states used
/// See Figure 54 – State machine of the Device command handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChConfState {
    /// (Handler changed to the ACTIVE state)
    Active,
    /// (Handler changed to the INACTIVE state)
    Inactive,
}

/// All the On-request data configuration states used
/// See Figure 49 – State machine of the Device On-request Data handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OhConfState {
    /// (Handler changed to the ACTIVE state)
    Active,
    /// (Handler changed to the INACTIVE state)
    Inactive,
}

/// All the ISDU Hanler configuration states used
/// See Figure 52 – State machine of the Device ISDU handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IhConfState {
    /// (Handler changed to the ACTIVE state)
    Active,
    /// (Handler changed to the INACTIVE state)
    Inactive,
}

/// All the Event Handler configuration states used
/// See Figure 56 – State machine of the Device Event handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EhConfState {
    /// (Handler changed to the ACTIVE state)
    Active,
    /// (Handler changed to the INACTIVE state)
    Inactive,
}

/// All the Process Data Handler configuration states used
/// See Figure 47 – State machine of the Device Process Data handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdConfState {
    /// (Handler changed to the ACTIVE state)
    Active,
    /// (Handler changed to the INACTIVE state)
    Inactive,
}

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

/// Physical layer status
/// See IO-Link v1.1.4 Section 5.2.3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PhysicalLayerStatus {
    /// No communication
    NoCommunication = 0,
    /// Communication established
    Communication = 1,
    /// Error state
    Error = 2,
}

/// See 7.2.1.18 DL_Control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DlControlCode {
    /// (Input Process Data valid; see 7.2.2.5, 8.2.2.12)
    VALID,
    /// (Input Process Data invalid)
    INVALID,
    /// (Output Process Data valid; see 7.3.7.1)
    PDOUTVALID,
    /// (Output Process Data invalid or missing)
    PDOUTINVALID,
}

/// See 7.2.2.5 PDInStatus
#[bitfield_support]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdStatus {
    /// (Input Process Data valid based on PD status flag (see A.1.5); see 7.2.1.18)
    VALID = 0,
    /// (Input Process Data invalid)
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

/// Error types for the IO-Link stack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoLinkError {
    /// Invalid parameter
    InvalidParameter,
    /// Communication timeout
    Timeout,
    /// Checksum error
    ChecksumError,
    /// Invalid frame format
    InvalidFrame,
    /// Buffer overflow
    BufferOverflow,
    /// Device not ready
    DeviceNotReady,
    /// Hardware error
    HardwareError,
    /// Protocol error
    ProtocolError,
    /// Cycle Error, This is a custom error type
    CycleError,
    /// Invalid Event Data, This is a custom error type
    InvalidEvent,
    /// Invalid Event Data, This is a custom error type
    InvalidData,
    /// Invalid M-sequence type, This is a custom error type
    InvalidMseqType,
    /// M-sequence checksum error, This is a custom error type
    InvalidMseqChecksum,
    /// Nothing to do, This is a custom error type for dummy trait functions
    NoImplFound,
    /// Event memory full, This is a custom error type for event handler
    EventMemoryFull,
    /// ISDU memory full, This is a custom error type for ISDU handler
    IsduVolatileMemoryFull,
    /// ISDU memory full, This is a custom error type for ISDU handler
    IsduNonVolatileMemoryFull,
    /// No event details supported in event memory, This is a custom error type for event handler
    NoEventDetailsSupported,
    /// Invalid address, This is a custom error type for address handling
    InvalidAddress,
    /// Read-only error, This is a custom error type for read-only operations
    ReadOnlyError,
    /// Invalid length, This is a custom error type for length handling
    InvalidLength,
    /// Parameter storage not set, This is a custom error type for parameter manager
    ParameterStorageNotSet,
    /// Failed to get parameter, This is a custom error type for parameter manager
    FailedToGetParameter,
    /// Failed to set parameter, This is a custom error type for parameter manager
    FailedToSetParameter,
    /// Function not available, This is a custom error type for parameter manager
    FuncNotAvailable,
    /// Invalid index, This is a custom error type for parameter manager
    InvalidIndex,
    /// Memory error, This is a custom error type for memory handling
    MemoryError,
}

/// Result type for IO-Link operations
pub type IoLinkResult<T> = Result<T, IoLinkError>;

/// Device identification structure
/// See IO-Link v1.1.4 Section 7.3.4
#[derive(Debug, Clone)]
pub struct DeviceIdentification {
    /// Vendor ID
    pub vendor_id: u16,
    /// Device ID
    pub device_id: u32,
    /// Function ID
    pub function_id: u16,
    /// Reserved field
    pub reserved: u8,
}

/// Device status word
/// See IO-Link v1.1.4 Section 7.3.4
#[derive(Debug, Clone, Copy)]
pub struct DeviceStatus {
    /// Raw status value
    pub raw: u8,
}

impl DeviceStatus {
    /// Create new device status
    pub fn new(raw: u8) -> Self {
        Self { raw }
    }

    /// Check if device is in error state
    pub fn is_error(&self) -> bool {
        (self.raw & 0x80) != 0
    }

    /// Check if device is ready
    pub fn is_ready(&self) -> bool {
        (self.raw & 0x40) != 0
    }

    /// Get device operating mode
    pub fn operating_mode(&self) -> u8 {
        (self.raw >> 4) & 0x03
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