//! # IO-Link Parameter Memory (PM) Handler Module
//!
//! This module provides types and logic for handling parameter memory operations in IO-Link devices.
//! It defines enums and bitfields for representing Data Storage state, parameter indices, subindices,
//! and validity checks, as well as methods for accessing and categorizing device parameters.
//!
//! ## Key Types
//! - [`DsState`]: Enumerates Data Storage states.
//! - [`StateProperty`]: Bitfield structure for Data Storage state property.
//! - [`DeviceParametersIndex`]: Enumerates device parameter indices as per IO-Link specification.
//! - [`DirectParameterPage1SubIndex`], [`DirectParameterPage2SubIndex`], [`DataStorageIndexSubIndex`]: Subindex enums for parameter access.
//! - [`SubIndex`]: Enum for specifying subindices for parameter indices.
//! - [`IndexCategory`]: Categorizes parameter indices by range.
//!
//! ## Key Methods
//! - [`DeviceParametersIndex::from_index`]: Creates a parameter index from a raw value.
//! - [`DeviceParametersIndex::index`]: Returns the raw index value.
//! - [`DeviceParametersIndex::name`]: Returns the human-readable parameter name.
//! - [`DeviceParametersIndex::category`]: Returns the parameter category.
//!
//! ## Specification Reference
//! - IO-Link v1.1.4, Section B.8 (Index assignment of data objects)
//! - Table B.8 – Index assignment of data objects
//!
//! This module is intended for use in IO-Link device implementations to manage parameter memory,
//! access device parameters, and ensure compliance with index assignment and categorization.

use bitfields::bitfield;
use iolinke_macros::bitfield_support;

/// State of Data Storage (for bits 1-2 of StateProperty)
#[bitfield_support]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsState {
    /// Data Storage is inactive
    Inactive = 0b00,
    /// Data Storage is active for upload
    Upload = 0b01,
    /// Data Storage is active for download
    Download = 0b10,
    /// Data Storage is locked
    Locked = 0b11,
}

impl DsState {
    /// Creates a new `DsState` instance with the default value of `Inactive`.
    pub const fn new() -> Self {
        Self::Inactive
    }
}

/// Data Storage State Property (8 bits)
/// Bit 0: Reserved
/// Bit 1-2: State of Data Storage (see `DsState`)
/// Bit 3-6: Reserved
/// Bit 7: DS_UPLOAD_FLAG ("1": DS_UPLOAD_REQ pending)
#[bitfield(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StateProperty {
    /// Bit 0: Reserved
    #[skip]
    #[bits(1)]
    __: u8,
    /// Bits 1-2: State of Data Storage
    #[bits(2)]
    pub ds_state: DsState,
    /// Bits 3-6: Reserved
    #[skip]
    #[bits(4)]
    __: u8,
    /// Bit 7: DS_UPLOAD_FLAG
    #[bits(1)]
    pub ds_upload_flag: bool,
}

/// Result of parameter validity check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidityCheckResult {
    /// Parameter is valid
    Valid,
    /// Parameter is invalid
    Invalid,
    /// Parameter is validation yet to be done
    YetToBeValidated,
}

/// Generates a comprehensive `DeviceParameterIndex` enum based on the IO-Link specification v1.1.4.
///
/// This procedural macro creates a strongly-typed representation of all device parameter indices
/// as defined in the IO-Link specification. It covers the complete range from 0x0000 to 0xFFFF,
/// categorizing parameters into standard, profile-specific, and device-specific ranges.
///
/// ## Index Ranges and Categories
///
/// ### Standard Parameters (0x0000-0x0030)
/// These are mandatory or conditional parameters that all IO-Link devices must support:
///
/// | Index  | Name                       | Access | Data Type         | M/O/C | Description                                         |
/// |--------|----------------------------|--------|-------------------|-------|-----------------------------------------------------|
/// | 0x0000 | Direct Parameter Page 1    | R      | RecordT           | M     | Redirected to page communication channel            |
/// | 0x0001 | Direct Parameter Page 2    | R/W    | RecordT           | M     | Redirected to page communication channel            |
/// | 0x0002 | System-Command             | W      | UIntegerT         | C     | Command code definition (1 octet)                   |
/// | 0x0003 | Data-Storage-Index         | R/W    | RecordT           | M     | Set of data objects for storage                     |
/// | 0x000C | Device-Access-Locks        | R/W    | RecordT           | O     | Standardized device locking functions (2 octets)    |
/// | 0x000D | Profile-Characteristic     | R      | ArrayT<UIntegerT16>| C     | Reserved for Common Profile                         |
/// | 0x000E | PDInput-Descriptor         | R      | ArrayT<OctetStringT3>| C  | Reserved for Common Profile                         |
/// | 0x000F | PDOutput-Descriptor        | R      | ArrayT<OctetStringT3>| C  | Reserved for Common Profile                         |
/// | 0x0010 | Vendor-Name                | R      | StringT           | M     | Vendor information (max 64 octets)                  |
/// | 0x0011 | Vendor-Text                | R      | StringT           | O     | Additional vendor information (max 64 octets)       |
/// | 0x0012 | Product-Name               | R      | StringT           | M     | Detailed product or type name (max 64 octets)       |
/// | 0x0013 | ProductID                  | R      | StringT           | O     | Product or type identification (max 64 octets)      |
/// | 0x0014 | Product-Text               | R      | StringT           | O     | Description of device function (max 64 octets)      |
/// | 0x0015 | Serial-Number              | R      | StringT           | O     | Vendor specific serial number (max 16 octets)       |
/// | 0x0016 | Hardware-Revision          | R      | StringT           | O     | Vendor specific format (max 64 octets)              |
/// | 0x0017 | Firmware-Revision          | R      | StringT           | O     | Vendor specific format (max 64 octets)              |
/// | 0x0018 | Application-Specific-Tag   | R/W    | StringT           | O     | Tag defined by user (16-32 octets)                  |
/// | 0x0019 | Function-Tag               | R/W    | StringT           | C     | Reserved for Common Profile (max 32 octets)         |
/// | 0x001A | Location-Tag               | R/W    | StringT           | C     | Reserved for Common Profile (max 32 octets)         |
/// | 0x001B | Product-URI                | R      | StringT           | C     | Reserved for Common Profile (max 100 octets)        |
/// | 0x0020 | ErrorCount                 | R      | UIntegerT         | O     | Errors since power-on or reset (2 octets)           |
/// | 0x0024 | Device-Status              | R      | UIntegerT         | O     | Current status of the device (1 octet)              |
/// | 0x0025 | Detailed-Device-Status     | R      | ArrayT<OctetStringT3>| O  | Detailed device status information                  |
/// | 0x0028 | Process-DataInput          | R      | Device specific   | O     | Read last valid process data from PDin channel      |
/// | 0x0029 | Process-DataOutput         | R      | Device specific   | O     | Read last valid process data from PDout channel     |
/// | 0x0030 | Offset-Time                | R/W    | RecordT           | O     | Synchronization of device timing to M-sequence      |
///
/// ### Reserved Ranges
/// The following ranges are reserved and should not be used:
/// - 0x0004-0x000B: Reserved for exceptional operations
/// - 0x001C-0x001F: Reserved
/// - 0x0021-0x0023: Reserved
/// - 0x0026-0x0027: Reserved
/// - 0x002A-0x002F: Reserved
/// - 0x00FF: Reserved
/// - 0x5100-0xFFFF: Reserved
///
/// ### Profile-Specific Parameters (0x0031-0x003F)
/// Reserved for device profiles and common profile extensions.
///
/// ### Preferred Device-Specific Parameters (0x0040-0x00FE)
/// 8-bit range for device-specific parameters that are commonly used.
///
/// ### Extended Device-Specific Parameters (0x0100-0x3FFF)
/// 16-bit range for extended device-specific functionality.
///
/// ### Device Profile Parameters (0x4000-0x41FF, 0x4300-0x4FFF)
/// Reserved ranges for device profile specifications.
///
/// ### Safety System Extensions (0x4200-0x42FF)
/// Reserved for safety system extensions as defined in [10].
///
/// ### Wireless System Extensions (0x5000-0x50FF)
/// Reserved for wireless system extensions as defined in [11].
///
/// ## Provided Methods
///
/// The following methods are implemented for the `DeviceParametersIndex` enum:
///
/// - `from_index(index: u16) -> Option<Self>`: Creates a parameter index from a raw value.
/// - `index(&self) -> u16`: Returns the raw index value.
/// - `name(&self) -> &'static str`: Returns the human-readable parameter name.
/// - `category(&self) -> IndexCategory`: Returns the parameter category.
///
/// ## Usage Example
///
/// ```rust
/// use crate::handlers::pm::{DeviceParametersIndex, IndexCategory};
///
/// // Access standard parameters
/// let vendor_name = DeviceParametersIndex::VendorName;
/// assert_eq!(vendor_name.index(), 0x0010);
/// assert_eq!(vendor_name.name(), "Vendor-Name");
/// assert_eq!(vendor_name.category(), IndexCategory::Standard);
///
/// // Create from raw index
/// let param = DeviceParametersIndex::from_index(0x0012).unwrap();
/// assert_eq!(param, DeviceParametersIndex::ProductName);
///
/// // Handle device-specific parameters
/// let custom_param = DeviceParametersIndex::PreferredIndex(0x50);
/// assert_eq!(custom_param.index(), 0x50);
/// assert_eq!(custom_param.category(), IndexCategory::PreferredIndex);
/// ```
///
/// ## References
///
/// - IO-Link Specification v1.1.4, Section B.8 - Index assignment of data objects
/// - Common Profile [7] for profile-specific parameters
/// - Safety system extensions [10]
/// - Wireless system extensions [11]
///
/// ## Notes
///
/// - All standard parameters with M (Mandatory) status must be implemented by IO-Link devices
/// - Parameters with O (Optional) status may be implemented based on device capabilities
/// - Parameters with C (Conditional) status are required when specific profiles are implemented
/// - Reserved ranges should not be used for custom parameters to avoid conflicts
/// - The macro automatically handles range validation and provides type-safe access to all valid indices
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeviceParametersIndex {
    /// 0x0000: Redirected to page communication channel (Direct Parameter Page 1, RecordT, Mandatory)
    DirectParameterPage1 = 0x0000,
    /// 0x0001: Redirected to page communication channel (Direct Parameter Page 2, RecordT, Mandatory)
    DirectParameterPage2 = 0x0001,
    /// 0x0002: Command code definition (System-Command, UIntegerT, Conditional, 1 octet)
    SystemCommand = 0x0002,
    /// 0x0003: Set of data objects for storage (Data-Storage-Index, RecordT, Mandatory)
    DataStorageIndex = 0x0003,
    /// 0x000C: Standardized device locking functions (Device-Access-Locks, RecordT, Optional, 2 octets)
    DeviceAccessLocks = 0x000C,
    /// 0x000D: Reserved for Common Profile (Profile-Characteristic, ArrayT<UIntegerT16>, Conditional)
    ProfileCharacteristic = 0x000D,
    /// 0x000E: Reserved for Common Profile (PDInput-Descriptor, ArrayT<OctetStringT3>, Conditional)
    PDInputDescriptor = 0x000E,
    /// 0x000F: Reserved for Common Profile (PDOutput-Descriptor, ArrayT<OctetStringT3>, Conditional)
    PDOutputDescriptor = 0x000F,
    /// 0x0010: Vendor information (Vendor-Name, StringT, Mandatory, max 64 octets)
    VendorName = 0x0010,
    /// 0x0011: Additional vendor information (Vendor-Text, StringT, Optional, max 64 octets)
    VendorText = 0x0011,
    /// 0x0012: Detailed product or type name (Product-Name, StringT, Mandatory, max 64 octets)
    ProductName = 0x0012,
    /// 0x0013: Product or type identification (ProductID, StringT, Optional, max 64 octets)
    ProductID = 0x0013,
    /// 0x0014: Description of device function (Product-Text, StringT, Optional, max 64 octets)
    ProductText = 0x0014,
    /// 0x0015: Vendor specific serial number (Serial-Number, StringT, Optional, max 16 octets)
    SerialNumber = 0x0015,
    /// 0x0016: Vendor specific format (Hardware-Revision, StringT, Optional, max 64 octets)
    HardwareRevision = 0x0016,
    /// 0x0017: Vendor specific format (Firmware-Revision, StringT, Optional, max 64 octets)
    FirmwareRevision = 0x0017,
    /// 0x0018: Tag defined by user (Application-Specific-Tag, StringT, Optional, 16-32 octets)
    ApplicationSpecificTag = 0x0018,
    /// 0x0019: Reserved for Common Profile (Function-Tag, StringT, Conditional, max 32 octets)
    FunctionTag = 0x0019,
    /// 0x001A: Reserved for Common Profile (Location-Tag, StringT, Conditional, max 32 octets)
    LocationTag = 0x001A,
    /// 0x001B: Reserved for Common Profile (Product-URI, StringT, Conditional, max 100 octets)
    ProductURI = 0x001B,
    /// 0x0020: Errors since power-on or reset (ErrorCount, UIntegerT, Optional, 2 octets)
    ErrorCount = 0x0020,
    /// 0x0024: Current status of the device (Device-Status, UIntegerT, Optional, 1 octet)
    DeviceStatus = 0x0024,
    /// 0x0025: Detailed device status information (Detailed-Device-Status, ArrayT<OctetStringT3>, Optional)
    DetailedDeviceStatus = 0x0025,
    /// 0x0028: Read last valid process data from PDin channel (Process-DataInput, Device specific, Optional)
    ProcessDataInput = 0x0028,
    /// 0x0029: Read last valid process data from PDout channel (Process-DataOutput, Device specific, Optional)
    ProcessDataOutput = 0x0029,
    /// 0x0030: Synchronization of device timing to M-sequence (Offset-Time, RecordT, Optional)
    OffsetTime = 0x0030,
    /// Profile-specific parameters range (0x0031-0x003F)
    ProfileSpecific(u16),
    /// Preferred device-specific parameters (8-bit) range (0x0040-0x00FE)
    PreferredIndex(u16),
    /// Extended device-specific parameters (16-bit) range (0x0100-0x3FFF)
    ExtendedIndex(u16),
    /// Device profile-specific parameters range (0x4000-0x41FF)
    DeviceProfileIndex(u16),
    /// Safety system extensions parameters range (0x4200-0x42FF)
    SafetySpecificIndex(u16),
    /// Secondary device profile-specific parameters range (0x4300-0x4FFF)
    SecondaryDeviceProfileIndex(u16),
    /// Wireless system extensions parameters range (0x5000-0x50FF)
    WirelessSpecificIndex(u16),
}

/// Resolves IO-Link direct parameter identifiers to their corresponding addresses.
///
/// This function maps parameter names to their standardized IO-Link addresses according
/// to the IO-Link specification v1.1. Direct parameters are organized into two pages:
///
/// ## Direct Parameter Page 1 (0x00-0x0F) - Standard Parameters
///
/// | Address | Parameter | Access | Description |
/// |---------|-----------|--------|-------------|
/// | 0x00u8 | `MasterCommand` | W | Master command to switch to operating states |
/// | 0x01u8 | `MasterCycleTime` | R/W | Actual cycle duration used by Master |
/// | 0x02u8 | `MinCycleTime` | R | Minimum cycle duration supported by Device |
/// | 0x03u8 | `MSequenceCapability` | R | M-sequences and physical configuration options |
/// | 0x04u8 | `RevisionID` | R/W | Protocol version ID (shall be 0x11) |
/// | 0x05u8 | `ProcessDataIn` | R | Input data type and length (Device to Master) |
/// | 0x06u8 | `ProcessDataOut` | R | Output data type and length (Master to Device) |
/// | 0x07u8 | `VendorID1` | R | Vendor identification MSB |
/// | 0x08u8 | `VendorID2` | R | Vendor identification LSB |
/// | 0x09u8 | `DeviceID1` | R/W | Device identification Octet 2 (MSB) |
/// | 0x0Au8 | `DeviceID2` | R/W | Device identification Octet 1 |
/// | 0x0Bu8 | `DeviceID3` | R/W | Device identification Octet 0 (LSB) |
/// | 0x0Cu8 | `FunctionID1` | R | Reserved (MSB) |
/// | 0x0Du8 | `FunctionID2` | R | Reserved (LSB) |
/// | 0x0Eu8 | `Reserved0E` | R | Reserved |
/// | 0x0Fu8 | `SystemCommand` | W | Command interface for end user applications |
///
/// ## Direct Parameter Page 2 (0x10-0x1F) - Vendor Specific
///
/// Addresses 0x10-0x1F are reserved for vendor-specific parameters.
///
/// # Parameters
///
/// * `param_ident` - The parameter identifier as a string slice
///
/// # Returns
///
/// Returns the corresponding 8-bit address for the given parameter.
///
/// # Panics
///
/// Panics if the provided parameter identifier is not recognized.
///
/// # Examples
///
/// ```rust
/// use crate::handlers::pm::DirectParameterPage1SubIndex;
/// let addr = DirectParameterPage1SubIndex::VendorID1 as u8;
/// assert_eq!(addr, 0x07u8);
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DirectParameterPage1SubIndex {
    /// Master command to switch to operating states (W)
    MasterCommand = 0x00u8,
    /// Actual cycle duration used by Master (R/W)
    MasterCycleTime = 0x01u8,
    /// Minimum cycle duration supported by Device (R)
    MinCycleTime = 0x02u8,
    /// M-sequences and physical configuration options (R)
    MSequenceCapability = 0x03u8,
    /// Protocol version ID (shall be 0x11) (R/W)
    RevisionID = 0x04u8,
    /// Input data type and length (Device to Master) (R)
    ProcessDataIn = 0x05u8,
    /// Output data type and length (Master to Device) (R)
    ProcessDataOut = 0x06u8,
    /// Vendor identification MSB (R)
    VendorID1 = 0x07u8,
    /// Vendor identification LSB (R)
    VendorID2 = 0x08u8,
    /// Device identification Octet 2 (MSB) (R/W)
    DeviceID1 = 0x09u8,
    /// Device identification Octet 1 (R/W)
    DeviceID2 = 0x0Au8,
    /// Device identification Octet 0 (LSB) (R/W)
    DeviceID3 = 0x0Bu8,
    /// Reserved (MSB) (R)
    FunctionID1 = 0x0Cu8,
    /// Reserved (LSB) (R)
    FunctionID2 = 0x0Du8,
    /// Reserved (R)
    Reserved0E = 0x0Eu8,
    /// Command interface for end user applications (W)
    SystemCommand = 0x0Fu8,
}

/// Subindices for Direct Parameter Page 2 (Index 0x0001).
///
/// These subindices (0x10–0x1F) are reserved for vendor-specific parameters,
/// allowing device manufacturers to define custom parameters beyond the standard set.
/// Each variant corresponds to a unique subindex value within the vendor-specific range.
///
/// # Specification Reference
/// - IO-Link Specification v1.1.4, Annex B (Parameter Assignment)
/// - Table B.8: Index assignment of data objects
///
/// # Usage
/// Use these variants to access or define vendor-specific parameters on Direct Parameter Page 2.
/// For example, `DirectParameterPage2SubIndex::VendorSpecific12` corresponds to subindex 0x10.
///
/// # Example
/// ```
/// use crate::handlers::pm::DirectParameterPage2SubIndex;
/// let subindex = DirectParameterPage2SubIndex::VendorSpecific12 as u8;
/// assert_eq!(subindex, 0x12);
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DirectParameterPage2SubIndex {
    /// Vendor-specific parameter at subindex 0x10
    VendorSpecific10 = 0x10u8,
    /// Vendor-specific parameter at subindex 0x11
    VendorSpecific11 = 0x11u8,
    /// Vendor-specific parameter at subindex 0x12
    VendorSpecific12 = 0x12u8,
    /// Vendor-specific parameter at subindex 0x13
    VendorSpecific13 = 0x13u8,
    /// Vendor-specific parameter at subindex 0x14
    VendorSpecific14 = 0x14u8,
    /// Vendor-specific parameter at subindex 0x15
    VendorSpecific15 = 0x15u8,
    /// Vendor-specific parameter at subindex 0x16
    VendorSpecific16 = 0x16u8,
    /// Vendor-specific parameter at subindex 0x17
    VendorSpecific17 = 0x17u8,
    /// Vendor-specific parameter at subindex 0x18
    VendorSpecific18 = 0x18u8,
    /// Vendor-specific parameter at subindex 0x19
    VendorSpecific19 = 0x19u8,
    /// Vendor-specific parameter at subindex 0x1A
    VendorSpecific1A = 0x1Au8,
    /// Vendor-specific parameter at subindex 0x1B
    VendorSpecific1B = 0x1Bu8,
    /// Vendor-specific parameter at subindex 0x1C
    VendorSpecific1C = 0x1Cu8,
    /// Vendor-specific parameter at subindex 0x1D
    VendorSpecific1D = 0x1Du8,
    /// Vendor-specific parameter at subindex 0x1E
    VendorSpecific1E = 0x1Eu8,
    /// Vendor-specific parameter at subindex 0x1F
    VendorSpecific1F = 0x1Fu8,
}

/// Subindices for the Data-Storage-Index parameter (Index 0x0003).
///
/// These subindices are used to access specific properties of the Data-Storage-Index
/// as defined in the IO-Link specification v1.1.4, Table B.8.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataStorageIndexSubIndex {
    /// Data Storage Command (Subindex 0x01)
    DsCommand = 0x01u8,
    /// State Property (Subindex 0x02)
    StateProperty = 0x02u8,
    /// Data Storage Size (Subindex 0x03)
    DataStorageSize = 0x03u8,
    /// Parameter Checksum (Subindex 0x04)
    ParameterChecksum = 0x04u8,
    /// Index List (Subindex 0x05)
    IndexList = 0x05u8,
}

/// Represents the subindex for a device parameter index.
///
/// This enum is used to specify the subindex for a given parameter index,
/// allowing access to specific fields or vendor-specific extensions.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SubIndex {
    /// Subindex for Direct Parameter Page 1 (Index 0x0000)
    DpPage1(DirectParameterPage1SubIndex),
    /// Subindex for Direct Parameter Page 2 (Index 0x0001)
    DpPage2(DirectParameterPage2SubIndex),
    /// Subindex for Data-Storage-Index (Index 0x0003)
    DataStorageIndex(DataStorageIndexSubIndex),
    /// Subindex for Vendor Name (Index 0x0010)
    VendorName,
    /// Subindex for Product Name (Index 0x0012)
    ProductName,
}

impl DeviceParametersIndex {
    /// Creates a DeviceParameterIndex from a raw index value.
    ///
    /// # Arguments
    /// * `index` - The raw index value to convert.
    ///
    /// # Returns
    /// `Some(DeviceParameterIndex)` if the index is valid, `None` otherwise.
    ///
    /// # Examples
    /// ```
    /// let param = DeviceParameterIndex::from_index(0x0010).unwrap();
    /// assert_eq!(param, DeviceParameterIndex::VendorName);
    /// ```
    pub fn from_index(index: u16) -> Option<Self> {
        match index {
            0x0000 => Some(Self::DirectParameterPage1),
            0x0001 => Some(Self::DirectParameterPage2),
            0x0002 => Some(Self::SystemCommand),
            0x0003 => Some(Self::DataStorageIndex),
            0x000C => Some(Self::DeviceAccessLocks),
            0x000D => Some(Self::ProfileCharacteristic),
            0x000E => Some(Self::PDInputDescriptor),
            0x000F => Some(Self::PDOutputDescriptor),
            0x0010 => Some(Self::VendorName),
            0x0011 => Some(Self::VendorText),
            0x0012 => Some(Self::ProductName),
            0x0013 => Some(Self::ProductID),
            0x0014 => Some(Self::ProductText),
            0x0015 => Some(Self::SerialNumber),
            0x0016 => Some(Self::HardwareRevision),
            0x0017 => Some(Self::FirmwareRevision),
            0x0018 => Some(Self::ApplicationSpecificTag),
            0x0019 => Some(Self::FunctionTag),
            0x001A => Some(Self::LocationTag),
            0x001B => Some(Self::ProductURI),
            0x0020 => Some(Self::ErrorCount),
            0x0024 => Some(Self::DeviceStatus),
            0x0025 => Some(Self::DetailedDeviceStatus),
            0x0028 => Some(Self::ProcessDataInput),
            0x0029 => Some(Self::ProcessDataOutput),
            0x0030 => Some(Self::OffsetTime),
            x @ 0x0031..=0x003F => Some(Self::ProfileSpecific(x as u16)),
            x @ 0x0040..=0x00FE => Some(Self::PreferredIndex(x as u16)),
            x @ 0x0100..=0x3FFF => Some(Self::ExtendedIndex(x)),
            x @ 0x4000..=0x41FF => Some(Self::DeviceProfileIndex(x)),
            x @ 0x4200..=0x42FF => Some(Self::SafetySpecificIndex(x)),
            x @ 0x4300..=0x4FFF => Some(Self::SecondaryDeviceProfileIndex(x)),
            x @ 0x5000..=0x50FF => Some(Self::WirelessSpecificIndex(x)),
            _ => None,
        }
    }

    /// Returns the raw index value for this parameter.
    ///
    /// # Examples
    /// ```
    /// let param = DeviceParameterIndex::VendorName;
    /// assert_eq!(param.index(), 0x0010);
    /// ```
    pub const fn index(&self) -> u16 {
        match *self {
            Self::DirectParameterPage1 => 0x0000,
            Self::DirectParameterPage2 => 0x0001,
            Self::SystemCommand => 0x0002,
            Self::DataStorageIndex => 0x0003,
            Self::DeviceAccessLocks => 0x000C,
            Self::ProfileCharacteristic => 0x000D,
            Self::PDInputDescriptor => 0x000E,
            Self::PDOutputDescriptor => 0x000F,
            Self::VendorName => 0x0010,
            Self::VendorText => 0x0011,
            Self::ProductName => 0x0012,
            Self::ProductID => 0x0013,
            Self::ProductText => 0x0014,
            Self::SerialNumber => 0x0015,
            Self::HardwareRevision => 0x0016,
            Self::FirmwareRevision => 0x0017,
            Self::ApplicationSpecificTag => 0x0018,
            Self::FunctionTag => 0x0019,
            Self::LocationTag => 0x001A,
            Self::ProductURI => 0x001B,
            Self::ErrorCount => 0x0020,
            Self::DeviceStatus => 0x0024,
            Self::DetailedDeviceStatus => 0x0025,
            Self::ProcessDataInput => 0x0028,
            Self::ProcessDataOutput => 0x0029,
            Self::OffsetTime => 0x0030,
            Self::ProfileSpecific(x) => x as u16,
            Self::PreferredIndex(x) => x as u16,
            Self::ExtendedIndex(x) => x,
            Self::DeviceProfileIndex(x) => x,
            Self::SafetySpecificIndex(x) => x,
            Self::SecondaryDeviceProfileIndex(x) => x,
            Self::WirelessSpecificIndex(x) => x,
        }
    }

    /// Returns the subindex for this parameter, if applicable.
    ///
    /// For most parameters, the subindex is 0. For parameter variants that
    /// encode a subindex, this method should be updated accordingly.
    ///
    /// # Arguments
    /// * `subindex` - The subindex variant to use for this parameter.
    ///
    /// # Returns
    /// The subindex value as a `u8`.
    ///
    /// # Panics
    /// Panics if the subindex is not valid for the parameter.
    ///
    /// # Examples
    /// ```
    /// let param = DeviceParameterIndex::VendorName;
    /// assert_eq!(param.subindex(SubIndex::VendorName), 0);
    /// ```
    pub const fn subindex(&self, subindex: SubIndex) -> u8 {
        match (*self, subindex) {
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::MasterCommand),
            ) => 0x00u8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::MasterCycleTime),
            ) => 0x01u8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::MinCycleTime),
            ) => 0x02u8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::MSequenceCapability),
            ) => 0x03u8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::RevisionID),
            ) => 0x04u8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::ProcessDataIn),
            ) => 0x05u8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::ProcessDataOut),
            ) => 0x06u8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::VendorID1),
            ) => 0x07u8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::VendorID2),
            ) => 0x08u8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::DeviceID1),
            ) => 0x09u8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::DeviceID2),
            ) => 0x0Au8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::DeviceID3),
            ) => 0x0Bu8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::FunctionID1),
            ) => 0x0Cu8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::FunctionID2),
            ) => 0x0Du8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::Reserved0E),
            ) => 0x0Eu8,
            (
                Self::DirectParameterPage1,
                SubIndex::DpPage1(DirectParameterPage1SubIndex::SystemCommand),
            ) => 0x0Fu8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific10),
            ) => 0x10u8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific11),
            ) => 0x11u8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific12),
            ) => 0x12u8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific13),
            ) => 0x13u8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific14),
            ) => 0x14u8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific15),
            ) => 0x15u8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific16),
            ) => 0x16u8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific17),
            ) => 0x17u8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific18),
            ) => 0x18u8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific19),
            ) => 0x19u8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1A),
            ) => 0x1Au8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1B),
            ) => 0x1Bu8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1C),
            ) => 0x1Cu8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1D),
            ) => 0x1Du8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1E),
            ) => 0x1Eu8,
            (
                Self::DirectParameterPage2,
                SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1F),
            ) => 0x1Fu8,
            (
                Self::DataStorageIndex,
                SubIndex::DataStorageIndex(DataStorageIndexSubIndex::DsCommand),
            ) => 0x01u8,
            (
                Self::DataStorageIndex,
                SubIndex::DataStorageIndex(DataStorageIndexSubIndex::StateProperty),
            ) => 0x02u8,
            (
                Self::DataStorageIndex,
                SubIndex::DataStorageIndex(DataStorageIndexSubIndex::DataStorageSize),
            ) => 0x03u8,
            (
                Self::DataStorageIndex,
                SubIndex::DataStorageIndex(DataStorageIndexSubIndex::ParameterChecksum),
            ) => 0x04u8,
            (
                Self::DataStorageIndex,
                SubIndex::DataStorageIndex(DataStorageIndexSubIndex::IndexList),
            ) => 0x05u8,
            (Self::VendorName, SubIndex::VendorName) => 0x00u8,
            (Self::ProductName, SubIndex::ProductName) => 0x00u8,
            _ => {
                panic!("Invalid subindex for parameter");
            }
        }
    }

    /// Returns the human-readable name of the parameter.
    ///
    /// # Returns
    /// A static string representing the name of the parameter.
    ///
    /// # Examples
    /// ```
    /// let param = DeviceParameterIndex::VendorName;
    /// assert_eq!(param.name(), "Vendor-Name");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            Self::DirectParameterPage1 => "Direct Parameter Page 1",
            Self::DirectParameterPage2 => "Direct Parameter Page 2",
            Self::SystemCommand => "System-Command",
            Self::DataStorageIndex => "Data-Storage-Index",
            Self::DeviceAccessLocks => "Device-Access-Locks",
            Self::ProfileCharacteristic => "Profile-Characteristic",
            Self::PDInputDescriptor => "PDInput-Descriptor",
            Self::PDOutputDescriptor => "PDOutput-Descriptor",
            Self::VendorName => "Vendor-Name",
            Self::VendorText => "Vendor-Text",
            Self::ProductName => "Product-Name",
            Self::ProductID => "ProductID",
            Self::ProductText => "Product-Text",
            Self::SerialNumber => "Serial-Number",
            Self::HardwareRevision => "Hardware-Revision",
            Self::FirmwareRevision => "Firmware-Revision",
            Self::ApplicationSpecificTag => "Application-Specific-Tag",
            Self::FunctionTag => "Function-Tag",
            Self::LocationTag => "Location-Tag",
            Self::ProductURI => "Product-URI",
            Self::ErrorCount => "ErrorCount",
            Self::DeviceStatus => "Device-Status",
            Self::DetailedDeviceStatus => "Detailed-Device-Status",
            Self::ProcessDataInput => "Process-DataInput",
            Self::ProcessDataOutput => "Process-DataOutput",
            Self::OffsetTime => "Offset-Time",
            Self::ProfileSpecific(_) => "Profile-Specific",
            Self::PreferredIndex(_) => "Preferred-Index",
            Self::ExtendedIndex(_) => "Extended-Index",
            Self::DeviceProfileIndex(_) => "Device-Profile-Index",
            Self::SafetySpecificIndex(_) => "Safety-Specific-Index",
            Self::SecondaryDeviceProfileIndex(_) => "Secondary-Device-Profile-Index",
            Self::WirelessSpecificIndex(_) => "Wireless-Specific-Index",
        }
    }

    /// Returns the category of the parameter index.
    ///
    /// # Returns
    /// The [`IndexCategory`] of the parameter.
    ///
    /// # Examples
    /// ```
    /// let param = DeviceParameterIndex::VendorName;
    /// assert_eq!(param.category(), IndexCategory::Standard);
    /// ```
    pub fn category(&self) -> IndexCategory {
        match self {
            Self::DirectParameterPage1
            | Self::DirectParameterPage2
            | Self::SystemCommand
            | Self::DataStorageIndex
            | Self::DeviceAccessLocks
            | Self::ProfileCharacteristic
            | Self::PDInputDescriptor
            | Self::PDOutputDescriptor
            | Self::VendorName
            | Self::VendorText
            | Self::ProductName
            | Self::ProductID
            | Self::ProductText
            | Self::SerialNumber
            | Self::HardwareRevision
            | Self::FirmwareRevision
            | Self::ApplicationSpecificTag
            | Self::FunctionTag
            | Self::LocationTag
            | Self::ProductURI
            | Self::ErrorCount
            | Self::DeviceStatus
            | Self::DetailedDeviceStatus
            | Self::ProcessDataInput
            | Self::ProcessDataOutput
            | Self::OffsetTime => IndexCategory::Standard,
            Self::ProfileSpecific(_) => IndexCategory::ProfileSpecific,
            Self::PreferredIndex(_) => IndexCategory::PreferredIndex,
            Self::ExtendedIndex(_) => IndexCategory::ExtendedIndex,
            Self::DeviceProfileIndex(_) => IndexCategory::DeviceProfile,
            Self::SafetySpecificIndex(_) => IndexCategory::SafetySpecific,
            Self::SecondaryDeviceProfileIndex(_) => IndexCategory::DeviceProfile,
            Self::WirelessSpecificIndex(_) => IndexCategory::WirelessSpecific,
        }
    }
}

/// Categorizes device parameter indices based on their range.
///
/// This enum is used to distinguish between standard, profile-specific,
/// preferred, extended, device profile, safety, and wireless parameter indices.
pub enum IndexCategory {
    /// Standard parameters (0x0000-0x0030)
    Standard,
    /// Profile-specific parameters (0x0031-0x003F)
    ProfileSpecific,
    /// Preferred device-specific parameters (8-bit) (0x0040-0x00FE)
    PreferredIndex,
    /// Extended device-specific parameters (16-bit) (0x0100-0x3FFF)
    ExtendedIndex,
    /// Device profile-specific parameters (0x4000-0x41FF, 0x4300-0x4FFF)
    DeviceProfile,
    /// Safety system extensions parameters (0x4200-0x42FF)
    SafetySpecific,
    /// Wireless system extensions parameters (0x5000-0x50FF)
    WirelessSpecific,
}
