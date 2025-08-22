//! M-sequence capability configuration for IO-Link devices.
//!
//! This module provides macros for configuring M-sequence capabilities in IO-Link devices
//! according to the IO-Link Specification v1.1.4. It includes configuration for:
//!
//! - PREOPERATE mode M-sequence types (Table A.8)
//! - OPERATE mode M-sequence types for legacy devices (Table A.9)
//! - OPERATE mode M-sequence types for standard devices (Table A.10)
//! - ISDU (Index Sequential Data Unit) support
//! - Complete M-sequence capability parameter construction
//!
//! ## M-sequence Types
//!
//! M-sequences define the communication timing and data exchange patterns between
//! IO-Link masters and devices. Different M-sequence types are used depending on:
//! - Device operation mode (PREOPERATE vs OPERATE)
//! - On-request data length requirements
//! - Process data (PDin/PDout) configurations
//! - Legacy vs standard device implementations
//!
//! ## Usage
//!
//! The macros in this module should be configured according to your device's specific
//! requirements and then used to construct the M-sequence capability parameter that
//! gets reported to the IO-Link master during device identification.
//!
//! ## Compliance Notes
//!
//! - TYPE_0 M-sequences are discouraged for better error detection
//! - Interleaved modes (TYPE_1_1/1_2) must not be implemented in devices
//! - Minimum recovery times must be observed by masters in PREOPERATE mode
//! - All configurations must comply with IO-Link Specification requirements
//! Configuration macros for IO-Link device setup.

pub mod pre_operate_m_sequence {
    use crate::config;


    /// M-sequence types for the PREOPERATE mode as per IO-Link Specification (v1.1.4, Table A.8).
    ///
    /// The following table describes valid M-sequence types, expected on-request data length (in octets),
    /// and the required minimum recovery time (`T_initcyc`) the master must observe in PREOPERATE mode:
    ///
    /// | PREOPERATE M-sequence code | On-request Data (Octets) | M-sequence Type | Minimum Recovery Time (`T_BIT`) |
    /// |----------------------------|---------------------------|------------------|----------------------------------|
    /// | 0                          | 1                         | TYPE_0           | 100                              |
    /// | 1                          | 2                         | TYPE_1_2         | 100                              |
    /// | 2                          | 8                         | TYPE_1_V         | 210                              |
    /// | 3                          | 32                        | TYPE_1_V         | 550                              |
    ///
    /// ---
    ///
    /// - ⚠️ **Note a**: The minimum recovery time in PREOPERATE mode is a requirement for the Master.
    /// - ⚠️ **Note b**: It is highly recommended for Devices **not to use TYPE_0**, as it improves error discovery
    ///   when the Master restarts communication.
    pub const fn m_sequence_code() -> u8 {
        match config::on_req_data::pre_operate::od_length() {
            1 => 0u8,
            2 => 1u8,
            8 => 2u8,
            32 => 3u8,
            _ => panic!("Invalid PREOPERATE M-sequence configuration"),
        }
    }

    pub const fn m_sequence_type() -> crate::types::MsequenceType {
        match m_sequence_code() {
            0 => crate::types::MsequenceType::Type0,
            1 => crate::types::MsequenceType::Type12,
            2 => crate::types::MsequenceType::Type1V,
            3 => crate::types::MsequenceType::Type1V,
            _ => panic!("Invalid PREOPERATE M-sequence configuration"),
        }
    }

    /// Represents the M-sequence type used in IO-Link communication, based on bits 6 and 7 of the
    /// Checksum / M-sequence Type (CKT) field. These bits define how the Master structures messages
    /// within an M-sequence, as specified in Table A.3 of the IO-Link specification (see Section A.1.3).
    ///
    /// This macro depends on the `operate_m_sequence` or `operate_m_sequence_legacy` macros. The selected
    /// M-sequence type in `operate_m_sequence` or `operate_m_sequence_legacy` determines the value of the
    /// `operate_m_sequence_base_type` macro.
    ///
    /// The mapping is as follows:
    /// - `TYPE_0`   → `0`
    /// - `TYPE_1_x` → `1`
    /// - `TYPE_2_x` → `2`
    /// - `3` is reserved and should not be used
    ///
    /// Ensure consistency with the selected M-sequence type when defining dependent macros.
    pub const fn m_sequence_base_type() -> crate::types::MsequenceBaseType {
        match m_sequence_type() {
            crate::types::MsequenceType::Type0 => crate::types::MsequenceBaseType::Type0,
            crate::types::MsequenceType::Type12 => crate::types::MsequenceBaseType::Type1,
            crate::types::MsequenceType::Type1V => crate::types::MsequenceBaseType::Type2,
            _ => panic!("Invalid M-sequence type"),
        }
    }
}

pub mod operate_m_sequence {
    /// Returns whether the device is configured to use interleaved mode for M-sequences, when device is in legacy mode.
    // #[cfg(feature = "legacy_device")]
    // pub const fn interleaved_mode() -> bool {
    //     const OPERATE_M_SEQUENCE_LEGACY: crate::types::MsequenceType = operate_m_sequence_legacy();
    //     (OPERATE_M_SEQUENCE_LEGACY as u8) == (crate::types::MsequenceType::Type11 as u8)
    // }

    /// Returns whether the device is configured to use interleaved mode for M-sequences.
    // #[cfg(not(feature = "legacy_device"))]
    // pub const fn interleaved_mode() -> bool {
    //     const OPERATE_M_SEQUENCE: crate::types::MsequenceType = operate_m_sequence();
    //     (OPERATE_M_SEQUENCE as u8) == (crate::types::MsequenceType::Type11 as u8)
    // }
    use crate::{config};

    /// M-sequence types for the OPERATE mode (standard protocol) as per IO-Link Specification (Table A.10).
    pub const fn operate_m_sequence_code() -> u8 {
        use crate::config::process_data::ProcessDataLength::{self, *};
        const PD_IN_LEN: ProcessDataLength = config::process_data::pd_in::config_length();
        const PD_OUT_LEN: ProcessDataLength = config::process_data::pd_out::config_length();
        const OD_LEN: u8 = config::on_req_data::operate::od_length();
        match (OD_LEN, PD_IN_LEN, PD_OUT_LEN) {
            (1, Bit(0), Bit(0)) | (1, Octet(0), Octet(0)) => 00u8,
            (2, Bit(0), Bit(0)) | (2, Octet(0), Octet(0)) => 01u8,
            (8, Bit(0), Bit(0)) | (8, Octet(0), Octet(0)) => 06u8,
            (32, Bit(0), Bit(0)) | (32, Octet(0), Octet(0)) => 07u8,
            (2, Octet(3..=32), Octet(0..=32)) => 00u8,
            (2, Octet(0..=32), Octet(3..=32)) => 00u8,
            (1, Bit(1..8), Bit(0)) => 00u8,
            (1, Bit(9..16), Bit(0)) => 00u8,
            (1, Bit(0), Bit(1..=8)) => 00u8,
            (1, Bit(0), Bit(9..=16)) => 00u8,
            (1, Bit(1..=8), Bit(1..=8)) => 00u8,
            (1, Bit(9..=16), Bit(1..=16)) => 00u8,
            (1, Bit(1..=16), Bit(9..=16)) => 00u8,
            (1, Octet(0..=32), Octet(3..=32)) => 04u8,
            (1, Octet(3..=32), Octet(0..=32)) => 04u8,
            (2, Bit(1..=16), Bit(0..=16)) => 05u8,
            (2, Bit(0..=16), Bit(1..=16)) => 05u8,
            (8, Bit(1..=16), Bit(0..=16)) => 06u8,
            (8, Bit(0..=16), Bit(1..=16)) => 06u8,
            (32, Bit(1..=16), Bit(0..=16)) => 07u8,
            (32, Bit(0..=16), Bit(1..=16)) => 07u8,
            _ => panic!("Invalid OPERATE M-sequence configuration"),
        }
    }

    /// M-sequence types for the OPERATE mode (per IO-Link Specification, Table A.10).
    ///
    /// This table outlines valid M-sequence types in the OPERATE mode for **Devices**, depending on the
    /// on-request data and process data (PDin and PDout). These combinations determine the M-sequence
    /// type that the Master and Device must use.
    ///
    /// | OPERATE M-sequence code | On-request Data (Octets)   | PDin                     | PDout                    | M-sequence Type          |
    /// |--------------------------|---------------------------|--------------------------|--------------------------|--------------------------|
    /// | 0                        | 1                         | 0                        | 0                        | TYPE_0 (⚠️ see NOTE 1)   |
    /// | 1                        | 2                         | 0                        | 0                        | TYPE_1_2                 |
    /// | 6                        | 8                         | 0                        | 0                        | TYPE_1_V                 |
    /// | 7                        | 32                        | 0                        | 0                        | TYPE_1_V                 |
    /// | 0                        | 2                         | 3…32 octets              | 0…32 octets              | TYPE_1_1 / 1_2 interleaved (⚠️ see NOTE 3) |
    /// | 0                        | 2                         | 0…32 octets              | 3…32 octets              | TYPE_1_1 / 1_2 interleaved (⚠️ see NOTE 3) |
    /// | 0                        | 1                         | 1…8 bit                  | 0                        | TYPE_2_1                 |
    /// | 0                        | 1                         | 9…16 bit                 | 0                        | TYPE_2_2                 |
    /// | 0                        | 1                         | 0                        | 1…8 bit                  | TYPE_2_3                 |
    /// | 0                        | 1                         | 0                        | 9…16 bit                 | TYPE_2_4                 |
    /// | 0                        | 1                         | 1…8 bit                  | 1…8 bit                  | TYPE_2_5                 |
    /// | 0                        | 1                         | 9…16 bit                 | 1…16 bit                 | TYPE_2_V (⚠️ see NOTE 2) |
    /// | 0                        | 1                         | 1…16 bit                 | 9…16 bit                 | TYPE_2_V (⚠️ see NOTE 2) |
    /// | 4                        | 1                         | 0…32 octets              | 3…32 octets              | TYPE_2_V                 |
    /// | 4                        | 1                         | 3…32 octets              | 0…32 octets              | TYPE_2_V                 |
    /// | 5                        | 2                         | >0 bit, octets           | ≥0 bit, octets           | TYPE_2_V                 |
    /// | 5                        | 2                         | ≥0 bit, octets           | >0 bit, octets           | TYPE_2_V                 |
    /// | 6                        | 8                         | >0 bit, octets           | ≥0 bit, octets           | TYPE_2_V                 |
    /// | 6                        | 8                         | ≥0 bit, octets           | >0 bit, octets           | TYPE_2_V                 |
    /// | 7                        | 32                        | >0 bit, octets           | ≥0 bit, octets           | TYPE_2_V                 |
    /// | 7                        | 32                        | ≥0 bit, octets           | >0 bit, octets           | TYPE_2_V                 |
    ///
    /// ---
    ///
    /// ⚠️ **NOTE 1**: It is highly recommended for Devices **not to use TYPE_0**, to improve error detection when the Master restarts communication.
    ///
    /// ⚠️ **NOTE 2**: Former `TYPE_2_6` has been deprecated in favor of `TYPE_2_V` due to inefficiency.
    ///
    /// ⚠️ **NOTE 3**: Interleaved mode (`TYPE_1_1/1_2`) **must not** be implemented in Devices, but should be supported by Masters.
    pub const fn m_sequence_type() -> crate::types::MsequenceType {
        use crate::config::process_data::ProcessDataLength::{self, *};
        use crate::types::MsequenceType;
        const PD_IN_LEN: ProcessDataLength = config::process_data::pd_in::config_length();
        const PD_OUT_LEN: ProcessDataLength = config::process_data::pd_out::config_length();
        const OD_LEN: u8 = config::on_req_data::operate::od_length();
        match (OD_LEN, PD_IN_LEN, PD_OUT_LEN) {
            (1, Bit(0), Bit(0)) | (1, Octet(0), Octet(0)) => MsequenceType::Type0, // TYPE_0
            (2, Bit(0), Bit(0)) | (2, Octet(0), Octet(0)) => MsequenceType::Type12, // TYPE_1_2
            (8, Bit(0), Bit(0)) | (8, Octet(0), Octet(0)) => MsequenceType::Type1V, // TYPE_1_V
            (32, Bit(0), Bit(0)) | (32, Octet(0), Octet(0)) => MsequenceType::Type1V, // TYPE_1_V
            (2, Octet(3..=32), Octet(0..=32)) => MsequenceType::Type11, // TYPE_1_1/1_2 interleaved
            (2, Octet(0..=32), Octet(3..=32)) => MsequenceType::Type11, // TYPE_1_1/1_2 interleaved
            (1, Bit(1..8), Bit(0)) => MsequenceType::Type21,            // TYPE_2_1
            (1, Bit(9..16), Bit(0)) => MsequenceType::Type22,           // TYPE_2_2
            (1, Bit(0), Bit(1..=8)) => MsequenceType::Type23,           // TYPE_2_3
            (1, Bit(0), Bit(9..=16)) => MsequenceType::Type24,          // TYPE_2_4
            (1, Bit(1..=8), Bit(1..=8)) => MsequenceType::Type25,       // TYPE_2_5
            (1, Bit(9..=16), Bit(1..=16)) => MsequenceType::Type2V,     // TYPE_2_V
            (1, Bit(1..=16), Bit(9..=16)) => MsequenceType::Type2V,     // TYPE_2_V
            (1, Octet(0..=32), Octet(3..=32)) => MsequenceType::Type2V, // TYPE_2_V
            (1, Octet(3..=32), Octet(0..=32)) => MsequenceType::Type2V, // TYPE_2_V
            (2, Bit(1..=16), Bit(0..=16)) => MsequenceType::Type2V,     // TYPE_2_V
            (2, Bit(0..=16), Bit(1..=16)) => MsequenceType::Type2V,     // TYPE_2_V
            (8, Bit(1..=16), Bit(0..=16)) => MsequenceType::Type2V,     // TYPE_2_V
            (8, Bit(0..=16), Bit(1..=16)) => MsequenceType::Type2V,     // TYPE_2_V
            (32, Bit(1..=16), Bit(0..=16)) => MsequenceType::Type2V,    // TYPE_2_V
            (32, Bit(0..=16), Bit(1..=16)) => MsequenceType::Type2V,    // TYPE_2_V
            _ => panic!("Invalid OPERATE M-sequence configuration"),
        }
    }

    /// Represents the M-sequence type used in IO-Link communication, based on bits 6 and 7 of the
    /// Checksum / M-sequence Type (CKT) field. These bits define how the Master structures messages
    /// within an M-sequence, as specified in Table A.3 of the IO-Link specification (see Section A.1.3).
    ///
    /// This macro depends on the `operate_m_sequence` or `operate_m_sequence_legacy` macros. The selected
    /// M-sequence type in `operate_m_sequence` or `operate_m_sequence_legacy` determines the value of the
    /// `operate_m_sequence_base_type` macro.
    ///
    /// The mapping is as follows:
    /// - `TYPE_0`   → `0`
    /// - `TYPE_1_x` → `1`
    /// - `TYPE_2_x` → `2`
    /// - `3` is reserved and should not be used
    ///
    /// Ensure consistency with the selected M-sequence type when defining dependent macros.
    pub const fn m_sequence_base_type() -> crate::types::MsequenceBaseType {
        match m_sequence_type() {
            crate::types::MsequenceType::Type0 => crate::types::MsequenceBaseType::Type0,
            crate::types::MsequenceType::Type12 => crate::types::MsequenceBaseType::Type1,
            crate::types::MsequenceType::Type1V => crate::types::MsequenceBaseType::Type2,
            _ => panic!("Invalid M-sequence type"),
        }
    }
}

/// ## B.1.4 M-sequenceCapability
/// Bit 0: ISDU
// This bit indicates whether or not the ISDU communication channel is supported. Permissible
// values for ISDU are listed in Table B.4.
/// - `0` = ISDU not supported
/// - `1` = ISDU supported
pub const fn isdu_supported() -> bool {
    true // Accepted values true for ISDU support, false for no support
}

/// ## M-sequenceCapability (B.1.4)
///
/// The structure of the `M-sequenceCapability` parameter is defined as follows:
///
/// ```text
/// Bit 7   | Bit 6   | Bit 5 - Bit 4  | Bit 3 - Bit 1  | Bit 0
/// ------- | ------- | -------------- | -------------- | -------
/// Reserved| Reserved| PREOP.         | OPERATE        | ISDU
///         |         | M-seq code     | M-seq code     |
/// ```
///
/// ### Bit 0: ISDU
/// - `0` = ISDU not supported  
/// - `1` = ISDU supported
///
/// ### Bits 1 to 3: OPERATE M-sequence type
/// Encodes the M-sequence type for the OPERATE state.  
/// - See Table A.9 for legacy Devices  
/// - See Table A.10 for standard Devices
///
/// ### Bits 4 to 5: PREOPERATE M-sequence type
/// Encodes the M-sequence type for the PREOPERATE state.  
/// - See Table A.8
///
/// ### Bits 6 to 7: Reserved
/// - These bits are reserved and shall be set to `0`
pub const fn config_m_sequence_capability() -> u8 {
    use crate::utils::page_params::page1;
    use operate_m_sequence::{m_sequence_type, operate_m_sequence_code};
    let mut m_sequence_cap: page1::MsequenceCapability = page1::MsequenceCapability::new();
    m_sequence_cap.set_isdu(isdu_supported());
    m_sequence_cap.set_preoperate_m_sequence(operate_m_sequence_code() as u8);
    m_sequence_cap.set_operate_m_sequence(m_sequence_type() as u8);
    m_sequence_cap.into_bits()
}
