
pub const MAX_WRITE_BUFFER_SIZE: usize = 65535;
pub const MAX_PARAM_ENTRIES: usize = 100;

/// Macro to configure block parameterization support and conditional code sections.
///
/// Usage:
/// ```ignore
/// block_param_support! {
///     supported {
///         // code if block parameterization is supported
///     }
///     not_supported {
///         // code if block parameterization is NOT supported
///     }
/// }
/// ```
#[macro_export]
macro_rules! block_param_support {
    (supported { $($supported_code:tt)* } not_supported { $($not_supported_code:tt)* }) => {
        #[cfg(feature = "block_parameterization")]
        {
            $($supported_code)*
        }
        #[cfg(not(feature = "block_parameterization"))]
        {
            $($not_supported_code)*
        }
    };
}


/// Macro to configure basic IO-Link device parameters.
#[macro_export]
macro_rules! iolink_device_config {
    ($vendor_id:expr, $device_id:expr, $revision:expr) => {
        const VENDOR_ID: u16 = $vendor_id;
        const DEVICE_ID: u32 = $device_id;
        const REVISION: u8 = $revision;
    };
}

/// Macro to define IO-Link process data structure.
#[macro_export]
macro_rules! process_data {
    ($name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        #[repr(C, packed)]
        pub struct $name {
            $(pub $field: $type,)*
        }
    };
}

/// Macro to configure communication parameters.
#[macro_export]
macro_rules! comm_config {
    (baudrate: $baudrate:expr, min_cycle_time: $cycle_time:expr) => {
        const BAUDRATE: u32 = $baudrate;
        const MIN_CYCLE_TIME: u16 = $cycle_time;
    };
}

/// Macro to define parameter data.
#[macro_export]
macro_rules! parameter {
    ($index:expr, $subindex:expr, $access:expr, $type:ty, $default:expr) => {
        pub const INDEX: u16 = $index;
        pub const SUBINDEX: u8 = $subindex;
        pub const ACCESS: u8 = $access;
        pub type DataType = $type;
        pub const DEFAULT_VALUE: DataType = $default;
    };
}
// pub mod storage_config {
//     use iolinke_macros::declare_parameter_storage;

//     declare_parameter_storage! {
//         // Note: Index 0x0000 and 0x0001 can only have subindexes 0x00-0x0F
//         // Index, Subindex, Length, Range, Access, Type, DefaultValue
//         (0x0000, 0x00, 16, 0..15, ReadOnly, u8, &[0; 16]),  // Direct Parameter Page 1
//         (0x0000, 0x0C, 16, 0..15, WriteOnly, u8, &[0; 16]),      // Split subindex space
//         (0x0001, 0x09, 2, 0..1, ReadWrite, u16, &[0, 1]),   // Fixed: length 2 for range 0..=1
//         (0x0002, 0x00, 1, 0..0, WriteOnly, u8, &[0, 1]),      // System-Command, 0..=0 is 1 value
//     }
// }

pub mod storage_config {
    use iolinke_macros::declare_parameter_storage;
    /// Parameter access rights
    pub enum AccessRight {
        ReadOnly,
        WriteOnly,
        ReadWrite,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for AccessRight {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    AccessRight::ReadOnly => "ReadOnly",
                    AccessRight::WriteOnly => "WriteOnly",
                    AccessRight::ReadWrite => "ReadWrite",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for AccessRight {
        #[inline]
        fn clone(&self) -> AccessRight {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for AccessRight {}
    /// Parameter error types
    pub enum ParameterError {
        IndexNotAvailable,
        SubindexNotAvailable,
        ServiceNotAvailable,
        ServiceNotAvailableLocalControl,
        ServiceNotAvailableDeviceControl,
        AccessDenied,
        ValueOutOfRange,
        ValueAboveLimit,
        ValueBelowLimit,
        LengthOverrun,
        LengthUnderrun,
        FunctionNotAvailable,
        FunctionTemporarilyUnavailable,
        InvalidParameterSet,
        InconsistentParameterSet,
        ApplicationNotReady,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ParameterError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    ParameterError::IndexNotAvailable => "IndexNotAvailable",
                    ParameterError::SubindexNotAvailable => "SubindexNotAvailable",
                    ParameterError::ServiceNotAvailable => "ServiceNotAvailable",
                    ParameterError::ServiceNotAvailableLocalControl => {
                        "ServiceNotAvailableLocalControl"
                    }
                    ParameterError::ServiceNotAvailableDeviceControl => {
                        "ServiceNotAvailableDeviceControl"
                    }
                    ParameterError::AccessDenied => "AccessDenied",
                    ParameterError::ValueOutOfRange => "ValueOutOfRange",
                    ParameterError::ValueAboveLimit => "ValueAboveLimit",
                    ParameterError::ValueBelowLimit => "ValueBelowLimit",
                    ParameterError::LengthOverrun => "LengthOverrun",
                    ParameterError::LengthUnderrun => "LengthUnderrun",
                    ParameterError::FunctionNotAvailable => "FunctionNotAvailable",
                    ParameterError::FunctionTemporarilyUnavailable => {
                        "FunctionTemporarilyUnavailable"
                    }
                    ParameterError::InvalidParameterSet => "InvalidParameterSet",
                    ParameterError::InconsistentParameterSet => {
                        "InconsistentParameterSet"
                    }
                    ParameterError::ApplicationNotReady => "ApplicationNotReady",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ParameterError {
        #[inline]
        fn clone(&self) -> ParameterError {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for ParameterError {}
    /// Parameter information structure
    pub struct ParameterInfo {
        pub index: u16,
        pub subindex: u8,
        pub length: usize,
        pub range: Option<core::ops::Range<u8>>,
        pub access: AccessRight,
        pub data_type: &'static str,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ParameterInfo {
        #[inline]
        fn clone(&self) -> ParameterInfo {
            ParameterInfo {
                index: ::core::clone::Clone::clone(&self.index),
                subindex: ::core::clone::Clone::clone(&self.subindex),
                length: ::core::clone::Clone::clone(&self.length),
                range: ::core::clone::Clone::clone(&self.range),
                access: ::core::clone::Clone::clone(&self.access),
                data_type: ::core::clone::Clone::clone(&self.data_type),
            }
        }
    }
    /// Parameter storage structure
    pub struct ParameterStorage {
        ///Parameter storage for index 0x0x0000 subindex 0x00
        pub index_0000_sub_00: heapless::Vec<u8, 16>,
        ///Parameter storage for index 0x0x0000 subindex 0x0C
        pub index_0000_sub_0c: heapless::Vec<u8, 16>,
        ///Parameter storage for index 0x0x0001 subindex 0x09
        pub index_0001_sub_09: heapless::Vec<u8, 2>,
        ///Parameter storage for index 0x0x0002 subindex 0x00
        pub index_0002_sub_00: heapless::Vec<u8, 1>,
    }
    impl ParameterStorage {
        /// Create new parameter storage with default values
        pub fn new() -> Self {
            Self {
                index_0000_sub_00: heapless::Vec::<u8, 16>::from_slice(&[0; 16])
                    .unwrap(),
                index_0000_sub_0c: heapless::Vec::<u8, 16>::from_slice(&[0; 16])
                    .unwrap(),
                index_0001_sub_09: heapless::Vec::<u8, 2>::from_slice(&[0, 1]).unwrap(),
                index_0002_sub_00: heapless::Vec::<u8, 1>::from_slice(&[0, 1]).unwrap(),
            }
        }
        pub fn clear(&mut self) {
            *self = Self::new();
        }
        /// Get parameter information
        pub fn get_parameter_info(
            &self,
            index: u16,
            subindex: u8,
        ) -> Result<ParameterInfo, ParameterError> {
            let param_infos = [
                ParameterInfo {
                    index: 0x0000,
                    subindex: 0x00,
                    length: 16,
                    range: Some(0..15),
                    access: AccessRight::ReadOnly,
                    data_type: "u8",
                },
                ParameterInfo {
                    index: 0x0000,
                    subindex: 0x0C,
                    length: 16,
                    range: Some(0..15),
                    access: AccessRight::WriteOnly,
                    data_type: "u8",
                },
                ParameterInfo {
                    index: 0x0001,
                    subindex: 0x09,
                    length: 2,
                    range: Some(0..1),
                    access: AccessRight::ReadWrite,
                    data_type: "u16",
                },
                ParameterInfo {
                    index: 0x0002,
                    subindex: 0x00,
                    length: 1,
                    range: Some(0..0),
                    access: AccessRight::WriteOnly,
                    data_type: "u8",
                },
            ];
            for info in &param_infos {
                if info.index == index && info.subindex == subindex {
                    return Ok(info.clone());
                }
            }
            Err(ParameterError::IndexNotAvailable)
        }
        /// Generic function to get parameter value
        pub fn get_parameter<'a>(
            &'a self,
            index: u16,
            subindex: u8,
        ) -> Result<&'a [u8], ParameterError> {
            let info = self.get_parameter_info(index, subindex)?;
            if !match info.access {
                AccessRight::ReadOnly | AccessRight::ReadWrite => true,
                _ => false,
            } {
                return Err(ParameterError::AccessDenied);
            }
            let field_data: &[u8] = match (index, subindex) {
                (0x0000, 0x00) => self.index_0000_sub_00.as_slice(),
                (0x0000, 0x0C) => self.index_0000_sub_0c.as_slice(),
                (0x0001, 0x09) => self.index_0001_sub_09.as_slice(),
                (0x0002, 0x00) => self.index_0002_sub_00.as_slice(),
                _ => return Err(ParameterError::IndexNotAvailable),
            };
            Ok(field_data)
        }
        /// Generic function to set parameter value
        pub fn set_parameter(
            &mut self,
            index: u16,
            subindex: u8,
            data: &[u8],
        ) -> Result<(), ParameterError> {
            let info = self.get_parameter_info(index, subindex)?;
            if !match info.access {
                AccessRight::WriteOnly | AccessRight::ReadWrite => true,
                _ => false,
            } {
                return Err(ParameterError::AccessDenied);
            }
            if data.len() > info.length {
                return Err(ParameterError::LengthOverrun);
            } else if data.len() < info.length {
                return Err(ParameterError::LengthUnderrun);
            }
            match (index, subindex) {
                (0x0000, 0x00) => {
                    self.index_0000_sub_00.copy_from_slice(data);
                    Ok(())
                }
                (0x0000, 0x0C) => {
                    self.index_0000_sub_0c.copy_from_slice(data);
                    Ok(())
                }
                (0x0001, 0x09) => {
                    self.index_0001_sub_09.copy_from_slice(data);
                    Ok(())
                }
                (0x0002, 0x00) => {
                    self.index_0002_sub_00.copy_from_slice(data);
                    Ok(())
                }
                _ => return Err(ParameterError::IndexNotAvailable),
            }
        }
        /// Read entire index memory
        pub fn read_index_memory(
            &self,
            index: u16,
        ) -> Result<heapless::Vec<u8, 16usize>, ParameterError> {
            let mut memory = heapless::Vec::new();
            let param_infos = [
                ParameterInfo {
                    index: 0x0000,
                    subindex: 0x00,
                    length: 16,
                    range: Some(0..15),
                    access: AccessRight::ReadOnly,
                    data_type: "u8",
                },
                ParameterInfo {
                    index: 0x0000,
                    subindex: 0x0C,
                    length: 16,
                    range: Some(0..15),
                    access: AccessRight::WriteOnly,
                    data_type: "u8",
                },
                ParameterInfo {
                    index: 0x0001,
                    subindex: 0x09,
                    length: 2,
                    range: Some(0..1),
                    access: AccessRight::ReadWrite,
                    data_type: "u16",
                },
                ParameterInfo {
                    index: 0x0002,
                    subindex: 0x00,
                    length: 1,
                    range: Some(0..0),
                    access: AccessRight::WriteOnly,
                    data_type: "u8",
                },
            ];
            let mut found = false;
            for info in &param_infos {
                if info.index == index {
                    found = true;
                    if !match info.access {
                        AccessRight::ReadOnly | AccessRight::ReadWrite => true,
                        _ => false,
                    } {
                        return Err(ParameterError::AccessDenied);
                    }
                    let data = self.get_parameter(index, info.subindex)?;
                    memory.extend_from_slice(&data);
                }
            }
            if !found {
                return Err(ParameterError::IndexNotAvailable);
            }
            Ok(memory)
        }
        /// Write entire index memory
        pub fn write_index_memory(
            &mut self,
            index: u16,
            data: &[u8],
        ) -> Result<(), ParameterError> {
            let param_infos = [
                ParameterInfo {
                    index: 0x0000,
                    subindex: 0x00,
                    length: 16,
                    range: Some(0..15),
                    access: AccessRight::ReadOnly,
                    data_type: "u8",
                },
                ParameterInfo {
                    index: 0x0000,
                    subindex: 0x0C,
                    length: 16,
                    range: Some(0..15),
                    access: AccessRight::WriteOnly,
                    data_type: "u8",
                },
                ParameterInfo {
                    index: 0x0001,
                    subindex: 0x09,
                    length: 2,
                    range: Some(0..1),
                    access: AccessRight::ReadWrite,
                    data_type: "u16",
                },
                ParameterInfo {
                    index: 0x0002,
                    subindex: 0x00,
                    length: 1,
                    range: Some(0..0),
                    access: AccessRight::WriteOnly,
                    data_type: "u8",
                },
            ];
            let mut found = false;
            let mut total_length = 0;
            for info in &param_infos {
                if info.index == index {
                    found = true;
                    if !match info.access {
                        AccessRight::WriteOnly | AccessRight::ReadWrite => true,
                        _ => false,
                    } {
                        return Err(ParameterError::AccessDenied);
                    }
                    total_length += info.length;
                }
            }
            if !found {
                return Err(ParameterError::IndexNotAvailable);
            }
            if data.len() != total_length {
                return Err(ParameterError::LengthOverrun);
            }
            let mut offset = 0;
            for info in &param_infos {
                if info.index == index {
                    let subindex_data = &data[offset..offset + info.length];
                    self.set_parameter(index, info.subindex, subindex_data)?;
                    offset += info.length;
                }
            }
            Ok(())
        }
        /// Get all parameter information
        pub fn get_all_parameters(&self) -> &'static [ParameterInfo] {
            static PARAM_INFOS: &[ParameterInfo] = &[
                ParameterInfo {
                    index: 0x0000,
                    subindex: 0x00,
                    length: 16,
                    range: Some(0..15),
                    access: AccessRight::ReadOnly,
                    data_type: "u8",
                },
                ParameterInfo {
                    index: 0x0000,
                    subindex: 0x0C,
                    length: 16,
                    range: Some(0..15),
                    access: AccessRight::WriteOnly,
                    data_type: "u8",
                },
                ParameterInfo {
                    index: 0x0001,
                    subindex: 0x09,
                    length: 2,
                    range: Some(0..1),
                    access: AccessRight::ReadWrite,
                    data_type: "u16",
                },
                ParameterInfo {
                    index: 0x0002,
                    subindex: 0x00,
                    length: 1,
                    range: Some(0..0),
                    access: AccessRight::WriteOnly,
                    data_type: "u8",
                },
            ];
            PARAM_INFOS
        }
        /// Validate parameter constraints
        pub fn validate_constraints(&self) -> Result<(), ParameterError> {
            let param_infos = self.get_all_parameters();
            for info in param_infos {
                if (info.index == 0x0000 || info.index == 0x0001) && info.subindex > 0x0F
                {
                    return Err(ParameterError::InvalidParameterSet);
                }
            }
            Ok(())
        }
    }
}