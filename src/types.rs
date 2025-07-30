//! IO-Link protocol types and enums
//!
//! This module defines all the data types, enums, and structures used throughout
//! the IO-Link protocol implementation, based on IO-Link Specification v1.1.4.

use heapless::Vec;

/// Maximum length of IO-Link message data
pub const MAX_MESSAGE_LENGTH: usize = 32;

/// Maximum number of process data bytes
pub const MAX_PROCESS_DATA_LENGTH: usize = 32;

/// Maximum number of events in event queue
pub const MAX_EVENT_QUEUE_SIZE: usize = 16;

/// IO-Link communication mode
/// See IO-Link v1.1.4 Section 5.2.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IoLinkMode {
    /// SIO mode (Standard I/O)
    Sio = 0,
    /// COM1 mode (4.8 kbaud)
    Com1 = 1,
    /// COM2 mode (38.4 kbaud)
    Com2 = 2,
    /// COM3 mode (230.4 kbaud)
    Com3 = 3,
    /// DI mode
    Di = 4,
    /// DO mode
    Do = 5,
    /// INACTIVE mode
    Inactive = 0xFF
}

/// See 7.2.2.2 OD Arguments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RwDirection {
    /// Read operation
    Read,
    /// Write operation
    Write,
}


impl TryFrom<u8> for RwDirection {
    type Error = IoLinkError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RwDirection::Read),
            1 => Ok(RwDirection::Write),
            _ => Err(IoLinkError::InvalidParameter),
        }
    }
}

/// See A.1.2 M-sequence control (MC)
/// Also see Table A.1 – Values of communication channel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComChannel {
    /// {Process} = 0
    Process = 0,
    /// {Page} = 1
    Page = 1,
    /// {Diagnosis} = 2
    Diagnosis = 2,
    /// {ISDU} = 3
    Isdu = 3,
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

/// See A.1.3 Checksum / M-sequence type (CKT)
/// Also see Bit 6 to 7: M-sequence type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsequenceBaseType {
    /// {Type 0} = 0
    Type0 = 0,
    /// {Type 1} = 1
    Type1 = 1,
    /// {Type 2} = 2
    Type2 = 2,
    /// {reserved} = 3
    Reserved = 3,
}

/// See A.6.4 EventQualifier
/// Bits 0 to 2: INSTANCE
/// These bits indicate the particular source (instance) of an Event thus refining its evaluation on
/// the receiver side. Permissible values for INSTANCE are listed in Table A.17.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventInstance {
    /// {Unknown} = 0
    Unknown = 0,
    // 1 to 3 Reserved
    /// {Application} = 4
    Application = 4,
    /// {System} = 5
    System = 5,
    // 6 to 7 Reserved
}

/// See A.6.4 EventQualifier
/// Bits 4 to 5: TYPE
/// These bits indicate the Event mode. Permissible values for MODE are listed in Table A.20.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// 0 is Reserved
    /// {Notification} = 1
    Notification = 1,
    /// {Warning} = 2
    Warning = 2,
    /// {Error} = 3
    Error = 3,
}

/// See A.6.4 EventQualifier
/// Bits 6 to 7: MODE
/// These bits indicate the Event mode. Permissible values for MODE are listed in Table A.20.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventMode {
    /// 0 is Reserved
    /// {Event single shot} = 1
    SingleShot = 1,
    /// {Event disappears} = 2
    Disappears = 2,
    /// {Event appears} = 3
    Appears = 3,
}

/// See Table 94 – SM_DeviceMode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceMode {
    /// (Device changed to the SM mode "SM_IdentStartup")
    IdentStartup,
    /// (Device changed to the SM mode "SM_IdentChange")
    IdentChange,
    /// (Device changed to waiting for configuration)
    Idle,
    /// (Device changed to the mode defined in service "SM_SetDeviceCom")
    Sio,
    /// (Device changed to the SM mode "SM_ComEstablish")
    Estabcom,
    /// (Device changed to the COM1 mode)
    Com1,
    /// (Device changed to the COM2 mode)
    Com2,
    /// (Device changed to the COM3 mode)
    Com3,
    /// (Device changed to the STARTUP mode)
    Startup,
    /// (Device changed to the SM mode "SM_IdentStartup")
    Identstartup,
    /// (Device changed to the SM mode "SM_IdentCheck")
    Identchange,
    /// (Device changed to the PREOPERATE mode)
    Preoperate,
    /// (Device changed to the OPERATE mode)
    Operate,
}

/// See 7.2.1.14 DL_Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DlMode {
    /// (Handler changed to the INACTIVE state)
    Inactive,
    /// (COM1 mode established)
    Com1,
    /// (COM2 mode established)
    Com2,
    /// (COM3 mode established)
    Com3,
    /// (Lost communication)
    Comlost,
    /// (Handler changed to the EstablishCom state)
    Estabcom,
    /// (Handler changed to the STARTUP state)
    Startup,
    /// (Handler changed to the PREOPERATE state)
    Preoperate,
    /// (Handler changed to the OPERATE state)
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
    INACTIVE,
    /// MasterCommand {Fallback} = 0x5Au8
    FALLBACK,
    /// MasterCommand {DeviceStartup} = 0x97u8
    STARTUP,
    /// MasterCommand {DevicePreoperate} = 0x9Au8
    PREOPERATE,
    /// MasterCommand {DeviceOperate} = 0x99u8
    /// This command is also known as PDOUTINVALID
    OPERATE,
    /// MasterCommand {MasterIdent} = 0x95u8
    MASTERIDENT,
    /// MasterCommand {DeviceIdent} = 0x96u8
    DEVICEIDENT,
    /// MasterCommand {ProcessDataOutputOperate} = 0x98u8
    /// This command aka PDOUTVALID
    PDOUT,
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
pub enum DlControlCodes {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdInStatus {
    /// (Input Process Data valid based on PD status flag (see A.1.5); see 7.2.1.18)
    VALID,
    /// (Input Process Data invalid)
    INVALID,
}

/// Message types for IO-Link communication
/// See IO-Link v1.1.4 Section 6.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// Type 0: Process Data
    ProcessData = 0,
    /// Type 1: Device command
    DeviceCommand = 1,
    /// Type 2: Parameter command
    ParameterCommand = 2,
}

impl TryFrom<u8> for MessageType {
    type Error = IoLinkError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::ProcessData),
            1 => Ok(MessageType::DeviceCommand),
            2 => Ok(MessageType::ParameterCommand),
            _ => Err(IoLinkError::InvalidParameter),
        }
    }
}

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        match self {
            MessageType::ProcessData => 0,
            MessageType::DeviceCommand => 1,
            MessageType::ParameterCommand => 2,
        }
    }
}

/// Process data input/output structure
/// See IO-Link v1.1.4 Section 8.4.2
#[derive(Debug, Clone)]
pub struct ProcessData {
    /// Input data from device
    pub input: Vec<u8, MAX_PROCESS_DATA_LENGTH>,
    /// Output data to device
    pub output: Vec<u8, MAX_PROCESS_DATA_LENGTH>,
    /// Data validity flag
    pub valid: bool,
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            input: Vec::new(),
            output: Vec::new(),
            valid: false,
        }
    }
}

/// ISDU (Index Service Data Unit) structure
/// See IO-Link v1.1.4 Section 8.4.3
#[derive(Debug, Clone)]
pub struct Isdu {
    /// Parameter index
    pub index: u16,
    /// Sub-index
    pub sub_index: u8,
    /// Data payload
    pub data: Vec<u8, MAX_MESSAGE_LENGTH>,
    /// Read/Write operation flag
    pub is_write: bool,
}

/// Event types for IO-Link devices
/// See IO-Link v1.1.4 Section 8.4.4
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// #[repr(u16)]
// pub enum EventType {
//     /// No event
//     None = 0x0000,
//     /// Device appears
//     DeviceAppears = 0x1000,
//     /// Device disappears
//     DeviceDisappears = 0x1001,
//     /// Communication lost
//     CommunicationLost = 0x1002,
//     /// Device fault
//     DeviceFault = 0x2000,
//     /// Parameter change
//     ParameterChange = 0x3000,
// }

/// Event structure
/// See IO-Link v1.1.4 Section 8.4.4
#[derive(Debug, Clone)]
pub struct Event {
    /// Event type
    pub event_type: EventType,
    /// Event qualifier
    pub qualifier: u8,
    /// Event mode
    pub mode: u8,
    /// Event data
    pub data: Vec<u8, 8>,
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
    /// ```
    /// MC | CKT | --OD-- | CKS
    ///          | RD WR  |    
    /// ```
    Type0 = 0,

    /// TYPE_1_1:
    /// ```
    /// MC | CKT | --PD₀ PD₁-- | CKS
    ///          |    RD WR    |    
    /// ```
    Type11,

    /// TYPE_1_2:
    /// ```
    /// MC | CKT | --OD₀ OD₁-- | CKS
    ///          |    RD WR    |    
    /// ```
    Type12,

    /// TYPE_1_V:
    /// ```
    /// MC | CKT | --OD₀ ... ODₙ-- | CKS
    ///          |      RD WR      |    
    /// ```
    Type1V,

    /// TYPE_2_1:
    /// ```
    /// MC | CKT |  --OD-- | PD | CKS
    ///          |  RD WR  |    
    /// ```
    Type21,

    /// TYPE_2_2:
    /// ```
    /// MC | CKT |  --OD-- | PD₀ PD₁ | CKS
    ///          |  RD WR  |    
    /// ```
    Type22,

    /// TYPE_2_3:
    /// ```
    /// MC | CKT | PD |  --OD-- | CKS
    ///               |  RD WR  |    
    /// ```
    Type23,

    /// TYPE_2_4:
    /// ```
    /// MC | CKT | PD₀ PD₁ |  --OD-- | CKS
    ///                    |  RD WR  |    
    /// ```
    Type24,

    /// TYPE_2_5:
    /// ```
    /// MC | CKT | PD |  --OD-- | PD | CKS
    ///               |  RD WR  |    
    /// ```
    Type25,

    /// TYPE_2_V:
    /// ```
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