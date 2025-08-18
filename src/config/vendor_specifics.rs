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

/// Macro to define vendor specific parameters.
pub const fn major_revision_id() -> u8 {
    0x09u8
}

/// Macro to define vendor specific parameters.
pub const fn minor_revision_id() -> u8 {
    0x4u8
}

pub const fn revision_id() -> u8 {
    const MAJOR_REVISION_ID: u8 = major_revision_id();
    const MINOR_REVISION_ID: u8 = minor_revision_id();

    if MAJOR_REVISION_ID > 0b1111 {
        panic!("Invalid major revision id provided is not in the range 0-15");
    }
    if MINOR_REVISION_ID > 0b1111 {
        panic!("Invalid minor revision id provided is not in the range 0-15");
    }
    MAJOR_REVISION_ID << 4 | MINOR_REVISION_ID
}

pub const fn vendor_id_1() -> u8 {
    0x01u8
}

pub const fn vendor_id_2() -> u8 {
    0x01u8
}

pub const fn device_id_1() -> u8 {
    0x01u8
}

pub const fn device_id_2() -> u8 {
    0x01u8
}

pub const fn device_id_3() -> u8 {
    0x01u8
}

pub const fn function_id_1() -> u8 {
    0x01u8
}

pub const fn function_id_2() -> u8 {
    0x01u8
}

pub const fn vendor_name() -> &'static [u8] {
    b"IOLinke"
}

pub const fn vendor_name_length() -> u8 {
    vendor_name().len() as u8
}

pub const fn product_name() -> &'static [u8] {
    b"IOLinke"
}

pub const fn product_name_length() -> u8 {
    product_name().len() as u8
}

/// Returns the number of entries in the Index List for the Data Storage Index (Index 0x0020).
///
/// According to IO-Link Specification Table B.19, the Index List contains 9 entries.
/// Each entry describes a parameter record for data storage.
///
/// # Example
/// ```
/// let entries = index_list_entries!(); // 9u8
/// ```
#[macro_export]
macro_rules! index_list_entries {
    () => {
        09u8
    };
}

/// Returns the bit offset to the start of the Data Storage Index (Index 0x0020).
///
/// The offset is calculated as `(index_list_entries!() * 3 + 2) * 8` bits, as specified in Table B.18.
/// This offset marks the start of the Data Storage Index structure in the parameter memory.
///
/// # Example
/// ```
/// let offset = data_storage_index_offset!();
/// ```
#[macro_export]
macro_rules! data_storage_index_offset_in_bytes {
    () => {
        (index_list_entries!() * 3 + 2)
    };
}

/// Returns the bit offset to the start of the Data Storage Index (Index 0x0020).
///
/// The offset is calculated as `(index_list_entries!() * 3 + 2) * 8` bits, as specified in Table B.18.
/// This offset marks the start of the Data Storage Index structure in the parameter memory.
///
/// # Example
/// ```
/// let offset = data_storage_index_offset!();
/// ```
#[macro_export]
macro_rules! data_storage_index_list_length {
    () => {
        data_storage_index_offset_in_bytes!() + 1
    };
}

/// Returns the subindex value for the Index List within the Data Storage Index (Index 0x0020).
///
/// As per Table B.19, the subindex for the Index List is 0x05.
///
/// # Example
/// ```
/// let subidx = index_list_subindex!(); // 0x05
/// ```
#[macro_export]
macro_rules! index_list_subindex {
    () => {
        05u8
    };
}

/// Returns the bit offset for the Index List field within the Data Storage Index (Index 0x0020).
///
/// The offset for the Index List is always 0, as it is the first field in the structure.
///
/// # Example
/// ```
/// let offset = index_list_offset!(); // 0u8
/// ```
#[macro_export]
macro_rules! index_list_offset {
    () => {
        00u8
    };
}

/// Returns the subindex value for the Parameter Checksum field in the Data Storage Index (Index 0x0020).
///
/// According to Table B.19, the subindex for the Parameter Checksum is 0x04.
///
/// # Example
/// ```
/// let subidx = parameter_checksum_subindex!(); // 0x04
/// ```
#[macro_export]
macro_rules! parameter_checksum_subindex {
    () => {
        04u8
    };
}

/// Returns the bit offset for the Parameter Checksum field in the Data Storage Index (Index 0x0020).
///
/// The offset is calculated as the start of the Data Storage Index structure, as per Table B.18.
///
/// # Example
/// ```
/// let offset = parameter_checksum_offset!();
/// ```
#[macro_export]
macro_rules! parameter_checksum_offset {
    () => {
        data_storage_index_offset!()
    };
}

/// Returns the subindex value for the Data Storage Size field in the Data Storage Index (Index 0x0020).
///
/// As specified in Table B.19, the subindex for Data Storage Size is 0x03.
///
/// # Example
/// ```
/// let subidx = data_storage_size_subindex!(); // 0x03
/// ```
#[macro_export]
macro_rules! data_storage_size_subindex {
    () => {
        03u8
    };
}

/// Returns the bit offset for the Data Storage Size field in the Data Storage Index (Index 0x0020).
///
/// The offset is calculated as `data_storage_index_offset!() + 32` bits, as per Table B.18.
///
/// # Example
/// ```
/// let offset = data_storage_size_offset!();
/// ```
#[macro_export]
macro_rules! data_storage_size_offset {
    () => {
        data_storage_index_offset!() + 32
    };
}

/// Returns the subindex value for the State/Property field in the Data Storage Index (Index 0x0020).
///
/// According to Table B.19, the subindex for State/Property is 0x02.
///
/// # Example
/// ```
/// let subidx = state_property_subindex!(); // 0x02
/// ```
#[macro_export]
macro_rules! state_property_subindex {
    () => {
        02u8
    };
}

/// Returns the bit offset for the State/Property field in the Data Storage Index (Index 0x0020).
///
/// The offset is calculated as `data_storage_index_offset!() + 64` bits, as per Table B.18.
///
/// # Example
/// ```
/// let offset = state_property_offset!();
/// ```
#[macro_export]
macro_rules! state_property_offset {
    () => {
        data_storage_index_offset!() + 64
    };
}

/// Returns the subindex value for the State/Storage Command field in the Data Storage Index (Index 0x0020).
///
/// As specified in Table B.19, the subindex for State/Storage Command is 0x01.
///
/// # Example
/// ```
/// let subidx = state_storage_command_subindex!(); // 0x01
/// ```
#[macro_export]
macro_rules! state_storage_command_subindex {
    () => {
        01u8
    };
}

/// Returns the bit offset for the State/Storage Command field in the Data Storage Index (Index 0x0020).
///
/// The offset is calculated as `data_storage_index_offset!() + 72` bits, as per Table B.18.
///
/// # Example
/// ```
/// let offset = state_storage_command_offset!();
/// ```
#[macro_export]
macro_rules! state_storage_command_offset {
    () => {
        data_storage_index_offset!() + 72
    };
}

pub mod storage_config {
    use iolinke_macros::declare_parameter_storage;

    use crate::{al, config};

    /// Provides grouped access to index and subindex constants for parameter storage.
    mod param_indices {
        use super::al;

        /// Returns the index for Direct Parameter Page 1 (0x0000).
        #[inline(always)]
        pub const fn page_1() -> u16 {
            al::DeviceParametersIndex::DirectParameterPage1.index()
        }

        /// Returns the index for System Command (0x0002).
        #[inline(always)]
        pub const fn system_command() -> u16 {
            al::DeviceParametersIndex::SystemCommand.index()
        }

        /// Returns the index for Data Storage Index (0x0003).
        #[inline(always)]
        pub const fn data_storage() -> u16 {
            al::DeviceParametersIndex::DataStorageIndex.index()
        }

        /// Returns the index for Vendor Name (0x0010).
        #[inline(always)]
        pub const fn vendor_name() -> u16 {
            al::DeviceParametersIndex::VendorName.index()
        }

        /// Returns the index for Product Name (0x0012).
        #[inline(always)]
        pub const fn product_name() -> u16 {
            al::DeviceParametersIndex::ProductName.index()
        }
    }

    /// Provides grouped subindex constants for Direct Parameter Page 1.
    mod page_1_subindices {
        use super::al;

        /// Subindices for Direct Parameter Page 1 (Index 0x0000).
        #[inline(always)]
        pub const fn master_command() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::MasterCommand),
            )
        }
        #[inline(always)]
        pub const fn master_cycle_time() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::MasterCycleTime),
            )
        }
        #[inline(always)]
        pub const fn min_cycle_time() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::MinCycleTime),
            )
        }
        #[inline(always)]
        pub const fn m_sequence_capability() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::MSequenceCapability),
            )
        }
        #[inline(always)]
        pub const fn revision_id() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::RevisionID),
            )
        }
        #[inline(always)]
        pub const fn process_data_in() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::ProcessDataIn),
            )
        }
        #[inline(always)]
        pub const fn process_data_out() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::ProcessDataOut),
            )
        }
        #[inline(always)]
        pub const fn vendor_id_1() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::VendorID1),
            )
        }
        #[inline(always)]
        pub const fn vendor_id_2() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::VendorID2),
            )
        }
        #[inline(always)]
        pub const fn device_id_1() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::DeviceID1),
            )
        }
        #[inline(always)]
        pub const fn device_id_2() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::DeviceID2),
            )
        }
        #[inline(always)]
        pub const fn device_id_3() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::DeviceID3),
            )
        }
        #[inline(always)]
        pub const fn function_id_1() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::FunctionID1),
            )
        }
        #[inline(always)]
        pub const fn function_id_2() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::FunctionID2),
            )
        }
        #[inline(always)]
        pub const fn system_command() -> u8 {
            al::DeviceParametersIndex::DirectParameterPage1.subindex(
                al::SubIndex::DpPage1(al::DirectParameterPage1SubIndex::SystemCommand),
            )
        }
    }

    /// Provides grouped constants for Data Storage Index List.
    mod data_storage_list {
        use super::al;

        /// Returns the offset for the index list in the Data Storage Index.
        #[inline(always)]
        pub const fn offset() -> u8 {
            index_list_offset!()
        }
        /// Returns the subindex for the index list in the Data Storage Index.
        #[inline(always)]
        pub const fn subindex() -> u8 {
            index_list_subindex!()
        }
        /// Returns the length of the index list in the Data Storage Index.
        #[inline(always)]
        pub const fn length() -> u8 {
            data_storage_index_list_length!()
        }

        /// Returns the length of the index list in the Data Storage Index.
        #[inline(always)]
        pub const fn state_property() -> u8 {
            al::DeviceParametersIndex::DataStorageIndex.subindex(
                al::SubIndex::DataStorageIndex(al::DataStorageIndexSubIndex::StateProperty),
            )
        }

        /// Returns the length of the index list in the Data Storage Index.
        #[inline(always)]
        pub const fn data_storage_size() -> u8 {
            al::DeviceParametersIndex::DataStorageIndex.subindex(
                al::SubIndex::DataStorageIndex(al::DataStorageIndexSubIndex::DataStorageSize),
            )
        }

        /// Returns the length of the index list in the Data Storage Index.
        #[inline(always)]
        pub const fn parameter_checksum() -> u8 {
            al::DeviceParametersIndex::DataStorageIndex.subindex(
                al::SubIndex::DataStorageIndex(al::DataStorageIndexSubIndex::ParameterChecksum),
            )
        }
        
        /// Returns the length of the index list in the Data Storage Index.
        #[inline(always)]
        pub const fn ds_command() -> u8 {
            al::DeviceParametersIndex::DataStorageIndex.subindex(
                al::SubIndex::DataStorageIndex(al::DataStorageIndexSubIndex::DsCommand),
            )
        }
    }

    /// Provides grouped constants for vendor and product information.
    mod vendor_product_info {
        use super::config;

        /// Returns the vendor name as a byte slice.
        #[inline(always)]
        pub const fn vendor_name() -> &'static [u8] {
            config::vendor_specifics::vendor_name()
        }
        /// Returns the length of the vendor name.
        #[inline(always)]
        pub const fn vendor_name_length() -> u8 {
            config::vendor_specifics::vendor_name_length()
        }
        /// Returns the product name as a byte slice.
        #[inline(always)]
        pub const fn product_name() -> &'static [u8] {
            config::vendor_specifics::product_name()
        }
        /// Returns the length of the product name.
        #[inline(always)]
        pub const fn product_name_length() -> u8 {
            config::vendor_specifics::product_name_length()
        }
    }

    /// Provides grouped constants for configuration values.
    mod config_values {
        use super::config;

        /// Returns the minimum cycle time configuration value.
        #[inline(always)]
        pub const fn min_cycle_time() -> u8 {
            config::timings::min_cycle_time()
        }
        /// Returns the M-Sequence capability configuration value.
        #[inline(always)]
        pub const fn m_sequence_capability() -> u8 {
            config::m_seq_capability::config_m_sequence_capability()
        }
        /// Returns the revision ID configuration value.
        #[inline(always)]
        pub const fn revision_id() -> u8 {
            config::vendor_specifics::revision_id()
        }
        /// Returns the process data in configuration value.
        #[inline(always)]
        pub const fn process_data_in() -> u8 {
            config::process_data::pd_in::pd_in()
        }
        /// Returns the process data out configuration value.
        #[inline(always)]
        pub const fn process_data_out() -> u8 {
            config::process_data::pd_out::pd_out()
        }
        /// Returns the vendor ID 1 configuration value.
        #[inline(always)]
        pub const fn vendor_id_1() -> u8 {
            config::vendor_specifics::vendor_id_1()
        }
        /// Returns the vendor ID 2 configuration value.
        #[inline(always)]
        pub const fn vendor_id_2() -> u8 {
            config::vendor_specifics::vendor_id_2()
        }
        /// Returns the device ID 1 configuration value.
        #[inline(always)]
        pub const fn device_id_1() -> u8 {
            config::vendor_specifics::device_id_1()
        }
        /// Returns the device ID 2 configuration value.
        #[inline(always)]
        pub const fn device_id_2() -> u8 {
            config::vendor_specifics::device_id_2()
        }
        /// Returns the device ID 3 configuration value.
        #[inline(always)]
        pub const fn device_id_3() -> u8 {
            config::vendor_specifics::device_id_3()
        }
        /// Returns the function ID 1 configuration value.
        #[inline(always)]
        pub const fn function_id_1() -> u8 {
            config::vendor_specifics::function_id_1()
        }
        /// Returns the function ID 2 configuration value.
        #[inline(always)]
        pub const fn function_id_2() -> u8 {
            config::vendor_specifics::function_id_2()
        }
    }

    // Now, use the grouped const fns to get values for the parameter storage macro.
    // Grouped by index for clarity and maintainability.

    // --- Direct Parameter Page 1 (0x0000) ---
    const PAGE_1_INDEX: u16 = param_indices::page_1();
    const MASTER_COMMAND_SUBINDEX: u8 = page_1_subindices::master_command();
    const MASTER_CYCLE_TIME_SUBINDEX: u8 = page_1_subindices::master_cycle_time();
    const MIN_CYCLE_TIME_SUBINDEX: u8 = page_1_subindices::min_cycle_time();
    const M_SEQUENCE_CAPABILITY_SUBINDEX: u8 = page_1_subindices::m_sequence_capability();
    const REVISION_ID_SUBINDEX: u8 = page_1_subindices::revision_id();
    const PROCESS_DATA_IN_SUBINDEX: u8 = page_1_subindices::process_data_in();
    const PROCESS_DATA_OUT_SUBINDEX: u8 = page_1_subindices::process_data_out();
    const VENDOR_ID_1_SUBINDEX: u8 = page_1_subindices::vendor_id_1();
    const VENDOR_ID_2_SUBINDEX: u8 = page_1_subindices::vendor_id_2();
    const DEVICE_ID_1_SUBINDEX: u8 = page_1_subindices::device_id_1();
    const DEVICE_ID_2_SUBINDEX: u8 = page_1_subindices::device_id_2();
    const DEVICE_ID_3_SUBINDEX: u8 = page_1_subindices::device_id_3();
    const FUNCTION_ID_1_SUBINDEX: u8 = page_1_subindices::function_id_1();
    const FUNCTION_ID_2_SUBINDEX: u8 = page_1_subindices::function_id_2();
    const SYSTEM_COMMAND_SUBINDEX: u8 = page_1_subindices::system_command();

    // --- System Command (0x0002) ---
    const SYSTEM_COMMAND_INDEX: u16 = param_indices::system_command();

    // --- Data Storage Index (0x0003) ---
    const DATA_STORAGE_INDEX_INDEX: u16 = param_indices::data_storage();
    const STATE_PROPERTY_SUBINDEX: u8 = data_storage_list::state_property();
    const DATA_STORAGE_SIZE_SUBINDEX: u8 = data_storage_list::data_storage_size();
    const PARAMETER_CHECKSUM_SUBINDEX: u8 = data_storage_list::parameter_checksum();
    const INDEX_LIST_SUBINDEX: u8 = data_storage_list::subindex();
    const DS_COMMAND_SUBINDEX: u8 = data_storage_list::ds_command();
    const INDEX_LIST_LENGTH: u8 = data_storage_list::length();
    const INDEX_LIST_OFFSET: u8 = data_storage_list::offset();

    // --- Vendor/Product Name (0x0010, 0x0012) ---
    const VENDOR_NAME_INDEX: u16 = param_indices::vendor_name();
    const VENDOR_NAME: &[u8] = vendor_product_info::vendor_name();
    const VENDOR_NAME_LENGTH: u8 = vendor_product_info::vendor_name_length();
    const PRODUCT_NAME_INDEX: u16 = param_indices::product_name();
    const PRODUCT_NAME: &[u8] = vendor_product_info::product_name();
    const PRODUCT_NAME_LENGTH: u8 = vendor_product_info::product_name_length();

    // --- Configuration Values ---
    const MIN_CYCLE_TIME_CONFIG_VALUE: u8 = config_values::min_cycle_time();
    const M_SEQUENCE_CAPABILITY_CONFIG_VALUE: u8 = config_values::m_sequence_capability();
    const REVISION_ID_CONFIG_VALUE: u8 = config_values::revision_id();
    const PROCESS_DATA_IN_CONFIG_VALUE: u8 = config_values::process_data_in();
    const PROCESS_DATA_OUT_CONFIG_VALUE: u8 = config_values::process_data_out();
    const VENDOR_ID_1_CONFIG_VALUE: u8 = config_values::vendor_id_1();
    const VENDOR_ID_2_CONFIG_VALUE: u8 = config_values::vendor_id_2();
    const DEVICE_ID_1_CONFIG_VALUE: u8 = config_values::device_id_1();
    const DEVICE_ID_2_CONFIG_VALUE: u8 = config_values::device_id_2();
    const DEVICE_ID_3_CONFIG_VALUE: u8 = config_values::device_id_3();
    const FUNCTION_ID_1_CONFIG_VALUE: u8 = config_values::function_id_1();
    const FUNCTION_ID_2_CONFIG_VALUE: u8 = config_values::function_id_2();

    // TODO: Integer literals are written because rust compiler does not support macro expansion in macro calls/constants
    // TODO: When rust compiler supports macro expansion in macro calls/constants, the integer literals can be removed and use the macro expansion directly or constants
    // TODO: May need to handle the DATA_STORAGE_INDEX_INDEX
    declare_parameter_storage! {
        // Note: Index 0x0000 and 0x0001 can only have subindexes 0x00-0x0F
        //                               Index,                                 Subindex,                        Length, IndexRange,    Access, Type, DefaultValue,
        (            /* PAGE_1_INDEX */ 0x0000,        /* MASTER_COMMAND_SUBINDEX */ 0x00,              /* Default */ 1,       0..0, WriteOnly,   u8, &[0]),
        (            /* PAGE_1_INDEX */ 0x0000,     /* MASTER_CYCLE_TIME_SUBINDEX */ 0x01,              /* Default */ 1,       0..0, ReadWrite,   u8, &[0]),
        (            /* PAGE_1_INDEX */ 0x0000,        /* MIN_CYCLE_TIME_SUBINDEX */ 0x02,              /* Default */ 1,       0..0, ReadOnly,    u8, &[MIN_CYCLE_TIME_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000, /* M_SEQUENCE_CAPABILITY_SUBINDEX */ 0x03,              /* Default */ 1,       0..0, ReadOnly,    u8, &[M_SEQUENCE_CAPABILITY_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,           /* REVISION_ID_SUBINDEX */ 0x04,              /* Default */ 1,       0..0, ReadOnly,    u8, &[REVISION_ID_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,       /* PROCESS_DATA_IN_SUBINDEX */ 0x05,              /* Default */ 1,       0..0, ReadOnly,    u8, &[PROCESS_DATA_IN_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,      /* PROCESS_DATA_OUT_SUBINDEX */ 0x06,              /* Default */ 1,       0..0, ReadOnly,    u8, &[PROCESS_DATA_OUT_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,           /* VENDOR_ID_1_SUBINDEX */ 0x07,              /* Default */ 1,       0..0, ReadOnly,    u8, &[VENDOR_ID_1_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,           /* VENDOR_ID_2_SUBINDEX */ 0x08,              /* Default */ 1,       0..0, ReadOnly,    u8, &[VENDOR_ID_2_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,           /* DEVICE_ID_1_SUBINDEX */ 0x09,              /* Default */ 1,       0..0, ReadWrite,   u8, &[DEVICE_ID_1_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,           /* DEVICE_ID_2_SUBINDEX */ 0x0A,              /* Default */ 1,       0..0, ReadWrite,   u8, &[DEVICE_ID_2_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,           /* DEVICE_ID_3_SUBINDEX */ 0x0B,              /* Default */ 1,       0..0, ReadWrite,   u8, &[DEVICE_ID_3_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,         /* FUNCTION_ID_1_SUBINDEX */ 0x0C,              /* Default */ 1,       0..0, ReadOnly,    u8, &[FUNCTION_ID_1_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,         /* FUNCTION_ID_2_SUBINDEX */ 0x0D,              /* Default */ 1,       0..0, ReadOnly,    u8, &[FUNCTION_ID_2_CONFIG_VALUE]),
        (            /* PAGE_1_INDEX */ 0x0000,        /* SYSTEM_COMMAND_SUBINDEX */ 0x0F,              /* Default */ 1,       0..0, WriteOnly,   u8, &[0]),
        // PAGE_2 is reserved for vendor-specific parameters.
        // Add entries here if your device requires additional vendor-specific configuration.
        // Example:
        // (0x0001, 0x10, 1, 0..0, ReadWrite, u8, &[0; 1]), // Vendor-specific parameter
        (    /* SYSTEM_COMMAND_INDEX */ 0x0002,        /* SYSTEM_COMMAND_SUBINDEX */ 0x00,             /* Default */  1,       0..0, WriteOnly,   u8, &[0]),
        (/* DATA_STORAGE_INDEX_INDEX */ 0x0003,            /* DS_COMMAND_SUBINDEX */ 0x01,             /* Default */  1,       0..0, ReadWrite,   u8, &[0]),
        (/* DATA_STORAGE_INDEX_INDEX */ 0x0003,        /* STATE_PROPERTY_SUBINDEX */ 0x02,             /* Default */  1,       0..0, ReadWrite,   u8, &[0]),
        (/* DATA_STORAGE_INDEX_INDEX */ 0x0003,     /* DATA_STORAGE_SIZE_SUBINDEX */ 0x03,             /* Default */  1,       0..0, ReadWrite,   u8, &[0]),
        (/* DATA_STORAGE_INDEX_INDEX */ 0x0003,    /* PARAMETER_CHECKSUM_SUBINDEX */ 0x04,             /* Default */  1,       0..0, ReadWrite,   u8, &[0]),
        (/* DATA_STORAGE_INDEX_INDEX */ 0x0003,            /* INDEX_LIST_SUBINDEX */ 0x05,   /* INDEX_LIST_LENGTH */ 30,      0..29, ReadWrite,   u8, &[0]),
        (       /* VENDOR_NAME_INDEX */ 0x0010,           /* VENDOR_NAME_SUBINDEX */ 0x00,  /* VENDOR_NAME_LENGTH */  7,       0..6, ReadOnly,    u8, VENDOR_NAME),
        (      /* PRODUCT_NAME_INDEX */ 0x0012,          /* PRODUCT_NAME_SUBINDEX */ 0x00, /* PRODUCT_NAME_LENGTH */  7,       0..6, ReadOnly,    u8, PRODUCT_NAME),
    }
}
