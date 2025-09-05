use bitfields::bitfield;
use iolinke_macros::bitfield_support;

/// State of Data Storage (for bits 1-2 of StateProperty)
#[bitfield_support]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsState {
    Inactive = 0b00,
    Upload = 0b01,
    Download = 0b10,
    Locked = 0b11,
}

impl DsState {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidityCheckResult {
    /// Parameter is valid
    Valid,
    /// Parameter is invalid
    Invalid,
    /// Parameter is validation yet to be done
    YetToBeValidated,
}

/// Represents all possible device parameter indices as defined in the specification.
///
/// This enum categorizes parameters into different ranges with specific purposes:
/// - Standard parameters (0x0000-0x0030)
/// - Profile-specific parameters (0x0031-0x003F)
/// - Preferred device-specific parameters (0x0040-0x00FE)
/// - Extended device-specific parameters (0x0100-0x3FFF)
/// - Various profile-specific ranges (0x4000-0x4FFF)
/// - Safety and wireless extensions (0x4200-0x42FF, 0x5000-0x50FF)
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeviceParametersIndex {
    DirectParameterPage1 = 0x0000,
    DirectParameterPage2 = 0x0001,
    SystemCommand = 0x0002,
    DataStorageIndex = 0x0003,
    DeviceAccessLocks = 0x000C,
    ProfileCharacteristic = 0x000D,
    PDInputDescriptor = 0x000E,
    PDOutputDescriptor = 0x000F,
    VendorName = 0x0010,
    VendorText = 0x0011,
    ProductName = 0x0012,
    ProductID = 0x0013,
    ProductText = 0x0014,
    SerialNumber = 0x0015,
    HardwareRevision = 0x0016,
    FirmwareRevision = 0x0017,
    ApplicationSpecificTag = 0x0018,
    FunctionTag = 0x0019,
    LocationTag = 0x001A,
    ProductURI = 0x001B,
    ErrorCount = 0x0020,
    DeviceStatus = 0x0024,
    DetailedDeviceStatus = 0x0025,
    ProcessDataInput = 0x0028,
    ProcessDataOutput = 0x0029,
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
/// Device parameter index as defined by the IO-Link Specification.
///
/// This enum represents the standard and extended parameter indices used for accessing
/// device parameters in IO-Link devices. The variants cover:
/// - Standard parameters (0x0000-0x0030), such as direct parameter pages, system commands, device identification, and process data descriptors.
/// - Profile-specific and device-specific parameter ranges, including:
///   - ProfileSpecific: 0x0031-0x003F
///   - PreferredIndex: 0x0040-0x00FE (8-bit preferred device-specific)
///   - ExtendedIndex: 0x0100-0x3FFF (16-bit extended device-specific)
///   - DeviceProfileIndex: 0x4000-0x41FF (device profile-specific)
///   - SafetySpecificIndex: 0x4200-0x42FF (safety system extensions)
///   - SecondaryDeviceProfileIndex: 0x4300-0x4FFF (secondary device profile-specific)
///   - WirelessSpecificIndex: 0x5000-0x50FF (wireless system extensions)
///
/// Use this enum to match or construct parameter indices for device parameter access,
/// including vendor-specific and profile-specific extensions.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DirectParameterPage1SubIndex {
    MasterCommand = 0x00u8,
    MasterCycleTime = 0x01u8,
    MinCycleTime = 0x02u8,
    MSequenceCapability = 0x03u8,
    RevisionID = 0x04u8,
    ProcessDataIn = 0x05u8,
    ProcessDataOut = 0x06u8,
    VendorID1 = 0x07u8,
    VendorID2 = 0x08u8,
    DeviceID1 = 0x09u8,
    DeviceID2 = 0x0Au8,
    DeviceID3 = 0x0Bu8,
    FunctionID1 = 0x0Cu8,
    FunctionID2 = 0x0Du8,
    Reserved0E = 0x0Eu8,
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
/// For example, `DirectParameterPage2SubIndex::VendorSpecific10` corresponds to subindex 0x10.
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
