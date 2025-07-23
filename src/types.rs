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
}

/// All the timers used in IO-Link
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timer {
    /// See Table 42 – Wake-up procedure and retry characteristics
    Tdsio
}

/// All the master commands used in IO-Link
/// See Table B.2 – Types of MasterCommands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasterCommand {
    ///MasterCommand = 0x5A
    INACTIVE,
    ///MasterCommand = 0x97
    STARTUP,
    ///MasterCommand = 0x9A
    PREOPERATE,
    ///MasterCommand = 0x99
    OPERATE,
    ///MasterCommand = 0x5A
    FALLBACK,
}

/// All the message handler information type
/// See 7.2.2.6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MHInfo {
    /// lost communication)
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum EventType {
    /// No event
    None = 0x0000,
    /// Device appears
    DeviceAppears = 0x1000,
    /// Device disappears
    DeviceDisappears = 0x1001,
    /// Communication lost
    CommunicationLost = 0x1002,
    /// Device fault
    DeviceFault = 0x2000,
    /// Parameter change
    ParameterChange = 0x3000,
}

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
    /// Invalid Event Error, This is a custom error type
    InvalidEvent,
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
