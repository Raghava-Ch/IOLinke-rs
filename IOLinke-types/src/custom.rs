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
    /// Device not ready, This is a custom error type for device state handling
    NotReady,
    /// Not enough memory, This is a custom error type for memory handling
    NotEnoughMemory,
}

/// Result type for IO-Link operations
pub type IoLinkResult<T> = Result<T, IoLinkError>;
