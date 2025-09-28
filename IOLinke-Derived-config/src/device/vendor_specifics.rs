use iolinke_dev_config::device as dev_config;
use iolinke_types::page::page1::RevisionId;

pub use core::result::{Result, Result::{Ok, Err}};

pub const fn revision_id_parameter() -> RevisionId {
    RevisionId::from_bits(dev_config::vendor_specifics::REVISION_ID)
}

pub const fn vendor_id_1() -> u8 {
    dev_config::vendor_specifics::VENDOR_ID[0]
}

pub const fn vendor_id_2() -> u8 {
    dev_config::vendor_specifics::VENDOR_ID[1]
}

pub const fn device_id_1() -> u8 {
    dev_config::vendor_specifics::DEVICE_ID[0]
}

pub const fn device_id_2() -> u8 {
    dev_config::vendor_specifics::DEVICE_ID[1]
}

pub const fn device_id_3() -> u8 {
    dev_config::vendor_specifics::DEVICE_ID[2]
}

pub const fn function_id_1() -> u8 {
    dev_config::vendor_specifics::FUNCTION_ID[0]
}

pub const fn function_id_2() -> u8 {
    dev_config::vendor_specifics::FUNCTION_ID[1]
}

pub const fn vendor_name() -> &'static str {
    dev_config::vendor_specifics::VENDOR_NAME
}

pub const fn product_name() -> &'static str {
    dev_config::vendor_specifics::PRODUCT_NAME
}

pub mod storage_config {
    use iolinke_macros::declare_parameter_storage;
    /// Provides grouped constants for configuration values.
    mod config_values {
        use crate::device::m_seq_capability;
        use crate::device::process_data;
        use crate::device::timings;
        use iolinke_dev_config::device as dev_config;

        /// Returns the minimum cycle time configuration value.
        #[inline(always)]
        pub const fn min_cycle_time() -> u8 {
            timings::min_cycle_time::min_cycle_time_parameter().into_bits()
        }
        /// Returns the M-Sequence capability configuration value.
        #[inline(always)]
        pub const fn m_sequence_capability() -> u8 {
            m_seq_capability::m_sequence_capability_parameter().into_bits()
        }
        /// Returns the revision ID configuration value.
        #[inline(always)]
        pub const fn revision_id() -> u8 {
            dev_config::vendor_specifics::REVISION_ID
        }
        /// Returns the process data in configuration value.
        #[inline(always)]
        pub const fn process_data_in() -> u8 {
            process_data::pd_in::pd_in_parameter().into_bits()
        }
        /// Returns the process data out configuration value.
        #[inline(always)]
        pub const fn process_data_out() -> u8 {
            process_data::pd_out::pd_out_parameter().into_bits()
        }
        /// Returns the vendor ID 1 configuration value.
        #[inline(always)]
        pub const fn vendor_id_1() -> u8 {
            dev_config::vendor_specifics::VENDOR_ID[0]
        }
        /// Returns the vendor ID 2 configuration value.
        #[inline(always)]
        pub const fn vendor_id_2() -> u8 {
            dev_config::vendor_specifics::VENDOR_ID[1]
        }
        /// Returns the device ID 1 configuration value.
        #[inline(always)]
        pub const fn device_id_1() -> u8 {
            dev_config::vendor_specifics::DEVICE_ID[0]
        }
        /// Returns the device ID 2 configuration value.
        #[inline(always)]
        pub const fn device_id_2() -> u8 {
            dev_config::vendor_specifics::DEVICE_ID[1]
        }
        /// Returns the device ID 3 configuration value.
        #[inline(always)]
        pub const fn device_id_3() -> u8 {
            dev_config::vendor_specifics::DEVICE_ID[2]
        }
        /// Returns the function ID 1 configuration value.
        #[inline(always)]
        pub const fn function_id_1() -> u8 {
            dev_config::vendor_specifics::FUNCTION_ID[0]
        }
        /// Returns the function ID 2 configuration value.
        #[inline(always)]
        pub const fn function_id_2() -> u8 {
            dev_config::vendor_specifics::FUNCTION_ID[1]
        }

        /// Returns the vendor name configuration value.
        #[inline(always)]
        pub const fn vendor_name() -> &'static str {
            dev_config::vendor_specifics::VENDOR_NAME
        }
        /// Returns the product name configuration value.
        #[inline(always)]
        pub const fn product_name() -> &'static str {
            dev_config::vendor_specifics::PRODUCT_NAME
        }
    }

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
    const VENDOR_NAME: &'static str = config_values::vendor_name();
    const PRODUCT_NAME: &'static str = config_values::product_name();

    // TODO: Integer literals are written because rust compiler not yet support macro expansion in macro calls/constants
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
        (/* DATA_STORAGE_INDEX_INDEX */ 0x0003,            /* INDEX_LIST_SUBINDEX */ 0x05,   /* INDEX_LIST_LENGTH */ 30,      0..29, ReadWrite,   u8, &[0; 30]),
        (       /* VENDOR_NAME_INDEX */ 0x0010,           /* VENDOR_NAME_SUBINDEX */ 0x00,  /* VENDOR_NAME_LENGTH */  7,       0..6, ReadOnly,    u8, VENDOR_NAME.as_bytes()),
        (      /* PRODUCT_NAME_INDEX */ 0x0012,          /* PRODUCT_NAME_SUBINDEX */ 0x00, /* PRODUCT_NAME_LENGTH */  7,       0..6, ReadOnly,    u8, PRODUCT_NAME.as_bytes()),
    }
}
